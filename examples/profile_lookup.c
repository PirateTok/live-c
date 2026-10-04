/*
 * profile_lookup.c — profile metadata + HD avatars via the cached profile scraper
 *
 *   ./profile_lookup <username> [username...]
 *
 * Second lookup of the same name is served from the runtime's ProfileCache.
 */
#include <stdio.h>
#include "../include/piratetok.h"

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s <username> [username...]\n", argv[0]);
        return 1;
    }
    PirateTokRuntime* rt = piratetok_init();
    if (!rt) {
        fprintf(stderr, "init failed: %s\n", piratetok_last_error());
        return 1;
    }
    for (int round = 0; round < 2; round++) {
        for (int i = 1; i < argc; i++) {
            char* json = NULL;
            PirateTokError err = piratetok_fetch_profile(rt, argv[i], &json);
            if (err == PIRATETOK_OK) {
                printf("[%s] %s\n", round == 0 ? "fetch" : "cache", json);
                piratetok_string_free(json);
            } else {
                printf("[%s] @%s failed: %d %s\n", round == 0 ? "fetch" : "cache", argv[i], err, piratetok_last_error());
            }
        }
    }
    piratetok_shutdown(rt);
    return 0;
}
