/*
 * basic_chat.c — connect to a TikTok Live room and print chat events.
 *
 * Build:
 *   gcc -o basic_chat basic_chat.c -L../target/release -lpiratetok -lpthread -ldl -lm
 *
 * Run:
 *   LD_LIBRARY_PATH=../target/release ./basic_chat <username>
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
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
    case CYCLED_EVENT_CHAT:
        printf("[chat] %s\n", json);
        break;
    case CYCLED_EVENT_GIFT:
        printf("[gift] %s\n", json);
        break;
    case CYCLED_EVENT_LIKE:
        printf("[like] %s\n", json);
        break;
    case CYCLED_EVENT_JOIN:
        printf("[join] %s\n", json);
        break;
    case CYCLED_EVENT_FOLLOW:
        printf("[follow] %s\n", json);
        break;
    case CYCLED_EVENT_SHARE:
        printf("[share] %s\n", json);
        break;
    case CYCLED_EVENT_ROOM_USER_SEQ:
        printf("[viewers] %s\n", json);
        break;
    case CYCLED_EVENT_LIVE_ENDED:
        printf("[live ended] %s\n", json);
        break;
    case CYCLED_EVENT_RECONNECTING:
        printf("[reconnecting] %s\n", json);
        break;
    case CYCLED_EVENT_DISCONNECTED:
        printf("[disconnected] %s\n", json);
        break;
    default:
        /* skip niche events */
        break;
    }

    fflush(stdout);
}

static void sigint_handler(int sig) {
    (void)sig;
    printf("\ncaught SIGINT, disconnecting...\n");
    if (g_client) {
        piratetok_disconnect(g_client);
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <tiktok_username>\n", argv[0]);
        return 1;
    }
    const char* username = argv[1];

    signal(SIGINT, sigint_handler);

    PirateTokRuntime* rt = piratetok_init();
    if (!rt) {
        fprintf(stderr, "failed to init runtime: %s\n", piratetok_last_error());
        return 1;
    }

    /* optional: check if user is online first */
    char* room_id = NULL;
    PirateTokError err = piratetok_check_online(rt, username, &room_id, NULL);
    if (err != PIRATETOK_OK) {
        fprintf(stderr, "check_online failed (code %d): %s\n", err, piratetok_last_error());
        piratetok_shutdown(rt);
        return 1;
    }
    printf("user '%s' is live, room_id=%s\n", username, room_id);
    piratetok_string_free(room_id);

    /* connect and stream events */
    g_client = piratetok_client_new(rt, username);
    if (!g_client) {
        fprintf(stderr, "failed to create client: %s\n", piratetok_last_error());
        piratetok_shutdown(rt);
        return 1;
    }

    err = piratetok_connect(g_client, on_event, NULL);
    if (err != PIRATETOK_OK) {
        fprintf(stderr, "connect failed (code %d): %s\n", err, piratetok_last_error());
        piratetok_client_free(g_client);
        piratetok_shutdown(rt);
        return 1;
    }

    printf("streaming events... press Ctrl+C to stop\n");

    /* block main thread until SIGINT */
    pause();

    piratetok_client_free(g_client);
    g_client = NULL;
    piratetok_shutdown(rt);

    return 0;
}
