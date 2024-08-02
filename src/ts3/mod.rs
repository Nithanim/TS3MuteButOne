pub mod ts3sdk;
pub mod plugin_static;
mod functions;


use core::ffi::c_int;
use std::ffi::{c_char, c_uint, CStr};
use std::ptr::{null, null_mut};
use std::{ptr, slice, thread};
use std::alloc::Layout;
use std::time::Duration;
use ts3sdk::{anyID, ConnectStatus, ConnectStatus_STATUS_CONNECTION_ESTABLISHED, uint64};
use crate::ts3::ts3sdk::{ClientProperties_CLIENT_UNIQUE_IDENTIFIER, Ts3ErrorType, Ts3ErrorType_ERROR_ok};
use crate::util::{log_error, log_info, ts3_get_error_message};


type ServerConnectionHandlerID = uint64;
type ClientId = anyID;
type ChannelId = uint64;

static mut TS3FUNCTIONS: Option<ts3sdk::TS3Functions> = None;

static TARGET_UNIQUE_ID: &str = "F05J9/DWIVowPTds0a/BlSa7rHI=";

pub fn get_ts3_functions() -> ts3sdk::TS3Functions {
    unsafe { TS3FUNCTIONS.unwrap() }
}


#[allow(unused)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[no_mangle]
pub extern "system" fn ts3plugin_onConnectStatusChangeEvent(serverConnectionHandlerID: uint64, newStatus: c_int, errorNumber: c_uint) {
    match newStatus as ConnectStatus {
        ConnectStatus_STATUS_CONNECTION_ESTABLISHED => {}
        _ => {}
    }
}


#[allow(unused)]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn ts3plugin_onClientMoveEvent(serverConnectionHandlerID: uint64, clientID: anyID, oldChannelID: uint64, newChannelID: uint64, visibility: c_int, moveMessage: *const c_char) {
    log_info("----------------------------------");
    log_info(format!("Client {} on conn {} moved from channel {} to {}. Vis: {}", clientID, serverConnectionHandlerID, oldChannelID, newChannelID, visibility));

    let my_client_id = get_own_client_id(serverConnectionHandlerID).unwrap();
    log_info(format!("My client id: {}", my_client_id));

    let my_channel_id = match get_client_channel(serverConnectionHandlerID, my_client_id) {
        None => {
            // When the TS3 client disconnects it also triggers a "client move" event.
            // So we land here, but it is too late to fetch the channel of the client because we already disconnected.
            // So if fetching the channel fails, assume we disconnected.
            // At least until we have a better solution.
            return;
        }
        Some(c) => c
    };

    log_info(format!("My channel id: {}", my_channel_id));

    if clientID == my_client_id {
        log_info("We moved...");

        let clients = get_client_channel_list(serverConnectionHandlerID, my_channel_id).unwrap();
        log_info(format!("There are {} clients in the channel (including us).", clients.len()));

        let mut clients_to_mute = Vec::new();
        for client in clients {
            if client == my_client_id {
                continue;
            }
            let uid = get_uid(serverConnectionHandlerID, client);
            match uid {
                None => {}
                Some(uid) => if uid == TARGET_UNIQUE_ID {
                    log_info(format!("Ignoring client {} because it is our target.", client));
                } else {
                    log_info(format!("Setting client {} on list to mute next.", client));
                    clients_to_mute.push(client);
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }

        log_info("Muting clients now...");
        mute_clients(serverConnectionHandlerID, &clients_to_mute);
    } else {
        log_info("Someone else moved...");

        if my_channel_id == newChannelID {
            log_info("They are in our channel!");

            match get_uid(serverConnectionHandlerID, clientID) {
                None => {}
                Some(uid) => if uid == TARGET_UNIQUE_ID {
                    log_info("Its our target, NOT muting!");
                } else {
                    mute_clients(serverConnectionHandlerID, &vec![clientID]);
                }
            }
        }
    }
}

fn mute_clients(connection_id: ServerConnectionHandlerID, client_ids: &Vec<ClientId>) {
    let error_code = unsafe {
        let client_ids_arrays = alloc_zero_terminated_array(client_ids);

        // TODO use error codes (see official example C plugin; search "returnCode")
        get_ts3_functions().requestMuteClients.unwrap()(connection_id, client_ids_arrays, null())
    };
    if error_code as Ts3ErrorType != Ts3ErrorType_ERROR_ok {
        log_error(format!("Unable to mute clients! error {}: {}", error_code, ts3_get_error_message(error_code)));
    }
}
fn get_uid(connection_id: ServerConnectionHandlerID, client_id: ClientId) -> Option<String> {
    let mut pointer: *mut c_char = null_mut();
    let error_code = unsafe {
        get_ts3_functions().getClientVariableAsString.unwrap()(connection_id, client_id, ClientProperties_CLIENT_UNIQUE_IDENTIFIER as usize, &mut pointer)
    };
    if error_code as Ts3ErrorType != Ts3ErrorType_ERROR_ok {
        log_error(format!("Unable to get channel id of client {} on server {}!", client_id, connection_id));
        return None;
    } else {
        if pointer == null_mut() {
            log_error("Got a NULL as string as UID?");
            None
        } else {
            let maybe = unsafe {
                CStr::from_ptr(pointer).to_str()
            };
            match maybe {
                Ok(str) => Some(str.to_owned()),
                Err(_) => {
                    log_error("Cannot convert string to UID");
                    None
                }
            }
        }
    }
}

fn alloc_zero_terminated_array<T>(data: &Vec<T>) -> *mut T {
    unsafe {
        let raw: *mut T = std::alloc::alloc(Layout::array::<T>(data.len() + 1).unwrap()) as *mut T;
        ptr::copy::<T>(data.as_ptr(), raw, data.len());
        ptr::write_bytes(raw.add(data.len()), 0, 1);
        raw
    }
}

fn get_client_channel_list(connection_id: ServerConnectionHandlerID, channel_id: ChannelId) -> Option<Vec<ClientId>> {
    let mut client_ids_pointer: *mut ClientId = null_mut();
    let error_code = unsafe {
        get_ts3_functions().getChannelClientList.unwrap()(connection_id, channel_id, &mut client_ids_pointer)
    };
    if error_code as Ts3ErrorType != Ts3ErrorType_ERROR_ok {
        log_error(format!("Unable to get clients of channel {} on server {}!", channel_id, connection_id));
        return None;
    } else {
        let len = count_zero_terminated_array(client_ids_pointer);
        let parts: &[ClientId] = unsafe { slice::from_raw_parts(client_ids_pointer, len) };
        let boxed: Box<[ClientId]> = Box::from(parts);
        let vec = boxed.into_vec();
        Some(vec)
    }
}

fn count_zero_terminated_array(arr: *mut ClientId) -> usize {
    let mut len = 0;
    let mut p = arr;
    unsafe {
        while *p != 0 {
            p = p.add(1);
            len += 1;
        }
    }
    len
}

fn get_client_channel(connection_id: ServerConnectionHandlerID, client_id: ClientId) -> Option<ChannelId> {
    let mut channel_id: ChannelId = 0;
    let error_code = unsafe {
        get_ts3_functions().getChannelOfClient.unwrap()(connection_id, client_id, &mut channel_id)
    };
    if error_code as Ts3ErrorType != Ts3ErrorType_ERROR_ok {
        log_error(format!("Unable to get channel id of client {} on server {}!", client_id, connection_id));
        return None;
    } else {
        Some(channel_id)
    }
}

fn get_own_client_id(connection_id: uint64) -> Option<ClientId> {
    let mut my_client_id: ClientId = 0;
    let error_code = unsafe {
        get_ts3_functions().getClientID.unwrap()(connection_id, &mut my_client_id)
    };
    if error_code as Ts3ErrorType != Ts3ErrorType_ERROR_ok {
        log_error(format!("Unable to get own client id!"));
        return None;
    } else {
        Some(my_client_id)
    }
}
