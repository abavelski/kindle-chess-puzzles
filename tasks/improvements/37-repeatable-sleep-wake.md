# Task 37 — Repeatable power-button sleep/wake

**Status:** Implemented
**Depends on:** Tasks 07 and 36
**Working branch:** main (maintenance of the current sleep overlay)

Fix repeated button sleep/wake without changing puzzle/progress or power policy.
Scribe 5.19.6 reproduction on 2026-10-05: first sleep/wake succeeds;
next button press produces `Splash screen is on. Ignoring power button` and
blanket load timeouts. Xorg remains stopped. A bounded CONT of the verified
Xorg PID immediately restores blanket responses and emits exitingScreenSaver;
pausing it again permits one further cycle, then the failure returns.

On wake, temporarily resume only the supervisor-owned Xorg process, wait for
the already-subscribed `exitingScreenSaver` completion event with a four-second
monotonic deadline, then pause the same process and redraw the chess frame. Preserve process identity
checks, finite waits and cleanup; never pause after the app has died. Errors
return through normal supervisor recovery. Core/render stay platform-neutral.

Acceptance: failing host contracts first for repeated handoffs, timeout cleanup,
PID reuse and app death; full automated gate and Kindle build. Deploy through
the existing installer preserving puzzle and progress hashes. Physical checkpoint:
at least three button sleep/wake cycles, Sleeping overlay appears/removes each
time, chess controls remain usable, X exits normally and native UI recovers.
Keep the task in progress until that checkpoint passes.

## Automated validation and deployment — 2026-10-05

Wake contracts failed first (missing hook returned exit 1), then passed for
three repeated handoffs, timeout cleanup, reused Xorg PID, app death, and
supervisor-owned environment exports. scripts/build-kindle.sh passed fmt,
clippy, workspace tests, Python/C/lifecycle/packaging contracts, asset checks
and ARMv7/glibc-2.35 release validation. Existing renderer snapshots unchanged.
Staged and deployed with the existing installer to root@192.168.1.20:2222.
Installed binary SHA-256:
`dc5144e76cb13ef1ab299b2ba61ba80094740232cee0a0b0636771a2edddb906`.
All five collection hashes and progress hash unchanged across installation;
loader dependencies resolve. Three-cycle physical acceptance is pending.

## Physical regression and revision

First candidate failed: blanket property response precedes native wake completion.
User confirmed third and subsequent presses still failed. Replaced that probe
with already-subscribed `exitingScreenSaver`, bounded by monotonic four-second
pipe polling. Native completion and early Awake are distinct events. Added
failing event/FFI contracts, then passing timeout/EINTR/error and completion tests.
Keep Sleeping visible through the wake handoff; restore board only after completion.
System logs verify goingToScreenSaver follows ACTIVE -> SCREEN SAVER, so the
notice already follows actual logical sleep entry rather than the raw button.

## Revised deployment — 2026-10-05

Revised build passed the full scripts/build-kindle.sh gate, including new native
completion/finite pipe polling contracts, then staged and deployed through the
existing installer after X exit. Installed binary SHA-256:
`c3029286a3c37062aa435c8d52deb08fed1f3f5b6caef73c664f8983bc1f63a3`.
Installed supervisor SHA-256:
`725b0c4604164c1c948419f83c4258970c311570e0c914aa8b65e6091b9f158c`.
Both match local build files; loader dependencies resolve. All five collection
hashes and progress `133662b61e2fdd710596eb0ca4813d579275d6682b68b30a9d0336bd8998c431`
are unchanged across deployment. No app or lock remains before user launch.
Repeated sleep/wake, visible overlay and normal exit acceptance remain pending.

## Physical acceptance — 2026-10-05

User confirmed the revised build works and requested commit/push/rebuild/deploy
from main. Device logs corroborate repeated Sleeping/Awake transitions, each
with beginning handoff and native completion before the board redraw, and X
exit 0 with Xorg/awesome resumed. User confirmation completes the requested
physical review; no extra idle, reboot or crash-exit observation is inferred.
