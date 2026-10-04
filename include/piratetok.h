/*
 * piratetok.h — C ABI for piratetok-live-rs
 *
 * Link against libpiratetok.so (Linux), piratetok.dll (Windows),
 * or libpiratetok.dylib (macOS).
 *
 * License: 0BSD
 */

#ifndef PIRATETOK_H
#define PIRATETOK_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ---- opaque handles ---- */

typedef struct PirateTokRuntime PirateTokRuntime;
typedef struct PirateTokClient  PirateTokClient;

/* ---- error codes ---- */

typedef enum {
    PIRATETOK_OK                  = 0,
    PIRATETOK_ERR_NULL_PARAM      = 1,
    PIRATETOK_ERR_USER_NOT_FOUND  = 2,
    PIRATETOK_ERR_HOST_NOT_ONLINE = 3,
    PIRATETOK_ERR_DEVICE_BLOCKED  = 4,
    PIRATETOK_ERR_AGE_RESTRICTED  = 5,
    PIRATETOK_ERR_INVALID_RESPONSE= 6,
    PIRATETOK_ERR_CONNECTION_CLOSED=7,
    PIRATETOK_ERR_HTTP            = 8,
    PIRATETOK_ERR_WEBSOCKET       = 9,
    PIRATETOK_ERR_ALREADY_RUNNING = 10,
    PIRATETOK_ERR_SESSION_REQUIRED= 11,  /* login-gated endpoint called without session cookies */
    PIRATETOK_ERR_API_ERROR       = 12,  /* TikTok API returned a non-zero statusCode (see last_error) */
    PIRATETOK_ERR_TIKTOK_BLOCKED  = 13,  /* HTTP 403/429, empty or non-JSON body: IP/fingerprint/rate/geo block */
    PIRATETOK_ERR_PROXY           = 14,  /* proxy handshake failed (CONNECT rejected, socks5 error) */
    PIRATETOK_ERR_INTERNAL        = 99
} PirateTokError;

/* ---- event types ---- */

typedef enum {
    /* lifecycle */
    CYCLED_EVENT_CONNECTED     = 0,
    CYCLED_EVENT_RECONNECTING  = 1,
    CYCLED_EVENT_DISCONNECTED  = 2,

    /* core */
    CYCLED_EVENT_CHAT          = 10,
    CYCLED_EVENT_GIFT          = 11,
    CYCLED_EVENT_LIKE          = 12,
    CYCLED_EVENT_MEMBER        = 13,
    CYCLED_EVENT_SOCIAL        = 14,
    CYCLED_EVENT_ROOM_USER_SEQ = 15,
    CYCLED_EVENT_CONTROL       = 16,

    /* convenience (sub-routed) */
    CYCLED_EVENT_FOLLOW        = 20,
    CYCLED_EVENT_SHARE         = 21,
    CYCLED_EVENT_JOIN          = 22,
    CYCLED_EVENT_LIVE_ENDED    = 23,

    /* useful */
    CYCLED_EVENT_LIVE_INTRO    = 30,
    CYCLED_EVENT_ROOM_MESSAGE  = 31,
    CYCLED_EVENT_CAPTION       = 32,
    CYCLED_EVENT_GOAL_UPDATE   = 33,
    CYCLED_EVENT_IM_DELETE     = 34,

    /* niche */
    CYCLED_EVENT_RANK_UPDATE   = 40,
    CYCLED_EVENT_POLL          = 41,
    CYCLED_EVENT_ENVELOPE      = 42,
    CYCLED_EVENT_ROOM_PIN      = 43,
    CYCLED_EVENT_UNAUTHORIZED_MEMBER = 44,
    CYCLED_EVENT_LINK_MIC_METHOD     = 45,
    CYCLED_EVENT_LINK_MIC_BATTLE     = 46,
    CYCLED_EVENT_LINK_MIC_ARMIES     = 47,
    CYCLED_EVENT_LINK_MESSAGE        = 48,
    CYCLED_EVENT_LINK_LAYER          = 49,
    CYCLED_EVENT_LINK_MIC_LAYOUT_STATE = 50,
    CYCLED_EVENT_GIFT_PANEL_UPDATE   = 51,
    CYCLED_EVENT_IN_ROOM_BANNER      = 52,
    CYCLED_EVENT_GUIDE               = 53,

    /* extended */
    CYCLED_EVENT_EMOTE_CHAT          = 60,
    CYCLED_EVENT_QUESTION_NEW        = 61,
    CYCLED_EVENT_SUB_NOTIFY          = 62,
    CYCLED_EVENT_BARRAGE             = 63,
    CYCLED_EVENT_HOURLY_RANK         = 64,
    CYCLED_EVENT_MSG_DETECT          = 65,
    CYCLED_EVENT_LINK_MIC_FAN_TICKET = 66,
    CYCLED_EVENT_ROOM_VERIFY         = 67,
    CYCLED_EVENT_OEC_LIVE_SHOPPING   = 68,
    CYCLED_EVENT_GIFT_BROADCAST      = 69,
    CYCLED_EVENT_RANK_TEXT           = 70,
    CYCLED_EVENT_GIFT_DYNAMIC_RESTRICTION = 71,
    CYCLED_EVENT_VIEWER_PICKS_UPDATE = 72,

    /* secondary */
    CYCLED_EVENT_SYSTEM_MESSAGE      = 80,
    CYCLED_EVENT_LIVE_GAME_INTRO     = 81,
    CYCLED_EVENT_ACCESS_CONTROL      = 82,
    CYCLED_EVENT_ACCESS_RECALL       = 83,
    CYCLED_EVENT_ALERT_BOX_AUDIT_RESULT = 84,
    CYCLED_EVENT_BINDING_GIFT        = 85,
    CYCLED_EVENT_BOOST_CARD          = 86,
    CYCLED_EVENT_BOTTOM_MESSAGE      = 87,
    CYCLED_EVENT_GAME_RANK_NOTIFY    = 88,
    CYCLED_EVENT_GIFT_PROMPT         = 89,
    CYCLED_EVENT_LINK_STATE          = 90,
    CYCLED_EVENT_LINK_MIC_BATTLE_PUNISH_FINISH = 91,
    CYCLED_EVENT_LINKMIC_BATTLE_TASK = 92,
    CYCLED_EVENT_MARQUEE_ANNOUNCEMENT = 93,
    CYCLED_EVENT_NOTICE              = 94,
    CYCLED_EVENT_NOTIFY              = 95,
    CYCLED_EVENT_PARTNERSHIP_DROPS_UPDATE = 96,
    CYCLED_EVENT_PARTNERSHIP_GAME_OFFLINE = 97,
    CYCLED_EVENT_PARTNERSHIP_PUNISH  = 98,
    CYCLED_EVENT_PERCEPTION          = 99,
    CYCLED_EVENT_SPEAKER             = 100,
    CYCLED_EVENT_SUB_CAPSULE         = 101,
    CYCLED_EVENT_SUB_PIN_EVENT       = 102,
    CYCLED_EVENT_SUBSCRIPTION_NOTIFY = 103,
    CYCLED_EVENT_TOAST               = 104,

    /* any message type not decoded by live-rs: JSON has "method" + "payload_b64" */
    CYCLED_EVENT_UNKNOWN       = 255
} PirateTokEventType;

/* ---- CDN endpoints ---- */

typedef enum {
    PIRATETOK_CDN_GLOBAL = 0,
    PIRATETOK_CDN_EU     = 1,
    PIRATETOK_CDN_US     = 2
} PirateTokCdn;

/* ---- callback signature ----
 *
 * Called from a background thread for each event.
 *
 * event_type : which event (see PirateTokEventType)
 * json       : null-terminated UTF-8 JSON with event data
 * json_len   : byte length of json (excludes null terminator)
 * user_data  : pointer you passed to piratetok_connect()
 *
 * The json pointer is valid ONLY during the callback. Copy if you need it later.
 */
typedef void (*PirateTokCallback)(
    PirateTokEventType event_type,
    const char*        json,
    size_t             json_len,
    void*              user_data
);

/* ---- runtime lifecycle ---- */

/* Create the async runtime. Call once before anything else.
 * Returns NULL on failure. */
PirateTokRuntime* piratetok_init(void);

/* Shut down the runtime. Call after all clients are freed. */
void piratetok_shutdown(PirateTokRuntime* rt);

/* ---- client lifecycle ---- */

/* Create a client for the given TikTok username.
 * username is copied internally; the pointer need not remain valid. */
PirateTokClient* piratetok_client_new(
    PirateTokRuntime* rt,
    const char*       username
);

/* Free client resources. Disconnects if still running. */
void piratetok_client_free(PirateTokClient* client);

/* ---- configuration (call before piratetok_connect) ---- */

void piratetok_client_set_cdn(PirateTokClient* c, PirateTokCdn cdn);
void piratetok_client_set_timeout_ms(PirateTokClient* c, uint32_t ms);
void piratetok_client_set_heartbeat_ms(PirateTokClient* c, uint32_t ms);
void piratetok_client_set_max_retries(PirateTokClient* c, uint32_t n);
void piratetok_client_set_stale_timeout_ms(PirateTokClient* c, uint32_t ms);
void piratetok_client_set_proxy(PirateTokClient* c, const char* proxy_url);
void piratetok_client_set_user_agent(PirateTokClient* c, const char* ua);
void piratetok_client_set_cookies(PirateTokClient* c, const char* cookies);
/* language/region default to the system locale; NULL is rejected (see piratetok_last_error) */
void piratetok_client_set_language(PirateTokClient* c, const char* language);
void piratetok_client_set_region(PirateTokClient* c, const char* region);
/* request gzip WSS frames (default 1); decode handles both either way */
void piratetok_client_set_compress(PirateTokClient* c, int enabled);

/* Event JSON notes:
 *  - every event: "type" (variant name), "type_id" (PirateTokEventType) and, for
 *    message events, "payload_b64" = the full protobuf message (decode any field)
 *  - common events also carry decoded fields; users carry badges[{scene,level,...}],
 *    follower_count/following_count, fans club, follow flags
 *  - GIFT: "streak" {streak_id,is_active,is_final,event_gift_count,total_gift_count,
 *    event_diamond_count,total_diamond_count} (GiftStreakTracker, per client)
 *  - LIKE: "like_stats" {event_like_count,total_like_count (monotonic),
 *    accumulated_count,went_backwards} (LikeAccumulator, per client)
 *  - absent optional fields are JSON null (not "" or 0)
 *  - CYCLED_EVENT_ROOM_USER_SEQ carries "top_viewers": [{rank, score, delta, user}]
 *    — the top-viewers box next to the counter, sorted by rank, no cookies needed
 *  - reconnects reuse the ttwid; a missing ttwid cookie is retried 8x750ms before
 *    counting as a failed attempt; max_retries counts consecutive failures only */

/* ---- connection ---- */

/* Start streaming events. Non-blocking: events arrive via callback
 * on a background thread. Returns PIRATETOK_OK on success. */
PirateTokError piratetok_connect(
    PirateTokClient*  client,
    PirateTokCallback callback,
    void*             user_data
);

/* Stop the connection. Blocks until the background task exits. */
PirateTokError piratetok_disconnect(PirateTokClient* client);

/* ---- standalone utilities (blocking calls) ---- */

/* Check if a user is online. On success, writes the room_id string and,
 * if out_anchor_id is not NULL, the streamer's user id (needed by
 * piratetok_fetch_room_audience). Free both with piratetok_string_free(). */
PirateTokError piratetok_check_online(
    PirateTokRuntime* rt,
    const char*       username,
    char**            out_room_id,
    char**            out_anchor_id
);

/* Fetch room info as JSON. cookies may be NULL (needed only for 18+ rooms).
 * Caller must free the returned string with piratetok_string_free(). */
PirateTokError piratetok_fetch_room_info(
    PirateTokRuntime* rt,
    const char*       room_id,
    const char*       cookies,
    char**            out_json
);

/* Full audience roster (every named viewer) as JSON:
 * {"total","anonymous","viewers":[{rank,score,user_id,username,nickname,sec_uid,
 *   avatar_url,follower_count,verified,is_follower,is_following,is_subscriber}],"raw_json"}
 * Login-gated: cookies ("sessionid=...; sid_tt=...") are required, otherwise
 * PIRATETOK_ERR_SESSION_REQUIRED. anchor_id may be NULL (resolved via room info).
 * Caller must free the returned string with piratetok_string_free(). */
PirateTokError piratetok_fetch_room_audience(
    PirateTokRuntime* rt,
    const char*       room_id,
    const char*       anchor_id,
    const char*       cookies,
    char**            out_json
);

/* Profile (HD avatars + metadata) scraped from the profile page, cached per
 * runtime (ProfileCache, 5 min TTL, negative cache for private/not-found) as JSON:
 * {user_id,unique_id,nickname,bio,avatar_thumb,avatar_medium,avatar_large,verified,
 *  private_account,is_organization,room_id,bio_link,follower_count,following_count,
 *  heart_count,video_count,friend_count}. Free with piratetok_string_free(). */
PirateTokError piratetok_fetch_profile(
    PirateTokRuntime* rt,
    const char*       username,
    char**            out_json
);

/* ---- memory management ---- */

/* Free a string returned by piratetok_check_online / _fetch_room_info / _fetch_room_audience. */
void piratetok_string_free(char* s);

/* ---- error info ---- */

/* Get human-readable message for the last error on this thread.
 * Returns a static or thread-local string; do NOT free it.
 * Returns "" if no error has occurred. */
const char* piratetok_last_error(void);

#ifdef __cplusplus
}
#endif

#endif /* PIRATETOK_H */
