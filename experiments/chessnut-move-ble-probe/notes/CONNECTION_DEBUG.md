# Connection-path investigation — 2026-11-04

Scope: M3 remains blocked before telemetry. No motors, new connection parameters,
stock service changes or unchanged connection retries. All work stays on the
experiment branch. DEVICE_RESULTS.md contains the actual hardware timeline.

Two observed failure classes must stay separate: stale daemon connection-list
routing (M3 and M2 reference), and post-restart establishment failure (HCI reason
0x3e, GATT 133, translated BLE failure 10). Mac discovery/disconnect succeeds;
the human confirmed the Kindle awake beside the board. H2's successful public
API arguments match both current code and the pinned upstream example.

Next bounded diagnostic change, before another physical test:

- Replace the misleading generic connect_address_stale_or_unavailable stage
  with neutral connect; absence of confirmation does not prove an address error.
- Extend the existing exact-pinned log observer to retain at most eight numeric
  connection callbacks for the supplied target. Never forward raw vendor logs,
  target addresses or connection pointer values. Ignore unrelated/unknown formats.
- After the public connect call returns, report its actual result and duration,
  observed callbacks (or explicit absence), and a bounded read-only snapshot of
  matching KindleBT contexts. Hold the existing callback/context mutexes.
- Keep callback/daemon failures unconfirmed: do not manufacture a connection,
  recover handles, send writes, retry or change cleanup based on these observations.
- Test no callback, failed callback, wrong target/function/format, bounded overflow,
  contradictory native state/API result and diagnostic redaction before hardware.

This improves evidence; it is not a claimed BLE fix. It will distinguish failure
before callback delivery from callback/state mismatch on the next specifically
coordinated discovery-only test. Existing daemon logs remain corroborating
private evidence. If no relevant callback is delivered, say so rather than
inventing a callback error from the public timeout.


Implementation: native connection_evidence/callback reports and neutral connect
stage are added. Portable lifecycle red test first failed on the old presumed-
address diagnosis; pinned-header adapter red compilation first failed on missing
observation state. Focused green tests cover numeric callback failure, wrong
format/function/target, eight-event bounds/overflow, absent callback, contradictory
context/API result, missing public handle, invalid context bounds and redaction.

Required final gates: independent fmt/clippy/Rust/Python tests, real pinned-header
adapter, ARM build/ELF and stage verification. Parent compatibility gate remains
separate. No physical test has run against this diagnostic build yet. The next
single discovery-only test asks: does a matching callback reach KindleBT, and do
its context flags agree with the public API result? A successful connection must
still pass UUID/property discovery and cleanup before any telemetry. On failure,
retain immediate private daemon logs and stop; do not automatically retry.


Verification PASS: independent fmt/clippy, 36 Rust integration tests, 19 Python
tests, native adapter against the real pinned headers, ARM/PIE/GLIBC inspection,
stage verification, parent fmt/clippy and all 237 parent tests. Source b4ce485
was deployed with matching installed receipt/checksums; preflight exit 0, lock
cleared and all 121 original app/data hashes unchanged. No connection ran during
deployment. Installed source_sha256:
c94bb0a216b6dd93e69530ebbf67f088dc4dca0aae8ed80a10278fda325fa23b.

No existing controller packet trace was found in the inspected normal log
locations. No global trace setting was enabled. The next coordinated diagnostic
will additionally capture the existing system message stream privately while
the probe runs, to preserve daemon evidence before log rotation. The capture
is a supervised read-only SSH tail, bounded on the host, with no Bluetooth API.
Physical validation of the new diagnostics and H3 remain NOT TESTED/BLOCKED.


## First instrumented hardware result — 2026-11-05

The callback is delivered: target match, state 3/status 10, no handle. The API
then waits 10 seconds and returns -2 because pinned bleConnStateChangedCallback
updates/signals its wait condition only for success status. The matching library
context remains unconnected with no handle. Daemon evidence reports GATT 133 /
controller reason 0x3e. This localizes the present failure before GATT discovery;
it does not establish why connection establishment fails. The earlier stale-
entry failure is a separate historical case, not the current callback outcome.

The final error now preserves explicit callback/API domains as
connect_callback_10_api_status_-2 when the most recent bounded target callback
reported failure. Missing, overflowed or successful callback histories cannot
invent a failure code. Registration failures are not reclassified. Red test:
real-header adapter first failed for missing connection_error_stage. No radio/API
parameters, dependency pins, stock files or retries change as a result.


## Operator-run read report — 2026-11-05

At the user's request, read-board.sh provides a direct on-Kindle diagnostic
without Mac advertisement timing or interactive command entry. It runs the new
noninteractive read-once path: one connect/discovery, both subscriptions, SDK
realtime, battery/piece status, 20s observation, MTU evidence and cleanup. It
never writes LEDs or motors. It prints/saves a redacted report including existing
system logs and an explicit outcome/exit code. A private address file is outside
the checksummed runtime and source; deployment provisions the known supplied
address without printing it. --check opens no session.

TDD red evidence: missing run_read_once, missing shell script, then the initial-
position prerequisite prevented battery/status reads. Green coverage includes
full observation without stdin/early EOF, status retrieval without initial
position (eventual missing-position failure), worker failures/no retry, CLI with
closed stdin, script pass/fail exit propagation, report retention/redaction,
setup-only check and signal cleanup. Packaging includes/checksums the new script
and validates it against the source. No additional board test runs automatically;
the user will enable Bluetooth and invoke the installed script themselves.
