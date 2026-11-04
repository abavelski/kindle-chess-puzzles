#define _GNU_SOURCE
#include <ctype.h>
#include <dirent.h>
#include <dlfcn.h>
#include <errno.h>
#include <grp.h>
#include <pwd.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/utsname.h>
#include <unistd.h>
#include "bluetooth_identity.h"
#ifdef __linux__
#include <gnu/libc-version.h>
#include <kindlebt/kindlebt.h>
/* Compile against precisely the same public headers as the staged library. */
_Static_assert(sizeof(void *) == 4, "Kindle requires 32-bit pointers");
_Static_assert(sizeof(((uuid_t *)0)->uu) == 16, "UUID must hold all 128 bits");
static status_t (*const database_contract)(bleConnHandle, bleGattsService_t **, uint32_t *)
    __attribute__((unused)) = bleGetDatabase;
#endif

static int failure(const char *reason) {
    fprintf(stderr, "phase=M1 result=FAIL reason=%s\n", reason);
    return 1;
}

#ifdef __linux__
static bool manager_identity_matches(uid_t uid, gid_t gid) {
    DIR *directory = opendir("/proc");
    if (directory == NULL) return false;
    struct dirent *entry;
    bool found = false;
    while ((entry = readdir(directory)) != NULL) {
        if (!isdigit((unsigned char)entry->d_name[0])) continue;
        char path[300], name[64];
        if (snprintf(path, sizeof(path), "/proc/%s/comm", entry->d_name) >= (int)sizeof(path)) continue;
        FILE *file = fopen(path, "r");
        if (file == NULL) continue;
        bool manager = fgets(name, sizeof(name), file) != NULL && strcmp(name, "btmanagerd\n") == 0;
        fclose(file);
        if (!manager) continue;
        if (snprintf(path, sizeof(path), "/proc/%s/status", entry->d_name) >= (int)sizeof(path)) continue;
        file = fopen(path, "r");
        if (file == NULL) continue;
        found = bluetooth_status_matches(file, (unsigned long)uid, (unsigned long)gid);
        fclose(file);
        if (found) break;
    }
    closedir(directory);
    return found;
}

static int preflight(const char *library, const char *required) {
    struct utsname info;
    if (uname(&info) != 0) return failure("uname_failed");
    printf("uname_m=%s kernel=%s\n", info.machine, info.release);
    if (strcmp(info.machine, "armv7l") != 0) return failure("unsupported_architecture");
    const char *libc = gnu_get_libc_version();
    printf("glibc=%s loader=/lib/ld-linux-armhf.so.3\n", libc);
    unsigned int major = 0, minor = 0;
    if (sscanf(libc, "%u.%u", &major, &minor) != 2 || major != 2 || minor < 35)
        return failure("glibc_below_build_baseline");
    if (access("/lib/ld-linux-armhf.so.3", X_OK) != 0) return failure("missing_hard_float_loader");
    FILE *firmware = fopen("/etc/prettyversion.txt", "r");
    if (firmware == NULL) firmware = fopen("/etc/prettyversion", "r");
    if (firmware == NULL) return failure("firmware_file_unreadable");
    char text[256];
    if (fgets(text, sizeof(text), firmware) == NULL) { fclose(firmware); return failure("firmware_file_empty"); }
    fclose(firmware);
    text[strcspn(text, "\r\n")] = '\0';
    printf("firmware=%s\n", text);
    struct passwd *account = getpwnam("bluetooth");
    if (account == NULL) return failure("bluetooth_account_missing");
    uid_t uid = account->pw_uid;
    gid_t primary = account->pw_gid;
    struct group *group = getgrnam("bluetooth");
    if (group == NULL || uid == 0 || group->gr_gid == 0)
        return failure("bluetooth_mapping_invalid_or_ambiguous");
    gid_t gid = group->gr_gid;
    printf("bluetooth_uid=%lu bluetooth_gid=%lu account_primary_gid=%lu\n", (unsigned long)uid, (unsigned long)gid, (unsigned long)primary);
    if (!manager_identity_matches(uid, gid)) return failure("btmanagerd_identity_not_verified_no_services_changed");
    puts("btmanagerd=present identity=verified");
    /* Only this worker changes identity; the controller/SSH process is untouched. */
    if (geteuid() != 0) return failure("run_preflight_from_root_ssh_to_verify_worker_privileges");
    if (setgroups(0, NULL) != 0 || setgid(gid) != 0 || setuid(uid) != 0)
        return failure("worker_privilege_transition_failed");
    if (getuid() != uid || geteuid() != uid || getgid() != gid || getegid() != gid || getgroups(0, NULL) != 0)
        return failure("worker_privilege_verification_failed");
    puts("worker_privileges=verified");
    FILE *libs = fopen(required, "r");
    if (libs == NULL) return failure("required_library_list_unreadable_after_privilege_drop");
    unsigned int count = 0;
    while (fgets(text, sizeof(text), libs) != NULL) {
        size_t len = strcspn(text, "\r\n");
        text[len] = '\0';
        if (len == 0 || len >= sizeof(text)-2 || ++count > 128) { fclose(libs); return failure("invalid_library_list"); }
        for (size_t i = 0; i < len; ++i) {
            if (!isalnum((unsigned char)text[i]) && strchr("_.+-", text[i]) == NULL) { fclose(libs); return failure("unsafe_library_name"); }
        }
        void *dependency = dlopen(text, RTLD_NOW | RTLD_LOCAL);
        if (dependency == NULL) {
            fprintf(stderr, "dependency=%s load_error=%s\n", text, dlerror());
            fclose(libs);
            return failure("device_dependency_unloadable");
        }
        printf("dependency=%s load=PASS\n", text);
        dlclose(dependency);
    }
    if (ferror(libs) || count == 0) { fclose(libs); return failure("invalid_library_list"); }
    fclose(libs);
    void *handle = dlopen(library, RTLD_NOW | RTLD_LOCAL);
    if (handle == NULL) {
        fprintf(stderr, "kindlebt_load_error=%s\n", dlerror());
        return failure("kindlebt_dlopen_failed");
    }
    const char *symbols[] = {
        "openSession", "closeSession", "bleRegister", "bleDeregister",
        "bleRegisterGattClient", "bleDeregisterGattClient", "bleConnect", "bleDisconnect",
        "bleDiscoverAllServices", "bleGetDatabase", "bleCleanupGattService",
        "bleWriteCharacteristic", "bleSetNotification"
    };
    for (size_t i = 0; i < sizeof(symbols)/sizeof(symbols[0]); ++i) {
        dlerror();
        if (dlsym(handle, symbols[i]) == NULL || dlerror() != NULL) {
            dlclose(handle);
            return failure("kindlebt_required_symbol_missing");
        }
    }
    dlclose(handle);
    puts("kindlebt_load=PASS required_symbols=PASS session=NOT_OPENED connection=NOT_ATTEMPTED result=PASS");
    return 0;
}
#endif

#ifdef __linux__
#include "kindlebt_session.h"
#endif

int main(int argc, char **argv) {
    /* Only preflight or the discovery-only M2 IPC worker can be selected. */
    setvbuf(stdout, NULL, _IOLBF, 0);
    if (argc != 4 || (strcmp(argv[1], "--preflight") != 0 && strcmp(argv[1], "--ipc") != 0))
        return failure("UNSUPPORTED_worker_mode");
#ifdef __linux__
    if (strcmp(argv[1], "--ipc") == 0) return session_worker(argv[2], argv[3]);
    return preflight(argv[2], argv[3]);
#else
    return failure("UNSUPPORTED_requires_Kindle_Linux");
#endif
}
