use piratetok_live_rs::errors::TikTokLiveError;

use crate::ffi_str::FfiError;
use crate::last_error;

pub struct Code;

impl Code {
    pub const OK: i32 = 0;
    pub const BAD_PARAM: i32 = 1;
    pub const USER_NOT_FOUND: i32 = 2;
    pub const HOST_NOT_ONLINE: i32 = 3;
    pub const DEVICE_BLOCKED: i32 = 4;
    pub const AGE_RESTRICTED: i32 = 5;
    pub const INVALID_RESPONSE: i32 = 6;
    pub const CONNECTION_CLOSED: i32 = 7;
    pub const HTTP: i32 = 8;
    pub const WEBSOCKET: i32 = 9;
    pub const ALREADY_RUNNING: i32 = 10;
    pub const SESSION_REQUIRED: i32 = 11;
    pub const INTERNAL: i32 = 99;
}

pub fn of_live(e: &TikTokLiveError) -> i32 {
    match e {
        TikTokLiveError::UserNotFound(..) => Code::USER_NOT_FOUND,
        TikTokLiveError::HostNotOnline(..) => Code::HOST_NOT_ONLINE,
        TikTokLiveError::DeviceBlocked => Code::DEVICE_BLOCKED,
        TikTokLiveError::AgeRestricted(..) => Code::AGE_RESTRICTED,
        TikTokLiveError::SessionRequired(..) => Code::SESSION_REQUIRED,
        TikTokLiveError::InvalidResponse(..) => Code::INVALID_RESPONSE,
        TikTokLiveError::ConnectionClosed => Code::CONNECTION_CLOSED,
        TikTokLiveError::Http(..) => Code::HTTP,
        TikTokLiveError::WebSocket(..) => Code::WEBSOCKET,
        other => {
            last_error::set(&other.to_string());
            Code::INTERNAL
        }
    }
}

pub fn fail_live(e: &TikTokLiveError) -> i32 {
    last_error::set(&e.to_string());
    of_live(e)
}

pub fn fail_ffi(e: &FfiError) -> i32 {
    last_error::set(&e.to_string());
    match e {
        FfiError::NullPointer | FfiError::Utf8(..) | FfiError::Nul(..) => Code::BAD_PARAM,
        FfiError::Json(..) => Code::INTERNAL,
    }
}
