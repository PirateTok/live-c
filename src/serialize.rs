use piratetok_live_rs::structs::proto::messages::{
    Contributor, WebcastGiftMessage, WebcastRoomUserSeqMessage,
};
use piratetok_live_rs::structs::proto::user::UserIdentity;
use piratetok_live_rs::structs::TikTokLiveEvent;
use serde_json::{json, Value};

pub struct EventType;

impl EventType {
    pub const CONNECTED: i32 = 0;
    pub const RECONNECTING: i32 = 1;
    pub const DISCONNECTED: i32 = 2;
    pub const CHAT: i32 = 10;
    pub const GIFT: i32 = 11;
    pub const LIKE: i32 = 12;
    pub const MEMBER: i32 = 13;
    pub const SOCIAL: i32 = 14;
    pub const ROOM_USER_SEQ: i32 = 15;
    pub const CONTROL: i32 = 16;
    pub const FOLLOW: i32 = 20;
    pub const SHARE: i32 = 21;
    pub const JOIN: i32 = 22;
    pub const LIVE_ENDED: i32 = 23;
    pub const LIVE_INTRO: i32 = 30;
    pub const ROOM_MESSAGE: i32 = 31;
    pub const CAPTION: i32 = 32;
    pub const GOAL_UPDATE: i32 = 33;
    pub const IM_DELETE: i32 = 34;
    pub const RANK_UPDATE: i32 = 40;
    pub const POLL: i32 = 41;
    pub const ENVELOPE: i32 = 42;
    pub const ROOM_PIN: i32 = 43;
    pub const OTHER: i32 = 200;
    pub const UNKNOWN: i32 = 255;
}

pub fn event_to_type_id(ev: &TikTokLiveEvent) -> i32 {
    match ev {
        TikTokLiveEvent::Connected { .. } => EventType::CONNECTED,
        TikTokLiveEvent::Reconnecting { .. } => EventType::RECONNECTING,
        TikTokLiveEvent::Disconnected => EventType::DISCONNECTED,
        TikTokLiveEvent::Chat(..) => EventType::CHAT,
        TikTokLiveEvent::Gift(..) => EventType::GIFT,
        TikTokLiveEvent::Like(..) => EventType::LIKE,
        TikTokLiveEvent::Member(..) => EventType::MEMBER,
        TikTokLiveEvent::Social(..) => EventType::SOCIAL,
        TikTokLiveEvent::RoomUserSeq(..) => EventType::ROOM_USER_SEQ,
        TikTokLiveEvent::Control(..) => EventType::CONTROL,
        TikTokLiveEvent::Follow(..) => EventType::FOLLOW,
        TikTokLiveEvent::Share(..) => EventType::SHARE,
        TikTokLiveEvent::Join(..) => EventType::JOIN,
        TikTokLiveEvent::LiveEnded(..) => EventType::LIVE_ENDED,
        TikTokLiveEvent::LiveIntro(..) => EventType::LIVE_INTRO,
        TikTokLiveEvent::RoomMessage(..) => EventType::ROOM_MESSAGE,
        TikTokLiveEvent::Caption(..) => EventType::CAPTION,
        TikTokLiveEvent::GoalUpdate(..) => EventType::GOAL_UPDATE,
        TikTokLiveEvent::ImDelete(..) => EventType::IM_DELETE,
        TikTokLiveEvent::RankUpdate(..) => EventType::RANK_UPDATE,
        TikTokLiveEvent::Poll(..) => EventType::POLL,
        TikTokLiveEvent::Envelope(..) => EventType::ENVELOPE,
        TikTokLiveEvent::RoomPin(..) => EventType::ROOM_PIN,
        TikTokLiveEvent::Unknown { .. } => EventType::UNKNOWN,
        _ => EventType::OTHER,
    }
}

pub fn event_to_json(ev: &TikTokLiveEvent) -> String {
    let value = match ev {
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
    };
    value.to_string()
}

pub fn user_json(user: &UserIdentity) -> Value {
    let fans_club = user.fans_club.as_ref().and_then(|fc| fc.data.as_ref());
    json!({
        "user_id": user.user_id,
        "nickname": user.nickname,
        "unique_id": user.unique_id,
        "bio": user.bio_description,
        "verified": user.verified,
        "follow_status": user.follow_status,
        "is_follower": user.is_follower,
        "is_following": user.is_following,
        "is_subscribe": user.is_subscribe,
        "avatar": user.avatar_thumb.as_ref().and_then(|img| img.url_list.first()),
        "fans_club_level": fans_club.map(|d| d.level),
        "fans_club_name": fans_club.map(|d| d.club_name.as_str()),
        "top_vip_no": user.top_vip_no,
        "pay_score": user.pay_score,
    })
}

pub fn contributor_json(c: &Contributor) -> Value {
    json!({ "rank": c.rank, "score": c.score, "delta": c.delta, "user": c.user.as_ref().map(user_json) })
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
        other => json!({ "event": format!("{other:?}").chars().take(200).collect::<String>() }),
    }
}
