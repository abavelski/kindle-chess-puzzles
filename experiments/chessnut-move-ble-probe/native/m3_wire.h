#ifndef PROBE_M3_WIRE_H
#define PROBE_M3_WIRE_H
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef struct { unsigned char bytes[35]; size_t length; bool without_response; } probe_packet;
static bool probe_m3_allowed(const probe_packet *p) {
    if(p->length==3 && !p->without_response) {
        return (p->bytes[0]==0x21 && p->bytes[1]==1 && p->bytes[2]==0) ||
          (p->bytes[0]==0x41 && p->bytes[1]==1 && (p->bytes[2]==0x0b || p->bytes[2]==0x0c));
    }
    if(p->length!=34 || !p->without_response || p->bytes[0]!=0x43 || p->bytes[1]!=0x20) return false;
    unsigned int lit=0;
    for(size_t i=2;i<34;++i) {
        unsigned int low=p->bytes[i]&15,high=p->bytes[i]>>4;
        if(low>3 || high>3) return false;
        lit+=(low!=0)+(high!=0);
    }
    return lit<=1;
}
static bool probe_decimal(const char **cursor,unsigned int maximum,unsigned int *out) {
    const char *p=*cursor; if(*p<'0' || *p>'9')return false;
    unsigned int value=0;
    do { unsigned int digit=(unsigned int)(*p-'0');if(value>(maximum-digit)/10 || digit>maximum)return false;value=value*10+digit;++p; } while(*p>='0' && *p<='9');
    *cursor=p;*out=value;return true;
}
/* A strict canonical JSON subset; no allocations, escapes or arbitrary writes. */
static bool probe_write_request(const char *frame,unsigned int *id,probe_packet *packet) {
    const char *prefix="{\"v\":1,\"id\":";size_t length=strlen(prefix);
    if(strncmp(frame,prefix,length)!=0)return false;
    const char *p=frame+length;unsigned int request_id;
    if(!probe_decimal(&p,UINT32_MAX,&request_id) || request_id==0)return false;
    const char *middle=",\"operation\":{\"type\":\"write\",\"bytes\":[";length=strlen(middle);
    if(strncmp(p,middle,length)!=0)return false;p+=length;
    memset(packet,0,sizeof(*packet));
    do {
        unsigned int byte;
        if(packet->length>=34 || !probe_decimal(&p,255,&byte))return false;
        packet->bytes[packet->length++]=(unsigned char)byte;
        if(*p!=',')break;++p;
    }while(true);
    if(strcmp(p,"],\"write_kind\":\"with_response\"}}\n")==0)packet->without_response=false;
    else if(strcmp(p,"],\"write_kind\":\"without_response\"}}\n")==0)packet->without_response=true;
    else return false;
    if(!probe_m3_allowed(packet))return false;
    char canonical[512];size_t used=(size_t)snprintf(canonical,sizeof(canonical),"{\"v\":1,\"id\":%u,\"operation\":{\"type\":\"write\",\"bytes\":[",request_id);
    for(size_t i=0;i<packet->length;++i)used+=(size_t)snprintf(canonical+used,sizeof(canonical)-used,"%s%u",i?",":"",packet->bytes[i]);
    snprintf(canonical+used,sizeof(canonical)-used,"],\"write_kind\":\"%s\"}}\n",packet->without_response?"without_response":"with_response");
    if(strcmp(frame,canonical)!=0)return false;*id=request_id;return true;
}
typedef struct { unsigned char bytes[512];size_t length;unsigned int source; } probe_notification;
typedef struct { probe_notification items[16];size_t start,count; } probe_notifications;
/* Caller holds its own mutex. Copy callback-owned data before returning. */
static bool probe_notification_push(probe_notifications *q,unsigned int source,const unsigned char *bytes,size_t length) {
    if(q->count>=16 || source>1 || bytes==NULL || length==0 || length>512)return false;
    probe_notification *event=&q->items[(q->start+q->count)%16];
    memcpy(event->bytes,bytes,length);event->length=length;event->source=source;++q->count;return true;
}
static bool probe_notification_pop(probe_notifications *q,probe_notification *event) {
    if(q->count==0)return false;*event=q->items[q->start];q->start=(q->start+1)%16;--q->count;return true;
}
#endif
