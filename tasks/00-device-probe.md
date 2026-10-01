# Task 00 — Probe the Kindle Scribe environment

**Status:** Ready  
**Depends on:** nothing

## Outcome

Replace hardware assumptions with a checked-in record for the actual first-generation Kindle Scribe.

This task produces documentation/probe tooling only. It does not implement the chess app.

## Questions to answer

Record:

- firmware version;
- kernel architecture and `uname -m`;
- hard-float/userland ABI evidence;
- libc version;
- board/device identifier;
- framebuffer device and geometry;
- FBInk identification/state and a known-good build/revision;
- current screen rotation;
- all relevant `/dev/input/event*` devices and capabilities;
- touchscreen absolute ranges;
- finger event sequence;
- stylus event sequence if present;
- raw-to-screen coordinate transform;
- whether the stock UI repaints over an FBInk test frame;
- whether input grabbing is necessary for a foreground experiment;
- available homebrew launch mechanisms on this jailbreak;
- writable/persistent paths suitable for puzzles, progress, logs;
- practical file-transfer path from the development machine.

## Safety

Use read-only commands first.

Do not stop or disable Kindle services in this task.

A framebuffer test may draw a temporary pattern only after the current screen/state can be restored. Keep the power button/reboot/manual recovery route documented.

Do not assume historical KUAL/framework commands.

## TDD/tooling approach

Create a small host-built/device probe script or binary whose output is deterministic enough to paste into the device record.

Where parsing is non-trivial, test the parser on saved sample output.

The probe may call existing safe tools available on the device, but it must degrade gracefully when a tool is missing.

## Required files

Create:

`docs/device/ks1-barolo.md`

It should include:

- date;
- Kindle firmware;
- jailbreak/homebrew mechanism;
- command outputs or summarized verified facts;
- FBInk revision/version tested;
- framebuffer facts;
- input device table;
- coordinate transform;
- lifecycle observations;
- chosen preliminary puzzle/progress locations;
- unresolved questions.

## HUMAN CHECKPOINT A

On the physical Scribe:

1. run the probe;
2. run a minimal FBInk identification/test frame;
3. restore/return to the stock UI;
4. touch at least the four screen corners and center while recording raw events;
5. verify which device is finger touch and which is stylus;
6. confirm no persistent display/input state was changed.

## Acceptance criteria

- `docs/device/ks1-barolo.md` exists with measured values;
- a Rust target/toolchain hypothesis is confirmed or corrected;
- FBInk can identify/use the Scribe enough for Task 04;
- touchscreen mapping is known enough for Task 04;
- the plan no longer depends on a fixed `eventN` guess;
- no stock service was permanently modified.

## Suggested commit

`docs: record Kindle Scribe device capabilities`
