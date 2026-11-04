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
