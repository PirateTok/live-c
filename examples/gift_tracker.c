/*
 * gift_tracker.c — connect and print gift events as JSON.
 *
 * The C ABI delivers events as JSON strings, so gift tracking logic
 * (combo detection, diamond totals) is done by parsing the JSON.
 * For a simpler approach, just print the raw JSON and post-process.
 *
 * Build:
 *   gcc -o gift_tracker gift_tracker.c -L../target/release -lpiratetok -lpthread -ldl -lm
 *
 * Run:
 *   LD_LIBRARY_PATH=../target/release ./gift_tracker <username>
 */

#include <stdio.h>
#include <stdlib.h>
#include <signal.h>
#include <unistd.h>
#include "../include/piratetok.h"

static PirateTokClient* g_client = NULL;

static void on_event(
    PirateTokEventType type,
    const char* json,
    size_t json_len,
    void* user_data
) {
    (void)json_len;
    (void)user_data;

    switch (type) {
    case CYCLED_EVENT_CONNECTED:
        printf("[connected] %s\n", json);
        break;
    case CYCLED_EVENT_GIFT:
        printf("[gift] %s\n", json);
        break;
    case CYCLED_EVENT_LIVE_ENDED:
        printf("[live ended]\n");
        break;
    case CYCLED_EVENT_DISCONNECTED:
        printf("[disconnected]\n");
        break;
    default:
        break;
    }

    fflush(stdout);
}

static void sigint_handler(int sig) {
    (void)sig;
    if (g_client) {
        piratetok_disconnect(g_client);
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <username>\n", argv[0]);
        return 1;
    }

    signal(SIGINT, sigint_handler);

    PirateTokRuntime* rt = piratetok_init();
    if (!rt) {
        fprintf(stderr, "failed to init runtime: %s\n", piratetok_last_error());
        return 1;
    }

    g_client = piratetok_client_new(rt, argv[1]);
    if (!g_client) {
        fprintf(stderr, "failed to create client: %s\n", piratetok_last_error());
        piratetok_shutdown(rt);
        return 1;
    }

    PirateTokError err = piratetok_connect(g_client, on_event, NULL);
    if (err != PIRATETOK_OK) {
        fprintf(stderr, "connect failed (code %d): %s\n", err, piratetok_last_error());
        piratetok_client_free(g_client);
        piratetok_shutdown(rt);
        return 1;
    }

    printf("tracking gifts... press Ctrl+C to stop\n");
    pause();

    piratetok_client_free(g_client);
    g_client = NULL;
    piratetok_shutdown(rt);
    return 0;
}
