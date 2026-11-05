#ifndef CHESSNUT_BLUETOOTH_IDENTITY_H
#define CHESSNUT_BLUETOOTH_IDENTITY_H
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

/* Validate the kernel's four real/effective/saved/filesystem IDs. The
   account's passwd primary group need not be the daemon's effective group. */
static bool bluetooth_status_matches(FILE *status, unsigned long uid, unsigned long gid) {
    if (uid == 0 || gid == 0) return false;
    char line[256];
    bool seen_uid = false, seen_gid = false;
    while (fgets(line, sizeof(line), status) != NULL) {
        if (strchr(line, '\n') == NULL && !feof(status)) return false;
        bool is_uid = strncmp(line, "Uid:", 4) == 0;
        bool is_gid = strncmp(line, "Gid:", 4) == 0;
        if (!is_uid && !is_gid) continue;
        if ((is_uid && seen_uid) || (is_gid && seen_gid)) return false;
        unsigned long real, effective, saved, filesystem;
        char extra;
        if (sscanf(line + 4, "%lu %lu %lu %lu %c", &real, &effective, &saved, &filesystem, &extra) != 4)
            return false;
        unsigned long expected = is_uid ? uid : gid;
        if (real != expected || effective != expected || saved != expected || filesystem != expected)
            return false;
        if (is_uid) seen_uid = true;
        if (is_gid) seen_gid = true;
    }
    return !ferror(status) && seen_uid && seen_gid;
}
#endif
