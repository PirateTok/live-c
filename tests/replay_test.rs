//! Replays live-testdata captures through live-rs decode + the C JSON layer
//! (EventEnricher) and checks everything a C caller sees against the manifests.

use std::collections::BTreeMap;
use std::path::PathBuf;

use base64::Engine;
use piratetok::catalog::describe;
use piratetok::serialize::EventEnricher;
use piratetok_live_rs::decode::mapper;
use piratetok_live_rs::structs::proto::frames::WebcastPushFrame;
use piratetok_live_rs::structs::proto::messages::WebcastResponse;
use piratetok_live_rs::websocket::frames::decompress_if_gzipped;
use prost::Message;
use serde_json::Value;

fn testdata() -> PathBuf {
    let dir = match std::env::var("PIRATETOK_TESTDATA") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => PathBuf::from("testdata"),
    };
    assert!(
        dir.join("captures").exists(),
        "no testdata at {} (symlink testdata -> ../live-rs/testdata or set PIRATETOK_TESTDATA)",
        dir.display()
    );
    dir
}

fn frames(name: &str) -> Vec<Vec<u8>> {
    let path = testdata().join("captures").join(format!("{name}.bin"));
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut out = Vec::new();
    let mut pos = 0;
    while pos + 4 <= data.len() {
        let len = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
        out.push(data[pos + 4..pos + 4 + len].to_vec());
        pos += 4 + len;
    }
    eprintln!("LOAD {} ({} frames)", path.display(), out.len());
    out
}

fn manifest(name: &str) -> Value {
    let path = testdata().join("manifests").join(format!("{name}.json"));
    serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
    )
    .unwrap()
}

#[derive(Default)]
struct Seen {
    events: u64,
    types: BTreeMap<String, u64>,
    likes: Vec<Value>,
    gifts: BTreeMap<String, Vec<Value>>,
    top_viewer_boxes: u64,
}

fn replay(capture: &str) -> Seen {
    let mut seen = Seen::default();
    let mut enricher = EventEnricher::new();
    for raw in frames(capture) {
        let frame = WebcastPushFrame::decode(raw.as_slice()).unwrap();
        if frame.payload_type != "msg" {
            continue;
        }
        let response =
            WebcastResponse::decode(decompress_if_gzipped(&frame.payload).unwrap().as_slice())
                .unwrap();
        for msg in &response.messages {
            for event in mapper::decode_message(&msg.r#type, &msg.payload) {
                let json: Value = serde_json::from_str(&enricher.json(&event)).unwrap();
                let described = describe(&event);
                assert_eq!(json["type"], described.name);
                assert_eq!(json["type_id"], described.id);
                if described.name != "Unknown" {
                    let b64 = json["payload_b64"].as_str().unwrap_or("");
                    let bytes = base64::engine::general_purpose::STANDARD
                        .decode(b64)
                        .unwrap();
                    assert_eq!(
                        bytes, described.payload,
                        "payload round-trip for {}",
                        described.name
                    );
                }
                seen.events += 1;
                *seen.types.entry(described.name.to_string()).or_default() += 1;
                match described.name {
                    "Like" => seen.likes.push(json),
                    "Gift" => seen
                        .gifts
                        .entry(json["group_id"].to_string())
                        .or_default()
                        .push(json),
                    "RoomUserSeq" if !json["top_viewers"].as_array().unwrap().is_empty() => {
                        seen.top_viewer_boxes += 1
                    }
                    _ => {}
                }
            }
        }
    }
    seen
}

fn check(capture: &str, manifest_name: &str) {
    let m = manifest(manifest_name);
    let seen = replay(capture);
    assert_eq!(
        seen.events,
        m["event_count"].as_u64().unwrap(),
        "{capture}: event_count"
    );
    let expected: BTreeMap<String, u64> = serde_json::from_value(m["event_types"].clone()).unwrap();
    assert_eq!(seen.types, expected, "{capture}: event_types via C JSON");
    assert_eq!(
        seen.types.get("Follow").copied().unwrap_or(0),
        m["sub_routed"]["follow"].as_u64().unwrap()
    );
    assert_eq!(
        seen.types.get("Join").copied().unwrap_or(0),
        m["sub_routed"]["join"].as_u64().unwrap()
    );

    let like_events = m["like_accumulator"]["events"].as_array().unwrap();
    assert_eq!(
        seen.likes.len(),
        like_events.len(),
        "{capture}: like events"
    );
    for (i, (got, want)) in seen.likes.iter().zip(like_events).enumerate() {
        assert_eq!(
            got["like_count"], want["wire_count"],
            "{capture}: like[{i}] wire_count"
        );
        assert_eq!(
            got["total_like_count"], want["wire_total"],
            "{capture}: like[{i}] wire_total"
        );
        assert_eq!(
            got["like_stats"]["total_like_count"], want["acc_total"],
            "{capture}: like[{i}] acc_total"
        );
        assert_eq!(
            got["like_stats"]["accumulated_count"], want["accumulated"],
            "{capture}: like[{i}] accumulated"
        );
        assert_eq!(
            got["like_stats"]["went_backwards"], want["went_backwards"],
            "{capture}: like[{i}] went_backwards"
        );
    }

    let groups = m["gift_streaks"]["groups"].as_object().unwrap();
    assert_eq!(seen.gifts.len(), groups.len(), "{capture}: gift groups");
    for (gid, want) in groups {
        let got = &seen.gifts[gid];
        let want = want.as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{capture}: gift group {gid}");
        for (g, w) in got.iter().zip(want) {
            assert_eq!(g["gift_id"], w["gift_id"]);
            assert_eq!(g["repeat_count"], w["repeat_count"]);
            assert_eq!(g["streak"]["event_gift_count"], w["delta"]);
            assert_eq!(g["streak"]["is_final"], w["is_final"]);
            assert_eq!(g["streak"]["total_diamond_count"], w["diamond_total"]);
        }
    }
    eprintln!(
        "{capture}: {} events, {} RoomUserSeq with top_viewers",
        seen.events, seen.top_viewer_boxes
    );
}

#[test]
fn replay_calvinterest6() {
    check("calvinterest6", "calvinterest6");
}

#[test]
fn replay_fox4newsdallasfortworth() {
    check("fox4newsdallasfortworth", "fox4newsdallasfortworth");
}

#[test]
fn replay_happyhappygaltv() {
    check("happyhappygaltv", "happyhappygaltv");
}

#[test]
fn replay_raw_captures() {
    for name in [
        "calvinterest6",
        "fox4newsdallasfortworth",
        "happyhappygaltv",
    ] {
        check(&format!("{name}_raw"), name);
    }
}

#[test]
fn every_rust_event_variant_has_a_c_type_id() {
    let ids = piratetok::catalog::all_ids();
    assert_eq!(ids.len(), 68, "message variants exposed to C");
    let mut unique: Vec<i32> = ids.iter().map(|(_, id)| *id).collect();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), ids.len(), "type ids are unique");
}
