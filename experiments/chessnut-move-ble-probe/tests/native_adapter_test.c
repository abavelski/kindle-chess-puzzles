/* Built/run against the pinned real public headers, never guessed FFI types. */
#define _GNU_SOURCE
#include <assert.h>
#include <ctype.h>
#include <dlfcn.h>
#include <errno.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/types.h>
#include <pthread.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <time.h>
/* Darwin also defines uuid_t. Rename only the public header's typedef token
 * in this host test, preserving its actual structure and every field type. */
#define uuid_t probe_kindle_uuid_t
#include <kindlebt/kindlebt.h>
static int preflight(const char *library, const char *required) { (void)library; (void)required; return 0; }
#include "kindlebt_session.h"
int main(void) {
    /* Reference every production entry point so -Werror remains enabled. */
    (void)session_worker;
    char uuid[37];
    uuid_t source={.type=ACEBT_UUID_TYPE_128,.uu={0x12,0x34,0x56,0x78,0x90,0xab,0xcd,0xef,0x11,0x22,0x33,0x44,0x55,0x66,0x77,0x88}};
    assert(uuid_text(&source,uuid));
    assert(strcmp(uuid,"12345678-90ab-cdef-1122-334455667788")==0);
    source.type=ACEBT_UUID_TYPE_16;
    assert(uuid_text(&source,uuid)); assert(strcmp(uuid,"1234")==0);
    source.type=ACEBT_UUID_TYPE_32;
    assert(uuid_text(&source,uuid)); assert(strcmp(uuid,"12345678")==0);
    source.type=(__typeof__(source.type))99; assert(!uuid_text(&source,uuid));
    source.type=ACEBT_UUID_TYPE_128;
    bleGattsService_t service={.uuid=source,.no_characteristics=1};
    struct aceBT_gattCharRec_t record={0}; record.value.gattRecord.uuid=source;
    record.value.gattRecord.handle=5; record.value.gattRecord.attProp=0x0c;
    STAILQ_INIT(&service.charsList); STAILQ_INSERT_TAIL(&service.charsList,&record,link);
    struct probe_native n={.database=&service,.count=1}; char frame[4097];
    assert(database_report(&n,7,frame));
    assert(strstr(frame,"\"id\":7")!=NULL); assert(strstr(frame,"\"properties\":12,\"handle\":5")!=NULL);
    assert(strlen(frame)<=4096 && frame[strlen(frame)-1]=='\n');
    service.no_characteristics=0; assert(!database_report(&n,7,frame));
    service.no_characteristics=2; assert(!database_report(&n,7,frame));
    service.no_characteristics=20; STAILQ_NEXT(&record,link)=&record;
    assert(!database_report(&n,7,frame)); /* cycle cannot run forever */
    STAILQ_NEXT(&record,link)=NULL; service.no_characteristics=1;
    struct probe_native too_many={.database=&service,.count=0}; assert(!database_report(&too_many,7,frame));
    bleGattCharacteristicsValue_t notification={0};
    notification.blobValue.data=(unsigned char *)1; notification.blobValue.size=65535;
    unexpected_notification(NULL,notification); assert(atomic_load(&unexpected_callback));
    discovered(NULL,(status_t)-9); assert(atomic_load(&discovery_status)==-9 && atomic_load(&discovery_callback));
    got_database(NULL,(bleGattsService_t *)1,65535); assert(atomic_load(&database_callback));
    puts("native adapter UUID/database/callback contracts PASS");
}
