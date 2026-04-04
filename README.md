<p align="center">
  <img src="https://raw.githubusercontent.com/PirateTok/.github/main/profile/assets/og-banner-v2.png" alt="PirateTok" width="640" />
</p>

# libpiratetok

C ABI for `piratetok-live-rs`. Link against `libpiratetok.so` (Linux), `piratetok.dll` (Windows), or `libpiratetok.dylib` (macOS) and receive TikTok Live events as JSON from any language with C FFI.

```c
#include "piratetok.h"
#include <stdio.h>

// Callback fires on a background thread — events arrive as JSON strings
void on_event(PirateTokEventType type, const char* json, size_t len, void* ud) {
    switch (type) {
    case CYCLED_EVENT_CHAT:   printf("[chat] %s\n", json); break;
    case CYCLED_EVENT_GIFT:   printf("[gift] %s\n", json); break;
    case CYCLED_EVENT_LIKE:   printf("[like] %s\n", json); break;
    default: break; // 60+ other event types available
    }
}

int main(void) {
    // Initialize the async runtime (Tokio under the hood)
    PirateTokRuntime* rt = piratetok_init();

    // Create client and connect — non-blocking, events arrive via callback
    PirateTokClient* c = piratetok_client_new(rt, "username_here");
    piratetok_connect(c, on_event, NULL);

    pause(); // wait for signal (Ctrl+C)

    // Clean up
    piratetok_client_free(c);
    piratetok_shutdown(rt);
}
```

## Other languages

| Language | Install | Repo |
|:---------|:--------|:-----|
| **Rust** | `cargo add piratetok-live-rs` | [live-rs](https://github.com/PirateTok/live-rs) |
| **Go** | `go get github.com/PirateTok/live-go` | [live-go](https://github.com/PirateTok/live-go) |
| **Python** | `pip install piratetok-live-py` | [live-py](https://github.com/PirateTok/live-py) |
| **JavaScript** | `npm install piratetok-live-js` | [live-js](https://github.com/PirateTok/live-js) |
| **C#** | `dotnet add package PirateTok.Live` | [live-cs](https://github.com/PirateTok/live-cs) |
| **Java** | `com.piratetok:live` | [live-java](https://github.com/PirateTok/live-java) |
| **Lua** | `luarocks install piratetok-live-lua` | [live-lua](https://github.com/PirateTok/live-lua) |
| **Elixir** | `{:piratetok_live, "~> 0.1"}` | [live-ex](https://github.com/PirateTok/live-ex) |
| **Dart** | `dart pub add piratetok_live` | [live-dart](https://github.com/PirateTok/live-dart) |
| **PowerShell** | `Install-Module PirateTok.Live` | [live-ps1](https://github.com/PirateTok/live-ps1) |
| **Shell** | `bpkg install PirateTok/live-sh` | [live-sh](https://github.com/PirateTok/live-sh) |

## Build

Requires Rust toolchain to compile the cdylib:

```bash
cargo build --release
```

Produces `target/release/libpiratetok.so` (or `.dll`/`.dylib`).

Link your C code against it:

```bash
gcc -o basic_chat basic_chat.c -L target/release -lpiratetok -lpthread -ldl -lm
LD_LIBRARY_PATH=target/release ./basic_chat username
```

## API

See [`include/piratetok.h`](include/piratetok.h) for the full C header.

**Lifecycle:**
- `piratetok_init()` -- create async runtime (call once)
- `piratetok_client_new(rt, username)` -- create client for a TikTok user
- `piratetok_connect(client, callback, user_data)` -- start streaming events (non-blocking)
- `piratetok_disconnect(client)` -- stop connection (blocking)
- `piratetok_client_free(client)` -- free client resources
- `piratetok_shutdown(rt)` -- shut down runtime

**Configuration (call before connect):**
- `piratetok_client_set_cdn(c, PIRATETOK_CDN_EU)`
- `piratetok_client_set_timeout_ms(c, 15000)`
- `piratetok_client_set_max_retries(c, 10)`
- `piratetok_client_set_proxy(c, "socks5://127.0.0.1:1080")`

**Utilities:**
- `piratetok_check_online(rt, username, &room_id)` -- check if user is live
- `piratetok_fetch_room_info(rt, room_id, cookies, &json)` -- fetch room metadata
- `piratetok_string_free(s)` -- free strings returned by utilities

Events arrive as null-terminated JSON on a background thread via the callback. The `PirateTokEventType` enum matches the event types from the Rust lib.

## Examples

Build all examples:

```bash
cargo build --release   # build libpiratetok first
cd examples
gcc -o basic_chat basic_chat.c -L../target/release -lpiratetok -lpthread -ldl -lm
gcc -o online_check online_check.c -L../target/release -lpiratetok -lpthread -ldl -lm
gcc -o stream_info stream_info.c -L../target/release -lpiratetok -lpthread -ldl -lm
gcc -o gift_tracker gift_tracker.c -L../target/release -lpiratetok -lpthread -ldl -lm
```

Run (set library path first):

```bash
export LD_LIBRARY_PATH=../target/release
./basic_chat <username>       # connect + print chat events
./online_check <username>     # check if user is live
./stream_info <username>      # fetch room metadata + stream URLs
./gift_tracker <username>     # track gifts as JSON
```

## License

0BSD
