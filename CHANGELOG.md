# Changelog

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
