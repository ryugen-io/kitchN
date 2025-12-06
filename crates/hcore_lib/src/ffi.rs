use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr;

use crate::config::HyprConfig;
use crate::{logger, packager, processor};

/// Opaque context pointer (wraps HyprConfig with error storage)
pub struct HCoreContext {
    pub config: HyprConfig,
    pub last_error: RefCell<Option<String>>,
}

impl HCoreContext {
    fn set_error(&self, err: String) {
        *self.last_error.borrow_mut() = Some(err);
    }

    fn clear_error(&self) {
        *self.last_error.borrow_mut() = None;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hcore_context_new() -> *mut HCoreContext {
    match HyprConfig::load() {
        Ok(cfg) => {
            let ctx = Box::new(HCoreContext {
                config: cfg,
                last_error: RefCell::new(None),
            });
            Box::into_raw(ctx)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `HCoreContext` created by `hcore_context_new`.
pub unsafe extern "C" fn hcore_context_free(ctx: *mut HCoreContext) {
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
/// * `ctx` must be a valid pointer to `HCoreContext` created by `hcore_context_new`.
/// * `buffer` must be a valid pointer to a writable memory region of at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hcore_get_last_error(
    ctx: *mut HCoreContext,
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

#[unsafe(no_mangle)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `HCoreContext`.
/// * `level`, `scope`, and `msg` must be valid, null-terminated C strings.
pub unsafe extern "C" fn hcore_log(
    ctx: *mut HCoreContext,
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

        logger::log_to_terminal(&context.config, &level_str, &scope_str, &msg_str);

        if context.config.layout.logging.write_by_default {
            let _ = logger::log_to_file(&context.config, &level_str, &scope_str, &msg_str);
        }
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `HCoreContext`.
/// * `src_dir` and `out_file` must be valid, null-terminated C strings.
pub unsafe extern "C" fn hcore_pack(
    ctx: *mut HCoreContext,
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

#[unsafe(no_mangle)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `HCoreContext`.
/// * `pkg_file` and `target_dir` must be valid, null-terminated C strings.
pub unsafe extern "C" fn hcore_unpack(
    ctx: *mut HCoreContext,
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

#[unsafe(no_mangle)]
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// * `ctx` must be a valid pointer to `HCoreContext`.
/// * `path` must be a valid, null-terminated C string.
pub unsafe extern "C" fn hcore_install(ctx: *mut HCoreContext, path: *const c_char) -> c_int {
    if ctx.is_null() {
        return 1;
    }
    let context = unsafe { &*ctx };
    context.clear_error();

    unsafe {
        let p = CStr::from_ptr(path).to_string_lossy();

        match processor::install(Path::new(&*p), &context.config) {
            Ok(_) => 0,
            Err(e) => {
                context.set_error(format!("{:#}", e));
                1
            }
        }
    }
}
