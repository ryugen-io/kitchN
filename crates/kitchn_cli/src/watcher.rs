//! Debug log watcher utilities

use colored::{ColoredString, Colorize};

/// Determines the log level from a line and returns a colorized version.
/// - ERROR -> Red
/// - WARN -> Yellow
/// - INFO -> Cyan
/// - DEBUG -> Blue
/// - Other -> Normal
pub fn colorize_line(line: &str) -> ColoredString {
    let lower = line.to_lowercase();

    // Check content first for more specific coloring
    if lower.contains("error") || lower.contains("fail") || lower.contains("stderr") {
        line.red().bold()
    } else if lower.contains("warn") {
        line.yellow()
    } else if lower.contains("success") || lower.contains("ok") || lower.contains("stdout") {
        // "stdout" in debug logs usually means we are printing output, which is good info
        line.green()
    } else if lower.contains("info") {
        line.cyan()
    } else if line.contains("DEBUG") || line.contains("[DEBUG]") {
        // Fallback for generic debug messages
        line.blue().dimmed()
    } else {
        line.normal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use colored::Color;

    #[test]
    fn test_colorize_error() {
        let result = colorize_line("[ERROR] Something went wrong");
        assert_eq!(result.fgcolor, Some(Color::Red));
    }

    #[test]
    fn test_colorize_warn() {
        let result = colorize_line("[WARN] Heads up");
        assert_eq!(result.fgcolor, Some(Color::Yellow));
    }

    #[test]
    fn test_colorize_info() {
        let result = colorize_line("[INFO] It's fine");
        assert_eq!(result.fgcolor, Some(Color::Cyan));
    }

    #[test]
    fn test_colorize_debug() {
        let result = colorize_line("[DEBUG] nitty gritty");
        assert_eq!(result.fgcolor, Some(Color::Blue));
    }

    #[test]
    fn test_colorize_normal() {
        let result = colorize_line("Just a plain line");
        assert_eq!(result.fgcolor, None);
    }
}
