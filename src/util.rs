use std::ffi::{c_char, c_void, CStr, CString};
use std::os::raw::c_uint;
use std::ptr::null_mut;
use crate::ts3::get_ts3_functions;
use crate::ts3::plugin_static::ts3plugin_name;
use crate::ts3::ts3sdk::{LogLevel, LogLevel_LogLevel_DEBUG, LogLevel_LogLevel_ERROR, LogLevel_LogLevel_INFO, Ts3ErrorType_ERROR_ok, uint64, VirtualServerProperties_VIRTUALSERVER_NAME};

pub fn log_info(msg: impl AsRef<str>) {
    log(msg, LogLevel_LogLevel_DEBUG);
}

pub fn log_debug(msg: impl AsRef<str>) {
    log(msg, LogLevel_LogLevel_INFO);
}

pub fn log_error(msg: impl AsRef<str>) {
    log(msg, LogLevel_LogLevel_ERROR);
}

pub fn log(msg: impl AsRef<str>, log_level: LogLevel) {
    let cstr = CString::new(msg.as_ref()).unwrap();

    let f = crate::ts3::get_ts3_functions().logMessage.unwrap();
    unsafe {
        let x: *const c_char = ts3plugin_name();
        f(cstr.as_ptr(), log_level, x, 1);
    }
}

pub fn ts3_get_error_message(error_code: c_uint) -> String {
    let mut server_name_raw: *mut c_char = null_mut();
    let get_error_message = get_ts3_functions().getErrorMessage.unwrap();
    let ret = unsafe { get_error_message(error_code, &mut server_name_raw) };

    if ret != Ts3ErrorType_ERROR_ok {
        log_error(format!("Unable to query server error!"));
        StringWrapper::new(server_name_raw).into()
    } else {
        String::from("<UNKNOWN>")
    }
}


pub struct StringWrapper {
    pointer: *mut c_char,
}

impl StringWrapper {
    pub fn new(pointer: *mut c_char) -> Self {
        Self { pointer }
    }
}

impl Drop for StringWrapper {
    fn drop(&mut self) {
        unsafe {
            ts3_free_char_pointer(self.pointer)
        }
    }
}

impl Into<String> for StringWrapper {
    fn into(mut self) -> String {
        c_char_to_string(self.pointer)
    }
}


fn c_char_to_string(char_pointer: *mut c_char) -> String {
    unsafe {
        String::from(CStr::from_ptr(char_pointer).to_owned().to_str().unwrap())
    }
}

fn ts3_free_char_pointer(char_pointer: *mut c_char) {
    unsafe {
        get_ts3_functions().freeMemory.unwrap()(char_pointer as *mut c_void);
    }
}

