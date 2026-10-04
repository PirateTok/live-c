use std::ffi::c_char;
use std::sync::Arc;
use std::time::Duration;

use piratetok_live_rs::structs::config::{CdnEndpoint, TikTokLiveConfig};
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::ffi_str::{optional_str, required_str, CStrArg, FfiError};
use crate::last_error;
use crate::runtime::PirateTokRuntime;

pub struct PirateTokClient {
    pub runtime: *const PirateTokRuntime,
    pub config: TikTokLiveConfig,
    pub task: Option<JoinHandle<()>>,
    pub stop: Arc<Notify>,
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_new(
    rt: *mut PirateTokRuntime,
    username: *const c_char,
) -> *mut PirateTokClient {
    if rt.is_null() {
        last_error::set("runtime is null");
        return std::ptr::null_mut();
    }
    let name = match required_str(username) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!(error = %e, "piratetok_client_new: bad username");
            last_error::set(&format!("username: {e}"));
            return std::ptr::null_mut();
        }
    };
    Box::into_raw(Box::new(PirateTokClient {
        runtime: rt.cast_const(),
        config: TikTokLiveConfig::new(name),
        task: None,
        stop: Arc::new(Notify::new()),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_free(client: *mut PirateTokClient) {
    if client.is_null() {
        return;
    }
    let owned: Box<PirateTokClient> = Box::from_raw(client);
    for task in owned.task.iter() {
        task.abort();
    }
    drop(owned);
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_cdn(c: *mut PirateTokClient, cdn: i32) {
    let Some(client) = c.as_mut() else { return };
    client.config.cdn = match cdn {
        1 => CdnEndpoint::Eu,
        2 => CdnEndpoint::Us,
        _ => CdnEndpoint::Global,
    };
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_timeout_ms(c: *mut PirateTokClient, ms: u32) {
    let Some(client) = c.as_mut() else { return };
    client.config.timeout = Duration::from_millis(u64::from(ms));
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_heartbeat_ms(c: *mut PirateTokClient, ms: u32) {
    let Some(client) = c.as_mut() else { return };
    client.config.heartbeat_interval = Duration::from_millis(u64::from(ms));
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_max_retries(c: *mut PirateTokClient, n: u32) {
    let Some(client) = c.as_mut() else { return };
    client.config.max_retries = n;
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_stale_timeout_ms(c: *mut PirateTokClient, ms: u32) {
    let Some(client) = c.as_mut() else { return };
    client.config.stale_timeout = Duration::from_millis(u64::from(ms));
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_compress(c: *mut PirateTokClient, enabled: i32) {
    let Some(client) = c.as_mut() else { return };
    client.config.compress = enabled != 0;
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_proxy(c: *mut PirateTokClient, url: *const c_char) {
    let Some(client) = c.as_mut() else { return };
    match owned_optional(url) {
        Ok(value) => client.config.proxy = value,
        Err(e) => {
            tracing::warn!(error = %e, "set_proxy rejected argument");
            reject("proxy", &e)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_user_agent(
    c: *mut PirateTokClient,
    ua: *const c_char,
) {
    let Some(client) = c.as_mut() else { return };
    match owned_optional(ua) {
        Ok(value) => client.config.user_agent = value,
        Err(e) => {
            tracing::warn!(error = %e, "set_user_agent rejected argument");
            reject("user_agent", &e)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_cookies(
    c: *mut PirateTokClient,
    cookies: *const c_char,
) {
    let Some(client) = c.as_mut() else { return };
    match owned_optional(cookies) {
        Ok(value) => client.config.cookies = value,
        Err(e) => {
            tracing::warn!(error = %e, "set_cookies rejected argument");
            reject("cookies", &e)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_language(
    c: *mut PirateTokClient,
    language: *const c_char,
) {
    let Some(client) = c.as_mut() else { return };
    match required_str(language) {
        Ok(value) => client.config.language = value.to_string(),
        Err(e) => {
            tracing::warn!(error = %e, "set_language rejected argument");
            reject("language", &e)
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_region(
    c: *mut PirateTokClient,
    region: *const c_char,
) {
    let Some(client) = c.as_mut() else { return };
    match required_str(region) {
        Ok(value) => client.config.region = value.to_string(),
        Err(e) => {
            tracing::warn!(error = %e, "set_region rejected argument");
            reject("region", &e)
        }
    }
}

unsafe fn owned_optional(p: *const c_char) -> Result<Option<String>, FfiError> {
    match optional_str(p)? {
        CStrArg::Given(value) => Ok(Some(value.to_string())),
        CStrArg::Absent => Ok(None),
    }
}

fn reject(field: &str, e: &FfiError) {
    last_error::set(&format!("{field}: {e}"));
}
