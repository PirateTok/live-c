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

    /* everything else */
    CYCLED_EVENT_OTHER         = 200,
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

/* Check if a user is online. On success, writes the room_id string.
 * Caller must free the returned string with piratetok_string_free(). */
PirateTokError piratetok_check_online(
    PirateTokRuntime* rt,
    const char*       username,
    char**            out_room_id
);

/* Fetch room info as JSON. cookies may be NULL (needed only for 18+ rooms).
 * Caller must free the returned string with piratetok_string_free(). */
PirateTokError piratetok_fetch_room_info(
    PirateTokRuntime* rt,
    const char*       room_id,
    const char*       cookies,
    char**            out_json
);

/* ---- memory management ---- */

/* Free a string returned by piratetok_check_online / piratetok_fetch_room_info. */
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
