#ifndef PROBE_KINDLEBT_M3_H
#define PROBE_KINDLEBT_M3_H
#ifndef PROBE_CALLBACK_TIMEOUT_MS
#define PROBE_CALLBACK_TIMEOUT_MS 5000
#endif
/* Included after the pinned types and session helpers. All callbacks copy only
 * bounded data. The owner thread alone writes protocol output and calls APIs. */
static bool same_record(const bleGattCharacteristicsValue_t *v,const struct aceBT_gattCharRec_t *record) {
    return record!=NULL && v->gattRecord.handle==record->value.gattRecord.handle &&
      v->gattRecord.uuid.type==record->value.gattRecord.uuid.type && memcmp(v->gattRecord.uuid.uu,record->value.gattRecord.uuid.uu,16)==0;
}
static void unexpected_notification(bleConnHandle connection,bleGattCharacteristicsValue_t value) {
    struct probe_native *n=atomic_load(&active_native);
    if(n==NULL) {atomic_store(&unexpected_callback,true);return;}
    pthread_mutex_lock(&n->state_lock);
    unsigned int source=same_record(&value,n->position_record)?0:same_record(&value,n->response_record)?1:2;
    bool valid=connection==n->connection && value.format==BLE_FORMAT_BLOB && value.blobValue.offset==0 &&
      probe_notification_push(&n->notifications,source,value.blobValue.data,value.blobValue.size);
    pthread_mutex_unlock(&n->state_lock);
    if(!valid)atomic_store(&unexpected_callback,true);
}
static void wrote(bleConnHandle connection,bleGattCharacteristicsValue_t value,status_t status) {
    struct probe_native *n=atomic_load(&active_native);
    if(n==NULL){atomic_store(&unexpected_callback,true);return;}
    pthread_mutex_lock(&n->state_lock);
    if(connection==n->connection && n->write_pending && !n->write_done && same_record(&value,n->command_record)) {
        n->write_status=(int)status;n->write_done=true;
    }else atomic_store(&unexpected_callback,true);
    pthread_mutex_unlock(&n->state_lock);
}
/* This observer reads only the exact pinned callback's numeric arguments. It
 * never forwards raw log messages, addresses, or other vendor diagnostics. */
static void mtu_observer(log_Event *event) {
    if(observe_connection(event))return;
    if(event->fmt==NULL || strcmp(event->fmt,"Callback %s(): status %d, mtu %d, conn_handle %p")!=0)return;
    va_list args;va_copy(args,event->ap);
    const char *name=va_arg(args,const char *);
    if(name==NULL || strcmp(name,"bleMtuUpdatedCallback")!=0){va_end(args);return;}
    int status=va_arg(args,int),mtu=va_arg(args,int);bleConnHandle connection=va_arg(args,void *);va_end(args);
    struct probe_native *n=atomic_load(&active_native);if(n==NULL)return;
    pthread_mutex_lock(&n->state_lock);n->mtu_seen=true;n->mtu_status=status;n->observed_mtu=mtu;n->mtu_connection=connection;pthread_mutex_unlock(&n->state_lock);
}
static unsigned int native_mtu(struct probe_native *n) {
    pthread_mutex_lock(&n->state_lock);
    unsigned int mtu=n->mtu_seen && n->mtu_status==0 && n->mtu_connection==n->connection && n->observed_mtu>=23 && n->observed_mtu<=517 ? (unsigned int)n->observed_mtu:0;
    pthread_mutex_unlock(&n->state_lock);return mtu;
}
static void capabilities_report(struct probe_native *n,unsigned int id) {
    unsigned int mtu=native_mtu(n);char value[16];
    if(mtu)snprintf(value,sizeof(value),"%u",mtu);else strcpy(value,"null");
    pthread_mutex_lock(&n->state_lock);bool seen=n->mtu_seen;int status=n->mtu_status;pthread_mutex_unlock(&n->state_lock);
    char callback[16];if(seen)snprintf(callback,sizeof(callback),"%d",status);else strcpy(callback,"null");
    fprintf(n->wire,"{\"v\":1,\"id\":%u,\"report\":{\"type\":\"capabilities\",\"mtu\":%s,\"callback_status\":%s}}\n",id,value,callback);fflush(n->wire);
}
static void flush_notifications(struct probe_native *n) {
    probe_notification event;
    for(;;){
        pthread_mutex_lock(&n->state_lock);bool present=probe_notification_pop(&n->notifications,&event);pthread_mutex_unlock(&n->state_lock);if(!present)break;
        const char *source=event.source==0?"1b7e8262-2877-41c3-b46e-cf057c562023":"1b7e8273-2877-41c3-b46e-cf057c562023";
        fprintf(session_diagnostics,"timestamp_unix=%ld phase=M3 stage=notification source=%s length=%zu\n",(long)time(NULL),source,event.length);
        fprintf(n->wire,"{\"v\":1,\"id\":0,\"report\":{\"type\":\"notification\",\"source\":\"%s\",\"bytes\":[",source);
        for(size_t i=0;i<event.length;++i)fprintf(n->wire,"%s%u",i?",":"",event.bytes[i]);
        fputs("]}}\n",n->wire);fflush(n->wire);
        if(ferror(n->wire))atomic_store(&unexpected_callback,true);
    }
}
static bool matches_uuid(const uuid_t *value,const char *canonical,bool reverse) {
    if(value->type!=ACEBT_UUID_TYPE_128)return false;
    unsigned char expected[16];unsigned int index=0,byte;
    for(const char *p=canonical;*p;){if(*p=='-'){++p;continue;}if(index>=16 || sscanf(p,"%2x",&byte)!=1)return false;expected[index++]=(unsigned char)byte;p+=2;}
    if(index!=16)return false;
    for(unsigned int i=0;i<16;++i)if(value->uu[i]!=expected[reverse?15-i:i])return false;
    return true;
}
static bool resolve_characteristics(struct probe_native *n) {
    const char *parents[]={"1b7e8261-2877-41c3-b46e-cf057c562023","1b7e8271-2877-41c3-b46e-cf057c562023","1b7e8271-2877-41c3-b46e-cf057c562023"};
    const char *ids[]={"1b7e8262-2877-41c3-b46e-cf057c562023","1b7e8272-2877-41c3-b46e-cf057c562023","1b7e8273-2877-41c3-b46e-cf057c562023"};
    unsigned int properties[]={0x10,0x0c,0x10};
    if(n->database==NULL || n->count==0 || n->count>32)return false;
    for(int reverse=0;reverse<=1;++reverse){
        struct aceBT_gattCharRec_t *found[3]={0};unsigned int matches[3]={0};
        for(uint32_t i=0;i<n->count;++i){
            unsigned int count=0;struct aceBT_gattCharRec_t *record;
            STAILQ_FOREACH(record,&n->database[i].charsList,link){
                if(++count>20 || count>n->database[i].no_characteristics)return false;
                for(unsigned int k=0;k<3;++k)if(matches_uuid(&n->database[i].uuid,parents[k],reverse!=0) && matches_uuid(&record->value.gattRecord.uuid,ids[k],reverse!=0)){
                    ++matches[k];if(record->value.gattRecord.handle!=0 && (record->value.gattRecord.attProp&properties[k])==properties[k])found[k]=record;
                }
            }
            if(count!=n->database[i].no_characteristics)return false;
        }
        if(matches[0]==1 && matches[1]==1 && matches[2]==1 && found[0] && found[1] && found[2]){
            pthread_mutex_lock(&n->state_lock);n->position_record=found[0];n->command_record=found[1];n->response_record=found[2];pthread_mutex_unlock(&n->state_lock);return true;
        }
    }
    return false;
}
/* SetNotification returns after submitting a CCCD write, before its completion.
 * Only the public descriptor callback permits the next GATT operation. */
static void descriptor_wrote(bleConnHandle connection,bleGattCharacteristicsValue_t value,status_t status) {
    struct probe_native *n=atomic_load(&active_native);
    if(n==NULL){atomic_store(&unexpected_callback,true);return;}
    pthread_mutex_lock(&n->state_lock);
    if(connection==n->connection && n->descriptor_pending && !n->descriptor_done &&
       value.gattDescriptor.gattRecord.handle==n->descriptor_handle){
        n->descriptor_status=(int)status;n->descriptor_done=true;
    }else atomic_store(&unexpected_callback,true);
    pthread_mutex_unlock(&n->state_lock);
}
static uint16_t notification_descriptor(struct aceBT_gattCharRec_t *record) {
    struct aceBT_gattDescRec_t *desc;uint16_t handle=0;unsigned int count=0;
    STAILQ_FOREACH(desc,&record->value.descList,link){
        if(++count>20)return 0;
        if(desc->value.is_set && desc->value.is_notify){
            if(handle!=0 || desc->value.gattRecord.handle==0)return 0;
            handle=desc->value.gattRecord.handle;
        }
    }
    return handle;
}
static int native_notification(struct probe_native *n,unsigned int index,bool enabled) {
    struct aceBT_gattCharRec_t *record=index==0?n->position_record:n->response_record;
    if(record==NULL)return -1;
    uint16_t handle=notification_descriptor(record);if(handle==0)return -1;
    pthread_mutex_lock(&n->state_lock);
    if(n->descriptor_pending || n->write_pending){pthread_mutex_unlock(&n->state_lock);return -1;}
    n->descriptor_pending=true;n->descriptor_done=false;n->descriptor_handle=handle;n->descriptor_status=-1;
    pthread_mutex_unlock(&n->state_lock);
    const char *stage=enabled?(index==0?"subscribe_position":"subscribe_response"):(index==0?"unsubscribe_position":"unsubscribe_response");
    int status=native_status(stage,n->bleSetNotification(n->session,n->connection,record->value,enabled));
    if(status!=0){pthread_mutex_lock(&n->state_lock);n->descriptor_pending=false;pthread_mutex_unlock(&n->state_lock);return status;}
    /* Conservatively retain ownership after acceptance even if completion fails. */
    if(enabled)n->subscribed[index]=true;
    struct timespec start;clock_gettime(CLOCK_MONOTONIC,&start);
    for(;;){
        pthread_mutex_lock(&n->state_lock);bool done=n->descriptor_done;status=n->descriptor_status;
        if(done)n->descriptor_pending=false;
        pthread_mutex_unlock(&n->state_lock);
        if(done){
            fprintf(session_diagnostics,"phase=M3 stage=%s_descriptor_callback handle=%u status=%d\n",stage,handle,status);fflush(session_diagnostics);
            if(status==0 && !enabled)n->subscribed[index]=false;
            return status;
        }
        struct timespec now;clock_gettime(CLOCK_MONOTONIC,&now);
        if((now.tv_sec-start.tv_sec)*1000000000LL+now.tv_nsec-start.tv_nsec>=(long long)PROBE_CALLBACK_TIMEOUT_MS*1000000LL || session_cancelled || atomic_load(&unexpected_callback)){
            fprintf(session_diagnostics,"phase=M3 stage=%s_descriptor_callback handle=%u result=TIMEOUT_OR_INVALID_CALLBACK api_accepted=true\n",stage,handle);fflush(session_diagnostics);
            return -1; /* Keep pending: cleanup must disconnect, without another GATT write. */
        }
        struct timespec pause={.tv_nsec=1000000};nanosleep(&pause,NULL);
    }
}
static int native_subscribe(struct probe_native *n) {
    pthread_mutex_lock(&n->state_lock);bool pending=n->descriptor_pending;pthread_mutex_unlock(&n->state_lock);
    if(pending || atomic_load(&unexpected_callback))return -1;
    if(n->subscribed[0] && n->subscribed[1])return 0;
    if(!resolve_characteristics(n))return -1;
    for(unsigned int i=0;i<2;++i){
        if(n->subscribed[i])continue;
        int status=native_notification(n,i,true);if(status!=0)return status;
    }
    return 0;
}
static bool packet_led(const probe_packet *packet){return packet->length==34;}
static int native_write_packet(struct probe_native *n,const probe_packet *packet) {
    if(!probe_m3_allowed(packet) || n->command_record==NULL || !n->subscribed[0] || !n->subscribed[1])return -1;
    unsigned int mtu=native_mtu(n);
    if(packet_led(packet) && (mtu==0 || packet->length>mtu-3)){
        fprintf(session_diagnostics,"phase=M3 stage=write_refused length=%zu reason=MTU_unknown_or_insufficient delivery=UNVERIFIED\n",packet->length);return -1;
    }
    pthread_mutex_lock(&n->state_lock);
    if(n->write_pending || n->descriptor_pending){pthread_mutex_unlock(&n->state_lock);return -1;}
    n->write_pending=true;n->write_done=false;n->write_status=-1;pthread_mutex_unlock(&n->state_lock);
    memcpy(n->write_bytes,packet->bytes,packet->length);
    n->write_value=n->command_record->value;
    n->write_value.blobValue.data=n->write_bytes;n->write_value.blobValue.size=(uint16_t)packet->length;n->write_value.blobValue.offset=0;n->write_value.format=BLE_FORMAT_BLOB;
    if(packet_led(packet))n->leds_maybe=true;
    ++n->write_api_calls;
    int status=n->bleWriteCharacteristic(n->session,n->connection,&n->write_value,packet->without_response?ACEBT_BLE_WRITE_TYPE_RESP_NO:ACEBT_BLE_WRITE_TYPE_RESP_REQUIRED);
    fprintf(session_diagnostics,"timestamp_unix=%ld phase=M3 stage=write_api length=%zu kind=%s status=%d\n",(long)time(NULL),packet->length,packet->without_response?"without_response":"with_response",status);fflush(session_diagnostics);
    if(status!=0){pthread_mutex_lock(&n->state_lock);n->write_pending=false;pthread_mutex_unlock(&n->state_lock);return status;}
    /* Both modes require a callback here. WithoutResponse completion remains a
     * stack observation, never a board acknowledgement. No automatic retries. */
    struct timespec start;clock_gettime(CLOCK_MONOTONIC,&start);
    for(;;){
        flush_notifications(n);
        pthread_mutex_lock(&n->state_lock);bool done=n->write_done;status=n->write_status;if(done)n->write_pending=false;pthread_mutex_unlock(&n->state_lock);
        if(done){
            fprintf(session_diagnostics,"timestamp_unix=%ld phase=M3 stage=write_callback length=%zu kind=%s status=%d delivery=NOT_INFERRED\n",(long)time(NULL),packet->length,packet->without_response?"without_response":"with_response",status);fflush(session_diagnostics);
            if(status==0 && packet_led(packet)){bool clear=true;for(size_t i=2;i<34;++i)if(packet->bytes[i])clear=false;if(clear)n->leds_maybe=false;}
            return status;
        }
        struct timespec now;clock_gettime(CLOCK_MONOTONIC,&now);
        if((now.tv_sec-start.tv_sec)*1000000000LL+now.tv_nsec-start.tv_nsec>=(long long)PROBE_CALLBACK_TIMEOUT_MS*1000000LL || atomic_load(&unexpected_callback)){
            fprintf(session_diagnostics,"phase=M3 stage=write_callback length=%zu result=TIMEOUT_OR_INVALID_CALLBACK api_accepted=true delivery=UNVERIFIED\n",packet->length);return -1;
        }
        struct timespec pause={.tv_nsec=5000000};nanosleep(&pause,NULL);
    }
}
static int native_m3_cleanup(struct probe_native *n) {
    int result=0;
    if(n->leds_maybe){
        probe_packet clear={.bytes={0x43,0x20},.length=34,.without_response=true};
        pthread_mutex_lock(&n->state_lock);bool pending=n->write_pending;pthread_mutex_unlock(&n->state_lock);
        if(!pending){if(native_write_packet(n,&clear)!=0)result=-1;}
        else if(n->command_record){
            /* Last, fixed non-motor cleanup write only. Its callback cannot be
             * correlated after a timeout, so cleanup must remain a failure. */
            memset(n->clear_bytes,0,34);n->clear_bytes[0]=0x43;n->clear_bytes[1]=0x20;n->clear_value=n->command_record->value;
            n->clear_value.blobValue.data=n->clear_bytes;n->clear_value.blobValue.size=34;n->clear_value.blobValue.offset=0;n->clear_value.format=BLE_FORMAT_BLOB;
            ++n->write_api_calls;
            int status=n->bleWriteCharacteristic(n->session,n->connection,&n->clear_value,ACEBT_BLE_WRITE_TYPE_RESP_NO);
            fprintf(session_diagnostics,"phase=M3 stage=led_clear_best_effort length=34 kind=without_response api_status=%d delivery=UNVERIFIED\n",status);result=-1;
        }else result=-1;
    }
    pthread_mutex_lock(&n->state_lock);bool gatt_pending=n->descriptor_pending || n->write_pending;pthread_mutex_unlock(&n->state_lock);
    if(gatt_pending){
        fprintf(session_diagnostics,"phase=M3 stage=unsubscribe result=SKIPPED_PENDING_GATT cleanup=FAIL disconnect_required=true\n");
        n->subscribed[0]=false;n->subscribed[1]=false;return -1;
    }
    for(unsigned int i=0;i<2;++i)if(n->subscribed[i]){
        int status=native_notification(n,i,false);
        if(status!=0){result=status;break;}
    }
    return result;
}
#endif
