#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "bluetooth_identity.h"

static bool check(const char *text, unsigned long uid, unsigned long gid) {
    FILE *file = tmpfile();
    assert(file != NULL);
    assert(fwrite(text, 1, strlen(text), file) == strlen(text));
    rewind(file);
    bool result = bluetooth_status_matches(file, uid, gid);
    fclose(file);
    return result;
}
int main(void) {
    /* Actual btmanagerd fields. The passwd account primary GID is 150;
       the daemon's verified group is the named bluetooth group's 1003. */
    const char *observed = "Name:\tbtmanagerd\nUid:\t1003\t1003\t1003\t1003\nGid:\t1003\t1003\t1003\t1003\nGroups:\t150 1003\n";
    assert(check(observed, 1003, 1003));
    assert(!check(observed, 1003, 150));
    assert(!check(observed, 1004, 1003));
    assert(!check(observed, 0, 1003));
    assert(!check(observed, 1003, 0));
    assert(!check("Uid: 1003 1003 1003\nGid: 1003 1003 1003 1003\n", 1003, 1003));
    assert(!check("Uid: 1003 1003 1003 1003\nGid: 1003 0 1003 1003\n", 1003, 1003));
    assert(!check("Uid: 1003 1003 1003 1003\nGid: 1003 1003 1003 1003 extra\n", 1003, 1003));
    assert(!check("Uid: 1003 1003 1003 1003\n", 1003, 1003));
    assert(!check("Uid: 1003 1003 1003 1003\nUid: 1003 1003 1003 1003\nGid: 1003 1003 1003 1003\n", 1003, 1003));
    return 0;
}
