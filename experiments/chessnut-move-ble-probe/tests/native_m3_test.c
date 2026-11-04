#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "m3_wire.h"
int main(void) {
    probe_packet packet={.bytes={0x21,1,0},.length=3,.without_response=false};
    assert(probe_m3_allowed(&packet));
    packet.bytes[0]=0x41;packet.bytes[2]=0x0c;assert(probe_m3_allowed(&packet));
    packet.bytes[2]=0x0b;assert(probe_m3_allowed(&packet));
    packet.without_response=true;assert(!probe_m3_allowed(&packet));
    memset(&packet,0,sizeof(packet));packet.length=34;packet.without_response=true;packet.bytes[0]=0x43;packet.bytes[1]=0x20;
    assert(probe_m3_allowed(&packet));packet.bytes[7]=0x20;assert(probe_m3_allowed(&packet));
    packet.bytes[8]=1;assert(!probe_m3_allowed(&packet));packet.bytes[8]=0;
    packet.bytes[7]=0x40;assert(!probe_m3_allowed(&packet));
    packet.length=35;packet.bytes[0]=0x42;packet.bytes[1]=0x21;assert(!probe_m3_allowed(&packet));
    unsigned int id=0;
    assert(probe_write_request("{\"v\":1,\"id\":7,\"operation\":{\"type\":\"write\",\"bytes\":[65,1,12],\"write_kind\":\"with_response\"}}\n",&id,&packet));
    assert(id==7 && packet.length==3 && probe_m3_allowed(&packet));
    const char *bad[]={
      "{\"v\":1,\"id\":0,\"operation\":{\"type\":\"write\",\"bytes\":[65,1,12],\"write_kind\":\"with_response\"}}\n",
      "{\"v\":1,\"id\":7,\"operation\":{\"type\":\"write\",\"bytes\":[256,1,12],\"write_kind\":\"with_response\"}}\n",
      "{\"v\":1,\"id\":7,\"operation\":{\"type\":\"write\",\"bytes\":[065,1,12],\"write_kind\":\"with_response\"}}\n",
      "{\"v\":1,\"id\":7,\"operation\":{\"type\":\"write\",\"bytes\":[65,1,12],\"write_kind\":\"without_response\"}}\n",
      "{\"v\":2,\"id\":7,\"operation\":{\"type\":\"write\",\"bytes\":[65,1,12],\"write_kind\":\"with_response\"}}\n"};
    for(size_t i=0;i<sizeof(bad)/sizeof(bad[0]);++i)assert(!probe_write_request(bad[i],&id,&packet));
    probe_notifications queue={0};unsigned char bytes[512]={0};probe_notification event;
    assert(!probe_notification_push(&queue,0,NULL,1));
    assert(!probe_notification_push(&queue,0,bytes,513));
    for(int i=0;i<16;++i)assert(probe_notification_push(&queue,1,bytes,512));
    assert(!probe_notification_push(&queue,1,bytes,1));
    for(int i=0;i<16;++i)assert(probe_notification_pop(&queue,&event) && event.length==512 && event.source==1);
    assert(!probe_notification_pop(&queue,&event));
    puts("M3 parser/policy/notification bounds PASS");
}
