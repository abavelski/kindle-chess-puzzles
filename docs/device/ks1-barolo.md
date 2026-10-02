# Kindle Scribe 1st generation (Barolo) — Task 00 device record

**Task status:** Awaiting HUMAN CHECKPOINT A  
**Target:** Kindle Scribe, first generation  
**Record date:** 2026-10-02 (partial; precise calibration and FBInk verification pending)
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

- Probe date: 2026-10-02; probe version 1 at repository commit `c95f740`.
- Device: Kindle Scribe, first generation; kernel/device tree report `Kindle Scribe`, firmware identifies Barolo.
- Board/model identifier: `com.lab126.eink.barolo.os`; software build `042-juno_1906_barolo_bellatrix3-483219`.
- Firmware: Kindle 5.19.6.
- Jailbreak/homebrew stack: Véra (user reported); `/mnt/us/koreader` present. `kpm`, `mrpi-helper`, and `dispatch-command` are not in the SSH session PATH; this does not establish whether they are installed elsewhere.
- Launch mechanism used for probe: KOReader SSH server, port 2222; direct `sh scripts/kindle_device_probe.sh` from `/mnt/us/kindle-chess-probe`.
- File-transfer mechanism: macOS `scp` over SSH, successfully deployed only the three Task 00 scripts and retrieved the report. Mac USB registry also detects a Kindle Scribe, but it is not mounted as an external disk.

## Kernel, ABI, and Rust target

- `uname -m`: `armv7l`.
- kernel: Linux `4.9.77-lab126`, SMP PREEMPT; build dated 2026-07-29.
- userland word size: 32 (`getconf LONG_BIT`); `/bin/sh` ELF header confirms ELF32, little-endian, ARM.
- libc: glibc 2.35 (`getconf GNU_LIBC_VERSION` and `ldd --version`).
- ARM hard-float/VFP evidence: `/bin/sh` resolves to `/bin/busybox.nosuid`; its ELF flags are `0x05000400` (EABI5, hard-float). `/lib/ld-linux-armhf.so.3` exists, and `/lib/ld-linux.so.3` links to it. CPU reports ARMv7, VFP/VFPv3/VFPv4 and NEON.
- planned Rust target: `armv7-unknown-linux-gnueabihf` is supported by the measured architecture/ABI. A compatible toolchain/sysroot and Rust hello-world execution remain unverified.

## Framebuffer / FBInk

- framebuffer node: sysfs `fb0`, driver name `hwtcon_v2`; `/dev/fb0` access not tested.
- visible/virtual size: sysfs `modes` reports `U:1860x2480p-0`; `virtual_size` is `1872,4960`. Visible dimensions and padding/double-buffer interpretation need verification before using a screen transform.
- bits per pixel: 8.
- stride: sysfs reports 1872 bytes.
- current rotation: sysfs `rotate=3`; its relationship to physical orientation is not yet verified.
- FBInk binary/version/revision: `fbink` not found in SSH PATH; no other binary located or tested. `input_scan` also not found.
- FBInk target reports Kindle: unknown; no CLI available in PATH.
- smoke draw succeeds: skipped because no known compatible FBInk CLI was available in PATH; no pixels written by Task 00.
- smoke text orientation: unknown; smoke test skipped.
- stock UI repaint observation: unknown; smoke test skipped.
- normal UI restored afterward: user confirmed normal stock UI and display/input after captures; FBInk-specific recovery was not tested.

## Input devices

| Role | Device | Kernel name | Important capabilities | Declared/observed ranges |
| --- | --- | --- | --- | --- |
| Finger | `/dev/input/event4` (`/dev/input/touch`) | `pt_mt` | multitouch position, tracking ID, pressure, touch-major; no button events observed | observed X 20–1792, Y 65–2446 across traces; declared ranges unknown |
| Physical stylus | `/dev/input/event3` | `WacomDigitizer` | `BTN_TOOL_PEN`, `BTN_TOUCH`; `ABS_X/Y`, pressure, distance, tilt | observed X 306–15240, Y 498–20462; declared ranges unknown |
| Virtual stylus stream | `/dev/input/event5` | `stylus-custom` | `BTN_TOOL_PEN`, `BTN_TOUCH`, `ABS_X/Y` observed | observed X 40–1814, Y 66–2435; declared ranges unknown |

### Finger raw-to-screen transform

- First finger capture: `finger-event4.bin` contained 0 bytes after 20 seconds while KOReader was open. User confirmed five taps during capture and KOReader responding. No transform can be inferred; exclusive input ownership is a hypothesis, not yet verified. `/dev/input/touch` was verified to link to `event4`. No capture process remained afterward.
- Second finger capture: `finger-stock-event4.bin`, collected after normal KOReader exit via a detached delayed invocation of the unchanged script, contained 1,184 bytes (74 events, 32-bit timeval layout, no trailing bytes). Four contacts were extracted: `(67,68)`, `(1763,107)`, `(878,1155)`, `(89,2399)`. Five-point transform remains pending because the fifth contact is missing. Observed event codes: `ABS_MT_TOUCH_MAJOR`, `ABS_MT_POSITION_X/Y`, `ABS_MT_TRACKING_ID`, `ABS_MT_PRESSURE`, and `EV_SYN`; no key/button events.
- Third finger capture: `finger-stock-inset-event4.bin` contained 1,936 bytes (121 events; 32-bit timeval, no trailing bytes). Five ordered contacts: `(20,530)`, `(1785,578)`, `(942,1236)`, `(82,2434)`, `(1789,2446)`. User reported stock menu interference during the previous attempt; this retry requested inset points. Actual inset distances are not measured, so these are not calibrated edges.
- axes swapped: no, based on the five ordered finger contacts.
- raw horizontal source/direction: `ABS_MT_POSITION_X`, increasing left to right.
- raw vertical source/direction: `ABS_MT_POSITION_Y`, increasing top to bottom.
- observed horizontal edges: five-point decoder estimates 51 to 1787 for the inset retry; earlier contact centers reached 67 and 1763. Neither is a declared/calibrated screen-edge range.
- observed vertical edges: five-point decoder estimates 554 to 2440 for the inset retry; earlier contact centers reached 68 and 2399. The retry upper points are substantially below the top screen edge.
- kernel-declared ABS ranges, if available: _pending_
- approximate pixel transform: axis assignment verified; scale/offset remain uncalibrated. Decoder was also run with the reported mode `1860x2480`, but treating inset tap positions as full-screen edges would produce an incorrect transform and must not be used.

### Finger/stylus separation

- separate event nodes: yes. Pen-only capture produced events on both physical `event3` and virtual `event5`, while simultaneous `event4` capture was empty. Finger events were observed on `event4`.
- distinctive BTN/ABS codes: finger uses `ABS_MT_POSITION_X/Y`, `ABS_MT_TRACKING_ID`, `ABS_MT_PRESSURE`, `ABS_MT_TOUCH_MAJOR`, without observed BTN codes. Pen uses `BTN_TOOL_PEN` and `BTN_TOUCH` plus `ABS_X/Y`; physical pen additionally reports pressure/distance/tilt.
- Pen traces: `stylus-event3.bin` contained 86,848 bytes / 5,428 records; `stylus-virtual-event5.bin` contained 7,136 bytes / 446 records. Both decode as 32-bit timeval with no trailing bytes and five contacts.
- Physical pen contact centers: `(548,583)`, `(15059,4309)`, `(7557,9474)`, `(379,20378)`, `(15234,20336)`. Virtual pen centers: `(65,70)`, `(1797,517)`, `(900,1128)`, `(45,2426)`, `(1813,2420)`. Both show unswapped axes increasing rightward/downward. User confirmed the second pen tap was below the stock menu, explaining its larger Y value; it is not a top-right screen-edge sample, so it must not be used for vertical scale/offset calibration.
- risk of duplicate finger+pen events: no finger events during this pen-only capture (`stylus-on-touch-event4.bin`, 0 bytes). Pen does appear on both `event3` and `event5`, so consuming both pen streams risks duplicate pen actions. Mixed finger/pen and palm behavior remain untested.

## Storage

- `/mnt/us` available: yes; mounted via `fsp`. `/mnt/base-us` is present, backed by `/dev/loop/0`.
- `/mnt/us` writable: yes; script deployment and report creation succeeded. Optional marker create/remove checks passed in `/mnt/us` and `/tmp`.
- `/var/local` available/writable: present, readable and reported writable for SSH root; backed by `/dev/mmcblk0p9` mounted on `/var/base-local`. No writes performed there.
- preliminary puzzle collection path: _pending_
- preliminary progress path: _pending_
- preliminary log path: Task 00 reports under `/mnt/us/kindle-chess-*`; local copies under ignored `probe-output/`.

No production path is final until the actual transfer/persistence workflow is verified.

## Lifecycle observations

- SSH lifecycle: first normal KOReader exit was followed by an SSH timeout; after reopening/restarting SSH, a later normal KOReader exit left SSH reachable. Cause of the first timeout is unknown. Detached delayed capture using existing `nohup` and `setsid` completed while the stock UI was open.
- stock UI repaints over direct FBInk output: unknown; smoke test skipped.
- input grab appears necessary for a foreground experiment: unknown. Zero finger events while KOReader was open, followed by events after normal exit, suggests exclusive ownership by KOReader; Task 00 never requested a grab and does not establish stock-UI grab requirements.
- service changes made during Task 00: **none by design**
- persistent display/input changes after probe: none observed; user confirmed normal stock UI, working finger touch and pen, and normal orientation/contrast after captures. The Task 00 probe/capture scripts made no service, framebuffer-mode, power, or network-configuration changes. SSH was enabled by the user through KOReader as a prerequisite; its server configuration is separate from the probe scripts.
- manual recovery used: normal KOReader exit/reopen through UI; no reboot or remote lifecycle/service commands used.

## Unresolved questions before Task 04

- Finger and pen traces establish axis assignment and device separation. Precise finger scale/offset and visible framebuffer geometry remain unresolved; physical recovery was confirmed by the user. Stock UI repaint over FBInk remains untested because no FBInk CLI was available.
- No compatible FBInk CLI has been tested; Task 04 needs a verified Kindle build/revision.
- `evtest` is present at `/usr/bin/evtest` according to PATH lookup, but execution returns `No such file or directory`; declared ABS ranges are therefore unknown.
- Rust hello-world execution and storage persistence across reboot/USB workflows remain unverified.

## HUMAN CHECKPOINT A result

- [x] read-only probe completed;
- [x] FBInk identification/smoke result recorded, or absence of FBInk recorded;
- [x] finger five-point trace captured/decoded;
- [x] stylus trace captured/decoded;
- [x] finger vs stylus device relationship identified;
- [ ] raw-to-screen transform known well enough for Task 04;
- [x] normal stock display/input verified after probe;
- [x] no persistent system state changed;
- [ ] measured sections above filled from evidence.

Task 00 must stay **Awaiting HUMAN CHECKPOINT A** until these boxes are complete.
