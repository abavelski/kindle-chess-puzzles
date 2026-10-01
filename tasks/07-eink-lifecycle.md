# Task 07 — Add damage tracking, e-ink refresh policy, and safe Kindle lifecycle

**Status:** Ready  
**Depends on:** Tasks 04-06

## Outcome

Turn the correctness-first prototype into a predictable e-ink foreground app that refreshes only what changed and returns safely to the stock Kindle UI.

This is hardware-sensitive work. Do not guess commands or waveform choices.

## Damage tracking

At the shared renderer boundary, calculate conservative dirty rectangles from previous/current view state or renderer regions.

At minimum distinguish:

- first/full frame;
- square selection/deselection;
- two-square piece move;
- feedback icon/hint change;
- toolbar selected-state change;
- header change;
- description region change;
- promotion modal open/close;
- collection picker;
- complete puzzle navigation/reset/full board change.

Merge rectangles when that is more efficient/safe.

A missed dirty pixel is a correctness bug; slight overdraw is acceptable.

## Refresh policy

Using the pinned FBInk API and measured Scribe behavior:

- choose a fast partial mode for small black/white UI changes only if it is visually reliable;
- choose a higher-fidelity grayscale mode for board/piece/full changes as needed;
- use periodic full refresh if measurements show ghosting accumulation;
- fall back safely when a requested mode is unsupported;
- do not assume Kobo waveform behavior applies to Kindle.

Document the chosen modes and why.

Add debug timings for:

- process start;
- FBInk initialized;
- first usable frame;
- touch-to-submit;
- update completion when reliable.

## Lifecycle investigation

Determine the minimum safe foreground handoff on the actual firmware.

Questions:

- Does stock UI repaint over the app?
- Is stopping/suspending a specific UI component necessary?
- Is exclusive input grab needed?
- What must be restored on exit?
- How does power/suspend affect FBInk/input descriptors?
- Is `fbink_reinit` needed after resume?

Do not copy old Kindle or Kobo launcher recipes blindly.

## Launcher safety

Any script/service transition must have:

- finite timeout;
- logging;
- traps for EXIT/INT/TERM;
- restoration only of state the app changed;
- manual recovery instructions;
- idempotent repeated launch/exit.

Do not alter Wi-Fi, Bluetooth, CPU governor, OTA, or unrelated services.

## Automated tests

Host-test:

- damage regions for all major transitions;
- region clipping/merging;
- refresh-policy mapping from region/content class;
- lifecycle script static checks where practical;
- cleanup command construction/state machine using mocks, not real services.

## HUMAN CHECKPOINT E

On the Scribe:

1. launch/exit normally five times;
2. kill the app process intentionally;
3. send SIGTERM;
4. leave it idle long enough for a normal power/suspend event, then resume;
5. solve/browse at least 30 interactions while watching ghosting;
6. open/close promotion and collection modals repeatedly;
7. verify stock UI touch/display after each exit;
8. verify no permanent rotation/bit-depth/input grab remains;
9. record timing/ghosting observations.

## Acceptance criteria

- partial refresh never leaves stale changed pixels;
- ghosting is acceptable for normal puzzle use;
- full refresh remains available for recovery;
- lifecycle cleanup works on normal and abnormal exit;
- resume path is documented/tested if applicable;
- manual recovery is documented;
- no unrelated system state is changed.

## Suggested commit

`kindle: add damage refresh policy and safe lifecycle handoff`
