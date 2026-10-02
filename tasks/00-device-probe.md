# Task 00 — Probe the Kindle Scribe environment

**Status:** Awaiting HUMAN CHECKPOINT A  
**Depends on:** nothing

## Outcome

Replace hardware assumptions with a checked-in record for the actual first-generation Kindle Scribe.

This task produces documentation/probe tooling only. It does not implement the chess app.

## Implemented tooling

The repository now contains:

- `scripts/kindle_device_probe.sh` — read-mostly, privacy-conscious environment probe;
- `scripts/kindle_fbink_smoke.sh` — a minimal one-message FBInk screen test;
- `scripts/kindle_capture_input.sh` — timed raw evdev capture without grabbing input;
- `tools/decode_evdev.py` — host-side decoder for common 32-bit/64-bit `input_event` layouts, contact extraction, and five-point transform hints;
- `tests/test_decode_evdev.py` — unit tests for the non-trivial raw-event parser;
- `docs/device/ks1-barolo.md` — exact human-checkpoint instructions plus the measured-device record template.

Host validation completed before this status change:

```sh
python3 -m unittest tests/test_decode_evdev.py
python3 -m py_compile tools/decode_evdev.py
sh -n scripts/kindle_device_probe.sh
sh -n scripts/kindle_fbink_smoke.sh
sh -n scripts/kindle_capture_input.sh
```

Task 00 is intentionally **not** marked Implemented until the physical Scribe checkpoint fills the measured sections in `docs/device/ks1-barolo.md`.

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
- touchscreen absolute ranges or observed edge ranges sufficient for mapping;
- finger event sequence;
- stylus event sequence if present;
- raw-to-screen coordinate transform;
- whether the stock UI repaints over an FBInk test frame;
- whether input grabbing appears necessary for a foreground experiment;
- available homebrew launch mechanisms on this jailbreak;
- writable/persistent paths suitable for puzzles, progress, logs;
- practical file-transfer path from the development machine.

## Safety

Use read-only commands first.

Do not stop or disable Kindle services in this task.

The general probe never writes the framebuffer and never grabs input. The FBInk smoke helper writes one small overlay message only; normal Kindle UI/power controls are used to redraw afterward.

Do not assume historical KUAL/framework commands.

Do not commit raw input captures or unreviewed probe logs. Summarize reviewed measurements in `docs/device/ks1-barolo.md`.

## TDD/tooling approach

The probe scripts degrade gracefully when optional tools are missing.

The non-trivial binary evdev parser is host-tested against synthetic 32-bit and 64-bit Linux `input_event` records, including multi-touch contact extraction and partial trailing records.

The five-point capture order is fixed:

1. top-left;
2. top-right;
3. center;
4. bottom-left;
5. bottom-right.

That known order allows the host decoder to infer whether axes are swapped/reversed and to produce an approximate raw-to-screen formula once framebuffer dimensions are known.

## Required files

Created:

`docs/device/ks1-barolo.md`

It contains:

- exact device-probe instructions;
- safety/recovery constraints;
- date/firmware/jailbreak placeholders;
- framebuffer/FBInk record;
- input device table;
- coordinate-transform record;
- lifecycle observations;
- storage/path record;
- unresolved questions;
- HUMAN CHECKPOINT A checklist.

## HUMAN CHECKPOINT A

Follow `docs/device/ks1-barolo.md` exactly.

On the physical Scribe:

1. run the read-only probe;
2. run the minimal FBInk identification/test message if a Kindle-compatible `fbink` binary is available;
3. restore/return to the stock UI using normal Kindle controls;
4. identify candidate finger/stylus event devices from the report;
5. capture the five-point finger trace;
6. capture the stylus trace;
7. decode the captures on the development machine;
8. verify which device/codes represent finger versus stylus;
9. verify the raw-to-screen mapping;
10. confirm no persistent display/input/system state was changed.

## Acceptance criteria

Automated/tooling side:

- [x] device record/checkpoint file exists;
- [x] read-mostly probe exists and avoids fixed `eventN` assumptions;
- [x] raw capture helper does not grab input;
- [x] non-trivial raw parser has automated tests;
- [x] host syntax/parser tests pass.

Physical side, still pending:

- [ ] `docs/device/ks1-barolo.md` contains measured values;
- [ ] Rust target/toolchain hypothesis is confirmed or corrected;
- [ ] FBInk can identify/use the Scribe enough for Task 04, or the missing-binary/build requirement is precisely identified;
- [ ] touchscreen mapping is known enough for Task 04;
- [ ] finger and stylus input relationship is known;
- [ ] stock UI repaint/lifecycle observation is recorded;
- [ ] no stock service was permanently modified;
- [ ] normal display/touch/pen behavior is confirmed after probing.

## Suggested final checkpoint commit

After the human results are reviewed and written into the device record:

`docs: record Kindle Scribe device capabilities`
