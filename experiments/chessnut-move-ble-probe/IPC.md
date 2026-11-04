# Worker IPC version 1 (M2 native discovery subset)

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
(`with_response` or `without_response`), `disconnect`, `shutdown`.
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
write means worker acceptance only, never physical movement or settled state.

ID **0** is reserved for asynchronous `notification` (`source` is the
characteristic UUID, `bytes` is a byte array) and unsolicited `disconnected`.
Position/response notification bytes are decoded by the pinned SDK; a command
characteristic or unknown source fails. Up to 16 decoded notifications may
wait during a request. A bounded 16-frame channel connects the stdout reader
to the client. Overflow ends the reader, causing session failure rather than
unbounded memory growth. Protocol frames have bounded allocation before JSON
parsing. The SDK position decoder accepts its documented minimum length and
piece codes; it does not validate the first two position-header bytes.

Each operation has one deadline including intervening notifications. Error,
timeout, EOF, malformed frame, or unexpected disconnect makes the client
terminal: close pipes, kill/reap child, join reader, discard cached events.
The mock worker never starts descendants or inherits stdout to another process;
the future native worker must follow the same ownership contract.

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
Only CONNECT, DISCOVER, DISCONNECT and SHUTDOWN are accepted; native SUBSCRIBE
and WRITE requests fail and clean up without executing a board operation.
The M0 typed encoders/decoder mocks still test future transport operations.

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
Application callbacks use atomics and never retain borrowed values. An unexpected
notification terminates M2 without decoding or dereferencing its payload.
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
