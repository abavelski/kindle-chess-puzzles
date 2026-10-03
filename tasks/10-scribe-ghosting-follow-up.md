# Task 10 — Diagnose and reduce residual Scribe ghosting

**Status:** Ready  
**Depends on:** Tasks 07-09 and the accepted regional-clean fix  
**Target:** Kindle Scribe first generation (Barolo), firmware 5.19.6  
**Baseline FBInk revision:** `92e127008145b2a22fba7c59815d810d716310dd`

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

Run this as a sequence of isolated experiments. Change **one display variable at a time**.
Do not combine several speculative fixes into one build and infer a cause from the result.

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

- `--clean-passes=1|2` — number of final clean refreshes for occupancy-changed clean
  regions; default `1`;
- `--clean-coverage=square|board|screen` — experimental clean coverage after a committed
  piece change; default `square`;
- `--explicit-gc16-clean` — request flashing explicit GC16 for clean regions instead of
  flashing AUTO; default off;
- `--serialize-clean-updates` — fence clean updates with completion waits; default off;
- `--mtk-fast-mode` — disable automatic REAGL promotion through FBInk's Kindle MTK
  helper; default off.

These are diagnostic controls, not user-facing product features. Keep their implementation
out of core and render crates.

Add host tests proving that with no new flags the presentation plan and C mapping are
byte-for-byte/field-for-field equivalent to today's baseline.

## Step 2 — Test a second refresh of the exact same final pixels

This is the highest-priority experiment.

Implement `--clean-passes=2` so each occupancy-changing clean rectangle is refreshed
twice **without writing an intermediate color or rerendering a different image**.

Requirements:

- pack the final region once or prove both passes use identical bytes;
- pass 1: submit current flashing AUTO clean request, then wait for its completion;
- pass 2: submit the same rectangle and same final bytes again, then wait again;
- do not duplicate unrelated partial UI regions;
- do not advance frame/damage history until the complete requested presentation sequence
  succeeds;
- log each pass separately.

Automated tests must prove:

- one pass remains the default;
- two-pass mode submits the same clean rectangle twice;
- both payloads are identical;
- a wait occurs after pass 1 before pass 2;
- there is no white/intermediate framebuffer write;
- partial-only selection changes are not duplicated.

### HUMAN CHECKPOINT 10A

Run the c7 -> d6 reproduction with:

1. baseline one-pass regional flashing AUTO;
2. two-pass regional flashing AUTO.

Use the same final board frame.

If pass 2 removes the recognizable ghost, record that result before trying another
waveform. Continue later steps only to determine whether the same quality can be obtained
with less flashing/latency.

If pass 2 makes no physical difference, keep it experimental and continue.

## Step 3 — Isolate refresh coverage

Using the same supported flashing AUTO behavior and the same final pixels, compare:

1. complete changed square;
2. complete board rectangle;
3. whole screen.

Do not change waveform at the same time.

The `board` and `screen` options are diagnostic fallbacks; they must not silently
become normal production behavior.

For each coverage, run one pass first. If Step 2 showed that two passes matter, repeat the
coverage comparison with two passes as a separate recorded series.

### HUMAN CHECKPOINT 10B

Record whether larger coverage clears c7 when square-only cleaning does not.

Interpretation:

- square fails, board/screen clears -> investigate MTK regional-update/controller history;
- all coverages clear equally -> prefer the smallest reliable region;
- all coverages fail -> proceed to explicit waveform/fencing tests.

## Step 4 — Add explicit flashing GC16 as a diagnostic mode

The existing `GrayPartial` uses explicit GC16 without flashing. Do not reuse that mode for
this experiment.

Add a project-owned FBInk refresh mode for **flashing explicit GC16**. Keep it inside the
platform/sys boundary. In the C bridge it must map to:

- `cfg.is_flashing = true`;
- `cfg.wfm_mode = WFM_GC16`.

Do not map it to REAGLD/GLD16.

For this experimental explicit mode, do not silently turn an unsupported error into a
successful AUTO result; that would invalidate the A/B test. Return/log the failure instead.
Keep the existing fallback behavior of existing production modes unchanged.

Add C contract tests for the exact fields and Rust tests for mode selection.

### HUMAN CHECKPOINT 10C

Compare flashing AUTO and flashing explicit GC16 with identical:

- c7 -> d6 final frame;
- region coverage;
- number of passes;
- wait policy.

If explicit GC16 is visibly cleaner, preserve that evidence and kernel log. If the result is
identical, do not claim FBInk AUTO selection was the cause.

## Step 5 — Test strict update fencing

Current production submits a batch and waits once on the final marker. Add an experimental
serialized-clean path without changing the default.

For `--serialize-clean-updates`:

- finish/wait for any preceding submitted partial work before the first clean request;
- submit a clean request;
- wait for that clean marker to complete before any next clean pass/region;
- if `--clean-passes=2`, wait between the two identical passes;
- do not advance presentation history until all requested work succeeds.

Keep timing logs separate enough to see the latency cost.

Add mock presenter tests asserting the exact submit/wait order.

### HUMAN CHECKPOINT 10D

Compare the best previous waveform/coverage combination with serialization off vs on.

If serialization removes the ghost, treat update ordering/controller history as the leading
explanation and then minimize the fencing needed while preserving the physical result.

## Step 6 — Test Kindle MTK fast mode / disable auto-REAGL promotion

Expose FBInk's `fbink_mtk_toggle_auto_reagl` only through the existing C/sys boundary.
Do not issue this ioctl from the application crate.

The desired experiment is "fast mode enabled", i.e. automatic REAGL promotion disabled.
The FBInk helper's naming is inverse to that desired state. Against the pinned header/source,
verify and document the polarity before implementation; the expected mapping is:

- experiment enabled -> `fbink_mtk_toggle_auto_reagl(fbfd, false)`;
- restore/default -> automatic REAGL behavior enabled again.

Add a safe Rust method with a name that describes the desired state, such as
`set_mtk_fast_mode(bool)`, so the inverse C API does not leak upward.

Requirements:

- default remains unchanged;
- if enabling the experimental mode fails, abort that experimental launch rather than
  pretending it is active;
- restore the previous/default state during orderly cleanup where supported;
- do not replace `fbink_wait_for_complete(LAST_MARKER)` with
  `fbink_wait_for_any_complete`; FBInk documents the latter as incompatible with fast
  mode;
- add C/Rust contract coverage where host-executable.

### HUMAN CHECKPOINT 10E

Run the same c7 -> d6 case with all other variables held constant, fast mode off vs on.

Also inspect kernel diagnostics for waveform promotion/missing-mode messages.

If fast mode changes the physical result, record it as a Scribe/MTK driver-policy finding,
not as a renderer fix.

## Step 7 — FBInk versus direct MTK ioctl A/B probe, only if still unresolved

Do this only if Steps 2-6 fail to produce a reliable clean result or if evidence specifically
implicates FBInk.

Create a tiny standalone diagnostic under the platform/sys boundary or `tools/`. It must
not become a second application renderer.

The probe should:

1. use the already-final known framebuffer content;
2. submit one carefully constructed MTK GC16/FULL refresh for a known rectangle;
3. wait for completion;
4. log every structure field used by the ioctl;
5. avoid lifecycle/service/framebuffer-format changes.

Compare that result with FBInk flashing explicit GC16 using the same pixels and rectangle.

If direct ioctl is cleaner, diff the complete request fields before changing FBInk
integration: waveform, update mode, histogram modes, temperature, flags, dithering,
marker, rectangle coordinates, and wait semantics.

If direct ioctl and FBInk are physically identical, do not label the remaining issue an
FBInk bug without new evidence.

### HUMAN CHECKPOINT 10F

A direct-ioctl result is valid only with physical observation and normal native recovery
after the probe.

## Step 8 — Select the smallest reliable production policy

Only after the experiment matrix identifies a physically reliable combination should Codex
propose a production default.

Preference order:

1. one clean pass on changed complete squares;
2. two serialized clean passes on changed complete squares;
3. larger board cleaning only if regional coverage is proven insufficient;
4. periodic/full-screen cleaning only if measured accumulation requires it.

Prefer fewer flashes and smaller coverage **only after** ghost removal is reliable.

Do not enable a faster waveform merely because it is faster. Pieces and dark board squares
contain grayscale transitions.

## Step 9 — Production verification

For the selected candidate:

- at least 30 committed piece moves;
- light and dark origin squares;
- captures;
- long moves;
- promotion;
- automatic replies;
- selection-only taps remain responsive and do not clean/flash unnecessarily;
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

Do not solve a refresh-quality problem by broadening lifecycle risk.

## Documentation requirements

After every physical checkpoint, update:

- `docs/GHOSTING_INVESTIGATION.md` with the exact result and interpretation;
- `docs/device/ks1-barolo.md` with verified device/firmware facts;
- this task's status only when the final production candidate passes.

Preserve hashes for important binaries/evidence. Do not commit user photos/raw framebuffer
dumps unless explicitly reviewed for inclusion.

## Acceptance criteria

Task 10 is Implemented only when one of these is true:

1. a smallest reliable refresh policy removes recognizable previous-piece silhouettes in
   the 30-move device verification and all regression/recovery checks pass; or
2. the controlled matrix exhausts the planned software/FBInk variables, the issue remains,
   and the documentation clearly narrows it to controller/firmware/panel behavior with no
   unsupported claim of a software fix.

In either case:

- baseline behavior remains recoverable;
- no white-pass or unloaded REAGLD mode is reintroduced;
- core/render architecture remains unchanged;
- host tests cover all new platform policy;
- physical findings are explicitly distinguished from framebuffer/API success.

## Suggested commits

Keep experimental code and accepted policy changes small. Examples:

- `test: specify repeated clean refresh sequencing`
- `kindle: add controlled ghosting refresh probes`
- `kindle: fence clean updates for Scribe experiment`
- `docs: record Scribe ghosting experiment results`
