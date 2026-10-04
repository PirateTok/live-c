use piratetok_live_rs::helpers::gift_streak::{GiftStreakEvent, GiftStreakTracker};
use piratetok_live_rs::helpers::like_accumulator::{LikeAccumulator, LikeStats};
use piratetok_live_rs::structs::proto::messages::{
    Contributor, WebcastGiftMessage, WebcastRoomUserSeqMessage,
};
use piratetok_live_rs::structs::proto::user::UserIdentity;
use piratetok_live_rs::structs::TikTokLiveEvent;
use serde_json::{json, Map, Value};

use crate::catalog::{describe, payload_b64};

pub struct EventEnricher {
    gifts: GiftStreakTracker,
    likes: LikeAccumulator,
}

impl EventEnricher {
    pub fn new() -> Self {
        Self {
            gifts: GiftStreakTracker::new(),
            likes: LikeAccumulator::new(),
        }
    }

    pub fn json(&mut self, ev: &TikTokLiveEvent) -> String {
        let mut value = event_value(ev);
        match ev {
            TikTokLiveEvent::Gift(msg) => value["streak"] = streak_json(&self.gifts.process(msg)),
            TikTokLiveEvent::Like(msg) => {
                value["like_stats"] = like_stats_json(&self.likes.process(msg))
            }
            _ => {}
        }
        value.to_string()
    }
}

pub fn event_type_id(ev: &TikTokLiveEvent) -> i32 {
    describe(ev).id
}

pub fn event_value(ev: &TikTokLiveEvent) -> Value {
    let described = describe(ev);
    let mut out = Map::new();
    out.insert("type".into(), json!(described.name));
    out.insert("type_id".into(), json!(described.id));
    if !described.payload.is_empty() {
        out.insert("payload_b64".into(), json!(payload_b64(&described)));
    }
    if let Value::Object(fields) = detail_json(ev) {
        out.extend(fields);
    }
    Value::Object(out)
}

pub fn user_json(user: &UserIdentity) -> Value {
    let fans_club = user.fans_club.as_ref().and_then(|fc| fc.data.as_ref());
    let follow = user.follow_info.as_ref();
    let badges: Vec<Value> = user
        .badge_list
        .iter()
        .map(|b| json!({ "scene": b.badge_scene, "display_type": b.display_type, "display": b.display, "level": b.log_extra.as_ref().map(|x| x.level.as_str()) }))
        .collect();
    json!({
        "user_id": user.user_id,
        "id_str": user.id_str,
        "sec_uid": user.sec_uid,
        "nickname": user.nickname,
        "unique_id": user.unique_id,
        "bio": user.bio_description,
        "verified": user.verified,
        "follow_status": user.follow_status,
        "is_follower": user.is_follower,
        "is_following": user.is_following,
        "is_subscribe": user.is_subscribe,
        "follower_count": follow.map(|f| f.follower_count),
        "following_count": follow.map(|f| f.following_count),
        "avatar": user.avatar_thumb.as_ref().and_then(|img| img.url_list.first()),
        "fans_club_level": fans_club.map(|d| d.level),
        "fans_club_name": fans_club.map(|d| d.club_name.as_str()),
        "top_vip_no": user.top_vip_no,
        "pay_score": user.pay_score,
        "badges": badges,
    })
}

pub fn contributor_json(c: &Contributor) -> Value {
    json!({ "rank": c.rank, "score": c.score, "delta": c.delta, "user": c.user.as_ref().map(user_json) })
}

fn streak_json(s: &GiftStreakEvent) -> Value {
    json!({
        "streak_id": s.streak_id,
        "is_active": s.is_active,
        "is_final": s.is_final,
        "event_gift_count": s.event_gift_count,
        "total_gift_count": s.total_gift_count,
        "event_diamond_count": s.event_diamond_count,
        "total_diamond_count": s.total_diamond_count,
    })
}

fn like_stats_json(s: &LikeStats) -> Value {
    json!({ "event_like_count": s.event_like_count, "total_like_count": s.total_like_count, "accumulated_count": s.accumulated_count, "went_backwards": s.went_backwards })
}

fn detail_json(ev: &TikTokLiveEvent) -> Value {
    match ev {
        TikTokLiveEvent::Connected { room_id } => json!({ "room_id": room_id }),
        TikTokLiveEvent::Reconnecting {
            attempt,
            max_retries,
            delay_secs,
        } => json!({ "attempt": attempt, "max_retries": max_retries, "delay_secs": delay_secs }),
        TikTokLiveEvent::Disconnected => json!({ "reason": "disconnected" }),
        TikTokLiveEvent::Gift(msg) => gift_json(msg),
        TikTokLiveEvent::RoomUserSeq(msg) => room_user_seq_json(msg),
        TikTokLiveEvent::Unknown { method, payload } => {
            json!({ "method": method, "payload_len": payload.len() })
        }
        other => social_json(other),
    }
}

fn room_user_seq_json(msg: &WebcastRoomUserSeqMessage) -> Value {
    let top: Vec<Value> = msg
        .top_viewers()
        .into_iter()
        .map(contributor_json)
        .collect();
    json!({
        "viewer_count": msg.viewer_count,
        "total_user": msg.total_user,
        "popularity": msg.popularity,
        "anonymous": msg.anonymous,
        "top_viewers": top,
        "msg_id": msg.common.as_ref().map(|c| c.msg_id),
    })
}

fn gift_json(msg: &WebcastGiftMessage) -> Value {
    let details = msg.gift_details.as_ref();
    json!({
        "user": msg.user.as_ref().map(user_json),
        "gift_id": msg.gift_id,
        "gift_name": details.map(|d| d.gift_name.as_str()),
        "diamond_count": details.map(|d| d.diamond_count),
        "repeat_count": msg.repeat_count,
        "repeat_end": msg.repeat_end,
        "combo_count": msg.combo_count,
        "is_combo": msg.is_combo_gift(),
        "streak_over": msg.is_streak_over(),
        "diamond_total": msg.diamond_total(),
        "is_first_sent": msg.is_first_sent,
        "group_id": msg.group_id,
        "msg_id": msg.common.as_ref().map(|c| c.msg_id),
    })
}

fn social_json(ev: &TikTokLiveEvent) -> Value {
    match ev {
        TikTokLiveEvent::Chat(msg) => {
            json!({ "user": msg.user.as_ref().map(user_json), "comment": msg.comment, "language": msg.content_language, "msg_id": msg.common.as_ref().map(|c| c.msg_id) })
        }
        TikTokLiveEvent::Like(msg) => {
            json!({ "user": msg.user.as_ref().map(user_json), "like_count": msg.like_count, "total_like_count": msg.total_like_count, "msg_id": msg.common.as_ref().map(|c| c.msg_id) })
        }
        TikTokLiveEvent::Member(msg) | TikTokLiveEvent::Join(msg) => {
            json!({ "user": msg.user.as_ref().map(user_json), "action": msg.action, "member_count": msg.member_count, "msg_id": msg.common.as_ref().map(|c| c.msg_id) })
        }
        TikTokLiveEvent::Social(msg)
        | TikTokLiveEvent::Follow(msg)
        | TikTokLiveEvent::Share(msg) => json!({
            "user": msg.user.as_ref().map(user_json),
            "action": msg.action,
            "share_type": msg.share_type,
            "follow_count": msg.follow_count,
            "share_count": msg.share_count,
            "msg_id": msg.common.as_ref().map(|c| c.msg_id),
        }),
        TikTokLiveEvent::Control(msg) | TikTokLiveEvent::LiveEnded(msg) => {
            json!({ "action": msg.action, "tips": msg.tips, "msg_id": msg.common.as_ref().map(|c| c.msg_id) })
        }
        TikTokLiveEvent::LiveIntro(msg) => {
            json!({ "content": msg.content, "language": msg.content_language, "host": msg.user.as_ref().map(user_json), "msg_id": msg.common.as_ref().map(|c| c.msg_id) })
        }
        TikTokLiveEvent::RoomMessage(msg) => {
            json!({ "content": msg.content, "msg_id": msg.common.as_ref().map(|c| c.msg_id) })
        }
        TikTokLiveEvent::Caption(msg) => {
            let captions: Vec<Value> = msg
                .content
                .iter()
                .map(|d| json!({ "language": d.language, "text": d.text }))
                .collect();
            json!({ "captions": captions, "msg_id": msg.common.as_ref().map(|c| c.msg_id) })
        }
        other => json!({ "debug": format!("{other:?}").chars().take(200).collect::<String>() }),
    }
}
