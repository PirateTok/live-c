# Changelog

## 0.4.0

- Builds on `piratetok-live-rs` 0.5. New error codes: `PIRATETOK_ERR_API_ERROR` (12, non-zero statusCode), `PIRATETOK_ERR_TIKTOK_BLOCKED` (13, HTTP 403/429 / empty / non-JSON), `PIRATETOK_ERR_PROXY` (14).
- `piratetok_client_set_proxy` now also takes `socks5://` / `socks5h://` and `user:pass@` for both HTTP and the WSS tunnel (via live-rs 0.5; proven there by `proxy_test` + `client_offline_test`).
- `piratetok_fetch_room_audience` with `anchor_id == NULL` resolves the anchor from room info (`AnchorId::FromRoomInfo`).

## 0.3.0

### Breaking (C ABI)
- `PirateTokEventType`: every live-rs event variant has its own id now (44–53, 60–72, 80–104); `CYCLED_EVENT_OTHER` (200) is gone.
- Event JSON keys: `type`, `type_id`, `payload_b64` on every event; the `{"event": "<debug>"}` fallback became `"debug"` next to the full payload.

### Added (parity with live-rs)
- Full protobuf payload (`payload_b64`) for every message event, Tier A/B and Unknown alike, so callers can decode any field.
- Enriched user JSON: `badges[{scene,display_type,display,level}]`, `follower_count`, `following_count`, `id_str`, `sec_uid`.
- GIFT `streak` (GiftStreakTracker) and LIKE `like_stats` (LikeAccumulator), tracked per client.
- `piratetok_fetch_profile(rt, username, &json)`: ProfileCache per runtime.
- Examples: `gift_streak.c`, `profile_lookup.c`.
- Tests: `tests/replay_test.rs` replays all 6 live-testdata captures through the C JSON layer and checks event counts, sub-routing, like stats event by event, and gift streak groups against the manifests exactly. It fails if `testdata/` is missing. `catalog` test: 68 unique variant ids.

## 0.2.0

### Fixed
- Builds again: 0.1.0 targeted a pre-`FetchParams` live-rs API and no longer compiled. Now depends on `piratetok-live-rs = "0.4"` from crates.io (was `path = "../live-rs"`).

### Breaking (C ABI)
- `piratetok_check_online(rt, username, &room_id, &anchor_id)`: new 4th out-param (may be `NULL`).
- Event JSON: absent optional fields are `null` instead of `""` / `0` (e.g. `gift_name`, `diamond_count`, `avatar`, `fans_club_level`, `msg_id`).
- `LIVE_INTRO` JSON: `host` comes from the intro's `user`, `language` was added, `room_id` was removed (it's no longer in the proto). `MEMBER`/`JOIN` JSON now carries `user` and the real `msg_id`.

### Added (parity with live-rs)
- `piratetok_client_set_language`, `piratetok_client_set_region`, `piratetok_client_set_compress`.
- `piratetok_fetch_room_audience(rt, room_id, anchor_id, cookies, &json)` + `PIRATETOK_ERR_SESSION_REQUIRED = 11`.
- `ROOM_USER_SEQ` JSON: `top_viewers` (sorted by rank, entries without a user skipped) and `anonymous`.
- Reconnect behavior inherited from live-rs 0.4: ttwid retry, session reuse, consecutive-failure budget.
- `examples/audience.c`; offline FFI tests (`cargo test`); crate also builds as `rlib` for those tests.

### Internal
- Source split into modules under lawkeeper rules (no logic in `lib.rs`, no silent error handling, no comments in `src/`). Setter argument errors are reported via `piratetok_last_error()`.
- Homepage → https://piratetok.rosint.org.
