use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::ptr;

use k_lib::config::Cookbook;
use k_lib::db::Pantry;
use k_lib::{ingredient, logger, packager, processor};

// --- Contexts ---

/// Opaque context pointer (wraps Cookbook with error storage)
pub struct KitchnContext {
    pub config: Cookbook,
    pub last_error: RefCell<Option<String>>,
    pub app_name: RefCell<Option<String>>,
}

/// Opaque pantry wrapper
pub struct KitchnPantry {
    pub inner: RefCell<Pantry>,
    pub last_error: RefCell<Option<String>>,
}

macro_rules! impl_error_handling {
    ($struct_name:ident) => {
        impl $struct_name {
            fn set_error(&self, err: String) {
                *self.last_error.borrow_mut() = Some(err);
            }

            fn clear_error(&self) {
                *self.last_error.borrow_mut() = None;
            }
        }
    };
}

impl_error_handling!(KitchnContext);
impl_error_handling!(KitchnPantry);

// --- KitchnContext API ---

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
pub unsafe extern "C" fn kitchn_context_free(ctx: *mut KitchnContext) {
    if !ctx.is_null() {
        unsafe {
            drop(Box::from_raw(ctx));
        }
    }
}

#[no_mangle]
/// # Safety
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
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
            *buffer.add(bytes.len()) = 0;
        }

        return bytes.len() as c_int;
    }

    0 // No error
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn kitchn_context_set_app_name(ctx: *mut KitchnContext, name: *const c_char) {
    if !ctx.is_null() && !name.is_null() {
        let context = &*ctx;
        let s = CStr::from_ptr(name).to_string_lossy();
        *context.app_name.borrow_mut() = Some(s.into_owned());
    }
}

#[no_mangle]
/// # Safety
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

    0
}

#[no_mangle]
/// # Safety
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
/// Cooks/Applies an ingredient file immediately to the current state/config context.
/// # Safety
pub unsafe extern "C" fn kitchn_cook_file(ctx: *mut KitchnContext, path: *const c_char) -> c_int {
    if ctx.is_null() {
        return 1;
    }
    let context = unsafe { &*ctx };
    context.clear_error();

    let p = unsafe { CStr::from_ptr(path).to_string_lossy() };
    match std::fs::read_to_string(Path::new(&*p)) {
        Ok(content) => match toml::from_str::<ingredient::Ingredient>(&content) {
            Ok(pkg) => match processor::apply(&pkg, &context.config, false) {
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

// --- KitchnPantry API ---

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn kitchn_pantry_load(path: *const c_char) -> *mut KitchnPantry {
    let p_str = if !path.is_null() {
        unsafe { CStr::from_ptr(path).to_string_lossy() }
    } else {
        return ptr::null_mut();
    };

    let p_path = PathBuf::from(p_str.as_ref());

    match Pantry::load(&p_path) {
        Ok(p) => {
            let ctx = Box::new(KitchnPantry {
                inner: RefCell::new(p),
                last_error: RefCell::new(None),
            });
            Box::into_raw(ctx)
        }
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn kitchn_pantry_free(pantry: *mut KitchnPantry) {
    if !pantry.is_null() {
        unsafe {
            drop(Box::from_raw(pantry));
        }
    }
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn kitchn_pantry_get_last_error(
    pantry: *mut KitchnPantry,
    buffer: *mut c_char,
    len: usize,
) -> c_int {
    if pantry.is_null() || buffer.is_null() {
        return -1;
    }

    let p = unsafe { &*pantry };
    let borrow = p.last_error.borrow();

    if let Some(msg) = &*borrow {
        let bytes = msg.as_bytes();
        if bytes.len() >= len {
            return -1;
        }

        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
            *buffer.add(bytes.len()) = 0;
        }

        return bytes.len() as c_int;
    }

    0
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn kitchn_pantry_save(pantry: *mut KitchnPantry) -> c_int {
    if pantry.is_null() {
        return 1;
    }
    let p = unsafe { &*pantry };
    p.clear_error();

    match p.inner.borrow().save() {
        Ok(_) => 0,
        Err(e) => {
            p.set_error(format!("{:#}", e));
            1
        }
    }
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn kitchn_pantry_add_toml(
    pantry: *mut KitchnPantry,
    toml_content: *const c_char,
) -> c_int {
    if pantry.is_null() || toml_content.is_null() {
        return 1;
    }
    let p = unsafe { &*pantry };
    p.clear_error();

    let content = unsafe { CStr::from_ptr(toml_content).to_string_lossy() };
    match toml::from_str::<ingredient::Ingredient>(&content) {
        Ok(ing) => match p.inner.borrow_mut().store(ing) {
            Ok(_) => 0,
            Err(e) => {
                p.set_error(format!("Store error: {:#}", e));
                1
            }
        },
        Err(e) => {
            p.set_error(format!("Parse error: {:#}", e));
            1
        }
    }
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn kitchn_pantry_remove(
    pantry: *mut KitchnPantry,
    name: *const c_char,
) -> c_int {
    if pantry.is_null() || name.is_null() {
        return 1;
    }
    let p = unsafe { &*pantry };
    p.clear_error();

    let n = unsafe { CStr::from_ptr(name).to_string_lossy() };
    p.inner.borrow_mut().discard(&n);
    0
}

#[no_mangle]
/// # Safety
pub unsafe extern "C" fn kitchn_pantry_count(pantry: *mut KitchnPantry) -> c_int {
    if pantry.is_null() {
        return -1;
    }
    let p = unsafe { &*pantry };
    p.inner.borrow().list().len() as c_int
}
