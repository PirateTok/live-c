/*
 * audience.c — viewer names: full roster (login-gated) + live top-viewers box
 *
 *   ./audience <username> "sessionid=...; sid_tt=..."
 *
 * The roster needs session cookies; without them you get
 * PIRATETOK_ERR_SESSION_REQUIRED. The top-viewers box arrives for free on
 * every ROOM_USER_SEQ event ("top_viewers" in the JSON).
 */
#include <stdio.h>
#include <string.h>
#include "../include/piratetok.h"

static int seen = 0;

static void on_event(PirateTokEventType type, const char* json, size_t len, void* user_data) {
    (void)len;
    (void)user_data;
    if (type == CYCLED_EVENT_ROOM_USER_SEQ && seen < 3) {
        printf("[room_user_seq] %s\n", json);
        seen++;
    }
    if (type == CYCLED_EVENT_DISCONNECTED) {
        printf("[disconnected] %s\n", json);
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <username> [cookies]\n", argv[0]);
        return 1;
    }
    const char* cookies = argc > 2 ? argv[2] : NULL;

    PirateTokRuntime* rt = piratetok_init();
    if (!rt) {
        fprintf(stderr, "init failed: %s\n", piratetok_last_error());
        return 1;
    }

    char* room_id = NULL;
    char* anchor_id = NULL;
    PirateTokError err = piratetok_check_online(rt, argv[1], &room_id, &anchor_id);
    if (err != PIRATETOK_OK) {
        fprintf(stderr, "check_online: %d %s\n", err, piratetok_last_error());
        piratetok_shutdown(rt);
        return 1;
    }
    printf("room_id=%s anchor_id=%s\n", room_id, anchor_id);

    char* roster = NULL;
    err = piratetok_fetch_room_audience(rt, room_id, anchor_id, cookies, &roster);
    if (err == PIRATETOK_OK) {
        printf("[roster] %.400s...\n", roster);
        piratetok_string_free(roster);
    } else if (err == PIRATETOK_ERR_SESSION_REQUIRED) {
        printf("[roster] login required — pass session cookies\n");
    } else {
        printf("[roster] failed: %d %s\n", err, piratetok_last_error());
    }

    PirateTokClient* client = piratetok_client_new(rt, argv[1]);
    piratetok_connect(client, on_event, NULL);
    printf("streaming top viewers, press Enter to stop\n");
    getchar();
    piratetok_disconnect(client);
    piratetok_client_free(client);

    piratetok_string_free(room_id);
    piratetok_string_free(anchor_id);
    piratetok_shutdown(rt);
    return 0;
}
