use std::ffi::{c_char, c_void, CString};
use std::sync::Arc;

use piratetok_live_rs::structs::config::TikTokLiveConfig;
use piratetok_live_rs::structs::TikTokLiveEvent;
use piratetok_live_rs::{TikTokLive, TikTokLiveBuilder};
use tokio::sync::Notify;

use crate::catalog::Lifecycle;
use crate::client::PirateTokClient;
use crate::codes::Code;
use crate::last_error;
use crate::serialize::{event_type_id, EventEnricher};

pub type RawCallback = unsafe extern "C" fn(i32, *const c_char, usize, *mut c_void);

struct CallbackCtx {
    callback: RawCallback,
    user_data: *mut c_void,
}

unsafe impl Send for CallbackCtx {}
unsafe impl Sync for CallbackCtx {}

#[no_mangle]
pub unsafe extern "C" fn piratetok_connect(
    client: *mut PirateTokClient,
    callback: RawCallback,
    user_data: *mut c_void,
) -> i32 {
    let Some(c) = client.as_mut() else {
        last_error::set("client is null");
        return Code::BAD_PARAM;
    };
    for running in c.task.iter() {
        tracing::warn!(
            finished = running.is_finished(),
            "piratetok_connect called twice"
        );
        last_error::set("client already connected");
        return Code::ALREADY_RUNNING;
    }
    let Some(runtime) = c.runtime.as_ref() else {
        last_error::set("runtime is null");
        return Code::BAD_PARAM;
    };
    let ctx = CallbackCtx {
        callback,
        user_data,
    };
    c.task = Some(
        runtime
            .rt
            .spawn(pump(builder_from(&c.config), c.stop.clone(), ctx)),
    );
    Code::OK
}

#[no_mangle]
pub unsafe extern "C" fn piratetok_disconnect(client: *mut PirateTokClient) -> i32 {
    let Some(c) = client.as_mut() else {
        last_error::set("client is null");
        return Code::BAD_PARAM;
    };
    let Some(task) = c.task.take() else {
        return Code::OK;
    };
    let Some(runtime) = c.runtime.as_ref() else {
        last_error::set("runtime is null");
        return Code::BAD_PARAM;
    };
    c.stop.notify_one();
    match runtime.rt.block_on(task) {
        Ok(()) => {}
        Err(e) => tracing::warn!(error = %e, "event task ended abnormally"),
    }
    c.stop = Arc::new(Notify::new());
    Code::OK
}

fn builder_from(config: &TikTokLiveConfig) -> TikTokLiveBuilder {
    let mut builder = TikTokLive::builder(&config.username)
        .cdn(config.cdn.clone())
        .timeout(config.timeout)
        .heartbeat_interval(config.heartbeat_interval)
        .max_retries(config.max_retries)
        .stale_timeout(config.stale_timeout)
        .language(config.language.clone())
        .region(config.region.clone())
        .compress(config.compress);
    for proxy in config.proxy.iter() {
        builder = builder.proxy(proxy.clone());
    }
    for ua in config.user_agent.iter() {
        builder = builder.user_agent(ua.clone());
    }
    for cookies in config.cookies.iter() {
        builder = builder.cookies(cookies.clone());
    }
    builder
}

async fn pump(builder: TikTokLiveBuilder, stop: Arc<Notify>, ctx: CallbackCtx) {
    let mut stream = match builder.connect().await {
        Ok(stream) => stream,
        Err(e) => {
            tracing::warn!(error = %e, "connect failed");
            fire(
                &ctx,
                Lifecycle::DISCONNECTED,
                &serde_json::json!({ "error": e.to_string() }).to_string(),
            );
            return;
        }
    };
    let mut enricher = EventEnricher::new();
    loop {
        tokio::select! {
            () = stop.notified() => {
                fire(&ctx, Lifecycle::DISCONNECTED, r#"{"reason":"user_disconnect"}"#);
                break;
            }
            event = stream.next_event() => {
                let event = match event {
                    Ok(event) => event,
                    Err(e) => {
                        tracing::warn!(error = %e, "event stream closed");
                        fire(&ctx, Lifecycle::DISCONNECTED, r#"{"reason":"stream_ended"}"#);
                        break;
                    }
                };
                fire(&ctx, event_type_id(&event), &enricher.json(&event));
                if matches!(event, TikTokLiveEvent::Disconnected) {
                    break;
                }
            }
        }
    }
}

fn fire(ctx: &CallbackCtx, type_id: i32, json: &str) {
    match CString::new(json) {
        Ok(text) => unsafe { (ctx.callback)(type_id, text.as_ptr(), json.len(), ctx.user_data) },
        Err(e) => tracing::error!(error = %e, type_id, "event json contained NUL, dropped"),
    }
}
