use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr;

use kitchn_lib::config::Cookbook;
use kitchn_lib::{ingredient, logger, packager, processor};

/// Opaque context pointer (wraps Cookbook with error storage)
pub struct KitchnContext {
    pub config: Cookbook,
    pub last_error: RefCell<Option<String>>,
    pub app_name: RefCell<Option<String>>,
}

impl KitchnContext {
    fn set_error(&self, err: String) {
        *self.last_error.borrow_mut() = Some(err);
    }

    fn clear_error(&self) {
        *self.last_error.borrow_mut() = None;
    }
}

#[no_mangle]
pub extern "C" fn kitchn_context_new() -> *mut KitchnContext {
    match Cookbook::load() {
        Ok(cfg) => {
            let ctx = Box::new(KitchnContext {
                config: cfg,
                last_error: RefCell::new(None),
                app_name: RefCell::new(None),
            });
            Box::into_raw(ctx)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `KitchnContext` created by `kitchn_context_new`.
pub unsafe extern "C" fn kitchn_context_free(ctx: *mut KitchnContext) {
    if !ctx.is_null() {
        unsafe {
            drop(Box::from_raw(ctx));
        }
    }
}

/// Copies the last error message into the provided buffer.
/// Returns the number of bytes written (excluding null terminator),
/// or -1 if the buffer was too small or no error exists.
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `KitchnContext` created by `kitchn_context_new`.
/// * `buffer` must be a valid pointer to a writable memory region of at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn kitchn_get_last_error(
    ctx: *mut KitchnContext,
    buffer: *mut c_char,
    len: usize,
) -> c_int {
    if ctx.is_null() || buffer.is_null() {
        return -1;
    }

    let context = unsafe { &*ctx };
    let borrow = context.last_error.borrow();

    if let Some(msg) = &*borrow {
        let bytes = msg.as_bytes();
        if bytes.len() >= len {
            return -1; // Buffer too small
        }

        unsafe {
            // Copy bytes
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
            // Null terminate
            *buffer.add(bytes.len()) = 0;
        }

        return bytes.len() as c_int;
    }

    0 // No error
}

#[no_mangle]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `KitchnContext`.
/// * `name` must be a valid, null-terminated C string.
pub unsafe extern "C" fn kitchn_context_set_app_name(ctx: *mut KitchnContext, name: *const c_char) {
    if !ctx.is_null() && !name.is_null() {
        let context = &*ctx;
        let s = CStr::from_ptr(name).to_string_lossy();
        *context.app_name.borrow_mut() = Some(s.into_owned());
    }
}

#[no_mangle]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `KitchnContext`.
/// * `level`, `scope`, and `msg` must be valid, null-terminated C strings.
pub unsafe extern "C" fn kitchn_log(
    ctx: *mut KitchnContext,
    level: *const c_char,
    scope: *const c_char,
    msg: *const c_char,
) {
    if ctx.is_null() {
        return;
    }

    unsafe {
        let context = &*ctx;
        let level_str = CStr::from_ptr(level).to_string_lossy();
        let scope_str = CStr::from_ptr(scope).to_string_lossy();
        let msg_str = CStr::from_ptr(msg).to_string_lossy();
        let app = context.app_name.borrow();

        logger::log_to_terminal(&context.config, &level_str, &scope_str, &msg_str);

        if context.config.layout.logging.write_by_default {
            let _ = logger::log_to_file(
                &context.config,
                &level_str,
                &scope_str,
                &msg_str,
                app.as_deref(),
            );
        }
    }
}

#[no_mangle]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `KitchnContext`.
/// * `preset_key` must be a valid, null-terminated C string.
/// * `msg_override` can be null. If not null, must be valid, null-terminated C string.
pub unsafe extern "C" fn kitchn_log_preset(
    ctx: *mut KitchnContext,
    preset_key: *const c_char,
    msg_override: *const c_char,
) -> c_int {
    if ctx.is_null() || preset_key.is_null() {
        return 1;
    }

    let context = unsafe { &*ctx };
    context.clear_error();

    let key = unsafe { CStr::from_ptr(preset_key).to_string_lossy() };

    let preset = match context.config.dictionary.presets.get(key.as_ref()) {
        Some(p) => p,
        None => {
            context.set_error(format!("Preset '{}' not found", key));
            return 1;
        }
    };

    let level = &preset.level;
    let scope = preset.scope.as_deref().unwrap_or("");

    let msg_default = &preset.msg;
    let msg_final = if !msg_override.is_null() {
        unsafe { CStr::from_ptr(msg_override).to_string_lossy() }
    } else {
        std::borrow::Cow::Borrowed(msg_default.as_str())
    };

    let app = context.app_name.borrow();

    logger::log_to_terminal(&context.config, level, scope, &msg_final);

    if context.config.layout.logging.write_by_default {
        let _ = logger::log_to_file(&context.config, level, scope, &msg_final, app.as_deref());
    }

    0 // Success
}

#[no_mangle]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `KitchnContext`.
/// * `src_dir` and `out_file` must be valid, null-terminated C strings.
pub unsafe extern "C" fn kitchn_pack(
    ctx: *mut KitchnContext,
    src_dir: *const c_char,
    out_file: *const c_char,
) -> c_int {
    if ctx.is_null() {
        return 1;
    }
    let context = unsafe { &*ctx };
    context.clear_error();

    unsafe {
        let src = CStr::from_ptr(src_dir).to_string_lossy();
        let out = CStr::from_ptr(out_file).to_string_lossy();

        match packager::pack(Path::new(&*src), Path::new(&*out)) {
            Ok(_) => 0,
            Err(e) => {
                context.set_error(format!("{:#}", e));
                1
            }
        }
    }
}

#[no_mangle]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `KitchnContext`.
/// * `pkg_file` and `target_dir` must be valid, null-terminated C strings.
pub unsafe extern "C" fn kitchn_unpack(
    ctx: *mut KitchnContext,
    pkg_file: *const c_char,
    target_dir: *const c_char,
) -> c_int {
    if ctx.is_null() {
        return 1;
    }
    let context = unsafe { &*ctx };
    context.clear_error();

    unsafe {
        let pkg = CStr::from_ptr(pkg_file).to_string_lossy();
        let target = CStr::from_ptr(target_dir).to_string_lossy();

        match packager::unpack(Path::new(&*pkg), Path::new(&*target)) {
            Ok(_) => 0,
            Err(e) => {
                context.set_error(format!("{:#}", e));
                1
            }
        }
    }
}

#[no_mangle]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `KitchnContext`.
/// * `path` must be a valid, null-terminated C string.
pub unsafe extern "C" fn kitchn_store(ctx: *mut KitchnContext, path: *const c_char) -> c_int {
    if ctx.is_null() {
        return 1;
    }
    let context = unsafe { &*ctx };
    context.clear_error();

    let p = unsafe { CStr::from_ptr(path).to_string_lossy() };
    match std::fs::read_to_string(Path::new(&*p)) {
        Ok(content) => match toml::from_str::<ingredient::Ingredient>(&content) {
            Ok(pkg) => match processor::apply(&pkg, &context.config) {
                Ok(_) => 0,
                Err(e) => {
                    context.set_error(format!("Apply error: {:#}", e));
                    1
                }
            },
            Err(e) => {
                context.set_error(format!("Parse error: {:#}", e));
                1
            }
        },
        Err(e) => {
            context.set_error(format!("File read error: {:#}", e));
            1
        }
    }
}
