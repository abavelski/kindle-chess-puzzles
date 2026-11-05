#ifndef PROBE_KINDLEBT_SESSION_H
#define PROBE_KINDLEBT_SESSION_H
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdatomic.h>
#include <time.h>
#include "session_core.h"
#include "m3_wire.h"
#include "log.h"
/* Exact pinned header types; no guessed layouts. MTU uses the verified ACE declaration. */
_Static_assert(ACE_STATUS_OK == 0, "portable lifecycle expects zero success");
#define PROBE_FUNCTIONS(X) \
 X(openSession) X(closeSession) X(bleRegister) X(bleDeregister) \
 X(bleRegisterGattClient) X(bleDeregisterGattClient) X(bleConnect) X(bleDisconnect) \
 X(bleDiscoverAllServices) X(bleGetDatabase) X(bleCleanupGattService) X(dca_find_by_handle) \
 X(bleSetNotification) X(bleWriteCharacteristic) X(aceBT_bleRequestMtu)
#define PROBE_FIELD(name) __typeof__(&name) name;
typedef struct { int state, status; bool handle_present; } probe_connection_event;
typedef struct {
    probe_connection_event events[8];
    size_t count;
    unsigned int unmatched;
    bool overflow;
} probe_connection_debug;
struct probe_native {
    PROBE_FUNCTIONS(PROBE_FIELD)
    device_context_array_t *contexts;
    sessionHandle session;
    bleConnHandle connection;
    bleGattsService_t *database;
    uint32_t count;
    size_t write_api_calls;
    unsigned char target_address[6];
    bool target_set;
    probe_connection_debug connection_debug;
    FILE *wire;
    pthread_mutex_t state_lock;
    probe_notifications notifications;
    struct aceBT_gattCharRec_t *position_record,*command_record,*response_record;
    bool motion_armed,motion_maybe;
    bool subscribed[2],leds_maybe,write_pending,write_done,mtu_seen;
    int write_status,mtu_status,observed_mtu;
    bool descriptor_pending,descriptor_done;
    uint16_t descriptor_handle;
    int descriptor_status;
    bleConnHandle mtu_connection;
    unsigned char write_bytes[35],clear_bytes[34],stop_bytes[35];
    bleGattCharacteristicsValue_t write_value,clear_value,stop_value;
};
#undef PROBE_FIELD
static _Atomic(struct probe_native *) active_native;
static int native_m3_cleanup(struct probe_native *n);
static volatile sig_atomic_t session_cancelled;
static atomic_bool unexpected_callback;
static atomic_int discovery_status;
static atomic_bool discovery_callback;
static atomic_bool database_callback;
static FILE *session_diagnostics;
#include "connection_diagnostics.h"
static void session_signal(int signal_number) { (void)signal_number; session_cancelled=1; }
static void discovered(bleConnHandle connection, status_t status) {
    (void)connection;
    atomic_store(&discovery_status,(int)status);
    atomic_store(&discovery_callback,true);
}
static void got_database(bleConnHandle connection, bleGattsService_t *database, uint32_t count) {
    (void)connection; (void)database; (void)count;
    /* Library deep-copies before this callback. Never retain the borrowed DB. */
    atomic_store(&database_callback,true);
}
static void unexpected_notification(bleConnHandle connection,bleGattCharacteristicsValue_t value);
static void wrote(bleConnHandle connection,bleGattCharacteristicsValue_t value,status_t status);
static void descriptor_wrote(bleConnHandle connection,bleGattCharacteristicsValue_t value,status_t status);
static bleGattClientCallbacks_t session_callbacks = {
    .size=sizeof(bleGattClientCallbacks_t),
    .on_ble_gattc_service_discovered_cb=discovered,
    .on_ble_gattc_get_gatt_db_cb=got_database,
    .notify_characteristics_cb=unexpected_notification,
    .on_ble_gattc_write_characteristics_cb=wrote,
    .on_ble_gattc_write_descriptor_cb=descriptor_wrote,
};
static int native_status(const char *stage, status_t status) {
    fprintf(session_diagnostics,"timestamp_unix=%ld phase=M3 stage=%s status=%d\n",(long)time(NULL),stage,(int)status);
    fflush(session_diagnostics);
    return (int)status;
}
static bool callback_wait(atomic_bool *flag) {
    for (unsigned int i=0;i<200 && !session_cancelled;++i) {
        if (atomic_load(flag)) return true;
        struct timespec pause={.tv_nsec=5000000}; nanosleep(&pause,NULL);
    }
    return false;
}
static int native_open(void *p) {
    struct probe_native *n=p;
    return native_status("open_session",n->openSession(ACEBT_SESSION_TYPE_DUAL_MODE,&n->session));
}
static int native_register(void *p) { struct probe_native *n=p; return session_cancelled ? -1 : native_status("ble_register",n->bleRegister(n->session)); }
static int native_gatt(void *p) { struct probe_native *n=p; return session_cancelled ? -1 : native_status("gatt_register",n->bleRegisterGattClient(n->session,&session_callbacks)); }
static int native_connect(void *p, const unsigned char address[6]) {
    struct probe_native *n=p; bdAddr_t target={0}; memcpy(target.address,address,6);
    if (session_cancelled) return -1;
    pthread_mutex_lock(&n->state_lock);
    memcpy(n->target_address,address,6);n->target_set=true;
    memset(&n->connection_debug,0,sizeof(n->connection_debug));
    pthread_mutex_unlock(&n->state_lock);
    struct timespec started,ended;clock_gettime(CLOCK_MONOTONIC,&started);
    int status=native_status("connect",n->bleConnect(n->session,&n->connection,&target,ACE_BT_BLE_CONN_PARAM_BALANCED,ACEBT_BLE_GATT_CLIENT_ROLE,ACE_BT_BLE_CONN_PRIO_MEDIUM));
    clock_gettime(CLOCK_MONOTONIC,&ended);
    long long elapsed=(ended.tv_sec-started.tv_sec)*1000000000LL+ended.tv_nsec-started.tv_nsec;
    connection_diagnostics_report(n,status,(unsigned long long)(elapsed<0?0:elapsed/1000000LL));
    if (status==0 && n->connection==NULL) return -1;
    return status;
}
static int native_discover(void *p) {
    struct probe_native *n=p; if (session_cancelled) return -1;
    atomic_store(&discovery_callback,false);
    int status=native_status("discover",n->bleDiscoverAllServices(n->session,n->connection));
    if (status!=0) return status;
    if (!callback_wait(&discovery_callback)) return -1;
    return atomic_load(&discovery_status);
}
static int native_database(void *p) {
    struct probe_native *n=p; if (session_cancelled) return -1;
    atomic_store(&database_callback,false);
    int status=native_status("database",n->bleGetDatabase(n->connection,&n->database,&n->count));
    if (status!=0) return status;
    if (!callback_wait(&database_callback)) return -1;
    return n->database!=NULL && n->count>0 && n->count<=32 ? 0 : -1;
}
static int native_free(void *p) {
    struct probe_native *n=p;
    int status=native_m3_cleanup(n);
    pthread_mutex_lock(&n->state_lock);n->position_record=NULL;n->command_record=NULL;n->response_record=NULL;pthread_mutex_unlock(&n->state_lock);
    if (n->count>32) { n->database=NULL; n->count=0; return -1; }
    if (n->database!=NULL) { int freed=native_status("cleanup_database",n->bleCleanupGattService(n->database,(int)n->count));if(freed!=0)status=freed; }
    n->database=NULL; n->count=0; return status;
}
static int native_disconnect(void *p) { struct probe_native *n=p; return native_status("cleanup_disconnect",n->bleDisconnect(n->connection)); }
static int native_ungatt(void *p) { struct probe_native *n=p; return native_status("cleanup_gatt",n->bleDeregisterGattClient(n->session)); }
static int native_unregister(void *p) { struct probe_native *n=p; return native_status("cleanup_ble",n->bleDeregister(n->session)); }
static int native_close(void *p) { struct probe_native *n=p; return native_status("cleanup_session",n->closeSession(n->session)); }
static bool native_connected(struct probe_native *n) {
    /* No public connection-state callback is forwarded by pinned KindleBT.
     * Observe its own callback-updated state under its lock, on this sole
     * API-owning thread. No concurrent array mutation or reconnect is allowed. */
    device_context_t *context=n->dca_find_by_handle(n->contexts,n->connection);
    if (context==NULL) return false;
    pthread_mutex_lock(&context->lock);
    bool connected=context->gattc_connected && !context->gattc_disconnected;
    pthread_mutex_unlock(&context->lock);
    return connected;
}
static bool uuid_text(const uuid_t *uuid, char text[37]) {
    unsigned int width;
    switch (uuid->type) {
        case ACEBT_UUID_TYPE_16: width=2; break;
        case ACEBT_UUID_TYPE_32: width=4; break;
        case ACEBT_UUID_TYPE_128: width=16; break;
        default: return false;
    }
    char *out=text;
    for (unsigned int i=0;i<width;++i) {
        if (width==16 && (i==4 || i==6 || i==8 || i==10)) *out++='-';
        snprintf(out,3,"%02x",uuid->uu[i]); out+=2;
    }
    *out=0; return true;
}
/* Serialize a complete bounded snapshot before writing any protocol bytes.
 * Never silently truncate a list or traverse an unbounded/cyclic list. */
static bool database_report(struct probe_native *n, unsigned int id, char frame[4097]) {
    if (n->database==NULL || n->count==0 || n->count>32) return false;
    size_t offset=(size_t)snprintf(frame,4097,"{\"v\":1,\"id\":%u,\"report\":{\"type\":\"database\",\"records\":[",id);
    unsigned int total=0;
    for (uint32_t i=0;i<n->count;++i) {
        bleGattsService_t *service=&n->database[i]; char parent[37];
        if (!uuid_text(&service->uuid,parent) || service->no_characteristics>20) return false;
        unsigned int count=0; struct aceBT_gattCharRec_t *record;
        STAILQ_FOREACH(record,&service->charsList,link) {
            if (++count>service->no_characteristics || ++total>20) return false;
            char uuid[37]; if (!uuid_text(&record->value.gattRecord.uuid,uuid)) return false;
            int written=snprintf(frame+offset,4097-offset,"%s{\"service\":\"%s\",\"uuid\":\"%s\",\"properties\":%u,\"handle\":%u}",total==1?"":",",parent,uuid,(unsigned int)record->value.gattRecord.attProp,(unsigned int)record->value.gattRecord.handle);
            if (written<0 || (size_t)written>=4097-offset) return false;
            offset+=(size_t)written;
        }
        if (count!=service->no_characteristics) return false;
    }
    if (total==0 || offset+5>4096) return false;
    memcpy(frame+offset,"]}}\n",5); return true;
}
static void ipc_report(FILE *wire, unsigned int id, const char *type) {
    fprintf(wire,"{\"v\":1,\"id\":%u,\"report\":{\"type\":\"%s\"}}\n",id,type); fflush(wire);
}
static void ipc_error(FILE *wire, unsigned int id, const char *stage, int status) {
    /* stage is a program constant; no vendor diagnostics or addresses cross IPC. */
    fprintf(wire,"{\"v\":1,\"id\":%u,\"report\":{\"type\":\"error\",\"code\":\"%s_status_%d\"}}\n",id,stage,status); fflush(wire);
}
#include "kindlebt_m3.h"

static int session_worker(const char *library, const char *required) {
    FILE *wire=fdopen(dup(STDOUT_FILENO),"w");
    session_diagnostics=fdopen(dup(STDERR_FILENO),"w");
    if (wire==NULL || session_diagnostics==NULL) return 1;
    /* Separate protocol from all upstream stdout/stderr, which may expose MACs.
     * Our bounded stage/status diagnostics use the saved stderr descriptor. */
    if (dup2(STDERR_FILENO,STDOUT_FILENO)<0) return 1;
    if (preflight(library,required)!=0) { ipc_error(wire,0,"preflight",-1); return 1; }
    fflush(stdout); fflush(stderr);
    int quiet=open("/dev/null",O_WRONLY);
    if (quiet<0 || dup2(quiet,STDOUT_FILENO)<0 || dup2(quiet,STDERR_FILENO)<0) return 1;
    close(quiet);
    struct sigaction action={0}; action.sa_handler=session_signal; sigemptyset(&action.sa_mask);
    if (sigaction(SIGINT,&action,NULL)!=0 || sigaction(SIGTERM,&action,NULL)!=0 || sigaction(SIGHUP,&action,NULL)!=0) return 1;
    signal(SIGPIPE,SIG_IGN);
    void *handle=dlopen(library,RTLD_NOW|RTLD_LOCAL);
    if (handle==NULL) { ipc_error(wire,0,"kindlebt_load",-1); return 1; }
    /* Callbacks may already have loaded active_native during final teardown.
     * This one-session process keeps the context/mutex alive until OS exit. */
    static struct probe_native n;
    n.wire=wire;
    if(pthread_mutex_init(&n.state_lock,NULL)!=0)return 1;
    active_native=&n;
#define PROBE_LOAD(name) n.name=(__typeof__(n.name))dlsym(handle,#name); if (n.name==NULL) { ipc_error(wire,0,"required_symbol",-1); return 1; }
    PROBE_FUNCTIONS(PROBE_LOAD)
#undef PROBE_LOAD
    __typeof__(&log_add_callback) observe=(__typeof__(&log_add_callback))dlsym(handle,"log_add_callback");
    if(observe!=NULL && observe(mtu_observer,NULL,LOG_DEBUG)==0)fprintf(session_diagnostics,"phase=M3 mtu_observer=registered source=pinned_KindleBT_callback_log connection_observer=registered\n");
    n.contexts=dlsym(handle,"devices_context");
    if (n.contexts==NULL) { ipc_error(wire,0,"required_state_symbol",-1); return 1; }
    const probe_api api={native_open,native_register,native_gatt,native_connect,native_discover,native_database,native_free,native_disconnect,native_ungatt,native_unregister,native_close};
    probe_session session={.api=&api,.context=&n};
    char input[4097], operation[16], address[18], snapshot[4097];
    size_t used=0; unsigned int previous=0, id=0; bool quit=false; int result=0;
    while (!quit && !session_cancelled) {
        flush_notifications(&n);
        if (atomic_load(&unexpected_callback)) { ipc_error(wire,0,"invalid_callback_or_notification_overflow",-1); result=1; break; }
        if (session.connected && !native_connected(&n)) { ipc_report(wire,0,"disconnected"); result=1; break; }
        struct pollfd fd={.fd=STDIN_FILENO,.events=POLLIN};
        int ready=poll(&fd,1,50);
        if (ready<0) { if (errno==EINTR) continue; result=1; break; }
        if (ready==0) continue;
        char byte; ssize_t got=read(STDIN_FILENO,&byte,1);
        if (got==0) break;
        if (got<0) { if (errno==EINTR) continue; result=1; break; }
        if (byte==0 || used>=4096) { ipc_error(wire,0,"invalid_frame",-1); result=1; break; }
        input[used++]=byte;
        if (byte!='\n') continue;
        input[used]=0; used=0;
        probe_packet packet;bool write_request=probe_write_request(input,&id,&packet);
        bool simple_request=write_request ? false : probe_request(input,&id,operation,address);
        if ((!write_request && !simple_request) || id<=previous) { ipc_error(wire,0,"invalid_or_forbidden_request",-1); result=1; break; }
        previous=id;
        if(write_request) {
            if(!session.connected || native_write_packet(&n,&packet)!=0) {ipc_error(wire,id,"write_rejected_or_callback_failed",-1);result=1;break;}
            ipc_report(wire,id,"ok");
        } else if(strcmp(operation,"subscribe")==0) {
            if(!session.connected || native_subscribe(&n)!=0) {ipc_error(wire,id,"subscribe_failed",-1);result=1;break;}
            ipc_report(wire,id,"ok");
        } else if(strcmp(operation,"arm")==0) {
            if(!session.connected || n.motion_maybe || n.write_pending || !n.subscribed[0] || !n.subscribed[1] || native_mtu(&n)<38){ipc_error(wire,id,"motion_arm_precondition",-1);result=1;break;}
            n.motion_armed=true;ipc_report(wire,id,"ok");
        } else if(strcmp(operation,"capabilities")==0) {
            capabilities_report(&n,id);
        } else if (strcmp(operation,"connect")==0) {
            unsigned char bytes[6];
            if (!probe_address(address,bytes) || probe_connect(&session,bytes)!=0) { char stage[96];connection_error_stage(&n,session.stage?session.stage:"address",session.status,stage);ipc_error(wire,id,stage,session.status); result=1; break; }
            ipc_report(wire,id,"connected");
        } else if (strcmp(operation,"discover")==0) {
            if (!session.database_attempted && probe_discover(&session)!=0) { ipc_error(wire,id,session.stage,session.status); result=1; break; }
            if (!session.connected || !database_report(&n,id,snapshot)) { ipc_error(wire,id,"invalid_or_oversized_database",-1); result=1; break; }
            fputs(snapshot,wire); fflush(wire);
        } else {
            int status=probe_cleanup(&session);
            if (status!=0) { ipc_error(wire,id,session.stage,status); result=1; break; }
            if (strcmp(operation,"shutdown")==0) { ipc_report(wire,id,"ok"); quit=true; }
            else ipc_report(wire,id,"disconnected");
        }
    }
    if (used!=0 && !session_cancelled) { ipc_error(wire,0,"partial_frame",-1); result=1; }
    if (probe_cleanup(&session)!=0) result=1;
    fprintf(session_diagnostics,"timestamp_unix=%ld phase=M3 stage=cleanup result=%s write_api_calls=%zu\n",(long)time(NULL),result?"FAIL":"PASS",n.write_api_calls); fflush(session_diagnostics);
    /* Do not dlclose callback code while upstream threads might still unwind.
     * This one-session process exits immediately; OS releases library mappings. */
    active_native=NULL;
    fclose(wire); fclose(session_diagnostics); return result;
}
#endif
