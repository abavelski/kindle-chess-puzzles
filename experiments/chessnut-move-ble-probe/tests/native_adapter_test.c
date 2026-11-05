/* Built/run against the pinned real public headers, never guessed FFI types. */
#define _GNU_SOURCE
#define PROBE_CALLBACK_TIMEOUT_MS 20
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
static int subscriptions, fail_subscription;
static int fake_subscribe(sessionHandle s,bleConnHandle c,bleGattCharacteristicsValue_t v,bool enabled) {
    (void)s;(void)c;(void)v;
    ++subscriptions;return enabled && subscriptions==fail_subscription ? -9 : 0;
}
static int write_api_status,write_callback_status;static bool omit_write_callback;
static bleGattCharacteristicsValue_t *borrowed_write;
static int writes;static unsigned int written_length;static responseType_t written_kind;
static int fake_write(sessionHandle s,bleConnHandle c,bleGattCharacteristicsValue_t *v,responseType_t kind) {
    (void)s;++writes;borrowed_write=v;written_length=v->blobValue.size;written_kind=kind;
    if(write_api_status==0 && !omit_write_callback)wrote(c,*v,(status_t)write_callback_status);return write_api_status;
}
static void emit_mtu(const char *fmt,...) {
    log_Event event={.fmt=fmt};va_start(event.ap,fmt);mtu_observer(&event);va_end(event.ap);
}
static uuid_t uuid128(const char *text) {
    uuid_t uuid={.type=ACEBT_UUID_TYPE_128};unsigned int i=0,value;
    while(*text) {if(*text=='-'){++text;continue;}assert(sscanf(text,"%2x",&value)==1);uuid.uu[i++]=(unsigned char)value;text+=2;}
    assert(i==16);return uuid;
}
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

    atomic_store(&unexpected_callback,false);
    struct probe_native live={0};
    pthread_mutex_init(&live.state_lock,NULL);active_native=&live;
    live.connection=(bleConnHandle)(uintptr_t)0x1234;live.bleSetNotification=fake_subscribe;live.bleWriteCharacteristic=fake_write;
    session_diagnostics=tmpfile();live.wire=tmpfile();assert(session_diagnostics && live.wire);
    assert(native_mtu(&live)==0);
    const char *connection_format="Callback %s(): state %d, status %d, conn_handle %p addr %s";
    unsigned char expected_address[]={0x12,0x34,0x56,0x78,0x90,0xab};
    memcpy(live.target_address,expected_address,6);live.target_set=true;
    emit_mtu("unknown format","bleConnStateChangedCallback",0,10,NULL,"12:34:56:78:90:AB");
    assert(live.connection_debug.count==0);
    emit_mtu(connection_format,"other",0,10,NULL,"12:34:56:78:90:AB");
    assert(live.connection_debug.count==0);
    emit_mtu(connection_format,"bleConnStateChangedCallback",0,10,NULL,"11:34:56:78:90:AB");
    assert(live.connection_debug.count==0 && live.connection_debug.unmatched==1);
    emit_mtu(connection_format,"bleConnStateChangedCallback",0,10,NULL,"12:34:56:78:90:AB");
    assert(live.connection_debug.count==1 && live.connection_debug.events[0].status==10);
    char failure_stage[96];
    connection_error_stage(&live,"connect",-2,failure_stage);
    assert(strcmp(failure_stage,"connect_callback_10_api")==0);
    connection_error_stage(&live,"gatt_register",-2,failure_stage);
    assert(strcmp(failure_stage,"gatt_register")==0);
    connection_error_stage(&live,"connect",0,failure_stage);
    assert(strcmp(failure_stage,"connect")==0);
    for(int i=0;i<12;++i)emit_mtu(connection_format,"bleConnStateChangedCallback",1,0,live.connection,"12:34:56:78:90:AB");
    assert(live.connection_debug.count==8 && live.connection_debug.overflow);
    live.connection_debug.events[7].status=10;
    connection_error_stage(&live,"connect",-2,failure_stage);
    assert(strcmp(failure_stage,"connect")==0); /* later history may be missing */
    live.connection_debug.events[7].status=0;
    device_context_t context={0};pthread_mutex_init(&context.lock,NULL);
    memcpy(context.address,expected_address,6);context.handle=live.connection;context.gattc_connected=true;
    device_context_array_t contexts={.items=&context,.count=1,.capacity=1};live.contexts=&contexts;
    FILE *saved_diagnostics=session_diagnostics;session_diagnostics=tmpfile();assert(session_diagnostics);
    connection_diagnostics_report(&live,-2,10000);
    rewind(session_diagnostics);char diagnostic[4096]={0};assert(fread(diagnostic,1,sizeof(diagnostic)-1,session_diagnostics)>0);
    assert(strstr(diagnostic,"api_status=-2") && strstr(diagnostic,"callback_status=10"));
    assert(strstr(diagnostic,"context_connected=true") && strstr(diagnostic,"connection=UNCONFIRMED"));
    assert(!strstr(diagnostic,"12:34:56:78:90") && !strstr(diagnostic,"0x1234"));
    fclose(session_diagnostics);session_diagnostics=tmpfile();assert(session_diagnostics);
    memset(&live.connection_debug,0,sizeof(live.connection_debug));live.contexts=NULL;
    connection_error_stage(&live,"connect",-2,failure_stage);
    assert(strcmp(failure_stage,"connect")==0);
    connection_diagnostics_report(&live,-2,10000);
    rewind(session_diagnostics);memset(diagnostic,0,sizeof(diagnostic));assert(fread(diagnostic,1,sizeof(diagnostic)-1,session_diagnostics)>0);
    assert(strstr(diagnostic,"callback_seen=false") && strstr(diagnostic,"context_snapshot=unavailable"));
    fclose(session_diagnostics);session_diagnostics=tmpfile();assert(session_diagnostics);
    bleConnHandle saved_connection=live.connection;live.connection=NULL;
    connection_diagnostics_report(&live,0,20);
    rewind(session_diagnostics);memset(diagnostic,0,sizeof(diagnostic));assert(fread(diagnostic,1,sizeof(diagnostic)-1,session_diagnostics)>0);
    assert(strstr(diagnostic,"api_status=0") && strstr(diagnostic,"connection=UNCONFIRMED"));
    live.connection=saved_connection;live.contexts=&contexts;contexts.count=33;
    fclose(session_diagnostics);session_diagnostics=tmpfile();assert(session_diagnostics);
    connection_diagnostics_report(&live,-2,10000);
    rewind(session_diagnostics);memset(diagnostic,0,sizeof(diagnostic));assert(fread(diagnostic,1,sizeof(diagnostic)-1,session_diagnostics)>0);
    assert(strstr(diagnostic,"context_snapshot=invalid"));live.contexts=NULL;
    fclose(session_diagnostics);session_diagnostics=saved_diagnostics;pthread_mutex_destroy(&context.lock);

    const char *mtu_format="Callback %s(): status %d, mtu %d, conn_handle %p";
    emit_mtu(mtu_format,"other",0,64,live.connection);assert(native_mtu(&live)==0);
    emit_mtu(mtu_format,"bleMtuUpdatedCallback",0,64,(void *)(uintptr_t)0x4321);assert(native_mtu(&live)==0);
    emit_mtu(mtu_format,"bleMtuUpdatedCallback",-1,64,live.connection);assert(native_mtu(&live)==0);
    emit_mtu(mtu_format,"bleMtuUpdatedCallback",0,22,live.connection);assert(native_mtu(&live)==0);
    emit_mtu(mtu_format,"bleMtuUpdatedCallback",0,64,live.connection);assert(native_mtu(&live)==64);
    bleGattsService_t db[2]={0};
    db[0].uuid=uuid128("1b7e8261-2877-41c3-b46e-cf057c562023");db[0].no_characteristics=1;
    db[1].uuid=uuid128("1b7e8271-2877-41c3-b46e-cf057c562023");db[1].no_characteristics=2;
    struct aceBT_gattCharRec_t chars[3]={0};
    const char *ids[]={"1b7e8262-2877-41c3-b46e-cf057c562023","1b7e8272-2877-41c3-b46e-cf057c562023","1b7e8273-2877-41c3-b46e-cf057c562023"};
    for(int i=0;i<3;++i){chars[i].value.gattRecord.uuid=uuid128(ids[i]);chars[i].value.gattRecord.handle=(uint16_t)(i+1);chars[i].value.gattRecord.attProp=i==1?0x0c:0x10;}
    STAILQ_INIT(&db[0].charsList);STAILQ_INSERT_TAIL(&db[0].charsList,&chars[0],link);
    STAILQ_INIT(&db[1].charsList);STAILQ_INSERT_TAIL(&db[1].charsList,&chars[1],link);STAILQ_INSERT_TAIL(&db[1].charsList,&chars[2],link);
    live.database=db;live.count=2;
    fail_subscription=2;assert(native_subscribe(&live)!=0);assert(live.subscribed[0] && !live.subscribed[1]);
    assert(native_m3_cleanup(&live)==0);assert(subscriptions==3);
    subscriptions=0;fail_subscription=0;assert(native_subscribe(&live)==0);
    probe_packet battery={.bytes={0x41,1,0x0c},.length=3};
    assert(native_write_packet(&live,&battery)==0);assert(writes==1 && written_length==3 && written_kind==ACEBT_BLE_WRITE_TYPE_RESP_REQUIRED);
    unsigned char payload[34]={0};
    bleGattCharacteristicsValue_t value=chars[0].value;value.blobValue.data=payload;value.blobValue.size=34;value.format=BLE_FORMAT_BLOB;
    unexpected_notification(live.connection,value);payload[2]=9;
    probe_notification copied;pthread_mutex_lock(&live.state_lock);assert(probe_notification_pop(&live.notifications,&copied));pthread_mutex_unlock(&live.state_lock);
    assert(copied.length==34 && copied.bytes[2]==0 && copied.source==0);
    value.blobValue.size=513;unexpected_notification(live.connection,value);assert(atomic_load(&unexpected_callback));atomic_store(&unexpected_callback,false);
    value.blobValue.size=34;value.format=(__typeof__(value.format))99;
    unexpected_notification(live.connection,value);assert(atomic_load(&unexpected_callback));atomic_store(&unexpected_callback,false);
    /* Full reversed UUIDs still resolve; a wrong parent cannot resolve. */
    for(int k=0;k<2;++k)for(int a=0;a<8;++a){unsigned char b=db[k].uuid.uu[a];db[k].uuid.uu[a]=db[k].uuid.uu[15-a];db[k].uuid.uu[15-a]=b;}
    for(int k=0;k<3;++k)for(int a=0;a<8;++a){unsigned char b=chars[k].value.gattRecord.uuid.uu[a];chars[k].value.gattRecord.uuid.uu[a]=chars[k].value.gattRecord.uuid.uu[15-a];chars[k].value.gattRecord.uuid.uu[15-a]=b;}
    assert(resolve_characteristics(&live));db[0].uuid.uu[0]^=1;assert(!resolve_characteristics(&live));db[0].uuid.uu[0]^=1;
    probe_packet led={.bytes={0x43,0x20},.length=34,.without_response=true};led.bytes[7]=0x20;
    emit_mtu(mtu_format,"bleMtuUpdatedCallback",0,23,live.connection);assert(native_write_packet(&live,&led)!=0);assert(writes==1);
    emit_mtu(mtu_format,"bleMtuUpdatedCallback",0,64,live.connection);assert(native_write_packet(&live,&led)==0);assert(writes==2 && written_length==34 && live.leds_maybe);
    assert(native_m3_cleanup(&live)==0);assert(writes==3 && !live.leds_maybe && !live.subscribed[0] && !live.subscribed[1]);
    assert(native_subscribe(&live)==0);
    write_api_status=-7;assert(native_write_packet(&live,&battery)==-7);assert(!live.write_pending);write_api_status=0;
    write_callback_status=-8;assert(native_write_packet(&live,&battery)==-8);assert(!live.write_pending);write_callback_status=0;
    omit_write_callback=true;assert(native_write_packet(&live,&led)!=0);assert(live.write_pending && live.leds_maybe);
    /* A timed-out asynchronous API must not retain a pointer into a returned stack frame. */
    assert(borrowed_write==&live.write_value && borrowed_write->blobValue.data==live.write_bytes);
    assert(memcmp(borrowed_write->blobValue.data,led.bytes,34)==0);
    int before_timeout=writes;assert(native_write_packet(&live,&battery)!=0);assert(writes==before_timeout);
    assert(native_m3_cleanup(&live)!=0);assert(writes==before_timeout+1 && !live.subscribed[0] && !live.subscribed[1]);
    assert(borrowed_write==&live.clear_value && borrowed_write->blobValue.data==live.clear_bytes);
    for(int i=2;i<34;++i)assert(live.clear_bytes[i]==0);
    assert(live.write_api_calls==(size_t)writes);
    fclose(live.wire);fclose(session_diagnostics);active_native=NULL;pthread_mutex_destroy(&live.state_lock);
    puts("native adapter UUID/database/callback contracts PASS");
}
