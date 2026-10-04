/*
 * stream_info.c — fetch room metadata and stream URLs.
 *
 * Build:
 *   gcc -o stream_info stream_info.c -L../target/release -lpiratetok -lpthread -ldl -lm
 *
 * Run:
 *   LD_LIBRARY_PATH=../target/release ./stream_info <username> [cookies]
 */

#include <stdio.h>
#include <stdlib.h>
#include "../include/piratetok.h"

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <username> [cookies]\n", argv[0]);
        return 1;
    }

    const char* username = argv[1];
    const char* cookies = argc > 2 ? argv[2] : NULL;

    PirateTokRuntime* rt = piratetok_init();
    if (!rt) {
        fprintf(stderr, "failed to init runtime: %s\n", piratetok_last_error());
        return 1;
    }

    char* room_id = NULL;
    PirateTokError err = piratetok_check_online(rt, username, &room_id, NULL);
    if (err != PIRATETOK_OK) {
        fprintf(stderr, "check_online failed (code %d): %s\n", err, piratetok_last_error());
        piratetok_shutdown(rt);
        return 1;
    }

    printf("=== Room Info ===\n");
    printf("Username: @%s\n", username);
    printf("Room ID:  %s\n", room_id);

    char* json = NULL;
    err = piratetok_fetch_room_info(rt, room_id, cookies, &json);
    piratetok_string_free(room_id);

    if (err == PIRATETOK_OK) {
        printf("\n%s\n", json);
        piratetok_string_free(json);
    } else {
        fprintf(stderr, "room info failed (code %d): %s\n", err, piratetok_last_error());
        if (!cookies) {
            fprintf(stderr, "hint: if this is an 18+ room, pass session cookies as the second argument\n");
        }
        piratetok_shutdown(rt);
        return 1;
    }

    piratetok_shutdown(rt);
    return 0;
}
