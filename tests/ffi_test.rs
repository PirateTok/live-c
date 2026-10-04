use std::ffi::{c_char, CStr, CString};
use std::time::Duration;

use piratetok::api::{piratetok_check_online, piratetok_fetch_room_audience};
use piratetok::client::{
    piratetok_client_free, piratetok_client_new, piratetok_client_set_compress,
    piratetok_client_set_heartbeat_ms, piratetok_client_set_language, piratetok_client_set_proxy,
    piratetok_client_set_region,
};
use piratetok::codes::{fail_live, Code};
use piratetok::last_error::piratetok_last_error;
use piratetok::runtime::{piratetok_init, piratetok_shutdown};
use piratetok::serialize::{event_to_json, event_to_type_id, EventType};
use piratetok_live_rs::errors::TikTokLiveError;
use piratetok_live_rs::structs::proto::messages::{Contributor, WebcastRoomUserSeqMessage};
use piratetok_live_rs::structs::proto::user::UserIdentity;
use piratetok_live_rs::structs::TikTokLiveEvent;

fn last_error() -> String {
    unsafe { CStr::from_ptr(piratetok_last_error()) }
        .to_string_lossy()
        .into_owned()
}

fn contributor(rank: i64, score: i64, nickname: Option<&str>) -> Contributor {
    Contributor {
        rank,
        score,
        delta: 0,
        user: nickname.map(|n| UserIdentity {
            nickname: n.into(),
            unique_id: format!("{n}_id"),
            ..Default::default()
        }),
    }
}

#[test]
fn client_new_rejects_null_runtime() {
    let name = CString::new("someone").unwrap();
    let client = unsafe { piratetok_client_new(std::ptr::null_mut(), name.as_ptr()) };
    assert!(client.is_null());
    assert_eq!(last_error(), "runtime is null");
}

#[test]
fn setters_reach_the_rust_config() {
    let rt = piratetok_init();
    let name = CString::new("someone").unwrap();
    let client = unsafe { piratetok_client_new(rt, name.as_ptr()) };
    assert!(!client.is_null());
    let ro = CString::new("ro").unwrap();
    let region = CString::new("RO").unwrap();
    let proxy = CString::new("http://127.0.0.1:8080").unwrap();
    unsafe {
        piratetok_client_set_language(client, ro.as_ptr());
        piratetok_client_set_region(client, region.as_ptr());
        piratetok_client_set_compress(client, 0);
        piratetok_client_set_heartbeat_ms(client, 15_000);
        piratetok_client_set_proxy(client, proxy.as_ptr());
        let config = &(*client).config;
        assert_eq!(config.language, "ro");
        assert_eq!(config.region, "RO");
        assert!(!config.compress);
        assert_eq!(config.heartbeat_interval, Duration::from_millis(15_000));
        assert_eq!(config.proxy.as_deref(), Some("http://127.0.0.1:8080"));
        piratetok_client_set_proxy(client, std::ptr::null());
        assert_eq!((*client).config.proxy, None);
        piratetok_client_set_language(client, std::ptr::null());
        assert_eq!(
            (*client).config.language,
            "ro",
            "null language is rejected, not applied"
        );
        assert!(last_error().starts_with("language:"));
        piratetok_client_free(client);
        piratetok_shutdown(rt);
    }
}

#[test]
fn room_user_seq_json_carries_sorted_top_viewers() {
    let msg = WebcastRoomUserSeqMessage {
        viewer_count: 120,
        total_user: 900,
        anonymous: 7,
        ranks_list: vec![
            contributor(3, 10, Some("c")),
            contributor(1, 500, Some("a")),
            contributor(2, 90, None),
            contributor(2, 100, Some("b")),
        ],
        ..Default::default()
    };
    let event = TikTokLiveEvent::RoomUserSeq(msg);
    assert_eq!(event_to_type_id(&event), EventType::ROOM_USER_SEQ);
    let json: serde_json::Value = serde_json::from_str(&event_to_json(&event)).unwrap();
    let top = json["top_viewers"].as_array().unwrap();
    let names: Vec<&str> = top
        .iter()
        .map(|v| v["user"]["nickname"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        vec!["a", "b", "c"],
        "sorted by rank, entries without a user skipped"
    );
    assert_eq!(top[0]["score"], 500);
    assert_eq!(top[0]["user"]["unique_id"], "a_id");
    assert_eq!(json["viewer_count"], 120);
    assert_eq!(json["anonymous"], 7);
}

#[test]
fn absent_fields_serialize_as_null() {
    let event = TikTokLiveEvent::RoomUserSeq(WebcastRoomUserSeqMessage::default());
    let json: serde_json::Value = serde_json::from_str(&event_to_json(&event)).unwrap();
    assert!(json["msg_id"].is_null());
    assert_eq!(json["top_viewers"], serde_json::json!([]));
}

#[test]
fn session_required_has_its_own_code() {
    let code = fail_live(&TikTokLiveError::SessionRequired("login".into()));
    assert_eq!(code, Code::SESSION_REQUIRED);
    assert!(last_error().contains("session required"));
}

#[test]
fn standalone_calls_validate_arguments_before_network() {
    let rt = piratetok_init();
    let mut out: *mut c_char = std::ptr::null_mut();
    unsafe {
        assert_eq!(
            piratetok_check_online(rt, std::ptr::null(), &mut out, std::ptr::null_mut()),
            Code::BAD_PARAM
        );
        assert_eq!(
            piratetok_fetch_room_audience(
                rt,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                &mut out
            ),
            Code::BAD_PARAM
        );
        let bad = [0xffu8, 0xfe, 0x00];
        assert_eq!(
            piratetok_fetch_room_audience(
                rt,
                bad.as_ptr().cast(),
                std::ptr::null(),
                std::ptr::null(),
                &mut out
            ),
            Code::BAD_PARAM
        );
        assert!(last_error().contains("utf-8"));
        assert_eq!(
            piratetok_check_online(
                std::ptr::null_mut(),
                std::ptr::null(),
                &mut out,
                std::ptr::null_mut()
            ),
            Code::BAD_PARAM
        );
        piratetok_shutdown(rt);
    }
    assert!(out.is_null());
}
