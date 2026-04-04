//! C ABI bindings for piratetok-live-rs.
//!
//! Produces libpiratetok.so / piratetok.dll / libpiratetok.dylib.
//! See include/piratetok.h for the C API.

mod serialize;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use piratetok_live_rs::errors::TikTokLiveError;
use piratetok_live_rs::structs::config::CdnEndpoint;
use piratetok_live_rs::structs::TikTokLiveEvent;
use piratetok_live_rs::TikTokLive;

use crate::serialize::{event_to_json, event_to_type_id};

// ---------------------------------------------------------------------------
// Thread-local last error
// ---------------------------------------------------------------------------

thread_local! {
    static LAST_ERROR: std::cell::RefCell<CString> =
        std::cell::RefCell::new(CString::new("").unwrap());
}

fn set_last_error(msg: &str) {
    let c = CString::new(msg).unwrap_or_else(|_| CString::new("(error contained null)").unwrap());
    LAST_ERROR.with(|e| *e.borrow_mut() = c);
}

fn map_error(e: &TikTokLiveError) -> i32 {
    match e {
        TikTokLiveError::UserNotFound(_) => 2,
        TikTokLiveError::HostNotOnline(_) => 3,
        TikTokLiveError::DeviceBlocked => 4,
        TikTokLiveError::AgeRestricted(_) => 5,
        TikTokLiveError::InvalidResponse(_) => 6,
        TikTokLiveError::ConnectionClosed => 7,
        TikTokLiveError::Http(_) => 8,
        TikTokLiveError::WebSocket(_) => 9,
        _ => 99,
    }
}

fn set_and_map(e: &TikTokLiveError) -> i32 {
    set_last_error(&e.to_string());
    map_error(e)
}

// ---------------------------------------------------------------------------
// Opaque types
// ---------------------------------------------------------------------------

pub struct PirateTokRuntime {
    rt: Runtime,
}

pub struct PirateTokClient {
    rt: *const PirateTokRuntime,
    username: String,
    cdn: CdnEndpoint,
    timeout_ms: u32,
    heartbeat_ms: u32,
    max_retries: u32,
    stale_timeout_ms: u32,
    proxy: Option<String>,
    user_agent: Option<String>,
    cookies: Option<String>,
    // running state
    task_handle: Option<JoinHandle<()>>,
    stop: Arc<Notify>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

unsafe fn str_from_ptr<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }
    CStr::from_ptr(p).to_str().ok()
}

fn to_c_string_ptr(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ---------------------------------------------------------------------------
// Runtime lifecycle
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn piratetok_init() -> *mut PirateTokRuntime {
    match Runtime::new() {
        Ok(rt) => Box::into_raw(Box::new(PirateTokRuntime { rt })),
        Err(e) => {
            set_last_error(&format!("failed to create runtime: {e}"));
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_shutdown(rt: *mut PirateTokRuntime) {
    if !rt.is_null() {
        drop(Box::from_raw(rt));
    }
}

// ---------------------------------------------------------------------------
// Client lifecycle
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_new(
    rt: *mut PirateTokRuntime,
    username: *const c_char,
) -> *mut PirateTokClient {
    if rt.is_null() {
        set_last_error("runtime is null");
        return std::ptr::null_mut();
    }
    let name = match str_from_ptr(username) {
        Some(s) => s.to_string(),
        None => {
            set_last_error("username is null or invalid utf-8");
            return std::ptr::null_mut();
        }
    };

    Box::into_raw(Box::new(PirateTokClient {
        rt: rt as *const PirateTokRuntime,
        username: name,
        cdn: CdnEndpoint::Global,
        timeout_ms: 10_000,
        heartbeat_ms: 10_000,
        max_retries: 5,
        stale_timeout_ms: 60_000,
        proxy: None,
        user_agent: None,
        cookies: None,
        task_handle: None,
        stop: Arc::new(Notify::new()),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_free(client: *mut PirateTokClient) {
    if client.is_null() {
        return;
    }
    let mut c = Box::from_raw(client);
    // abort running task if any
    if let Some(h) = c.task_handle.take() {
        h.abort();
    }
}

// ---------------------------------------------------------------------------
// Configuration setters
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_cdn(c: *mut PirateTokClient, cdn: i32) {
    if c.is_null() { return; }
    (*c).cdn = match cdn {
        1 => CdnEndpoint::Eu,
        2 => CdnEndpoint::Us,
        _ => CdnEndpoint::Global,
    };
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_timeout_ms(c: *mut PirateTokClient, ms: u32) {
    if !c.is_null() { (*c).timeout_ms = ms; }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_heartbeat_ms(c: *mut PirateTokClient, ms: u32) {
    if !c.is_null() { (*c).heartbeat_ms = ms; }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_max_retries(c: *mut PirateTokClient, n: u32) {
    if !c.is_null() { (*c).max_retries = n; }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_stale_timeout_ms(c: *mut PirateTokClient, ms: u32) {
    if !c.is_null() { (*c).stale_timeout_ms = ms; }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_proxy(c: *mut PirateTokClient, url: *const c_char) {
    if c.is_null() { return; }
    (*c).proxy = str_from_ptr(url).map(|s| s.to_string());
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_user_agent(c: *mut PirateTokClient, ua: *const c_char) {
    if c.is_null() { return; }
    (*c).user_agent = str_from_ptr(ua).map(|s| s.to_string());
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_client_set_cookies(c: *mut PirateTokClient, cookies: *const c_char) {
    if c.is_null() { return; }
    (*c).cookies = str_from_ptr(cookies).map(|s| s.to_string());
}

// ---------------------------------------------------------------------------
// Connection
// ---------------------------------------------------------------------------

type RawCallback = unsafe extern "C" fn(i32, *const c_char, usize, *mut std::ffi::c_void);

/// Safety: user_data and callback are sent across threads.
/// The caller guarantees they remain valid for the lifetime of the connection.
struct CallbackCtx {
    callback: RawCallback,
    user_data: *mut std::ffi::c_void,
}

unsafe impl Send for CallbackCtx {}
unsafe impl Sync for CallbackCtx {}

#[no_mangle]
pub unsafe extern "C" fn piratetok_connect(
    client: *mut PirateTokClient,
    callback: RawCallback,
    user_data: *mut std::ffi::c_void,
) -> i32 {
    if client.is_null() {
        set_last_error("client is null");
        return 1;
    }
    let c = &mut *client;
    if c.task_handle.is_some() {
        set_last_error("client already connected");
        return 10;
    }

    let rt = &(*c.rt).rt;
    let stop = c.stop.clone();

    // build the TikTokLive builder from config
    let username = c.username.clone();
    let cdn = c.cdn.clone();
    let timeout = Duration::from_millis(c.timeout_ms as u64);
    let heartbeat = Duration::from_millis(c.heartbeat_ms as u64);
    let max_retries = c.max_retries;
    let stale_timeout = Duration::from_millis(c.stale_timeout_ms as u64);
    let proxy = c.proxy.clone();
    let user_agent = c.user_agent.clone();
    let cookies = c.cookies.clone();

    let ctx = CallbackCtx { callback, user_data };

    let handle = rt.spawn(async move {
        let mut builder = TikTokLive::builder(&username)
            .cdn(cdn)
            .timeout(timeout)
            .heartbeat_interval(heartbeat)
            .max_retries(max_retries)
            .stale_timeout(stale_timeout);

        if let Some(ref p) = proxy {
            builder = builder.proxy(p.clone());
        }
        if let Some(ref ua) = user_agent {
            builder = builder.user_agent(ua.clone());
        }
        if let Some(ref ck) = cookies {
            builder = builder.cookies(ck.clone());
        }

        let mut stream = match builder.connect().await {
            Ok(s) => s,
            Err(e) => {
                let json = serde_json::json!({
                    "error": e.to_string()
                }).to_string();
                fire_callback(&ctx, 2, &json); // disconnected with error
                return;
            }
        };

        loop {
            tokio::select! {
                _ = stop.notified() => {
                    fire_callback(&ctx, 2, r#"{"reason":"user_disconnect"}"#);
                    break;
                }
                event = stream.next_event() => {
                    match event {
                        Some(ev) => {
                            let type_id = event_to_type_id(&ev);
                            let json = event_to_json(&ev);
                            fire_callback(&ctx, type_id, &json);

                            if matches!(ev, TikTokLiveEvent::Disconnected) {
                                break;
                            }
                        }
                        None => {
                            fire_callback(&ctx, 2, r#"{"reason":"stream_ended"}"#);
                            break;
                        }
                    }
                }
            }
        }
    });

    c.task_handle = Some(handle);
    0
}

fn fire_callback(ctx: &CallbackCtx, type_id: i32, json: &str) {
    if let Ok(cstr) = CString::new(json) {
        let ptr = cstr.as_ptr();
        let len = json.len();
        unsafe { (ctx.callback)(type_id, ptr, len, ctx.user_data); }
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_disconnect(client: *mut PirateTokClient) -> i32 {
    if client.is_null() {
        set_last_error("client is null");
        return 1;
    }
    let c = &mut *client;

    if let Some(handle) = c.task_handle.take() {
        c.stop.notify_one();
        let rt = &(*c.rt).rt;
        // block until the task finishes
        let _ = rt.block_on(handle);
        // reset stop for potential re-connect
        c.stop = Arc::new(Notify::new());
    }
    0
}

// ---------------------------------------------------------------------------
// Standalone utilities
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn piratetok_check_online(
    rt_ptr: *mut PirateTokRuntime,
    username: *const c_char,
    out_room_id: *mut *mut c_char,
) -> i32 {
    if rt_ptr.is_null() || username.is_null() || out_room_id.is_null() {
        set_last_error("null parameter");
        return 1;
    }
    let rt = &(*rt_ptr).rt;
    let name = match str_from_ptr(username) {
        Some(s) => s,
        None => {
            set_last_error("invalid utf-8 in username");
            return 1;
        }
    };

    let result = rt.block_on(piratetok_live_rs::http::api::fetch_room_id(
        name,
        Duration::from_secs(10),
        None,
    ));

    match result {
        Ok(resp) => {
            *out_room_id = to_c_string_ptr(&resp.room_id);
            0
        }
        Err(e) => set_and_map(&e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_fetch_room_info(
    rt_ptr: *mut PirateTokRuntime,
    room_id: *const c_char,
    cookies: *const c_char,
    out_json: *mut *mut c_char,
) -> i32 {
    if rt_ptr.is_null() || room_id.is_null() || out_json.is_null() {
        set_last_error("null parameter");
        return 1;
    }
    let rt = &(*rt_ptr).rt;
    let rid = match str_from_ptr(room_id) {
        Some(s) => s,
        None => {
            set_last_error("invalid utf-8 in room_id");
            return 1;
        }
    };
    let ck = str_from_ptr(cookies);

    let result = rt.block_on(piratetok_live_rs::http::api::fetch_room_info(
        rid,
        Duration::from_secs(10),
        ck,
        None,
    ));

    match result {
        Ok(info) => {
            let json = serde_json::to_string(&info).unwrap_or_default();
            *out_json = to_c_string_ptr(&json);
            0
        }
        Err(e) => set_and_map(&e),
    }
}

// ---------------------------------------------------------------------------
// Memory management
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn piratetok_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

// ---------------------------------------------------------------------------
// Error info
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn piratetok_last_error() -> *const c_char {
    LAST_ERROR.with(|e| e.borrow().as_ptr())
}
