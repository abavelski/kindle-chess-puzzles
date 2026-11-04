# Worker IPC version 1 (host scaffold)

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
characteristic properties and native UUID byte order remain M2 work. `ok` on a
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

`interrupt()` sends DISCONNECT then SHUTDOWN; Drop forcibly reaps a child if
this sequence fails or is skipped. No live controller signal handler or native
STOP cleanup is claimed by this scaffold. The only public CLI path is offline.
A future live session must add motor authorization, STOP, timeouts for native
calls, signal handling, privilege validation, and confirmation before exposing
this generic transport. No automatic writes/retries/reconnect are present.
