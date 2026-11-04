#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "session_core.h"
static char calls[64];
static int fail_at, step;
static int call(char c) { size_t n = strlen(calls); calls[n] = c; calls[n+1] = 0; return ++step == fail_at ? -7 : 0; }
static int open_api(void *p) { (void)p; return call('O'); }
static int reg_api(void *p) { (void)p; return call('R'); }
static int gatt_api(void *p) { (void)p; return call('G'); }
static int conn_api(void *p, const unsigned char a[6]) { (void)p; assert(a[0]==0x12 && a[5]==0xab); return call('C'); }
static int discover_api(void *p) { (void)p; return call('D'); }
static int db_api(void *p) { (void)p; return call('B'); }
static int free_api(void *p) { (void)p; return call('F'); }
static int disconnect_api(void *p) { (void)p; return call('X'); }
static int ungatt_api(void *p) { (void)p; return call('g'); }
static int unreg_api(void *p) { (void)p; return call('r'); }
static int close_api(void *p) { (void)p; return call('o'); }
int main(void) {
    probe_api api = {open_api, reg_api, gatt_api, conn_api, discover_api, db_api, free_api, disconnect_api, ungatt_api, unreg_api, close_api};
    unsigned char addr[6];
    assert(probe_address("12:34:56:78:90:ab", addr));
    assert(!probe_address("12:34:56:78:90:zz", addr));
    assert(!probe_address("12:34:56:78:90:ab;", addr));
    assert(!probe_address("12-34-56-78-90-ab", addr));
    const char *expected[] = {"ORGCDBFXgro", "O", "ORo", "ORGro", "ORGCgro", "ORGCDXgro", "ORGCDBFXgro"};
    for (int failure = 0; failure <= 6; ++failure) {
        calls[0]=0; step=0; fail_at=failure;
        probe_session s = {.api=&api};
        if (probe_connect(&s, addr)==0) (void)probe_discover(&s);
        if (failure==4) { assert(strcmp(s.stage,"connect")==0); assert(s.status==-7); }
        (void)probe_cleanup(&s);
        assert(strcmp(calls,expected[failure])==0);
        size_t n = strlen(calls); assert(probe_cleanup(&s)==0); assert(strlen(calls)==n);
    }
    calls[0]=0; step=0; fail_at=0;
    probe_session s = {.api=&api};
    assert(probe_discover(&s)!=0);
    assert(probe_connect(&s, addr)==0);
    assert(probe_connect(&s, addr)!=0); /* one connection only */
    assert(probe_discover(&s)==0);
    assert(probe_discover(&s)!=0); /* cached database only, never repeat async retrieval */
    fail_at=step+1;
    assert(probe_cleanup(&s)!=0); /* cleanup continues after a failure */
    assert(strcmp(calls,"ORGCDBFXgro")==0);
    unsigned int id; char operation[16], address[18];
    assert(probe_request("{\"v\":1,\"id\":1,\"operation\":{\"type\":\"connect\",\"address\":\"12:34:56:78:90:ab\"}}\n", &id, operation,address));
    assert(id==1 && strcmp(operation,"connect")==0);
    assert(probe_request("{\"v\":1,\"id\":2,\"operation\":{\"type\":\"discover\"}}\n", &id,operation,address));
    const char *bad[] = {"{}\n", "{\"v\":2,\"id\":1,\"operation\":{\"type\":\"discover\"}}\n", "{\"v\":1,\"id\":0,\"operation\":{\"type\":\"discover\"}}\n", "{\"v\":1,\"id\":1,\"operation\":{\"type\":\"write\"}}\n", "{\"v\":1,\"id\":1,\"operation\":{\"type\":\"discover\"}}\nextra"};
    for (size_t i=0;i<sizeof(bad)/sizeof(bad[0]);++i) assert(!probe_request(bad[i],&id,operation,address));
    puts("native session lifecycle/parser PASS");
}
