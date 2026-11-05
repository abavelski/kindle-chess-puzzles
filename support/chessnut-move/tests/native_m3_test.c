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
    probe_packet motor={.bytes={0x42,0x21},.length=35,.without_response=true};
    assert(probe_stop(&motor) && probe_wire_allowed(&motor));
    motor.bytes[2]=7;assert(!probe_wire_allowed(&motor)); /* Force is forbidden. */
    motor.bytes[34]=1;assert(probe_motor(&motor) && probe_wire_allowed(&motor));
    motor.bytes[2]=0xff;assert(!probe_wire_allowed(&motor));
    motor.bytes[2]=7;motor.without_response=false;assert(!probe_wire_allowed(&motor));
    unsigned int id=0;
    motor.without_response=true;
    char motor_frame[512];size_t used=(size_t)snprintf(motor_frame,sizeof(motor_frame),"{\"v\":1,\"id\":8,\"operation\":{\"type\":\"write\",\"bytes\":[");
    for(size_t i=0;i<35;++i)used+=(size_t)snprintf(motor_frame+used,sizeof(motor_frame)-used,"%s%u",i?",":"",motor.bytes[i]);
    snprintf(motor_frame+used,sizeof(motor_frame)-used,"],\"write_kind\":\"without_response\"}}\n");
    assert(probe_write_request(motor_frame,&id,&packet) && id==8 && packet.length==35 && memcmp(packet.bytes,motor.bytes,35)==0);

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
