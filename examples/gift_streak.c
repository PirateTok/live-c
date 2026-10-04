/*
 * gift_streak.c — per-event gift deltas from the built-in GiftStreakTracker
 *
 *   ./gift_streak <username>
 *
 * Every GIFT event JSON carries "streak": event_gift_count is the delta for
 * this event (TikTok only sends running totals), is_final marks the end.
 */
#include <stdio.h>
#include <string.h>
#include "../include/piratetok.h"

static void on_event(PirateTokEventType type, const char* json, size_t len, void* user_data) {
    (void)len;
    (void)user_data;
    if (type == CYCLED_EVENT_GIFT) {
        const char* streak = strstr(json, "\"streak\"");
        printf("[gift] %s\n", streak ? streak : json);
    } else if (type == CYCLED_EVENT_DISCONNECTED) {
        printf("[disconnected] %s\n", json);
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <username>\n", argv[0]);
        return 1;
    }
    PirateTokRuntime* rt = piratetok_init();
    if (!rt) {
        fprintf(stderr, "init failed: %s\n", piratetok_last_error());
        return 1;
    }
    PirateTokClient* client = piratetok_client_new(rt, argv[1]);
    if (piratetok_connect(client, on_event, NULL) != PIRATETOK_OK) {
        fprintf(stderr, "connect failed: %s\n", piratetok_last_error());
        return 1;
    }
    printf("tracking gift streaks, press Enter to stop\n");
    getchar();
    piratetok_disconnect(client);
    piratetok_client_free(client);
    piratetok_shutdown(rt);
    return 0;
}
