/*
 * online_check.c — check if a TikTok user is currently live.
 *
 * Build:
 *   gcc -o online_check online_check.c -L../target/release -lpiratetok -lpthread -ldl -lm
 *
 * Run:
 *   LD_LIBRARY_PATH=../target/release ./online_check <username> [username2] ...
 */

#include <stdio.h>
#include <stdlib.h>
#include "../include/piratetok.h"

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <username> [username2] ...\n", argv[0]);
        return 1;
    }

    PirateTokRuntime* rt = piratetok_init();
    if (!rt) {
        fprintf(stderr, "failed to init runtime: %s\n", piratetok_last_error());
        return 1;
    }

    for (int i = 1; i < argc; i++) {
        char* room_id = NULL;
        PirateTokError err = piratetok_check_online(rt, argv[i], &room_id);

        if (err == PIRATETOK_OK) {
            printf("  LIVE  @%s — room %s\n", argv[i], room_id);
            piratetok_string_free(room_id);
        } else if (err == PIRATETOK_ERR_USER_NOT_FOUND) {
            printf("  404   @%s — user does not exist\n", argv[i]);
        } else if (err == PIRATETOK_ERR_HOST_NOT_ONLINE) {
            printf("  OFF   @%s — not currently live\n", argv[i]);
        } else {
            printf("  ERR   @%s — %s\n", argv[i], piratetok_last_error());
        }
    }

    piratetok_shutdown(rt);
    return 0;
}
