#ifndef PROBE_SESSION_CORE_H
#define PROBE_SESSION_CORE_H
/* Portable, host-tested orchestration. Only the adapter knows KindleBT types. */
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>
typedef struct {
    int (*open)(void *); int (*reg)(void *); int (*gatt)(void *);
    int (*connect)(void *, const unsigned char[6]); int (*discover)(void *);
    int (*database)(void *); int (*free_database)(void *); int (*disconnect)(void *);
    int (*ungatt)(void *); int (*unreg)(void *); int (*close)(void *);
} probe_api;
typedef struct {
    const probe_api *api; void *context;
    bool opened, registered, gatt, connected, database_attempted;
    const char *stage; int status;
} probe_session;
static bool probe_address(const char *text, unsigned char bytes[6]) {
    if (strlen(text)!=17) return false;
    for (int i=0;i<6;++i) {
        if (!isxdigit((unsigned char)text[i*3]) || !isxdigit((unsigned char)text[i*3+1]) || (i<5 && text[i*3+2]!=':')) return false;
        unsigned int octet; if (sscanf(text+i*3,"%2x",&octet)!=1) return false;
        bytes[i]=(unsigned char)octet;
    }
    return true;
}
/* Canonical JSON subset emitted by Rust's serde serializer, not a general JSON
 * parser. Reconstruct the entire frame to reject unknown/duplicate fields,
 * partial EOF, escaped addresses, extra frames, versions or operations. */
static bool probe_request(const char *frame, unsigned int *id, char operation[16], char address[18]) {
    unsigned long long parsed=0; int consumed=0;
    if (sscanf(frame,"{\"v\":1,\"id\":%llu,\"operation\":{\"type\":\"%15[a-z]\"%n",&parsed,operation,&consumed)!=2 || consumed==0 || parsed==0 || parsed>UINT32_MAX) return false;
    *id=(unsigned int)parsed;
    char expected[256];
    if (strcmp(operation,"connect")==0) {
        if (sscanf(frame+consumed,",\"address\":\"%17[0-9a-fA-F:]\"}}",address)!=1) return false;
        unsigned char bytes[6]; if (!probe_address(address,bytes)) return false;
        snprintf(expected,sizeof(expected),"{\"v\":1,\"id\":%u,\"operation\":{\"type\":\"connect\",\"address\":\"%s\"}}\n",*id,address);
    } else {
        if (strcmp(operation,"discover")!=0 && strcmp(operation,"disconnect")!=0 && strcmp(operation,"shutdown")!=0 && strcmp(operation,"subscribe")!=0 && strcmp(operation,"capabilities")!=0) return false;
        snprintf(expected,sizeof(expected),"{\"v\":1,\"id\":%u,\"operation\":{\"type\":\"%s\"}}\n",*id,operation);
    }
    return strcmp(frame,expected)==0;
}
static int probe_step(probe_session *s, const char *stage, int status) { s->stage=stage; s->status=status; return status; }
static int probe_connect(probe_session *s, const unsigned char address[6]) {
    if (s->opened) return probe_step(s,"already_open",-1);
    if (probe_step(s,"open_session",s->api->open(s->context))!=0) return s->status;
    s->opened=true;
    if (probe_step(s,"ble_register",s->api->reg(s->context))!=0) return s->status;
    s->registered=true;
    if (probe_step(s,"gatt_register",s->api->gatt(s->context))!=0) return s->status;
    s->gatt=true;
    if (probe_step(s,"connect_address_stale_or_unavailable",s->api->connect(s->context,address))!=0) return s->status;
    s->connected=true;
    return 0;
}
static int probe_discover(probe_session *s) {
    if (!s->connected || s->database_attempted) return probe_step(s,"invalid_discovery_state",-1);
    if (probe_step(s,"discover",s->api->discover(s->context))!=0) return s->status;
    /* The library may produce a partial clone on an error; always release it. */
    s->database_attempted=true;
    return probe_step(s,"database",s->api->database(s->context));
}
static int probe_cleanup(probe_session *s) {
    int result=0, status;
#define PROBE_CLEAN(flag, fn, label) do { if (s->flag) { s->flag=false; status=s->api->fn(s->context); if (status!=0) { result=status; probe_step(s,label,status); } } } while (0)
    PROBE_CLEAN(database_attempted,free_database,"cleanup_database");
    PROBE_CLEAN(connected,disconnect,"cleanup_disconnect");
    PROBE_CLEAN(gatt,ungatt,"cleanup_gatt");
    PROBE_CLEAN(registered,unreg,"cleanup_ble");
    PROBE_CLEAN(opened,close,"cleanup_session");
#undef PROBE_CLEAN
    return result;
}
#endif
