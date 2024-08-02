use std::ffi::{c_char, c_int};
use c_str_macro::c_str;
use crate::ts3::{TS3FUNCTIONS, ts3sdk};
use crate::util::log_info;

#[no_mangle]
pub extern "system" fn ts3plugin_name() -> *const c_char {
    c_str!("MuteButOne").as_ptr()
}

#[no_mangle]
pub extern "system" fn ts3plugin_version() -> *const c_char {
    c_str!("0.0.1").as_ptr()
}

/* Plugin API version. Must be the same as the clients API major version, else the plugin fails to load. */
#[no_mangle]
pub extern "system" fn ts3plugin_apiVersion() -> c_int {
    return 26;
}

#[no_mangle]
pub extern "system" fn ts3plugin_author() -> *const c_char {
    c_str!("0.0.1").as_ptr()
}


#[no_mangle]
pub extern "system" fn ts3plugin_description() -> *const c_char {
    c_str!("Magic...").as_ptr()
}


#[no_mangle]
pub extern "system" fn ts3plugin_init() -> c_int {
    // 0 = success, 1 = failure
    log_info("Init plugin");
    return 0;
}


#[no_mangle]
pub extern "system" fn ts3plugin_shutdown() {
    log_info("Un-init plugin");
}

#[no_mangle]
pub extern "system" fn ts3plugin_setFunctionPointers(f: ts3sdk::TS3Functions) {
    unsafe { TS3FUNCTIONS = Some(f); }
}
