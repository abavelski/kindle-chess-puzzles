# Kindle Scribe 1st generation (Barolo) — Task 00 device record

**Task status:** Awaiting HUMAN CHECKPOINT A  
**Target:** Kindle Scribe, first generation  
**Record date:** _pending device probe_  
**Repository task:** `tasks/00-device-probe.md`

This file is the checked-in summary of facts measured on the actual device. Do not paste unreviewed logs into Git: review them first even though the probe intentionally excludes common serial/network/account identifiers.

## Safety rules for this checkpoint

The Task 00 tooling is deliberately conservative:

- do **not** stop/disable Kindle services;
- do **not** run historical `stop framework` recipes;
- do **not** change framebuffer rotation, bit depth, waveform defaults, CPU settings, Wi-Fi, Bluetooth, or OTA settings;
- do **not** use `EVIOCGRAB`/exclusive input grabs;
- do **not** read/copy the raw framebuffer;
- use the normal Kindle UI or normal lock/unlock to redraw after the FBInk smoke message;
- keep the power button/reboot path available in case the UI behaves unexpectedly.

The general probe is read-only except for writing its report. The optional `--write-check` creates and immediately removes one tiny marker in `/mnt/us` and `/tmp`. The FBInk smoke helper is the only Task 00 helper that deliberately writes pixels to the screen.

## Files used by the checkpoint

From this repository:

- `scripts/kindle_device_probe.sh` — privacy-conscious environment/hardware report;
- `scripts/kindle_fbink_smoke.sh` — one small temporary FBInk message;
- `scripts/kindle_capture_input.sh` — short raw evdev capture, **without grabbing the device**;
- `tools/decode_evdev.py` — host-side raw evdev decoder and five-point transform helper.

You can run the shell scripts with `sh ...`; executable bits do not have to survive MTP/file transfer.

## Human checkpoint — exact procedure

### 0. Prepare

1. Charge the Scribe enough that a reboot is not a concern.
2. Put the Kindle on a harmless screen before touch capture. A disposable blank notebook/PDF page is ideal, because the stock UI will still receive the test taps.
3. Copy the repository (or just the three scripts) to a temporary user-storage directory, for example:

   ```text
   /mnt/us/kindle-chess-probe/
   ```

   Use your existing jailbreak transfer path (SSH/SFTP/MTP/etc.). Do **not** install anything into system directories for Task 00.
4. Open the jailbreak shell/SSH context you normally use for homebrew commands.

### 1. Run the read-only device probe

From the copied repository directory:

```sh
sh scripts/kindle_device_probe.sh
```

Expected output file:

```text
/mnt/us/kindle-chess-probe.txt
```

If `/mnt/us` is unavailable/unwritable it falls back to `/tmp/kindle-chess-probe.txt`.

Copy the report back to the development machine and review it. It should contain sections for:

- kernel/architecture and ABI clues;
- firmware/userland;
- board/model identifiers (not unique serials);
- framebuffer sysfs geometry/rotation;
- FBInk/input_scan availability;
- `/proc/bus/input/devices` plus event sysfs capabilities;
- storage candidate permissions;
- KPM/homebrew command/path hints.

The script intentionally does **not** collect serial numbers, network/MAC/IP configuration, account information, or framebuffer contents.

Optional, after the read-only run succeeds:

```sh
sh scripts/kindle_device_probe.sh --write-check /mnt/us/kindle-chess-probe-writecheck.txt
```

This only verifies create/remove access in `/mnt/us` and `/tmp`.

### 2. Run the minimal FBInk smoke test

First inspect the probe's **FBInk availability** section.

If `fbink` is in `PATH`:

```sh
sh scripts/kindle_fbink_smoke.sh
```

If you already have a known Kindle-compatible FBInk binary elsewhere:

```sh
FBINK=/path/to/fbink sh scripts/kindle_fbink_smoke.sh
```

Do **not** substitute an arbitrary binary built for another e-reader.

The helper:

- records the FBInk help/version target header;
- draws one small centered overlay message using FBInk defaults;
- does not clear the screen;
- does not change waveform/rotation/bit depth explicitly;
- does not stop the stock UI.

Record these observations:

1. Did FBInk return success?
2. Did `Kindle Chess FBInk probe` appear?
3. Was it upright and approximately centered?
4. Did the stock UI immediately overwrite it? If so, approximately how quickly?
5. If it remained, did a normal Home/back/page action remove it?
6. If not, did a normal power-button lock/unlock restore the stock screen?
7. Was the stock UI fully usable afterward?

The helper log is normally:

```text
/mnt/us/kindle-chess-fbink-smoke.txt
```

If the probe reports **no FBInk binary**, stop this sub-step and record that result. Do not guess a build target yet; the architecture/ABI report is specifically meant to tell us what to build next.

### 3. Identify candidate finger and stylus event devices

Use the general report. Look at:

- the `input_scan likely touch/tablet matches` section if `input_scan` exists;
- `/proc/bus/input/devices` names and `Handlers=... eventN` lines;
- the `[eventN]` sysfs summaries and `cap_abs`/`cap_key` fields.

Do not assume an event number from another Scribe.

Write candidates here before capture:

| Role | Candidate device | Kernel name | Why |
| --- | --- | --- | --- |
| Finger touchscreen | _pending_ | _pending_ | _pending_ |
| Stylus/tablet | _pending_ | _pending_ | _pending_ |
| Other relevant input | _pending_ | _pending_ | _pending_ |

### 4. Capture the finger five-point trace

Open the harmless blank page/notebook first because the Kindle UI will still see the taps.

For the finger candidate, replacing `eventN`:

```sh
sh scripts/kindle_capture_input.sh /dev/input/eventN finger 20
```

During the 20-second window, tap once at each point **in exactly this order**, with roughly a one-second pause between taps:

1. top-left, just inside the usable screen edge;
2. top-right;
3. center;
4. bottom-left;
5. bottom-right.

Avoid dragging. Use a single finger. The script does not grab input.

It writes, for example:

```text
/mnt/us/kindle-chess-input/finger-eventN.bin
/mnt/us/kindle-chess-input/finger-eventN.meta.txt
```

### 5. Capture the stylus trace

For the stylus/tablet candidate, repeat the same physical point order with the pen:

```sh
sh scripts/kindle_capture_input.sh /dev/input/eventM stylus 20
```

Use simple tip taps only; avoid side buttons/eraser unless you deliberately want to identify them.

If stylus and finger appear on the same event device, record that fact and make two separately labeled captures from the same event node.

### 6. Decode the captures on the development machine

Run the decoder tests first:

```sh
python3 -m unittest tests/test_decode_evdev.py
```

Then decode each capture:

```sh
python3 tools/decode_evdev.py /path/to/finger-eventN.bin --summary-only --five-point
python3 tools/decode_evdev.py /path/to/stylus-eventM.bin --summary-only --five-point
```

The decoder auto-detects common 16-byte (32-bit timeval) and 24-byte (64-bit timeval) Linux `input_event` layouts. If auto-detection fails, retry explicitly with `--layout 32` or `--layout 64` and record which one works.

After reading the framebuffer `virtual_size` from the main probe, rerun the finger trace with the exact screen size, for example (replace with measured numbers):

```sh
python3 tools/decode_evdev.py /path/to/finger-eventN.bin \
  --summary-only --five-point --screen WIDTHxHEIGHT
```

The five-point section reports:

- extracted contact centers in the known tap order;
- whether raw X/Y appear swapped;
- whether each raw axis increases or decreases across the screen;
- observed edge values;
- an approximate raw-to-pixel formula when `--screen` is supplied.

Corner taps are an **observed** range, not necessarily the kernel-declared ABS min/max. If `evtest` or another trusted input diagnostic is already available on the device, record its declared ABS min/max as well; do not install a random tool solely for this checkpoint.

### 7. Verify normal recovery

After the captures:

1. return to the Kindle home/normal UI using only ordinary UI/power controls;
2. confirm touch still works;
3. confirm pen still works;
4. confirm display orientation/contrast looks normal;
5. confirm no probe process is still running:

   ```sh
   ps | grep -E 'kindle_(device_probe|capture_input|fbink_smoke)|[c]at /dev/input/event'
   ```

6. reboot only if you normally would or if the UI behaves unexpectedly; a reboot should not be necessary for a successful Task 00 run.

## What to send back for completing Task 00

Please provide:

- `kindle-chess-probe.txt`;
- `kindle-chess-fbink-smoke.txt` if FBInk was available;
- the **decoded text output** from the finger and stylus captures;
- the two `.meta.txt` files;
- your answers to the seven FBInk observations above;
- the jailbreak/homebrew launch method you actually used (for example SSH, scriptlet, KPM launch, or something else);
- how you transferred the files to/from the Scribe.

Raw `.bin` input captures are useful if decoding is ambiguous, but do not commit them to Git. The repository ignores `probe-output/` for local copies.

---

# Measured device record

Fill/update this section only from the actual checkpoint evidence.

## Device and firmware

- Probe date: _pending_
- Device: Kindle Scribe, first generation: _pending confirmation_
- Board/model identifier: _pending_
- Firmware: _pending_
- Jailbreak/homebrew stack: _pending_
- Launch mechanism used for probe: _pending_
- File-transfer mechanism: _pending_

## Kernel, ABI, and Rust target

- `uname -m`: _pending_
- kernel: _pending_
- userland word size: _pending_
- libc: _pending_
- ARM hard-float/VFP evidence: _pending_
- planned Rust target: _pending; expected hypothesis is `armv7-unknown-linux-gnueabihf`, not confirmed yet_

## Framebuffer / FBInk

- framebuffer node: _pending_
- visible/virtual size: _pending_
- bits per pixel: _pending_
- stride: _pending_
- current rotation: _pending_
- FBInk binary/version/revision: _pending_
- FBInk target reports Kindle: _pending_
- smoke draw succeeds: _pending_
- smoke text orientation: _pending_
- stock UI repaint observation: _pending_
- normal UI restored afterward: _pending_

## Input devices

| Role | Device | Kernel name | Important capabilities | Declared/observed ranges |
| --- | --- | --- | --- | --- |
| Finger | _pending_ | _pending_ | _pending_ | _pending_ |
| Stylus | _pending_ | _pending_ | _pending_ | _pending_ |

### Finger raw-to-screen transform

- axes swapped: _pending_
- raw horizontal source/direction: _pending_
- raw vertical source/direction: _pending_
- observed horizontal edges: _pending_
- observed vertical edges: _pending_
- kernel-declared ABS ranges, if available: _pending_
- approximate pixel transform: _pending_

### Finger/stylus separation

- separate event nodes: _pending_
- distinctive BTN/ABS codes: _pending_
- risk of duplicate finger+pen events: _pending_

## Storage

- `/mnt/us` available: _pending_
- `/mnt/us` writable: _pending_
- `/var/local` available/writable: _pending_
- preliminary puzzle collection path: _pending_
- preliminary progress path: _pending_
- preliminary log path: _pending_

No production path is final until the actual transfer/persistence workflow is verified.

## Lifecycle observations

- stock UI repaints over direct FBInk output: _pending_
- input grab appears necessary for a foreground experiment: _pending; Task 00 itself never grabs input_
- service changes made during Task 00: **none by design**
- persistent display/input changes after probe: _pending confirmation_
- manual recovery used: _pending / none_

## Unresolved questions before Task 04

- _pending_

## HUMAN CHECKPOINT A result

- [ ] read-only probe completed;
- [ ] FBInk identification/smoke result recorded, or absence of FBInk recorded;
- [ ] finger five-point trace captured/decoded;
- [ ] stylus trace captured/decoded;
- [ ] finger vs stylus device relationship identified;
- [ ] raw-to-screen transform known well enough for Task 04;
- [ ] normal stock display/input verified after probe;
- [ ] no persistent system state changed;
- [ ] measured sections above filled from evidence.

Task 00 must stay **Awaiting HUMAN CHECKPOINT A** until these boxes are complete.
