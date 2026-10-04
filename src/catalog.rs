use base64::Engine;
use piratetok_live_rs::structs::TikTokLiveEvent;
use prost::Message;

pub struct Described {
    pub id: i32,
    pub name: &'static str,
    pub payload: Vec<u8>,
}

pub struct Lifecycle;

impl Lifecycle {
    pub const CONNECTED: i32 = 0;
    pub const RECONNECTING: i32 = 1;
    pub const DISCONNECTED: i32 = 2;
    pub const UNKNOWN: i32 = 255;
}

macro_rules! catalog {
    ($($variant:ident = $id:literal),* $(,)?) => {
        pub fn describe(ev: &TikTokLiveEvent) -> Described {
            match ev {
                TikTokLiveEvent::Connected { .. } => Described { id: Lifecycle::CONNECTED, name: "Connected", payload: Vec::new() },
                TikTokLiveEvent::Reconnecting { .. } => Described { id: Lifecycle::RECONNECTING, name: "Reconnecting", payload: Vec::new() },
                TikTokLiveEvent::Disconnected => Described { id: Lifecycle::DISCONNECTED, name: "Disconnected", payload: Vec::new() },
                TikTokLiveEvent::Unknown { payload, .. } => Described { id: Lifecycle::UNKNOWN, name: "Unknown", payload: payload.clone() },
                $(TikTokLiveEvent::$variant(msg) => Described { id: $id, name: stringify!($variant), payload: msg.encode_to_vec() },)*
            }
        }

        pub fn all_ids() -> Vec<(&'static str, i32)> {
            vec![$((stringify!($variant), $id)),*]
        }
    };
}

catalog! {
    Chat = 10, Gift = 11, Like = 12, Member = 13, Social = 14, RoomUserSeq = 15, Control = 16,
    Follow = 20, Share = 21, Join = 22, LiveEnded = 23,
    LiveIntro = 30, RoomMessage = 31, Caption = 32, GoalUpdate = 33, ImDelete = 34,
    RankUpdate = 40, Poll = 41, Envelope = 42, RoomPin = 43, UnauthorizedMember = 44, LinkMicMethod = 45,
    LinkMicBattle = 46, LinkMicArmies = 47, LinkMessage = 48, LinkLayer = 49, LinkMicLayoutState = 50,
    GiftPanelUpdate = 51, InRoomBanner = 52, Guide = 53,
    EmoteChat = 60, QuestionNew = 61, SubNotify = 62, Barrage = 63, HourlyRank = 64, MsgDetect = 65,
    LinkMicFanTicket = 66, RoomVerify = 67, OecLiveShopping = 68, GiftBroadcast = 69, RankText = 70,
    GiftDynamicRestriction = 71, ViewerPicksUpdate = 72,
    SystemMessage = 80, LiveGameIntro = 81, AccessControl = 82, AccessRecall = 83, AlertBoxAuditResult = 84,
    BindingGift = 85, BoostCard = 86, BottomMessage = 87, GameRankNotify = 88, GiftPrompt = 89, LinkState = 90,
    LinkMicBattlePunishFinish = 91, LinkmicBattleTask = 92, MarqueeAnnouncement = 93, Notice = 94, Notify = 95,
    PartnershipDropsUpdate = 96, PartnershipGameOffline = 97, PartnershipPunish = 98, Perception = 99,
    Speaker = 100, SubCapsule = 101, SubPinEvent = 102, SubscriptionNotify = 103, Toast = 104,
}

pub fn payload_b64(described: &Described) -> String {
    base64::engine::general_purpose::STANDARD.encode(&described.payload)
}
