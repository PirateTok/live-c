//! Serialize TikTokLiveEvent variants to JSON strings for the C callback.

use piratetok_live_rs::structs::proto::messages::UserIdentity;
use piratetok_live_rs::structs::TikTokLiveEvent;
use serde_json::{json, Value};

/// Map event variant to the integer type ID matching PirateTokEventType in piratetok.h.
pub fn event_to_type_id(ev: &TikTokLiveEvent) -> i32 {
    match ev {
        TikTokLiveEvent::Connected { .. } => 0,
        TikTokLiveEvent::Reconnecting { .. } => 1,
        TikTokLiveEvent::Disconnected => 2,

        TikTokLiveEvent::Chat(_) => 10,
        TikTokLiveEvent::Gift(_) => 11,
        TikTokLiveEvent::Like(_) => 12,
        TikTokLiveEvent::Member(_) => 13,
        TikTokLiveEvent::Social(_) => 14,
        TikTokLiveEvent::RoomUserSeq(_) => 15,
        TikTokLiveEvent::Control(_) => 16,

        TikTokLiveEvent::Follow(_) => 20,
        TikTokLiveEvent::Share(_) => 21,
        TikTokLiveEvent::Join(_) => 22,
        TikTokLiveEvent::LiveEnded(_) => 23,

        TikTokLiveEvent::LiveIntro(_) => 30,
        TikTokLiveEvent::RoomMessage(_) => 31,
        TikTokLiveEvent::Caption(_) => 32,
        TikTokLiveEvent::GoalUpdate(_) => 33,
        TikTokLiveEvent::ImDelete(_) => 34,

        TikTokLiveEvent::RankUpdate(_) => 40,
        TikTokLiveEvent::Poll(_) => 41,
        TikTokLiveEvent::Envelope(_) => 42,
        TikTokLiveEvent::RoomPin(_) => 43,

        TikTokLiveEvent::Unknown { .. } => 255,
        _ => 200, // CYCLED_EVENT_OTHER
    }
}

fn user_to_json(u: &Option<UserIdentity>) -> Value {
    match u {
        Some(user) => json!({
            "user_id": user.user_id,
            "nickname": user.nickname,
            "unique_id": user.unique_id,
            "bio": user.bio_description,
            "verified": user.verified,
            "follow_status": user.follow_status,
            "is_follower": user.is_follower,
            "is_following": user.is_following,
            "is_subscribe": user.is_subscribe,
            "avatar": user.avatar_thumb.as_ref()
                .and_then(|img| img.url_list.first())
                .unwrap_or(&String::new()),
            "fans_club_level": user.fans_club.as_ref()
                .and_then(|fc| fc.data.as_ref())
                .map(|d| d.level)
                .unwrap_or(0),
            "fans_club_name": user.fans_club.as_ref()
                .and_then(|fc| fc.data.as_ref())
                .map(|d| d.club_name.as_str())
                .unwrap_or(""),
            "top_vip_no": user.top_vip_no,
            "pay_score": user.pay_score,
        }),
        None => Value::Null,
    }
}

/// Serialize an event to a JSON string. Extracts key fields for common events;
/// returns a minimal object for niche/unknown events.
pub fn event_to_json(ev: &TikTokLiveEvent) -> String {
    let val: Value = match ev {
        // -- lifecycle --
        TikTokLiveEvent::Connected { room_id } => {
            json!({ "room_id": room_id })
        }
        TikTokLiveEvent::Reconnecting { attempt, max_retries, delay_secs } => {
            json!({
                "attempt": attempt,
                "max_retries": max_retries,
                "delay_secs": delay_secs
            })
        }
        TikTokLiveEvent::Disconnected => {
            json!({ "reason": "disconnected" })
        }

        // -- core --
        TikTokLiveEvent::Chat(msg) => {
            json!({
                "user": user_to_json(&msg.user),
                "comment": msg.comment,
                "language": msg.content_language,
                "msg_id": msg.common.as_ref().map(|c| c.msg_id).unwrap_or(0),
            })
        }
        TikTokLiveEvent::Gift(msg) => {
            let details = msg.gift_details.as_ref();
            json!({
                "user": user_to_json(&msg.user),
                "gift_id": msg.gift_id,
                "gift_name": details.map(|d| d.gift_name.as_str()).unwrap_or(""),
                "diamond_count": details.map(|d| d.diamond_count).unwrap_or(0),
                "repeat_count": msg.repeat_count,
                "repeat_end": msg.repeat_end,
                "combo_count": msg.combo_count,
                "is_combo": msg.is_combo_gift(),
                "streak_over": msg.is_streak_over(),
                "diamond_total": msg.diamond_total(),
                "is_first_sent": msg.is_first_sent,
                "group_id": msg.group_id,
                "msg_id": msg.common.as_ref().map(|c| c.msg_id).unwrap_or(0),
            })
        }
        TikTokLiveEvent::Like(msg) => {
            json!({
                "user": user_to_json(&msg.user),
                "like_count": msg.like_count,
                "total_like_count": msg.total_like_count,
                "msg_id": msg.common.as_ref().map(|c| c.msg_id).unwrap_or(0),
            })
        }
        TikTokLiveEvent::Member(msg) => {
            json!({
                "action": msg.action,
                "member_count": msg.member_count,
                "msg_id": 0,
            })
        }
        TikTokLiveEvent::Social(msg) | TikTokLiveEvent::Follow(msg) | TikTokLiveEvent::Share(msg) => {
            json!({
                "user": user_to_json(&msg.user),
                "action": msg.action,
                "share_type": msg.share_type,
                "follow_count": msg.follow_count,
                "share_count": msg.share_count,
                "msg_id": msg.common.as_ref().map(|c| c.msg_id).unwrap_or(0),
            })
        }
        TikTokLiveEvent::RoomUserSeq(msg) => {
            json!({
                "viewer_count": msg.viewer_count,
                "total_user": msg.total_user,
                "popularity": msg.popularity,
                "msg_id": msg.common.as_ref().map(|c| c.msg_id).unwrap_or(0),
            })
        }
        TikTokLiveEvent::Control(msg) | TikTokLiveEvent::LiveEnded(msg) => {
            json!({
                "action": msg.action,
                "tips": msg.tips,
                "msg_id": msg.common.as_ref().map(|c| c.msg_id).unwrap_or(0),
            })
        }
        TikTokLiveEvent::Join(msg) => {
            json!({
                "action": msg.action,
                "member_count": msg.member_count,
                "msg_id": 0,
            })
        }

        // -- useful --
        TikTokLiveEvent::LiveIntro(msg) => {
            json!({
                "room_id": msg.room_id,
                "content": msg.content,
                "host": user_to_json(&msg.host),
                "msg_id": msg.common.as_ref().map(|c| c.msg_id).unwrap_or(0),
            })
        }
        TikTokLiveEvent::RoomMessage(msg) => {
            json!({
                "content": msg.content,
                "msg_id": msg.common.as_ref().map(|c| c.msg_id).unwrap_or(0),
            })
        }
        TikTokLiveEvent::Caption(msg) => {
            let captions: Vec<Value> = msg.caption_data.iter().map(|d| {
                json!({ "language": d.language, "text": d.text })
            }).collect();
            json!({
                "captions": captions,
                "msg_id": msg.common.as_ref().map(|c| c.msg_id).unwrap_or(0),
            })
        }

        // -- unknown --
        TikTokLiveEvent::Unknown { method, payload } => {
            json!({
                "method": method,
                "payload_len": payload.len(),
            })
        }

        // everything else: minimal JSON with the debug representation
        _ => {
            json!({ "event": format!("{ev:?}").chars().take(200).collect::<String>() })
        }
    };

    val.to_string()
}
