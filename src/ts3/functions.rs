use std::ffi::{c_char, c_void};
use std::os::raw::{c_int, c_uint};
use std::ptr::null_mut;
use crate::ts3::get_ts3_functions;
use crate::ts3::ts3sdk::{ConnectStatus_STATUS_CONNECTION_ESTABLISHED, Ts3ErrorType_ERROR_ok, uint64, VirtualServerProperties_VIRTUALSERVER_NAME};
use crate::util::{StringWrapper, ts3_get_error_message};

pub fn get_established_server_connections() -> Result<Vec<uint64>, Ts3Error> {
    let all = get_all_server_connection_ids();

    match all {
        Err(e) => Err(e),
        Ok(ok) => {
            Ok(ok.into_iter().filter(|e| {
                match get_server_connection_status(*e) {
                    Ok(status) => status == ConnectStatus_STATUS_CONNECTION_ESTABLISHED as i32,
                    Err(e) => false
                }
            }).collect())
        }
    }
}

pub fn get_all_server_connection_ids() -> Result<Vec<uint64>, Ts3Error> {
    let mut connection_ids_pointer: *mut uint64 = null_mut();

    let error_code = unsafe {
        get_ts3_functions().getServerConnectionHandlerList.unwrap()(&mut connection_ids_pointer)
    };

    if error_code != Ts3ErrorType_ERROR_ok {
        Err(to_ts3_error(error_code))
    } else {
        let mut connection_ids: Vec<uint64> = Vec::new();

        let mut i = 0;
        loop {
            let next = unsafe { *connection_ids_pointer.add(i) };

            if next == 0 {
                break;
            } else {
                i += 1;
            }

            connection_ids.push(next);
        }

        unsafe {
            get_ts3_functions().freeMemory.unwrap()(connection_ids_pointer as *mut c_void);
        }

        Ok(connection_ids)
    }
}

pub fn get_server_connection_status(connection_id: uint64) -> Result<i32, Ts3Error> {
    let mut status: c_int = 0;
    let error_code = unsafe {
        get_ts3_functions().getConnectionStatus.unwrap()(connection_id, &mut status)
    };


    if error_code != Ts3ErrorType_ERROR_ok {
        Err(to_ts3_error(error_code))
    } else {
        Ok(status)
    }
}

pub fn get_server_name(connection_id: uint64) -> Result<String, Ts3Error> {
    let mut server_name_raw: *mut c_char = null_mut();
    let get_server_variable = get_ts3_functions().getServerVariableAsString.unwrap();
    let error_code = unsafe {
        get_server_variable(connection_id, VirtualServerProperties_VIRTUALSERVER_NAME as usize, &mut server_name_raw)
    };


    if error_code != Ts3ErrorType_ERROR_ok {
        Err(to_ts3_error(error_code))
    } else {
        Ok(StringWrapper::new(server_name_raw).into())
    }
}

fn to_ts3_error(error_code: c_uint) -> Ts3Error {
    Ts3Error {
        code: error_code,
        message: ts3_get_error_message(error_code),
    }
}

pub struct Ts3Error {
    pub code: u32,
    pub message: String,
}