#ifndef PROBE_CONNECTION_DIAGNOSTICS_H
#define PROBE_CONNECTION_DIAGNOSTICS_H
/* Exact pinned log.c/callback contract, included after the native state types.
 * Never write from a callback or retain/emit an address or connection pointer. */
static bool observe_connection(log_Event *event) {
    const char *format="Callback %s(): state %d, status %d, conn_handle %p addr %s";
    if(event->fmt==NULL || strcmp(event->fmt,format)!=0)return false;
    va_list args;va_copy(args,event->ap);
    const char *name=va_arg(args,const char *);
    if(name==NULL || strcmp(name,"bleConnStateChangedCallback")!=0){va_end(args);return true;}
    int state=va_arg(args,int),status=va_arg(args,int);
    void *connection=va_arg(args,void *);
    const char *address=va_arg(args,const char *);va_end(args);
    struct probe_native *n=atomic_load(&active_native);if(n==NULL)return true;
    unsigned char bytes[6];
    bool parsed=address!=NULL && strnlen(address,18)==17 && probe_address(address,bytes);
    pthread_mutex_lock(&n->state_lock);
    if(n->target_set && parsed && memcmp(bytes,n->target_address,6)==0){
        probe_connection_debug *debug=&n->connection_debug;
        if(debug->count<8)debug->events[debug->count++]=(probe_connection_event){state,status,connection!=NULL};
        else debug->overflow=true;
    }else if(n->connection_debug.unmatched<65535)++n->connection_debug.unmatched;
    pthread_mutex_unlock(&n->state_lock);
    return true;
}
/* Keep API and observed callback status in separate named fields of the error
 * code. A missing/overflowed history cannot justify a callback diagnosis. */
static void connection_error_stage(struct probe_native *n,const char *stage,int api_status,char out[96]) {
    snprintf(out,96,"%s",stage?stage:"unknown");
    if(stage==NULL || strcmp(stage,"connect")!=0 || api_status==0)return;
    pthread_mutex_lock(&n->state_lock);
    probe_connection_debug debug=n->connection_debug;
    pthread_mutex_unlock(&n->state_lock);
    if(!debug.overflow && debug.count>0 && debug.events[debug.count-1].status!=0)
        snprintf(out,96,"connect_callback_%d_api",debug.events[debug.count-1].status);
}
static void connection_diagnostics_report(struct probe_native *n,int api_status,unsigned long long elapsed_ms) {
    pthread_mutex_lock(&n->state_lock);
    probe_connection_debug debug=n->connection_debug;
    bool target_set=n->target_set;unsigned char target[6];memcpy(target,n->target_address,6);
    pthread_mutex_unlock(&n->state_lock);
    /* Only this process's single API-owning thread mutates the array. Callback
     * threads update individual contexts under their pinned per-context lock. */
    const char *snapshot="unavailable";
    size_t matches=0;bool connected=false,disconnected=false,handle_present=false;
    if(n->contexts!=NULL){
        device_context_array_t *contexts=n->contexts;
        snapshot="invalid";
        if(contexts->count<=32 && contexts->count<=contexts->capacity && (contexts->count==0 || contexts->items!=NULL)){
            snapshot="bounded";
            for(size_t i=0;i<contexts->count;++i){
                device_context_t *context=&contexts->items[i];
                pthread_mutex_lock(&context->lock);
                if(target_set && memcmp(context->address,target,6)==0){
                    ++matches;connected|=context->gattc_connected;
                    disconnected|=context->gattc_disconnected;handle_present|=context->handle!=NULL;
                }
                pthread_mutex_unlock(&context->lock);
            }
        }
    }
    fprintf(session_diagnostics,
      "timestamp_unix=%ld phase=M3 stage=connect_evidence api_status=%d elapsed_ms=%llu connection=%s callback_seen=%s callback_count=%zu callback_overflow=%s unmatched_callbacks=%u context_snapshot=%s context_matches=%zu context_connected=%s context_disconnected=%s context_handle_present=%s public_handle_present=%s\n",
      (long)time(NULL),api_status,elapsed_ms,api_status==0 && n->connection!=NULL?"CONFIRMED":"UNCONFIRMED",
      debug.count?"true":"false",debug.count,debug.overflow?"true":"false",debug.unmatched,snapshot,matches,
      connected?"true":"false",disconnected?"true":"false",handle_present?"true":"false",n->connection?"true":"false");
    for(size_t i=0;i<debug.count;++i){
        probe_connection_event event=debug.events[i];
        fprintf(session_diagnostics,
          "phase=M3 stage=connect_callback index=%zu callback_state=%d callback_status=%d callback_handle_present=%s target_matches=true\n",
          i,event.state,event.status,event.handle_present?"true":"false");
    }
    fflush(session_diagnostics);
}
#endif
