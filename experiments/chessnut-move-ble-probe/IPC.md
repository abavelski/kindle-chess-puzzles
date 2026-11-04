# Worker IPC version 1 (M3 non-motor subset)

UTF-8, one JSON object per LF-terminated frame. Maximum **4096 bytes including
LF**, notification payload maximum **512 bytes**, command maximum **35 bytes**.
Empty writes, partial EOF frames, unknown fields/types, wrong versions, and
nonmatching IDs are errors. EOF before a frame reports WorkerExited. No
stderr output is interpreted as protocol. Address validation accepts only six
two-digit hexadecimal octets; diagnostics never echo the CONNECT request.

Request shape (ID starts at 1 and increases, no concurrent requests):

```json
{"v":1,"id":1,"operation":{"type":"discover"}}
```

Operations: `connect` with `address`, `discover`, `subscribe` (both position and
response), `write` with integer-array `bytes` and explicit `write_kind`
(`with_response` or `without_response`), `capabilities`, `disconnect`, `shutdown`.
`Operation::from_command` copies the SDK's complete bytes and write kind;
there is no truncation, fragmentation, or write-mode substitution.

Response shape:

```json
{"v":1,"id":1,"report":{"type":"services","uuids":["1b7e8261-2877-41c3-b46e-cf057c562023"]}}
```

Reports: `connected` for connect, `services` for discovery, `ok` for
subscribe/write/shutdown, `disconnected` for disconnect, `error` with `code`
for failures. Discovery checks all five SDK UUIDs; actual service membership,
characteristic properties and native UUID byte order are validated by the M2
controller using the native `database` report below. The legacy flat `services`
variant remains only for M0 mock transport tests. `ok` on a
write means native API acceptance and successful callback, never board acknowledgement,
physical movement or settled state.

ID **0** is reserved for asynchronous `notification` (`source` is the
characteristic UUID, `bytes` is a byte array) and unsolicited `disconnected`.
Position/response notification bytes are decoded by the pinned SDK; a command
characteristic or unknown source fails. Up to 16 decoded notifications may
wait during a request. A bounded 16-frame channel connects the stdout reader
to the client. Overflow ends the reader, causing session failure rather than
unbounded memory growth. Protocol frames have bounded allocation before JSON
parsing. The SDK position decoder accepts its documented minimum length and
piece codes; it does not validate the first two position-header bytes.

Each request has one deadline including intervening notifications. A quiet timed
observation poll returns no event without closing; the caller owns the query or
watch deadline. Error,
timeout, EOF, malformed frame, or unexpected disconnect makes the client
terminal: close pipes, kill/reap child, join reader, discard cached events.
The mock worker never starts descendants or inherits stdout to another process;
the native worker follows the same ownership contract.

`interrupt()` sends DISCONNECT then SHUTDOWN. The M2 CLI wires SIGINT/SIGTERM/HUP
to cancellation, checks for it during requests, and polls for unsolicited link
loss while idle. Session requests have a 45-second deadline. Native errors may
use ID 0 before a request is available; their stage/status codes remain visible.
On error/Drop, the live client closes input, sends TERM to the owned worker PID,
allows up to 55 seconds for cleanup, then kills/reaps a stuck child and joins its
reader. Forced or failed cleanup cannot report success. M0 mock clients retain
immediate reaping. No motor authorization or STOP support is added at M2.

## Native M2 subset

The C parser accepts only the **canonical serialized request shape** produced by
Rust serde, with no whitespace, LF termination, exact field order, version 1,
and increasing nonzero 32-bit IDs. It reconstructs and compares the complete
frame, rejecting duplicate/unknown fields, alternate/escaped spellings, extra
bytes, NUL and partial EOF. This is intentionally a strict JSON subset rather
than a general JSON parser. Input allocation is bounded before parsing.
CONNECT, DISCOVER, SUBSCRIBE, CAPABILITIES, WRITE, DISCONNECT and SHUTDOWN
are accepted. WRITE has an independent native allowlist: SDK realtime-enable,
battery/piece queries (three bytes, WithResponse), or 34-byte single-square LED
and all-off packets (WithoutResponse). Unknown packets, alternate modes and all
35-byte motor/STOP packets fail before the API. LED writes additionally require
observed MTU >=37 and both successful subscriptions. No automatic retry exists.

Native discovery returns a complete, bounded snapshot of up to 20 characteristic
records across at most 32 services (larger, cyclic or inconsistent lists fail):

```json
{"v":1,"id":2,"report":{"type":"database","records":[{"service":"1b7e8271-2877-41c3-b46e-cf057c562023","uuid":"1b7e8272-2877-41c3-b46e-cf057c562023","properties":12,"handle":5}]}}
```

This example shows one record; acceptance requires all three expected records.
Each record preserves its parent service, native handle and property bitmask.
Native 128-bit UUID bytes are formatted in returned array order, using all 16
bytes; 16/32-bit UUIDs are displayed as 4/8 hex digits and cannot match the
required 128-bit IDs. Rust tests full forward and full reversed byte order
consistently across both services and their characteristics, requires unique
matches and expected properties, and reports the selected order. Hardware H2
must confirm which ordering the actual firmware returns. No mixed/partial
UUID or service-less lookup is accepted.

The native worker owns one library/session/connection and cloned database.
KindleBT deep-copies borrowed database callback data; the worker retains only
the returned owned clone and frees it before disconnect/deregistration/close.
Application callbacks copy 1..512 notification bytes into a mutex-protected,
16-slot ring; connection, record handle/full UUID, blob format and offset are
validated first. Only the owner thread serializes notifications. Unknown records,
malformed callbacks and overflow terminate the session. WRITE waits up to five
seconds for its matching callback for both modes. Pending payload memory remains
owned by the worker through cleanup, including timeout. Further writes are
refused after a callback timeout, except a final fixed, explicitly best-effort
all-off cleanup packet. Partial subscriptions are unwound independently.
Repeated SERVICES uses the cached clone; it never starts overlapping retrieval.
Explicit reconnect from the CLI starts a fresh worker; there is no automatic retry.

Pinned KindleBT does not forward a public connection-state callback. The adapter
observes its exported, callback-updated `devices_context` through
`dca_find_by_handle`, locking the context mutex on the sole API-owning thread.
The exact pinned header defines these types; no Rust FFI layouts are duplicated.
This internal observation is version-specific and needs H2 hardware validation.

M1 preflight remains a separate supervised text command that never opens a
session. In M2, the worker repeats those checks and drops identity before opening
any session. Protocol output uses a dedicated saved descriptor; upstream
stdout/stderr are suppressed during session APIs to prevent address leaks and
protocol corruption. Only our stage/numeric-status diagnostics reach stderr.
No optional raw vendor logging switch is exposed.

## M3 capabilities and observation

```json
{"v":1,"id":3,"operation":{"type":"capabilities"}}
{"v":1,"id":3,"report":{"type":"capabilities","mtu":64,"callback_status":0}}
```

64 above is illustrative, not a device result. MTU is null unless an actual
successful 23..517 callback belongs to the current connection. The optional
log.c observer reads the exact pinned callback format/name and numeric arguments
without forwarding raw vendor logs or addresses. An absent observer/callback
means unknown; it never infers MTU from notification lengths. No negotiation API
is added. 35-byte delivery is always UNVERIFIED in M3.

Decoded observations retain raw bytes and canonical source UUID. The CLI prints
piece placement without fabricated FEN metadata. Every telemetry/status command
requires initial realtime position and bounded response deadlines. Native cleanup
attempts LEDs off, unsubscribes acquired subscriptions, frees the owned database,
disconnects and releases registrations/session, reporting failures at each stage.
No realtime-disable packet exists in the pinned SDK; disconnect ends streaming.


Connection errors now use the neutral native stage `connect` and retain the
public numeric status in the existing error code. Sanitized `connect_evidence`
and `connect_callback` lines are stderr diagnostics, not protocol frames. They
report actual bounded callback/context observations; they do not acknowledge a
connection or change any request/cleanup behavior.
