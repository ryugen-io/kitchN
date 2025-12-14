use anyhow::{Context, Result};
use colored::Colorize;
use k_lib::config::Cookbook;
use k_lib::logger;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use tracing::{debug, warn};
use tracing_subscriber::{EnvFilter, Layer, prelude::*};

/// Helper to log using k_lib::logger directly
pub fn log(config: &Cookbook, preset_key: &str) {
    if let Some(preset) = config.dictionary.presets.get(preset_key)
        && let Some(scope) = &preset.scope
    {
        // 1. User Facing Log
        logger::log_to_terminal(config, &preset.level, scope, &preset.msg);

        // 2. Debug Watcher (Socket Broadcast)
        // RE-ENABLED MANUAL TRACE to ensure high level semantic logs are visible
        tracing::info!(scope = scope, message = &preset.msg);

        if config.layout.logging.write_by_default {
            let _ = logger::log_to_file(config, &preset.level, scope, &preset.msg, None);
        }
    }
}

/// Helper to log with custom msg
pub fn log_msg(config: &Cookbook, preset_key: &str, msg_override: &str) {
    if let Some(preset) = config.dictionary.presets.get(preset_key)
        && let Some(scope) = &preset.scope
    {
        // 1. User Facing Log
        logger::log_to_terminal(config, &preset.level, scope, msg_override);

        // 2. Debug Watcher Log (Socket Broadcast)
        // RE-ENABLED MANUAL TRACE to ensure high level semantic logs are visible
        tracing::info!(scope = scope, message = msg_override);

        if config.layout.logging.write_by_default {
            let _ = logger::log_to_file(config, &preset.level, scope, msg_override, None);
        }
    }
}

pub fn get_socket_path() -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir());
    runtime_dir.join("kitchn-debug.sock")
}

pub struct SocketSubscriberLayer {
    socket: Mutex<Option<UnixStream>>,
}

impl SocketSubscriberLayer {
    pub fn new(socket_path: &Path) -> Self {
        // Try to connect, if fail, we just won't log to socket
        let socket = UnixStream::connect(socket_path).ok();
        Self {
            socket: Mutex::new(socket),
        }
    }
}

impl<S> Layer<S> for SocketSubscriberLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Only lock if we have a socket.
        // But we need to lock to check if we have one.
        if let Ok(mut guard) = self.socket.lock()
            && let Some(stream) = guard.as_mut()
        {
            // 1. Get Metadata (Level)
            let metadata = event.metadata();
            let level_color = match *metadata.level() {
                tracing::Level::ERROR => "ERROR".red(),
                tracing::Level::WARN => "WARN".yellow(),
                tracing::Level::INFO => "INFO".green(),
                tracing::Level::DEBUG => "DEBUG".blue(),
                tracing::Level::TRACE => "TRACE".magenta(),
            };

            // 2. Timestamp
            let timestamp = chrono::Local::now().format("%H:%M:%S").to_string().dimmed();

            // 3. Visitor with Scope support
            struct MessageVisitor {
                message: String,
                scope: Option<String>,
            }

            impl tracing::field::Visit for MessageVisitor {
                fn record_debug(
                    &mut self,
                    field: &tracing::field::Field,
                    value: &dyn std::fmt::Debug,
                ) {
                    if field.name() == "message" {
                        self.message.push_str(&format!("{:?}", value));
                    }
                }
                fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                    if field.name() == "message" {
                        self.message.push_str(value);
                    } else if field.name() == "scope" {
                        self.scope = Some(value.to_string());
                    }
                }
            }

            let mut visitor = MessageVisitor {
                message: String::new(),
                scope: None,
            };
            event.record(&mut visitor);

            if visitor.message.is_empty() {
                return;
            }

            // 4. Format: TIME [LEVEL] [SCOPE] Message
            let scope_part = if let Some(s) = visitor.scope {
                format!("[{}] ", s)
            } else {
                String::new()
            };

            // Strip tags for clean debug output
            let clean_message = strip_tags(&visitor.message);

            let final_msg = format!(
                "{} [{}] {}{}\n",
                timestamp, level_color, scope_part, clean_message
            );

            if stream.write_all(final_msg.as_bytes()).is_err() {
                *guard = None;
            }
        }
    }
}

// Helper to strip XML-like tags from log messages
fn strip_tags(msg: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < msg.len() {
        if let Some(start) = msg[i..].find('<') {
            result.push_str(&msg[i..i + start]);
            i += start;
            if let Some(end) = msg[i..].find('>') {
                i += end + 1;
            } else {
                result.push('<');
                i += 1;
            }
        } else {
            result.push_str(&msg[i..]);
            break;
        }
    }
    result
}

pub fn init_logging(force_debug: bool) -> Result<bool> {
    // Basic EnvFilter
    // Init LogTracer to bridge log crate events to tracing
    // Ignore error if already initialized (for tests/multiple calls safety)
    let _ = tracing_log::LogTracer::init();

    let socket_path = get_socket_path();
    let watcher_active = socket_path.exists();

    // Enable debug if flag is passed OR if the debug watcher is active/socket exists
    let enable_debug = force_debug || watcher_active;

    let env_filter = if enable_debug {
        // Force debug level, ignoring environment variables
        EnvFilter::new("debug")
    } else {
        // Use environment variable RUST_LOG, defaulting to info
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    let registry = tracing_subscriber::registry().with(env_filter);

    // Always attempt to connect to socket if it exists/is valid.
    let layer = SocketSubscriberLayer::new(&socket_path);
    // We use set_global_default instead of init() to avoid panics if called multiple times,
    // and because we manually initialized LogTracer above.
    // init() would try to init LogTracer again if the feature was enabled, causing a panic/error.
    let _ = tracing::subscriber::set_global_default(registry.with(layer));

    Ok(true)
}

pub fn spawn_debug_viewer() -> Result<()> {
    let socket_path = get_socket_path();

    // Check if socket connectable
    if UnixStream::connect(&socket_path).is_ok() {
        return Ok(());
    }

    // Remove stale socket file
    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    // Detect terminal
    let terminal = env::var("TERMINAL").ok().or_else(|| {
        let terminals = ["rio", "alacritty", "kitty", "gnome-terminal", "xterm"];
        for term in terminals {
            if which::which(term).is_ok() {
                return Some(term.to_string());
            }
        }
        None
    });

    if let Some(term) = terminal {
        debug!("Spawning debug viewer with: {}", term);
        let self_exe = env::current_exe().context("Failed to get current executable path")?;

        // Spawn terminal running internal-watch
        let _ = Command::new(&term)
            .arg("-e")
            .arg(&self_exe)
            .arg("internal-watch")
            .arg(&socket_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn debug terminal")?;

        println!("Debug Mode Started.");
        println!("Socket: {:?}", socket_path);

        // Wait for socket to appear (max 2s)
        let start = std::time::Instant::now();
        while !socket_path.exists() && start.elapsed() < Duration::from_secs(2) {
            std::thread::sleep(Duration::from_millis(50));
        }
        std::thread::sleep(Duration::from_millis(100)); // Grace period for bind
    } else {
        warn!("No supported terminal emulator found.");
        println!("Cannot spawn debug terminal.");
    }

    Ok(())
}

pub fn run_socket_watcher(socket_path: &Path) -> Result<()> {
    // Note: colors are provided by the sender now (like kitchnsink)
    // receiver is dumb and just prints.

    if socket_path.exists() {
        let _ = fs::remove_file(socket_path);
    }

    let listener = UnixListener::bind(socket_path).context("Failed to bind debug socket")?;

    println!(
        "{}",
        "Kitchn Debug Watcher (Socket Mode)".bold().underline()
    );
    println!("Listening on: {:?}\n", socket_path);

    // Accept connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Handle client in thread to allow concurrent clients
                std::thread::spawn(move || {
                    let reader = BufReader::new(stream);
                    for line in reader.lines() {
                        if let Ok(l) = line {
                            // Dumb print: just output what we get
                            // kitchnsink doesn't use println! because msg includes newline?
                            // sender sends with \n.
                            println!("{}", l);
                        } else {
                            break;
                        }
                    }
                });
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }

    Ok(())
}
