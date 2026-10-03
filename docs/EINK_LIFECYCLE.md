# Task 07 implementation and device verification

Task 07 is **Implemented with a user-authorized deferred suspend/resume check**.
On 2026-10-03 the user confirmed that only the sleeping test remains and
requested committing/pushing Task 07 while deferring that test.
Device tests verify responsive regional updates, exclusive finger touch during
promotion/cancel, and native display/finger/pen recovery after timeout and
SIGKILL. Exclusive finger input is required on every launch; exit repaint uses
verified xrefresh. The remaining natural suspend/resume check is deferred. Host checks alone do not establish waveform reliability or resume
support.

## Damage and presentation

`chess-render::calculate_damage` compares visible pixels in previous/current
Gray8 frames using stable 64-pixel tiles. It clips edge tiles and merges
intersecting or aligned adjacent rectangles. Renderer-region compaction then
bounds submission count without merging unrelated board/header/status content.
Small separated piece changes remain separate; heavily fragmented board
changes use their board bounding box. Frame size changes and the first
frame produce full damage; identical frames produce no submissions. Comparing
pixels covers selection, origin/destination moves, automatic replies, feedback,
header, toolbar, descriptions/warnings, navigation/reset/orientation, and modal
open/close/page changes without relying on an incomplete list of state fields.

Host tests replay just the dirty rectangles and assert equality with the full
render after each transition. Stride padding is excluded. Existing visible
snapshots remain unchanged because this task changes presentation, not artwork.

The Scribe development build originally fragmented navigation into 38
serialized updates, taking over 10 seconds. Region compaction and one final
completion wait reduced this to three updates; the optimized retest completed
navigation in approximately 0.43 seconds with clean redraws confirmed by the
user. Keep optimized builds for further device checkpoints.

The Kindle adapter validates the entire region batch before submission, packs
only each region's visible scanlines, and uses raw-data destination offsets.
The FBInk boundary checks signed-short offset limits as well as buffer size and
screen bounds. Updates are submitted as a batch with one final completion wait; damage history is
committed only after successful presentation. An error exits to the supervisor
rather than treating an incomplete submission as a successfully presented frame.

## Refresh policy and current measurements

The pinned FBInk revision remains `92e127008145b2a22fba7c59815d810d716310dd`.
Its `refresh_kindle_mtk` implementation maps flashing AUTO to GC16/FULL and
non-flashing requests to PARTIAL. That is source evidence, not a new physical
measurement.

| Content/request | Current default | Available after measurement |
| --- | --- | --- |
| First frame or explicit recovery | flashing AUTO, whole frame | Same |
| Changed rectangles | non-flashing AUTO | GC16 for grayscale; DU for verified black/white regions |
| Unchanged frame | no update | Same |
| Periodic cleaning | disabled | `RefreshPolicy.full_every` after observing accumulation |

Both previous and current pixels in the entire submitted rectangle must be
black/white for the fast-mode content class. Gray backgrounds, antialiasing,
and erased grayscale pieces therefore exclude DU. Explicit GC16/DU modes are
not enabled in the app until their reliability is measured. The C bridge retries
an unsupported explicit mode once using AUTO, logging the fallback; it does
not retry unrelated I/O errors. Compiled mock tests use the pinned header and
exercise mapping, offsets, fallback, and completion calls.

The binary's `--full-refresh` argument provides the previous flashing full-frame
presentation on every interaction for recovery/comparison. It intentionally
bypasses damage only when explicitly requested. No periodic cadence or faster
waveform is claimed verified yet.

## Timings

Stderr records process entry, FBInk initialization, first usable frame,
recognized-tap-to-final-submit, submission, and completion durations. Process
entry is the first line of `run`, not loader startup. Tap timing begins when the
evdev adapter recognizes a completed tap, not initial finger contact. For
multiple rectangles, final-submit time includes all submissions; completion includes the last wait. Driver completion reliability still
needs device verification. No-op frames have no update timing entry.

## Overlay supervisor and recovery

Stage the cross-built binary and `scripts/kindle_launch.sh` together. On the
Scribe, with a disposable blank notebook beneath the overlay:

```sh
sh /mnt/us/kindle-chess-task07/kindle_launch.sh /mnt/us/kindle-chess-task07/kindle-chess
```

The supervisor uses `/tmp/kindle-chess.lock` for atomic single-instance launch
and appends logs to `/mnt/us/kindle-chess/lifecycle.log`. Tests can override these
with `KINDLE_CHESS_LOCK` and `KINDLE_CHESS_LOG`. Puzzle/progress environment
variables retain Task 06 semantics; uploaded collections are never rewritten
for lifecycle purposes.

EXIT/INT/TERM/HUP traps clean up only the supervisor's child and owned lock.
SIGINT/SIGTERM to the supervisor forwards TERM, waits at most five polling
seconds, then escalates to KILL if necessary. A normal/nonzero exit or SIGKILL
of the app child is reaped and releases the lock. A pre-existing lock is never
removed by a second instance. Launch also rejects a `kindle-chess` process
started outside the supervisor. Arbitrary binaries supplied for host tests must
not fork persistent children; the production binary runs in one process.

The cleanup repaint hook uses `KINDLE_CHESS_XREFRESH` (defaults to `xrefresh`)
and `KINDLE_CHESS_TIMEOUT` (defaults to `/usr/bin/timeout`). After its child has
run, the supervisor requests `xrefresh -display :0.0` with a five-second TERM
timeout and one-second KILL escalation, before releasing its lock. Failed
preflight/duplicate launches never request a repaint. The child exit code is
preserved even if repaint fails. The display address was probed from the actual
awesome environment, and the user confirmed the command removes all remaining chess pixels on the
Scribe. The hook is enabled by default; set an explicitly empty
`KINDLE_CHESS_XREFRESH` to disable it in host tests. Automatic cleanup after
each exit type still needs device confirmation. The existing native UI can
remain partly covered by chess pixels after process cleanup, so process exit
alone does not pass the display-restoration requirement.

The launcher does not stop or suspend stock services, change rotation/depth,
or change power/network policy. Native repaint uses the device-verified
`xrefresh -display :0.0` command. The app requires exclusive finger input on every launch and
acquires EVIOCGRAB on only its discovered finger-touch file before the first
frame. Acquisition errors abort startup; the scoped guard releases on drop and
the kernel closes descriptors on process death. No pen or power-button node is
grabbed. The Scribe promotion test and native finger/pen recovery after timeout and
SIGKILL passed on 2026-10-03; no opt-in flag is required.

Manual recovery, including a supervisor killed with SIGKILL:

1. In a second SSH shell, inspect `/tmp/kindle-chess.lock/launcher.pid` and
   `child.pid`; check that each still belongs to this launch using `ps` and its
   command. Do not blindly signal stale recorded PIDs.
2. Send TERM to the live supervisor; if it is gone, send TERM to the verified
   app child. Use KILL only if the app remains after five seconds.
3. Confirm `pidof kindle-chess` reports no app and the recorded supervisor is
   gone. Remove only the two PID files and empty lock directory if a stale lock
   remains.
4. Use normal Home/back or lock/unlock for a native repaint; verify native finger
   and pen behavior. Keep SSH available during lifecycle experiments.

## Remaining hardware work

Read-only inspection of the actual 5.19.6 device confirms `x` starts
Xorg/awesome/blanket through lxinit, and `kppmainapp`/`pillow` have respawn or
recovery paths. `lab126_gui` also contains restart/reboot monitoring. These
observations invalidate treating a broad service shutdown as a harmless recipe;
no shutdown was attempted. Exclusive finger input plus native repaint passed the bounded promotion and
exit probes. No stock service suspension is needed for that tested flow.
The user accepted the remaining non-sleep checks on 2026-10-03; natural
suspend/resume remains deferred.

The user deferred the natural 10-minute idle/suspend test on 2026-10-02 for
a later session. It remains a pending checkpoint.

Suspend/resume is unresolved: `fbink_reinit` is called before each presentation,
but it is not a resume detector, does not reopen evdev, and does not invalidate
the previous frame after a stock screensaver repaint. No claim of resume support
is made. Checkpoint E must establish power events, descriptor survival, repaint
ownership, and the needed recovery/invalidation before release.

The remaining physical checkpoint is natural idle/suspend/resume. Record power
events, descriptor survival and recovery/invalidation findings in
`docs/device/ks1-barolo.md` when the user resumes this test. The accepted
non-sleep checklist covers repeated exits, app kill, SIGTERM, interaction/
ghosting and modals, native recovery, unchanged rotation/depth/input ownership,
and timing observations. Do not infer resume reliability from those checks.
