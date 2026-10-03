# Task 10 — Diagnose and reduce residual Scribe ghosting

**Status:** Ready  
**Depends on:** Tasks 07-09 and the accepted regional-clean fix  
**Target:** Kindle Scribe first generation (Barolo), firmware 5.19.6  
**Baseline FBInk revision:** `92e127008145b2a22fba7c59815d810d716310dd`  
**KOReader reference:** `koreader@06c87b8c424e78f7b7f889defcfd1fecc662a990` and `koreader-base@4c5f7a487d9ff4b707282fdea9bd3a8fdbae988f`

## Outcome

Determine why a faint previous-piece silhouette can remain physically visible after a
piece move even though the final visible framebuffer is already correct, then keep the
smallest refresh-policy change that reliably removes the artifact on the target Scribe.

This is a display-path investigation. Do not change chess state, puzzle behavior, piece
artwork, renderer clearing, or damage calculation to hide the symptom.

The accepted production control is the current regional-clean behavior:

- startup/recovery: flashing AUTO over the whole frame;
- occupancy-changing board cells: flashing AUTO over complete changed squares;
- ordinary UI damage: partial AUTO;
- no white intermediate frame;
- no REAGLD/GLD16 experiment;
- no periodic full-screen cleaning.

Read [the measured investigation](../docs/GHOSTING_INVESTIGATION.md) and
[the earlier fix plan](../docs/PIECE_GHOSTING_FIX.md) before changing code.

## New same-device evidence: KOReader is the primary reference

KOReader is installed on the same first-generation Scribe and ordinary book page turns do
not show comparable visible ghosting. This is stronger evidence than a Kobo comparison
because the panel, controller, Kindle firmware, temperature, and physical device are held
constant.

The pinned KOReader sources show a materially different Kindle MTK refresh policy:

- Kindle MTK initialization enables the driver's so-called **fast mode** to prevent silent
  promotion of refresh requests to REAGL;
- on MTK Kindles, KOReader sets `waveform_reagl = MTK_WAVEFORM_MODE_GLR16` and
  `waveform_partial = waveform_reagl`;
- GLR16 is MTK waveform **mode 4**;
- KOReader promotes REAGL/GLR16 requests to `UPDATE_MODE_FULL`;
- it waits for the previous relevant update before REAGL/GC16 work;
- it fences every FULL update by waiting for that marker to complete immediately after
  submission.

This strategy must be tested before generic double-GC16 or larger-region workarounds.

Important distinction:

- **GLR16 / REAGL = mode 4** on the Kindle MTK API;
- **GLD16 / REAGLD = mode 5**.

The already-failed white-screen experiment was mode 5. It does **not** rule out mode 4.

Pinned reference sources:

- https://github.com/koreader/koreader/blob/06c87b8c424e78f7b7f889defcfd1fecc662a990/frontend/device/kindle/device.lua
- https://github.com/koreader/koreader-base/blob/4c5f7a487d9ff4b707282fdea9bd3a8fdbae988f/ffi/framebuffer_mxcfb.lua
- https://github.com/NiLuJe/FBInk/blob/92e127008145b2a22fba7c59815d810d716310dd/fbink.c
- https://github.com/NiLuJe/FBInk/blob/92e127008145b2a22fba7c59815d810d716310dd/eink/mtk-kindle.h

## Established facts that must not be re-litigated

The controlled queen c7 -> d6 reproduction already proved that the origin square in
`/dev/fb0` is uniformly the correct dark-square gray after the move and is byte-for-byte
identical to another empty dark square. The physical screen can still show a faint queen.
Treat the remaining artifact as downstream of the final visible framebuffer unless a new
measurement disproves that finding.

Two experiments are already rejected:

1. **White intermediate clearing pass.** It worsened native UI ghosting and did not
   reliably remove the piece ghost. Do not restore it.
2. **REAGLD/GLD16.** The target kernel logged
   `waveform mode[5] not loaded night_mode[0]` while the framebuffer remained correct.
   Do not retry mode 5 on this firmware.

A successful FBInk call is not enough to declare a waveform visually valid. Every
experimental mode requires a physical-screen checkpoint.

## Research basis

These are hypothesis sources, not proof of this device's exact mechanism:

- FBInk's pinned Kindle/MTK refresh implementation and API:
  https://github.com/NiLuJe/FBInk/blob/92e127008145b2a22fba7c59815d810d716310dd/fbink.c
- FBInk documents the Kindle MTK auto-REAGL/fast-mode helper:
  https://github.com/NiLuJe/FBInk/blob/886f25f13368859ad8a899b88d04c26e19cda32e/fbink.h
- KOReader enables MTK "fast mode" on Kindle so the driver does not silently promote
  refreshes to REAGL:
  https://github.com/koreader/koreader/blob/06c87b8c424e78f7b7f889defcfd1fecc662a990/frontend/device/kindle/device.lua
- KOReader's MTK framebuffer backend contains explicit waveform selection and refresh
  fencing behavior:
  https://github.com/koreader/koreader-base/blob/4c5f7a487d9ff4b707282fdea9bd3a8fdbae988f/ffi/framebuffer_mxcfb.lua
- KOReader issue #8880 records a case where a second full refresh of the same final image
  clears residue left by the first:
  https://github.com/koreader/koreader/issues/8880
- KOReader issue #15757 records Scribe-class display state/corruption that clears after a
  stronger full refresh:
  https://github.com/koreader/koreader/issues/15757
- Community reports are consistent with imperfect first-pass cleaning, including:
  https://www.reddit.com/r/koreader/comments/1otl6gs/full_page_refresh_isnt_perfect/

## Operating rules for Codex

Run this as a sequence of controlled experiments.

Normally change **one display variable at a time**. There is one deliberate exception:
the first KOReader-parity probe reproduces the complete known-good KOReader policy bundle —
GLR16/FULL + MTK fast mode + strict completion fencing — because the first question is
whether the mature same-device strategy solves the artifact at all. If it succeeds, perform
the ablation in Step 3 to identify which parts are actually required.

For every host-testable behavioral change:

1. add the failing test;
2. prove that it fails for the expected reason;
3. implement the smallest change;
4. run the focused test;
5. run the relevant workspace checks.

For every device experiment:

1. record source commit and installed binary SHA-256;
2. record the exact experimental flags/settings;
3. reproduce the same controlled queen c7 -> d6 move on the black-facing Free Board;
4. preserve the final framebuffer pixels; do not alter chess/render state between
   comparison refreshes;
5. record the refresh rectangle, requested waveform/mode, flashing/full strength,
   marker/wait behavior, submission/completion timing, and relevant kernel diagnostics;
6. obtain a human physical-screen observation or photo;
7. classify c7 as **none**, **faint but recognizable**, or **clear ghost**;
8. restore the accepted baseline after a rejected experiment.

Do not mark a step successful from framebuffer captures alone.

## Step 0 — Preserve a reproducible control

Before adding experimental behavior:

- run all existing checks;
- confirm the current default still maps `RefreshMode::Clean` and
  `RefreshMode::Full` to flashing AUTO in `crates/fbink-sys/c/fbink_bridge.c`;
- confirm `submit_regions` still batches submissions and performs one final wait;
- preserve the current release binary hash and the known c7 -> d6 reproduction notes.

Do not modify the default behavior in this step.

Expected control result: the accepted substantial improvement remains, with the known faint
dark-square ghost still reproducible.

## Step 1 — Add experimental controls without changing defaults

Add narrowly scoped controls so later device builds can vary refresh behavior without
rewriting production code for every probe. Prefer command-line options parsed in the Kindle
binary and represented as a small platform experiment configuration.

Required controls:

- `--koreader-clean` — use the KOReader-parity clean path defined in Step 2; default off;
- `--glr16-clean` — request GLR16/REAGL mode 4 with FULL semantics for clean regions;
  default off;
- `--mtk-fast-mode` — enable Kindle MTK fast mode / disable automatic REAGL promotion;
  default off;
- `--serialize-clean-updates` — fence clean updates with completion waits; default off;
- `--clean-passes=1|2` — number of final clean refreshes; default `1`;
- `--clean-coverage=square|board|screen` — experimental clean coverage; default `square`;
- `--explicit-gc16-clean` — request flashing explicit GC16; default off.

`--koreader-clean` is a convenience preset equivalent to GLR16 clean + MTK fast mode +
strict clean-update fencing + one pass + square coverage. These remain diagnostic controls,
not user-facing product features.

These are diagnostic controls, not user-facing product features. Keep their implementation
out of core and render crates.

Add host tests proving that with no new flags the presentation plan and C mapping are
byte-for-byte/field-for-field equivalent to today's baseline.

## Step 2 — Reproduce KOReader's MTK page-refresh policy first

This is now the highest-priority experiment.

### 2.1 Add GLR16/REAGL mode 4

Add a project-owned clean mode inside the Kindle platform/sys boundary. On the pinned
FBInk build, map it through `WFM_REAGL`; on Kindle MTK this resolves to
`MTK_WAVEFORM_MODE_GLR16` / mode 4.

The effective request must be GLR16/REAGL + FULL using the final framebuffer pixels only.
With FBInk, pair `WFM_REAGL` with `cfg.is_flashing = true` and verify the resulting
low-level request against the pinned FBInk source.

Do **not** use `WFM_REAGLD`; that is mode 5 and is already rejected on this firmware.
Do not silently fall back to AUTO for this experimental mode, because that would invalidate
the A/B test. Add C contract tests and Rust mode-selection tests.

### 2.2 Match KOReader's MTK fast mode

Expose FBInk's `fbink_mtk_toggle_auto_reagl` only through the existing C/sys boundary.

KOReader's `_MTK_ToggleFastMode(true)` sets
`UPDATE_FLAGS_FAST_MODE | UPDATE_FLAGS_MODE_FAST_FLAG`.

FBInk's helper uses inverse naming. Against the pinned implementation:

- `fbink_mtk_toggle_auto_reagl(fbfd, false)` corresponds to KOReader fast mode enabled;
- `fbink_mtk_toggle_auto_reagl(fbfd, true)` restores the default that permits automatic
  REAGL upgrades.

Wrap this as a safe Rust method such as `set_mtk_fast_mode(bool)`. Default app behavior
must remain unchanged. Experimental fast-mode failure must be surfaced rather than ignored.
Restore the default on orderly cleanup where supported.

Do not replace `fbink_wait_for_complete(LAST_MARKER)` with
`fbink_wait_for_any_complete`; FBInk documents the latter as incompatible with fast mode.

### 2.3 Match KOReader's FULL-update fencing

For the KOReader-parity path:

1. wait for preceding relevant work before the GLR16 request;
2. submit the GLR16/FULL clean rectangle;
3. wait for that FULL update to complete before allowing the next refresh;
4. advance frame/damage history only after the sequence succeeds.

Add mock presenter tests proving the exact submit/wait order.

### 2.4 Optional runtime confirmation from installed KOReader

If practical without changing KOReader's normal behavior, capture debug output for one or
two page turns and record the actual `WFM` and `UPD` values. The source-level expectation
is mode 4 + FULL. This is useful evidence but must not block the experiment.

### HUMAN CHECKPOINT 10A — KOReader parity

Compare:

1. baseline one-pass regional flashing AUTO;
2. `--koreader-clean` on the same c7 -> d6 final frame and square coverage.

Confirm final framebuffer bytes are unchanged before judging the panel. Record whether mode
4 is accepted, physical c7 classification, timing, kernel diagnostics, and native recovery.

If the KOReader-parity path removes the recognizable ghost, proceed to Step 3 and minimize
the policy. If it does not materially improve the result, proceed to Step 4.

## Step 3 — Ablate the successful KOReader policy

Run this only if Step 2 materially improves or removes the ghost.

Keep the same pixels, square coverage, and one pass. Remove one component at a time:

1. GLR16/FULL + fast mode + strict fencing — reference;
2. GLR16/FULL + strict fencing, fast mode off;
3. GLR16/FULL + fast mode, old batch/final-wait behavior;
4. GLR16/FULL alone.

Interpretation:

- only the full bundle works -> preserve and document the interaction;
- GLR16/FULL alone works -> waveform/update pairing is the leading explanation;
- fencing is required -> update ordering/controller state matters;
- fast mode is required -> driver promotion/state policy matters.

Choose the smallest combination that reproduces the clean physical result.

### HUMAN CHECKPOINT 10B

Repeat c7 -> d6 at least five times with the smallest candidate and include another move
that vacates a dark square. If stable, jump to Step 8 production verification.

## Step 4 — Test a second refresh of the exact same final pixels

Use this fallback only if the KOReader-parity path does not solve the artifact.

Implement `--clean-passes=2` so each occupancy-changing clean rectangle is refreshed twice
without an intermediate color or different rendered frame. Start with the accepted flashing
AUTO control so pass count is the only changed variable.

Requirements:

- both passes use identical final pixels;
- wait after pass 1 before pass 2;
- do not duplicate unrelated partial UI regions;
- advance history only after the sequence succeeds;
- log each pass separately.

Automated tests must prove one pass remains the default, payloads are identical, the wait is
ordered correctly, no white/intermediate write occurs, and selection-only changes are not
duplicated.

### HUMAN CHECKPOINT 10C

Compare one-pass AUTO against two-pass AUTO on the same c7 -> d6 final frame.

## Step 5 — Isolate refresh coverage

Using the best supported single-waveform behavior found so far, compare complete changed
square, complete board, and whole screen. Do not change waveform, pass count, or fencing at
the same time.

### HUMAN CHECKPOINT 10D

Record whether larger coverage clears c7 when square-only cleaning does not.

- square fails, board/screen clears -> investigate MTK regional-update/controller history;
- all clear equally -> prefer the smallest reliable region;
- all fail -> proceed to explicit GC16.

## Step 6 — Compare explicit flashing GC16

Add a project-owned **flashing explicit GC16** mode:

- `cfg.is_flashing = true`;
- `cfg.wfm_mode = WFM_GC16`.

Do not map it to REAGL or REAGLD. Do not silently convert an unsupported experimental
request into AUTO. Compare against flashing AUTO with identical pixels, coverage, pass
count, and fencing.

### HUMAN CHECKPOINT 10E

If explicit GC16 is visibly cleaner, preserve the evidence and kernel log. If it is
identical, do not claim FBInk AUTO selection was the cause.

## Step 7 — FBInk versus direct MTK ioctl A/B, only if still unresolved

Do this only if Steps 2-6 fail or evidence specifically implicates FBInk.

The first direct comparison must reproduce the KOReader request, not an arbitrary GC16
request:

1. use the already-final known framebuffer content;
2. submit MTK GLR16/mode 4 + FULL for the known rectangle;
3. match KOReader's relevant temperature, flags, dithering, histogram fields, marker, and
   wait semantics;
4. wait for completion;
5. log every request field.

Compare against the FBInk `WFM_REAGL` + FULL path with the same pixels and rectangle.

If direct ioctl is cleaner, diff the complete requests before changing FBInk integration.
If direct ioctl and FBInk are physically identical, do not label the remaining issue an
FBInk bug without new evidence.

### HUMAN CHECKPOINT 10F

A direct-ioctl result is valid only with physical observation and normal native recovery.

## Step 8 — Select and verify the production policy

Only after the experiment matrix identifies a physically reliable combination should Codex
propose a production default.

Preference order is evidence-driven:

1. one GLR16/FULL clean on changed complete squares, with only the KOReader-derived
   fast-mode/fencing pieces proven necessary;
2. one existing AUTO/GC16 clean if another sequencing change alone is sufficient;
3. two serialized clean passes on changed complete squares;
4. larger board cleaning only if regional coverage is proven insufficient;
5. periodic/full-screen cleaning only if measured accumulation requires it.

For the selected candidate verify:

- at least 30 committed piece moves;
- light and dark origin squares;
- captures, long moves, promotion, and automatic replies;
- selection-only taps remain responsive and do not flash unnecessarily;
- navigation, flip, reset, modals, collection switching, progress persistence, X exit, and
  native repaint recovery remain correct;
- record touch-to-submit and completion timings;
- inspect for recognizable previous-piece silhouettes after every move.

Also rerun:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

and the documented Kindle cross-build/package checks.

## Stop/rollback conditions

Immediately stop an experimental series and restore the accepted baseline if any of these
occur:

- physical screen becomes blank/white while the framebuffer is correct;
- kernel reports an unloaded waveform;
- native UI recovery fails;
- awesome/Xorg ownership is not restored correctly;
- finger/pen recovery regresses;
- experiment requires changing framebuffer format/rotation or stopping additional services.

Mode 5 / GLD16 / REAGLD remains rejected. A failure of that mode must never be generalized to mode 4 / GLR16 / REAGL without actually testing mode 4.

Do not solve a refresh-quality problem by broadening lifecycle risk.

## Documentation requirements

After every physical checkpoint, update:

- `docs/GHOSTING_INVESTIGATION.md` with the exact result and interpretation;
- `docs/device/ks1-barolo.md` with verified device/firmware facts;
- this task's status only when the final production candidate passes.

Preserve hashes for important binaries/evidence. Do not commit user photos/raw framebuffer
dumps unless explicitly reviewed for inclusion.

If the KOReader-parity path succeeds, document the precise difference from the old app path: waveform, FULL/PARTIAL mode, MTK fast-mode state, wait ordering, and coverage. Do not summarize it merely as "KOReader refresh."

## Acceptance criteria

Task 10 is Implemented only when one of these is true:

1. a smallest reliable refresh policy removes recognizable previous-piece silhouettes in
   the 30-move device verification and all regression/recovery checks pass; or
2. the controlled matrix exhausts the planned KOReader/FBInk/software variables, the issue remains,
   and the documentation clearly narrows it to controller/firmware/panel behavior with no
   unsupported claim of a software fix.

In either case:

- baseline behavior remains recoverable;
- no white-pass or unloaded REAGLD/mode-5 path is reintroduced;
- GLR16/mode 4 is evaluated independently from failed GLD16/mode 5;
- core/render architecture remains unchanged;
- host tests cover all new platform policy;
- physical findings are explicitly distinguished from framebuffer/API success.

## Suggested commits

Keep experimental code and accepted policy changes small. Examples:

- `test: specify KOReader-parity refresh sequencing`
- `kindle: add GLR16 full-refresh probe`
- `kindle: match MTK fast-mode policy for ghosting test`
- `kindle: fence Scribe GLR16 clean updates`
- `docs: record Scribe KOReader comparison`
