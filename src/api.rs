use std::ffi::c_char;

use piratetok_live_rs::http::api::{
    fetch_room_audience, fetch_room_id, fetch_room_info, AnchorId, FetchParams,
};

use crate::codes::{fail_ffi, fail_live, Code};
use crate::ffi_str::{optional_str, required_str, write_out, CStrArg, FfiError};
use crate::runtime::PirateTokRuntime;

#[no_mangle]
pub unsafe extern "C" fn piratetok_check_online(
    rt: *mut PirateTokRuntime,
    username: *const c_char,
    out_room_id: *mut *mut c_char,
    out_anchor_id: *mut *mut c_char,
) -> i32 {
    let Some(runtime) = rt.as_ref() else {
        return fail_ffi(&FfiError::NullPointer);
    };
    let name = match required_str(username) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!(error = %e, "check_online: bad username");
            return fail_ffi(&e);
        }
    };
    let room = match runtime
        .rt
        .block_on(fetch_room_id(name, FetchParams::default()))
    {
        Ok(room) => room,
        Err(e) => {
            tracing::warn!(error = %e, "check_online failed");
            return fail_live(&e);
        }
    };
    let written = write_out(out_room_id, &room.room_id)
        .and_then(|()| write_optional_out(out_anchor_id, &room.anchor_id));
    match written {
        Ok(()) => Code::OK,
        Err(e) => {
            tracing::warn!(error = %e, "check_online: output");
            fail_ffi(&e)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_fetch_room_info(
    rt: *mut PirateTokRuntime,
    room_id: *const c_char,
    cookies: *const c_char,
    out_json: *mut *mut c_char,
) -> i32 {
    let Some(runtime) = rt.as_ref() else {
        return fail_ffi(&FfiError::NullPointer);
    };
    let args = match (required_str(room_id), optional_str(cookies)) {
        (Ok(room), Ok(cookies)) => (room, cookies),
        (Err(e), Ok(..)) | (Ok(..), Err(e)) | (Err(e), Err(..)) => {
            tracing::warn!(error = %e, "fetch_room_info: bad argument");
            return fail_ffi(&e);
        }
    };
    let params = FetchParams {
        cookies: as_option(args.1),
        ..Default::default()
    };
    match runtime.rt.block_on(fetch_room_info(args.0, params)) {
        Ok(info) => emit_json(out_json, serde_json::to_string(&info)),
        Err(e) => {
            tracing::warn!(error = %e, "fetch_room_info failed");
            fail_live(&e)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_fetch_room_audience(
    rt: *mut PirateTokRuntime,
    room_id: *const c_char,
    anchor_id: *const c_char,
    cookies: *const c_char,
    out_json: *mut *mut c_char,
) -> i32 {
    let Some(runtime) = rt.as_ref() else {
        return fail_ffi(&FfiError::NullPointer);
    };
    let parsed = required_str(room_id)
        .and_then(|room| Ok((room, optional_str(anchor_id)?, optional_str(cookies)?)));
    let (room, anchor, cookie_arg) = match parsed {
        Ok(parsed) => parsed,
        Err(e) => {
            tracing::warn!(error = %e, "fetch_room_audience: bad argument");
            return fail_ffi(&e);
        }
    };
    let params = FetchParams {
        cookies: as_option(cookie_arg),
        ..Default::default()
    };
    match runtime
        .rt
        .block_on(fetch_room_audience(room, to_anchor(anchor), params))
    {
        Ok(audience) => emit_json(out_json, serde_json::to_string(&audience)),
        Err(e) => {
            tracing::warn!(error = %e, "fetch_room_audience failed");
            fail_live(&e)
        }
    }
}

fn to_anchor(arg: CStrArg<'_>) -> AnchorId<'_> {
    match arg {
        CStrArg::Given(id) => AnchorId::Known(id),
        CStrArg::Absent => AnchorId::FromRoomInfo,
    }
}

fn as_option(arg: CStrArg<'_>) -> Option<&str> {
    match arg {
        CStrArg::Given(value) => Some(value),
        CStrArg::Absent => None,
    }
}

unsafe fn write_optional_out(out: *mut *mut c_char, value: &str) -> Result<(), FfiError> {
    if out.is_null() {
        return Ok(());
    }
    write_out(out, value)
}

unsafe fn emit_json(out_json: *mut *mut c_char, json: Result<String, serde_json::Error>) -> i32 {
    let written = json
        .map_err(FfiError::Json)
        .and_then(|text| write_out(out_json, &text));
    match written {
        Ok(()) => Code::OK,
        Err(e) => {
            tracing::warn!(error = %e, "json output");
            fail_ffi(&e)
        }
    }
}
