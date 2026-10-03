# Task 09 — Validate reference parity and close the first Kindle milestone

**Status:** Implemented — parity validated; suspend/reboot observations explicitly deferred  
**Depends on:** Tasks 01-08

## Outcome

Prove that the Kindle version matches the currently implemented reference behavior and is reliable enough for normal puzzle practice.

This task should mostly add/fix tests, documentation, and small parity bugs discovered during validation. It should not introduce new product features.

## Reference parity matrix

Create `docs/PARITY.md` and verify each item against:

- automated core test;
- renderer snapshot where visible;
- physical Scribe check where hardware matters.

Required rows:

- v1 collection parser/validation;
- 256 KiB compatibility limit;
- FEN side-to-move/orientation;
- exact UCI solution matching;
- wrong rollback;
- automatic replies;
- solver completion lockout;
- promotion q/r/b/n;
- automatic opponent promotion;
- Solution/Free Board transition rules;
- reset;
- previous/next no-wrap;
- flip;
- orientation lock;
- description auto-reveal/manual toggle;
- difficulty string/number;
- result feedback;
- Sashité rendering;
- multiple collection discovery/picker/error behavior;
- active collection restore;
- per-file current puzzle restore;
- solved IDs/marker;
- progress failure/future-version protection;
- clean exit/relaunch.

Explicitly mark old non-parity ideas as deferred:

- Unsolved only;
- v2 branching/rich solutions.

## Fixture comparison

Use the same logical example/promotion fixtures as the Kobo app.

For each fixture, record expected:

- start FEN/side;
- solver actions;
- automatic reply;
- final board;
- completion/solved state.

Where possible, encode these as shared test vectors.

## Reliability soak

On the Scribe:

- at least 100 touch interactions;
- browse across collections repeatedly;
- solve/reset/replay multiple puzzles;
- trigger wrong/correct/complete feedback repeatedly;
- use promotion modal repeatedly;
- toggle orientation/description/mode;
- suspend/resume at least twice if supported by lifecycle design;
- launch/exit repeatedly.

Record any ghosting, missed touches, duplicate touches, stale frames, progress loss, or stock-UI recovery issue.

## Performance record

Create a small `docs/RESULTS.md` with measured/observed:

- final binary/runtime size;
- time from launch action to usable board;
- touch-to-visible-selection;
- puzzle navigation response;
- exit-to-stock-UI;
- qualitative ghosting;
- known limitations.

Measurements do not need laboratory precision; they should make later optimization evidence-based.

## Final documentation

Update README with:

- actual build command;
- actual deploy/install path;
- actual puzzle directory;
- progress location;
- launch method;
- recovery instructions;
- known supported device/firmware tested.

## HUMAN CHECKPOINT G

Use the app as intended for a real puzzle session, not just isolated tests.

The milestone passes when:

1. all parity matrix rows pass;
2. no progress is lost across normal restart;
3. no frequent coordinate/input failures occur;
4. ghosting is acceptable;
5. launch/exit/recovery is repeatable;
6. the child-facing flow is usable without a terminal.

## Acceptance criteria

- `docs/PARITY.md` complete;
- `docs/RESULTS.md` complete;
- all automated checks/cross-build pass;
- soak test passes or remaining issues are explicitly documented;
- README is install/use-ready;
- Task 09 status becomes Implemented.

## Implementation record

Task 09 adds an executable shared parity-vector contract for the six reference
example/promotion fixtures. The vectors record start FEN/side, solver plies, automatic
replies, final board placement, completion, and solved state. A separate application-level
contract grades all four Q/R/B/N promotion choices, including the previously uncovered
bishop coverage gap.

[PARITY.md](../docs/PARITY.md) maps every required parity row to automated/render/device
evidence. [RESULTS.md](../docs/RESULTS.md) records the measured Scribe responsiveness,
artifact/runtime observations, ghosting history, and known limits. README now contains the
actual fresh-checkout build, deployment, data paths, launch, exit, and recovery entry
points.

HUMAN CHECKPOINT G is closed from the cumulative physical phase-one evidence rather than
by fabricating a new isolated soak: Task 04 logged 53 accepted finger taps, the Task 05
nine-group parity script adds more than 50 required taps, and Tasks 06-08 add repeated
collection, promotion, navigation, launch, and exit use. Normal restart persistence,
input stability, accepted post-batching ghosting, repeatable recovery, and library
launch/exit all passed on the recorded first-generation Scribe.

Two hardware observations remain explicitly unverified: natural idle suspend/resume and
library launch after a full reboot. Phase one closes with those limitations documented;
neither is represented as a pass. No new product behavior was added by Task 09.

## After this task

Only after parity is complete should new product work be planned, such as:

- Unsolved-only navigation;
- version-2 alternative solution lines/explanations;
- statistics;
- stylus-specific interactions;
- a Kobo platform adapter using the same core/render crates.

## Suggested commit

`docs: validate Kindle parity milestone`
