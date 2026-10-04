# Kindle Scribe 1st generation (Barolo) — Task 00 device record

**Task status:** Implemented
**Target:** Kindle Scribe, first generation  
**Record date:** 2026-10-02; HUMAN CHECKPOINT A passed
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
| Finger touchscreen | `/dev/input/event4` | `pt_mt` | multitouch axes; `/dev/input/touch` links here |
| Stylus/tablet | `/dev/input/event3` | `WacomDigitizer` | pen buttons, position, pressure, distance, tilt |
| Other relevant input | `/dev/input/event5` | `stylus-custom` | virtual pen stream; captured alongside physical pen |

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

After verifying the visible framebuffer size with `fbset`/FBInk, rerun the finger trace with that screen size. Do not use padded virtual dimensions as visible dimensions. For example (replace with measured numbers):

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

Corner taps are an **observed** range, not necessarily the kernel-declared ABS min/max. Inset/menu-displaced taps must not be treated as full-screen edges. If `evtest` or another trusted input diagnostic is already available on the device, record its declared ABS min/max as well; do not install a random tool solely for this checkpoint. The stock diagnostic on this Scribe uses `evtest info /dev/input/eventN`, which reports capabilities/ranges and exits without capturing or grabbing input; `--help` is interpreted as a device filename.

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

- framebuffer node: `fb0`, driver ID `hwtcon_v2`; FBInk successfully initialized and drew through the framebuffer.
- visible size: **1860×2480**, independently reported by plain `fbset` and FBInk variable framebuffer info.
- virtual size: **1872×4960**, reported by sysfs and `fbset`; these padded virtual dimensions are not the visible viewport.
- bits per pixel / format: 8, FBInk reports `Y8` grayscale.
- stride: 1872 bytes (sysfs and FBInk fixed framebuffer info).
- framebuffer memory length: 9,285,120 bytes (FBInk fixed framebuffer info; no raw framebuffer read/copy performed).
- current rotation: `3`, reported by FBInk as counterclockwise 270°. Text was physically upright in the tested stock notebook orientation.
- FBInk binary/version/revision: existing `/mnt/us/koreader/fbink`, build identifier `92e1270`. It is outside the SSH PATH; the initial PATH-only probe therefore missed it. No FBInk binary was downloaded or installed.
- FBInk target reports Kindle: yes; identifies Kindle Scribe / Barolo on Bellatrix3, 300 dpi. Help reports Draw/Bitmap/Fonts/OpenType enabled, Image/Input/ButtonScan disabled.
- smoke command: `FBINK=/mnt/us/koreader/fbink sh scripts/kindle_fbink_smoke.sh` from `/mnt/us/kindle-chess-probe`; scripts unchanged from `c95f740`.
- smoke draw succeeds: yes, exit status 0. Logged overlay region: left 546, top 1224, width 768, height 32; no full-screen clear requested.
- smoke text orientation/position: user saw “Kindle Chess FBInk probe” upright and centered both horizontally and vertically over the open blank notebook page.
- stock UI repaint observation: text stayed visible on the idle notebook page during the human observation; no timed persistence duration was measured. Normal Home action repainted the display and removed it.
- normal UI restored afterward: yes; user confirmed Home restored the normal screen and the UI remained usable. Lock/unlock and reboot were unnecessary.
- `input_scan`: not found in SSH PATH.

## Input devices

| Role | Device | Kernel name | Important capabilities | Declared/observed ranges |
| --- | --- | --- | --- | --- |
| Finger | `/dev/input/event4` (`/dev/input/touch`) | `pt_mt` | multitouch position, tracking ID, pressure, touch-major; no button events observed | declared X 0–1859, Y 0–2479; observed X 20–1792, Y 65–2446 across traces |
| Physical stylus | `/dev/input/event3` | `WacomDigitizer` | `BTN_TOOL_PEN`, `BTN_TOUCH`; `ABS_X/Y`, pressure, distance, tilt | declared X 0–15624, Y 0–20832; observed X 306–15240, Y 498–20462 |
| Virtual stylus stream | `/dev/input/event5` | `stylus-custom` | `BTN_TOOL_PEN`, `BTN_TOUCH`, `ABS_X/Y` observed | declared X 0–1860, Y 0–2480; observed X 40–1814, Y 66–2435 |

### Finger raw-to-screen transform

- First finger capture: `finger-event4.bin` contained 0 bytes after 20 seconds while KOReader was open. User confirmed five taps during capture and KOReader responding. No transform can be inferred; exclusive input ownership is a hypothesis, not yet verified. `/dev/input/touch` was verified to link to `event4`. No capture process remained afterward.
- Second finger capture: `finger-stock-event4.bin`, collected after normal KOReader exit via a detached delayed invocation of the unchanged script, contained 1,184 bytes (74 events, 32-bit timeval layout, no trailing bytes). Four contacts were extracted: `(67,68)`, `(1763,107)`, `(878,1155)`, `(89,2399)`. That trace alone could not establish a five-point transform because the fifth contact was missing. Observed event codes: `ABS_MT_TOUCH_MAJOR`, `ABS_MT_POSITION_X/Y`, `ABS_MT_TRACKING_ID`, `ABS_MT_PRESSURE`, and `EV_SYN`; no key/button events.
- Third finger capture: `finger-stock-inset-event4.bin` contained 1,936 bytes (121 events; 32-bit timeval, no trailing bytes). Five ordered contacts: `(20,530)`, `(1785,578)`, `(942,1236)`, `(82,2434)`, `(1789,2446)`. User reported stock menu interference during the previous attempt; this retry requested inset points. Actual inset distances are not measured, so these are not calibrated edges.
- axes swapped: no, based on the five ordered finger contacts.
- raw horizontal source/direction: `ABS_MT_POSITION_X`, increasing left to right.
- raw vertical source/direction: `ABS_MT_POSITION_Y`, increasing top to bottom.
- observed horizontal edges: five-point decoder estimates 51 to 1787 for the inset retry; earlier contact centers reached 67 and 1763. Neither is a declared/calibrated screen-edge range.
- observed vertical edges: five-point decoder estimates 554 to 2440 for the inset retry; earlier contact centers reached 68 and 2399. The retry upper points are substantially below the top screen edge.
- kernel-declared ABS ranges: `evtest info /dev/input/event4` reports MT X 0–1859 and MT Y 0–2479. Slots 0–1, touch-major/pressure/distance 0–255, tool type 0–2, tracking IDs 0–65535. Traces use tracking ID -1 for contact release.
- raw-to-visible-pixel transform for the tested orientation: `px = clamp(raw_x, 0, 1859)`, `py = clamp(raw_y, 0, 2479)`. Declared ranges match visible pixel bounds exactly, and ordered taps verify unswapped, increasing axes. This is sufficient for initial Task 04 mapping; other orientations and subpixel/physical-edge accuracy remain untested. Decoder was also run with `--screen 1860x2480`; its corner-fitted formula must not be used because the requested inset/menu-displaced tap positions are not screen-edge calibration points.

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
- preliminary puzzle collection path: unknown; `/mnt/us` is a verified writable/transferable parent, but no production collection directory was created or selected.
- preliminary progress path: unknown; `/mnt/us` is a verified writable parent, but no production progress directory was created or selected.
- preliminary log path: Task 00 reports under `/mnt/us/kindle-chess-*`; local copies under ignored `probe-output/`.

No production path is final until the actual transfer/persistence workflow is verified.

## Lifecycle observations

- SSH lifecycle: first normal KOReader exit was followed by an SSH timeout; after reopening/restarting SSH, a later normal KOReader exit left SSH reachable. Cause of the first timeout is unknown. Detached delayed capture using existing `nohup` and `setsid` completed while the stock UI was open.
- stock UI repaints over direct FBInk output: idle notebook retained the overlay during observation; pressing Home removed it and restored the normal Home screen. No stock service ownership changes were needed for this one-message test.
- input grab appears necessary for a foreground experiment: unknown. Zero finger events while KOReader was open, followed by events after normal exit, suggests exclusive ownership by KOReader; Task 00 never requested a grab and does not establish stock-UI grab requirements.
- service changes made during Task 00: **none by design**
- persistent display/input changes after probe: none observed; user confirmed normal stock UI, working finger touch and pen, and normal orientation/contrast after captures. The Task 00 probe/capture scripts made no service, framebuffer-mode, power, or network-configuration changes. SSH was enabled by the user through KOReader as a prerequisite; its server configuration is separate from the probe scripts.
- manual recovery used: normal KOReader exit/reopen for captures; normal Home action after FBInk smoke. No lock/unlock, reboot, or remote lifecycle/service commands used.

## Unresolved questions before Task 04

- FBInk `92e1270` is verified for identification/text overlay only. Pin the full corresponding upstream revision for reproducible builds; the bundled CLI has Image/Input disabled, so project-owned Gray8 presentation still needs a configured library build and device verification in Task 04.
- Continuous app refresh/input ownership and long-term stock repaint behavior remain untested; this checkpoint establishes only idle overlay persistence and normal Home recovery. Do not infer a safe service shutdown recipe.
- Other display orientations, mixed finger/pen/palm input, and exact physical-edge accuracy remain untested. Initial portrait finger mapping is supported by declared ranges, framebuffer geometry, and ordered traces.
- Rust target ABI is confirmed from the device; cross-toolchain/sysroot selection and Rust hello-world execution remain future build validation, not claims established by this probe.
- Storage persistence across reboot and the Mac MTP workflow remain untested; production collection/progress paths will be chosen in the storage/deployment tasks. SSH/SCP and writable `/mnt/us` are verified.

## HUMAN CHECKPOINT A result

- [x] read-only probe completed;
- [x] FBInk identification/smoke result recorded, or absence of FBInk recorded;
- [x] finger five-point trace captured/decoded;
- [x] stylus trace captured/decoded;
- [x] finger vs stylus device relationship identified;
- [x] raw-to-screen transform known well enough for Task 04;
- [x] normal stock display/input verified after probe;
- [x] no persistent system state changed;
- [x] measured sections above filled from evidence, with remaining unknowns explicitly recorded.

HUMAN CHECKPOINT A passed on 2026-10-02. Task 00 is **Implemented**; later-task validation limits are listed above.


---

# Task 04 — HUMAN CHECKPOINT B

**Status:** Checkpoint completed, with documented lifecycle/refresh limitations
**Automated implementation:** host gates and a glibc-2.35-compatible ARMv7 cross-build passed after the device-discovered correction.

The FBInk-linked Rust binary was exercised on the actual Scribe on 2026-10-02. See the observations and final checkpoint result below; the initial CI artifact could not launch on the device.

## Safety constraints

Use the same conservative rules as Task 00:

- do not stop or disable Kindle services;
- do not change framebuffer bit depth, rotation, waveform defaults, power, networking, or OTA settings;
- do not use `EVIOCGRAB` or another exclusive input grab;
- do not install the experiment into system directories;
- keep the normal power-button/reboot recovery path available.

The Task 04 binary opens FBInk and the selected finger event node only. It does not write puzzle/progress files and it does not open either stylus event stream.

## Obtain and stage the checkpoint binary

1. Open the latest green GitHub Actions run for the current Task 04 commit.
2. Download the artifact named `kindle-chess-task04-armv7`.
3. Extract the `kindle-chess` binary on the development machine.
4. Transfer it with the same SSH/SCP path already verified in Task 00 to:

   ```text
   /mnt/us/kindle-chess-task04/kindle-chess
   ```

5. In the Kindle shell:

   ```sh
   mkdir -p /mnt/us/kindle-chess-task04
   chmod 755 /mnt/us/kindle-chess-task04/kindle-chess
   ```

Do not copy the binary into `/usr`, `/opt`, or another system location.

## Launch

Task 00 observed that finger events were unavailable while KOReader itself was foreground, then became readable after a normal KOReader exit. Therefore:

1. keep the existing jailbreak SSH shell open;
2. exit KOReader through its normal UI so the stock UI is foreground;
3. confirm the SSH shell still responds;
4. start the app in that same shell:

   ```sh
   /mnt/us/kindle-chess-task04/kindle-chess 2>/mnt/us/kindle-chess-task04/checkpoint-b.log
   ```

The app runs in the foreground. **Ctrl-C** in that shell is the Task 04 controlled exit path.

If the SSH connection does not survive the normal KOReader exit, stop here and report that result. Do not compensate by stopping Kindle services or adding an input grab; use the already-verified detached-launch technique only after the launch/exit path is adjusted deliberately.

## What to verify

While the app is running:

1. Verify the board is upright, square, and fully visible with the toolbar/navigation/status regions on-screen.
2. With a finger, tap near the four board corners and the center. The log records normalized coordinates and the renderer hit target as `Square(...)`; confirm the visible coordinate labels and logged target agree.
3. Tap every visible control: Free Board, Note/description, orientation lock, Reset, Flip, Previous, and Next. Confirm the expected visible state changes.
4. Make at least ten additional selections/moves across different parts of the board. Confirm there is no accumulating offset or coordinate drift.
5. Use the stylus for several taps over the board and controls. Confirm the app does not react and no new `kindle-chess: tap ...` line is produced for those stylus contacts.
6. Press Ctrl-C in the controlling SSH shell.
7. Use a normal Home/back action or lock/unlock if needed to make the stock UI repaint. Confirm stock finger touch, pen input, orientation, and contrast are normal.

Reboot only if normal UI recovery fails or the device behaves unexpectedly.

## Evidence to report

Send back:

- whether the initial frame was upright and correctly fitted;
- the observed result of corner/center finger taps;
- whether every visible control responded correctly;
- whether at least ten additional selections/moves remained aligned;
- whether stylus taps caused any app action;
- whether Ctrl-C exited cleanly enough for the stock UI to recover normally;
- `/mnt/us/kindle-chess-task04/checkpoint-b.log` if anything is ambiguous or fails.

After those observations are recorded, this section will be updated with the measured result and Task 04 can be marked **Implemented**.

## Task 04 launch attempt — 2026-10-02

- SSH root login on port 2222 with an empty password succeeded while the user reported the stock blank notebook open.
- Tested commit: `e8cd032cd02a6ff327fe8493f6874973ded239ab`, successful CI run 17 (`37040572579`), artifact `kindle-chess-task04-armv7` (`11241607301`).
- Artifact ZIP SHA-256 matched GitHub: `25a381f8aa70ef861f67e133f18009e4b9f2779e70b817bc1c2e0fcd10891ed6`.
- Binary staged at `/mnt/us/kindle-chess-task04/kindle-chess`; host/device SHA-256 both `e19756a148dc576472accb5fd4ebeb30d6afcd56020326d07371bbb19c0837cb`.
- `fbset` reconfirmed visible 1860×2480, virtual 1872×4960, 8 bpp.
- Launch failed in the dynamic loader before app initialization: `/lib/libc.so.6: version GLIBC_2.38 not found`. No app frame was drawn. The device's measured glibc is 2.35; a green cross-build alone did not ensure runtime compatibility.
- Revised checkpoint plan: build against glibc 2.35 or earlier, verify that binary on the device, then proceed with HUMAN CHECKPOINT B. Keep all service/input/framebuffer safety constraints above. Physical observations remain pending.

### Compatible build and touch-discovery correction

- Rebuilt unchanged application source with Zig 0.13.0 (`zig cc -target arm-linux-gnueabihf.2.35`, `zig ar`, `zig ranlib`) through the existing `KINDLE_LINKER`, `KINDLE_AR`, and `KINDLE_RANLIB` overrides in `scripts/check-kindle.sh`. Local tools/wrappers and logs are under ignored `probe-output/task04/`.
- The device loader successfully resolved that binary's dependencies. Startup then failed with `no unique finger touchscreen matched name/capabilities` before first presentation.
- Read-only sysfs inspection measured `pt_mt` event mask `f` and ABS mask `ee18000 0`. The original parser discarded all but the last word, losing the upper 32-bit MT capability bits.
- Added host regression tests for the measured 32-bit mask and 64-bit masks. Confirmed failure with the original parser, then corrected parsing to combine the low two words on the 32-bit Scribe. No fixed event-node selection was added.
- `scripts/check.sh` passed after the correction (format, clippy, workspace tests, Python decoder tests); the glibc-2.35 cross-build passed.
- Corrected local binary SHA-256: `ae69860f6ad79ebe9d34557ee089b8515b7271bc8460ab1dab37dfa6bf197e4a`. It includes the uncommitted parser correction over `e8cd032` and is staged as `/mnt/us/kindle-chess-task04/kindle-chess-glibc235`.
- Launch command: `/mnt/us/kindle-chess-task04/kindle-chess-glibc235 2>/mnt/us/kindle-chess-task04/checkpoint-b-fixed.log` in a foreground SSH PTY. Ctrl-C in that PTY is the controlled termination route.
- Startup log reports pinned FBInk `v1.25.0-520-g92e12700`, 1860×2480, stride 1872, 8 bpp, rotation 3. Discovery selects `pt_mt` by capabilities, rejecting `WacomDigitizer` and `stylus-custom`. The process remains running awaiting human checks; visible output and touch correctness are not yet confirmed.
- CI currently still uses a toolchain that produced the incompatible artifact. Its build environment must target glibc 2.35 or earlier before future CI artifacts can be relied on for deployment.

### HUMAN CHECKPOINT B observations

- Initial frame geometry: **PASS**, user confirmed the board is upright, square, fully visible, and the controls readable. Corner/center touch mapping, controls, repeated moves, pen filtering, and exit/recovery checks remain pending.
- Checkpoint interrupted while the user was away: user reported the Scribe slept and, after waking, showed the stock blank notebook. On reconnect, SSH worked and no `kindle-chess` process was listed; the cause of termination is unverified. Restarted the same corrected binary in a foreground SSH PTY with log `/mnt/us/kindle-chess-task04/checkpoint-b-restart.log`. No service, power-policy, or framebuffer-mode changes made. Human checks restart from the initial board.
- After restart, user confirmed the initial board visible and visually correct, enabled Free Board, and moved the white king f4→a8 successfully. Log independently records `ToggleMode`, `Square(37)` (f4), and `Square(0)` (a8). Remaining corners, center, repeated interactions, other controls, pen filtering, and recovery remain pending.
- Corner/center move mapping: **PASS**, user confirmed a8→h8→a1→h1→d4 all landed correctly. Log agrees: square indices 0→7→56→63→35 in white orientation.
- Stock UI interference observed: user saw the native menu briefly after tapping a8, then it disappeared quickly. Input is deliberately not grabbed and stock services remain running; concurrent stock input handling is an inference consistent with this observation. This is a lifecycle limitation to preserve in the checkpoint findings, not grounds for an unverified service/grab change during Task 04.
- Visible controls: **PASS**, user confirmed Note, Lock, Flip, Reset, Next, Previous, and mode toggle behave as requested. Logged targets match the instructed controls.
- Stock UI interference is repeatable on the top board row, including e8, per user report. It is not limited to the first a8 tap. Correct hit mapping does not eliminate this concurrent stock-UI behavior; stock ownership/handoff remains unresolved for the later lifecycle task.
- Repeated move stability: **PASS**, user confirmed all ten instructed king moves f4→e4→d4→c3→d3→e3→f3→g3→h3→h2→g2 landed correctly, king remained centered, and taps had no coordinate drift or misses.
- Refresh artifact: user observed small residual traces on the previous square temporarily after moving the king, followed by a full flash/redraw that cleared them. This is a transient ghosting/refresh observation, not evidence of persistent missing pixels. Exact duration was not measured. Preserve for later refresh-policy investigation; no waveform changes made during Task 04.
- Pen-only test: the restart log stayed at 53 accepted taps, with last target `Square(54)` (g2); no additional chess action was logged during the reported pen contacts. User reported the board disappeared on the first stylus tap and the underlying stock notebook took over. This demonstrates stock repaint/input contention; pen filtering has supporting log evidence, but must be repeated in the single-instance retest below.
- Correction to the sleep/restart process observation: plain `ps` did not show the app's other PTYs. A later `ps -ef`/`pidof` check found both the pre-sleep instance (PID 19611) and the restarted instance (PID 21175). The earlier inference of termination was incorrect; sleep survival/stock redraw occurred with the original app still present. Thus the restarted touch/control observations were made with two app instances and refresh observations may be confounded by concurrent presentation. Preserve those observations, but repeat key checks with exactly one process before declaring checkpoint success.
- Stopped restarted instance through Ctrl-C in its controlling SSH PTY; terminated original instance with SIGTERM after verifying its `/proc/19611/cmdline` matched the experiment binary. `pidof kindle-chess-glibc235` then returned no PIDs. No stock services or display modes changed. Normal stock-UI recovery confirmation and a single-instance retest remain pending.
- Stock recovery after both app instances stopped: **PASS**, user confirmed native notebook finger/menu interaction and pen drawing work normally. No reboot was needed for this recovery. Single-instance retest follows to remove the concurrent-process uncertainty.
- Single-instance retest launch: verified no existing binary PIDs with `pidof`, then launched the same compatible binary in one foreground SSH PTY, logging to `/mnt/us/kindle-chess-task04/checkpoint-b-single.log`. Post-launch `pidof` reported exactly one PID. Startup selected the finger touchscreen successfully.
- Single-instance retest interrupted: user reported the underlying stock screen was Home, not the blank notebook. A pen tap at the overlay FREE control opened a stock Amazon book/purchase suggestion. The app log contains startup only, with no accepted taps, consistent with intentional pen filtering and stock input handling. No evidence of a purchase was reported. This confirms stock input/repaint contention with one instance, independent of the earlier duplicate-process issue.
- Stopped the single instance with Ctrl-C; subsequent `pidof kindle-chess-glibc235` returned no PIDs. The app remains a finger-only foreground overlay experiment; it does not own input or suppress stock UI. Repeat only over a harmless blank notebook, with finger taps for app controls. The single-instance move/refresh retest remains pending.
- User returned to a blank notebook through the normal UI. Verified no existing app PIDs before relaunch; launched one instance with log `/mnt/us/kindle-chess-task04/checkpoint-b-notebook-single.log` and confirmed exactly one PID afterward. Finger-only move/refresh retest requested.
- Single-instance notebook retest: user confirmed f4→e4→d4→e4→f4 finger moves work. Temporary traces on previous squares still appear; user also sees the underlying stock UI while the board refreshes. These artifacts persist with one app instance and cannot be attributed solely to duplicate presentation. Record for later lifecycle/refresh-policy work; no service/grab/waveform changes made.


### Final HUMAN CHECKPOINT B result — 2026-10-02

The user confirmed the final single-instance corner/control checks worked and requested completion without further repeated finger tests. The single-instance log confirms corner targets and Note/Lock/Flip/Reset/Next/Previous; the final d4/f4 pair from the instructed sequence is not present in that log, so do not infer extra logged moves beyond the evidence. Earlier corner/center checks and the ten-move sequence were confirmed by both user observations and their logs; single-instance center/repeated moves were subsequently confirmed separately.

- [x] shell launch and correct initial shared-renderer board geometry;
- [x] corner/center mapping and all visible finger controls;
- [x] at least ten aligned selections/moves, no coordinate drift;
- [x] pen contacts produce no chess tap/actions; stock UI still responds to pen;
- [x] Ctrl-C and SIGTERM terminate the experiment; native finger and pen functionality recovered normally;
- [x] no persistent display/input changes observed; no reboot needed for normal recovery;
- [x] host quality gates and compatible ARMv7 cross-build passed.

Final single instance stopped with Ctrl-C at the user's request. `pidof kindle-chess-glibc235` returned no PIDs afterward. No app instances remain. Device logs retained locally under ignored `probe-output/task04/`.

Known limitations: stock input/repaint contention on top-row taps and pen contacts, underlying stock UI visible during refresh, and temporary previous-square traces before the full flash. The overlay is finger-only and should be tested over a disposable blank notebook, not Home/store screens. These do not establish exclusive UI ownership or a production lifecycle solution. No destructive lifecycle commands were tried; later task files remain unchanged.

CI remediation: the cross-build runner is pinned to Ubuntu 22.04 instead of `ubuntu-latest`, so the GNU cross-toolchain targets glibc 2.35. The deployed binary was independently built with Zig 0.13.0 against glibc 2.35. Task 04 is **Implemented** with the above limitations recorded.


---

# Task 05 — HUMAN CHECKPOINT C

**Status:** Passed on 2026-10-02
**Automated implementation:** complete; host gates and the glibc-2.35-compatible ARMv7 cross-build are green.

Task 05 uses one bundled six-puzzle collection and no durable collection/progress filesystem work. Keep the Task 04 safety/lifecycle constraints: use a disposable blank notebook underneath the overlay, run exactly one app instance, do not stop Kindle services, do not add an input grab, and use Ctrl-C in the controlling SSH shell to exit.

## Obtain and stage the Task 05 binary

1. Open the latest green GitHub Actions run for the current Task 05 checkpoint commit.
2. Download artifact `kindle-chess-task05-armv7`.
3. Extract and transfer the `kindle-chess` binary to:

   ```text
   /mnt/us/kindle-chess-task05/kindle-chess
   ```

4. On the Scribe:

   ```sh
   mkdir -p /mnt/us/kindle-chess-task05
   chmod 755 /mnt/us/kindle-chess-task05/kindle-chess
   pidof kindle-chess || true
   ```

   Do not launch if another experiment instance is still running.

5. With a harmless blank notebook open under the overlay, launch one foreground instance:

   ```sh
   /mnt/us/kindle-chess-task05/kindle-chess 2>/mnt/us/kindle-chess-task05/checkpoint-c.log
   ```

The header identifies the current puzzle as `1/6` through `6/6`. Use finger taps for app interaction; stylus behavior remains the Task 04 finger-filtering/lifecycle limitation and does not need to be re-proven here.

## Checklist

### 1. One-move puzzle — `1/6 one-move`

- Confirm the pieces are legible at normal reading distance and controls are comfortably tappable.
- Try wrong move `d7→d8`: the board must roll back and show the obvious wrong result.
- Then solve `d7→e8`: completion feedback appears, the description auto-reveals, and the solved marker appears.
- Tap another board square after completion: the solved position must not change.
- Tap Reset: the original FEN returns and the attempt feedback/description clears.

### 2. Three-ply puzzle — `2/6 three-ply`

Tap Next, then:

- play `e2→e6`;
- verify the stored black reply `f7→f8` appears automatically and intermediate Correct feedback is visible;
- finish with `e6→f7`;
- verify Complete feedback and description reveal.

### 3. White queen promotion — `3/6 promotion-white-queen`

Tap Next:

- play `a7→a8`;
- verify the promotion modal blocks ordinary board/control taps;
- tap Cancel once and confirm the pawn returns to a7 with no solution progress;
- repeat `a7→a8`, choose **Q**, and confirm completion.

### 4. Underpromotion — `4/6 promotion-white-knight`

Tap Next:

- play `b7→b8`, choose **Q** first; confirm Wrong and rollback to the original pawn;
- repeat `b7→b8`, choose **N**; confirm the knight appears and the puzzle completes.

### 5. Black promotion / auto-orientation — `5/6 promotion-black-rook`

Tap Next:

- verify the board automatically faces Black before making a move;
- play `h2→h1` in logical board coordinates and choose **R**;
- confirm the rook promotion completes.

### 6. Automatic opponent promotion — `6/6 promotion-auto-reply`

Tap Next:

- confirm Next is visibly disabled/inert at the collection end;
- play `h2→h3`;
- verify the stored reply automatically promotes `a2→a1=Q` without opening the promotion modal;
- finish `h3→h8` and confirm completion.

### 7. Free Board + reset

Navigate back to `1/6 one-move` with Previous:

- confirm Previous becomes visibly disabled/inert at `1/6`;
- toggle **FREE**;
- move the white king `f4→a8`; it must move without grading/solving;
- tap Reset; the original FEN returns while FREE remains selected;
- toggle FREE off; Solution mode restarts from the original FEN.

### 8. Orientation lock + flip + navigation

From `1/6`:

- tap **LOCK**, then **FLIP**;
- navigate forward several puzzles, including the black-to-move promotion puzzle;
- verify the manually chosen orientation remains fixed while LOCK is selected;
- Flip again while locked and verify it still works;
- unlock; the board must not jump immediately;
- navigate once more and verify normal side-to-move auto-orientation resumes.

### 9. Description toggle before/after solve

Return to `1/6 one-move` and Reset:

- tap **NOTE** before solving; description must show without solving or advancing;
- hide it again;
- solve `d7→e8`; description must auto-reveal;
- tap NOTE again and verify it can be hidden after solving.

## Exit and recovery

- Stop the single app instance with Ctrl-C in its controlling SSH shell.
- Verify `pidof kindle-chess` returns no app PID.
- Use normal Home/back or lock/unlock if the stock UI needs a repaint.
- Confirm native finger/pen interaction, orientation, and contrast are normal.
- Do not reboot unless normal recovery fails.

## What to report

Please send:

- pass/fail for checklist items 1–9;
- whether the Sashité pieces were legible at normal reading distance;
- whether all controls felt comfortably tappable;
- any wrong/complete/promotion feedback that was unclear;
- any coordinate drift, missed taps, stale pixels, or stock-UI repaint behavior that differs materially from Task 04;
- `/mnt/us/kindle-chess-task05/checkpoint-c.log` if anything is ambiguous or fails.

Task 05 is **Implemented**; completed observations and recovery are recorded below.

## Task 05 deployment — 2026-10-02

- Tested commit: `a79e1d525461e1a49fd9cc8123774d956c66a871`, successful CI run 24 (`37049603381`), artifact `kindle-chess-task05-armv7` (`11245434619`).
- Artifact ZIP SHA-256 verified against GitHub: `9ce270c584d79f311c07dda07576e00a3a9db5bc43137fa87f693dd5ef5b54fa`.
- Binary staged at `/mnt/us/kindle-chess-task05/kindle-chess`; host/device SHA-256 matched: `27a44712e94243980adc9aab523722632e398675f017c1fa8b02a7cca19fd721`.
- SSH root login with empty password on port 2222 succeeded. User reported a blank stock notebook open. `fbset` reconfirmed visible 1860×2480, virtual 1872×4960, 8 bpp.
- Device dynamic loader resolved all dependencies successfully. Verified no Task 04/05 app PIDs before launch, then exactly one app PID afterward.
- Launch: `/mnt/us/kindle-chess-task05/kindle-chess 2>/mnt/us/kindle-chess-task05/checkpoint-c.log` in a foreground SSH PTY. Ctrl-C is the controlled exit path. Startup reports pinned FBInk, expected Scribe geometry, and capability-selected `pt_mt` finger input.
- Local artifact/evidence directory: ignored `probe-output/task05/`. Human checklist observations remain pending. No stock-service, input-grab, rotation, or waveform changes made.

## HUMAN CHECKPOINT C observations

- Test 1 (`1/6 one-move`): **PASS**, user confirmed pieces legible/controls comfortably tappable, wrong d7→d8 rollback and Wrong feedback, correct d7→e8 completion with description/solved marker, unchanged solved position after extra board tap, and Reset restoring the original position with transient feedback/description cleared.
- Test 2 (`2/6 three-ply`): **PASS**, user confirmed e2→e6 produces Correct feedback and automatic f7→f8 black reply, followed by e6→f7 completion and description reveal.
- Test 3 (`3/6 promotion-white-queen`): **PASS**, user confirmed a7→a8 opens the Q/R/B/N chooser, ordinary board/Reset taps leave the chooser and position unchanged, Cancel preserves the pawn on a7, and repeating the move with Q produces queen promotion and completion.
- Test 4 (`4/6 promotion-white-knight`): **PASS**, user confirmed b7→b8 with Q produces Wrong and rollback to b7, then b7→b8 with N produces a knight on b8 and completion.
- Test 5 (`5/6 promotion-black-rook`): **PASS**, user confirmed automatic Black-facing orientation (h1 top-left, files h→a), h2→h1 promotion with R, and black rook/completion feedback.
- Test 6 (`6/6 promotion-auto-reply`): **PASS**, user confirmed White-facing orientation restored, Next visibly disabled/inert at 6/6, h2→h3 triggers automatic a2→a1 queen promotion without a chooser, and h3→h8 completes the puzzle.
- Test 7 (Free Board/reset and first collection boundary): **PASS**, user confirmed Previous visibly disabled/inert at 1/6, FREE accepts f4→a8 without grading, Reset restores f4 while retaining FREE, and switching back to Solution restores the original position.
- Test 8 (orientation lock/flip/navigation): **PASS**, user confirmed locked manual Black-facing orientation stays fixed from 1/6 through 5/6, Flip works while locked, unlocking does not immediately reorient, and subsequent navigation restores side-to-move automatic orientation.
- Test 9 (description visibility): **PASS**, user confirmed NOTE reveals the description before solving without changing the board/attempt, can hide it, solving d7→e8 automatically reveals it, and NOTE can hide it after completion. This records the user's visual observations; the tail of the log does not contain every requested NOTE toggle.
- All nine feature checklist items passed by user confirmation. Stopped the one Task 05 app instance through Ctrl-C in its controlling SSH PTY; no further feature tests requested. Native UI recovery confirmation remains pending.


## HUMAN CHECKPOINT C final result — 2026-10-02

**PASS.** All nine device checklist groups passed by user confirmation, covering every Task 05 physical acceptance item including black-to-move orientation. Sashité pieces were legible at normal reading distance and controls comfortably tappable. No new device behavior or defect was reported during this checkpoint; existing Task 04 stock UI/refresh limitations remain documented, without a claim that this task resolves them.

After Ctrl-C, `pidof kindle-chess kindle-chess-glibc235` returned no PIDs. The device log was retrieved to ignored `probe-output/task05/checkpoint-c.log`. User confirmed normal native finger/menu interaction, pen drawing, and display appearance after returning to the stock UI. No reboot, service changes, input grabs, rotation changes, or waveform tuning were required.

Task 05 is **Implemented**. No puzzle/progress files were modified by the app, and later task files remain untouched.

Completion checks: `scripts/check.sh` passed formatting, clippy, workspace tests, asset regeneration, and Python/tooling checks; `scripts/check-kindle.sh` passed with the previously verified Zig/glibc-2.35 cross-toolchain overrides.

---

# Task 06 — HUMAN CHECKPOINT D

**Status:** Passed on 2026-10-02; Task 06 Implemented.

- Commit: `027ae82b0b7c026bde628723dcaef5461ce7da42`; CI run `37055795926` completed successfully for host checks and the ARMv7 cross-build.
- Artifact: `kindle-chess-task06-armv7`, ID `11247968147`. Downloaded ZIP SHA-256 matched GitHub's digest: `343da46a5d917c185b491d0948ec4948cd7d4f1f885fab66bd4ba559a569f770`.
- Binary: `/mnt/us/kindle-chess-task06/kindle-chess`; host/device SHA-256 matched: `8b363331208c7733d118cd375f46765953ea90b373e2048f8c0ef5d533255cc2`.
- SSH: root, empty password, port 2222 at the user-supplied LAN address. Device reconfirmed ARMv7, firmware 5.19.6, visible 1860×2480 / virtual 1872×4960 / 8 bpp. Dynamic loader resolved all binary dependencies successfully.
- `/mnt/us/kindle-chess` did not exist before deployment. Copied repository fixtures `tests/fixtures/puzzles.json` and `tests/fixtures/puzzles-endgames.json` into `/mnt/us/kindle-chess/puzzles`.
- Source baseline hashes: `puzzles.json` = `95d9f37c13f9df9ce686a75bee0288c2c54c1f84e53d292a619de4a53e95a9bf`; `puzzles-endgames.json` = `e6376ecd34dbd6f8a47072c42d191c2c99f5478b2d8c2989aed4f5efcbb28da2`.
- User confirmed a disposable blank notebook was open before launch. Verified no existing app process, launched one foreground SSH PTY instance, then confirmed exactly one PID (`30966` at initial launch).
- Launch helper: `sh /mnt/us/kindle-chess-task06/launch.sh`. It refuses launch while an existing chess instance is present, explicitly sets the documented puzzle/progress paths, creates a timestamped stderr log, and execs the app. Exit remains Ctrl-C in the controlling PTY; the helper changes no stock services, input ownership, framebuffer mode, or power policy.
- Initial log: `/mnt/us/kindle-chess-task06/checkpoint-d-20261002-215104.log`. Startup reports both valid collections, active `puzzles-endgames.json`, persistent storage enabled, pinned FBInk `92e1270`, expected geometry, and capability-selected `pt_mt` finger input.
- Initial progress file was created at `/mnt/us/kindle-chess/state/progress.json` with active `puzzles-endgames.json`, current ID `end-1`, and no solved IDs. Collection hashes still matched immediately after startup. This is startup evidence, not a completed human persistence check.
- Local artifacts and launch helper are retained in ignored `probe-output/task06/`.

## Human touch sequence and checkpoint pauses

Use finger taps and logical board coordinates. Existing stock-UI/refresh contention remains the Task 04 limitation; keep a blank notebook underneath the overlay.

1. On initial **Endgames 1/2 end-1**, solve `g6→g7`. Confirm Complete and solved marker. Tap Next to **2/2 end-2** and leave it unsolved.
2. Tap FILES and choose **Lichess sample puzzles**. Solve **1/2 lichess-001cr** with `d7→e8`, then tap Next to **2/2 lichess-000hf**. Solve `e2→e6`, observe automatic `f7→f8`, then finish `e6→f7`.
3. Switch via FILES to Endgames: it should return to **2/2 end-2**, unsolved. Tap Previous: **end-1** should have its solved marker. Leave Endgames at **1/2**.
4. Switch back to Lichess: it should return to **2/2 lichess-000hf** with its solved marker. Pause and report results. The operator stops the foreground app with Ctrl-C, checks no PID remains, checks progress and source hashes, and relaunches through the helper.
5. After relaunch, confirm active **Lichess 2/2**, solved marker visible. Switch to Endgames: confirm **1/2 end-1**, solved marker visible. Pause for the computer-copy portion.
6. The operator stops the app and copies a third valid matching collection into the puzzle directory, plus malformed `puzzles-broken.json` containing `{ broken`. Preserve both original collection files and all progress. Relaunch and confirm the new valid collection is listed in FILES.
7. Select `puzzles-broken.json`. Confirm a visible error and that the current valid collection/puzzle/board remains active after closing the picker. Pause and report.
8. The operator stops the app, removes only the deliberately created malformed test file, and relaunches. Confirm the invalid entry is gone and previous collection positions/solved markers remain.
9. Stop the app; use normal Home/back or lock/unlock to repaint if needed. Confirm native finger/pen interaction and display appearance are normal. Retrieve logs/progress, compare unchanged source hashes, and record user observations before marking Task 06 Implemented.

For manual launch from a computer, connect with `ssh -tt -p 2222 root@192.168.1.20`, submit an empty password, and run the launch helper. Do not launch a second instance. Computer-copy and malformed-file preparation are checkpoint operations, not app writes to puzzle data.

## HUMAN CHECKPOINT D observations

- Initial touch checks 1–5: **PASS** by user confirmation, covering solving in both collections, navigation, collection switching, remembered positions, and solved markers.
- Before restart, device progress independently recorded Endgames current `end-1`, solved `end-1`, and Lichess current `lichess-000hf`, solved `lichess-001cr` and `lichess-000hf`. The final active collection was **Endgames**, consistent with additional logged switches/navigation after the requested sequence; use this actual saved state for restart expectations.
- Both uploaded collection SHA-256 hashes remained unchanged. Retrieved the first-run log and pre-restart progress to ignored `probe-output/task06/`.
- Stopped the first instance with Ctrl-C in its controlling SSH PTY, verified no app PID remained, and relaunched once through the helper. Restart log: `/mnt/us/kindle-chess-task06/checkpoint-d-20261002-215527.log`. Human restoration confirmation and later computer-copy/invalid-file tests remain pending.
- Restart restoration: **PASS** by user confirmation. App reopened on Endgames `1/2 end-1` with its solved marker; switching to Lichess restored `2/2 lichess-000hf` with its solved marker.
- Stopped the second instance with Ctrl-C and verified no app PID remained. Copied `tests/fixtures/promotion-puzzles.json` from the computer as `puzzles-promotions.json`, plus deliberately malformed `puzzles-broken.json` containing `{ broken`. No original collection or progress file was replaced. Retrieved the restart log and launched one instance again; log `/mnt/us/kindle-chess-task06/checkpoint-d-20261002-215733.log`. New-file discovery, failed selection, removal, and final recovery checks remain pending.
- Computer-copy/new-file and failed-selection checks: **PASS** by user confirmation. The added Promotion test puzzles collection opened at `1/4` with the white pawn on a7. Selecting malformed `puzzles-broken.json` displayed an error and preserved the active promotion puzzle/board. Switching back to Lichess restored `2/2` and its solved marker.
- Stopped the third instance with Ctrl-C and verified no app PID remained. All three valid source files and the malformed test file retained their baseline hashes. Progress retained both collections' solved/current IDs and added the new promotion collection's current ID without solved IDs. Retrieved the third log, removed only the deliberately created malformed test file, and saved a local progress copy before final relaunch. Final restoration/removal and native recovery confirmation remain pending.
- Final relaunch/removal check: **PASS** by user confirmation. Startup restored Lichess `2/2` with its solved marker, FILES no longer listed the malformed file, and switching to Endgames restored `1/2` with its solved marker. Final launch log: `/mnt/us/kindle-chess-task06/checkpoint-d-20261002-220014.log`; exactly one app PID was verified after launch.
- Stopped the final instance with Ctrl-C and verified `pidof kindle-chess kindle-chess-glibc235` returned no PIDs. Retrieved the final log and progress to ignored `probe-output/task06/`. All three valid collection hashes still match their computer-copy baselines. Final active collection is Endgames, current `end-1`; all previously recorded solved/current IDs remain. Native UI recovery confirmation remains pending.


## HUMAN CHECKPOINT D final result — 2026-10-02

**PASS.** User confirmed all collection/persistence checks and normal native finger/menu interaction, pen drawing, and display appearance after the final app exit. The checklist verified switching collections, browsing/solving in each, durable active/current/solved restoration, computer-copy discovery of a third collection, safe failed selection of malformed data, and removal of the malformed file without losing progress.

All app instances were stopped through Ctrl-C, with no app PIDs remaining. All three valid source collection hashes remained unchanged; progress is separate at `/mnt/us/kindle-chess/state/progress.json`. The added promotion collection remains available, and only the deliberately malformed test file was removed. No stock-service, input-grab, framebuffer-mode, rotation, or power-policy changes were made. Existing Task 04 stock-UI/refresh limitations remain unresolved by this storage task.

Completion checks: `scripts/check.sh` passed formatting, clippy, workspace tests, asset regeneration, and Python/tooling checks. `scripts/check-kindle.sh` passed using the previously verified Zig/glibc-2.35 toolchain overrides. Logs and final progress are retained in ignored `probe-output/task06/`. Task 06 is **Implemented**; later task files remain untouched.


---

# Task 07 — read-only investigation and checkpoint preparation

**Status:** In progress; HUMAN CHECKPOINT E has not run.

On 2026-10-02, read-only SSH access at the previously recorded address/port
succeeded with root/empty-password authentication. The device reported kernel
4.9.77-lab126, ARMv7, and Kindle firmware 5.19.6. Inspected the actual Upstart
configuration for `x`, `lab126_gui`, `framework`, `kppmainapp`, and `pillow`:

- `x` runs lxinit and manages Xorg, awesome, and blanket; its stop path kills
  these components.
- `kppmainapp` uses respawn and KPPMainAppCrashRecovery on stop.
- `pillow` uses respawn.
- `lab126_gui` includes restart/reboot monitoring and additional recovery side
  effects; it is not an established minimal handoff boundary.

No service stop/suspension, input grab, framebuffer change, power change, or
app launch was performed during this investigation. The safe foreground
transition remains unverified. Host-tested implementation and the overlay
supervisor's exact cleanup/manual recovery procedure are documented in
[Task 07 notes](../EINK_LIFECYCLE.md).

All nine physical checkpoint items, waveform/ghosting measurements, timing
observations, and resume behavior remain **pending**, not passed. Task 07 must
not be marked Implemented from the automated changes alone.

Prepared local artifact: `probe-output/task07/kindle-chess`, SHA-256
`70db0fe48077d2db06e75a254238d14f0ef49d82e2880dbe2e129850e4e39f49`.
This is the uncommitted Task 07 working-tree build, not a deployed/tested device
binary. The overlay supervisor is beside it as `kindle_launch.sh`. Both
`scripts/check.sh` and `scripts/check-kindle.sh` passed; the cross-build used the
previously verified Zig/glibc-2.35 wrappers. Logs are retained in the same ignored
artifact directory. No physical checkpoint result is implied by these checks.


## Task 07 device staging — 2026-10-02

At the user's request to begin checkpoint testing, staged the host-tested binary
and supervisor at `/mnt/us/kindle-chess-task07/`. Device SHA-256 matched the
prepared artifact (`70db0fe48077d2db06e75a254238d14f0ef49d82e2880dbe2e129850e4e39f49`).
The device loader resolved all dependencies. Reconfirmed firmware 5.19.6,
ARMv7, visible 1860×2480 / virtual 1872×4960 / 8 bpp, no app process, and
`powerd.state=active`. No stale supervisor lock was present.

The installed `/mnt/us/koreader/koreader.sh` uses pillow disable/enable and
awesome STOP/CONT for its overlay handoff. These are investigation references,
not yet verified chess lifecycle transitions. The current awesome PID was
3509, sleeping rather than stopped. Read-only winmgr probes returned
`liglPause=0`; `eatTapMode` read failed with `lipcErrNoSuchProperty` despite
appearing in the property listing. Do not rely on that property.

No app launch or stock-service/property transition has occurred yet. The first
observed overlay test awaits the user's disposable-notebook readiness.


## HUMAN CHECKPOINT E: baseline partial updates and first exit

Initial app PID 32489, supervisor PID 32483. User confirmed Reset, g6
selection/deselection, NOTE reveal/hide, and FILES open/close all worked fine.
This passes the first visual smoke checks; it does not establish 30-interaction
ghosting acceptance, exclusive foreground ownership, or suspend/resume.

First usable frame: 996 ms from process entry; FBInk initialization: 20 ms.
Initial whole-frame submit/completion: 97/466 ms. Selection used four partial
regions: recognized-tap-to-final-submit approximately 1.17–1.62 seconds,
completion 828–1440 ms. NOTE changes used two regions: approximately 761 ms
recognized-tap-to-final-submit, completion 448 ms. FILES open/close used one
region: completion 312/599 ms. These include serialized region waits as
specified in the timing documentation; selection latency warrants later
measurement before changing batching.

Exited through Ctrl-C in the controlling SSH PTY. Supervisor forwarded TERM
to the child, logged exit 130, and removed its lock. `pidof kindle-chess`
returned no PID; framebuffer remains visible 1860×2480, virtual 1872×4960,
8 bpp. This is exit cycle 1 of the required five. Native finger/menu/pen/display
recovery awaits user confirmation. No stock-service or input-grab changes were
made in this run. Log: `/mnt/us/kindle-chess-task07/checkpoint-e-baseline.log`.


Baseline exit recovery: **PASS** by user confirmation. Native menu/finger,
pen stroke, and display worked normally after returning to the disposable
notebook. The first observed partial-refresh launch/exit cycle is complete.


SIGTERM case: launched one instance with app PID 358; first usable frame
995 ms, initial submit/completion 97/466 ms. Confirmed the child PID against
`pidof kindle-chess`, then sent TERM directly to the app process from the
second SSH shell. The supervisor logged exit 143 and released the lock.
No app PID remained. Framebuffer geometry/depth were unchanged and awesome
PID 3509 remained running (Sl). Native recovery confirmation is pending for
this specific abnormal-exit test. Log:
`/mnt/us/kindle-chess-task07/checkpoint-e-sigterm.log`.


### SIGTERM recovery finding: stale framebuffer contents

User confirmed native interaction works, but chess pixels remained behind the
native menu and partly visible after navigating Home. Rechecked: no app PID,
no supervisor lock; awesome remained running. Thus process/lock cleanup passed,
but **visual restoration failed**. Earlier native recovery confirmations must
not be interpreted as evidence that the entire chess frame was erased.

Verified X display from awesome's environment: `DISPLAY=:0.0`, socket X0.
Executed `/usr/bin/xrefresh -display :0` on the device; exit status 0. The user's
visual confirmation of its effectiveness is pending. BusyBox timeout usage
was probed: `timeout [-s SIG] [-k KILL_SECS] SECS PROG ARGS` is supported.

A host-tested, opt-in native repaint cleanup hook is being prepared. It runs
only if this supervisor started its child, runs before releasing the lock,
preserves the app's exit status, and bounds repaint to TERM after five seconds
and KILL one second later. No service stop, framebuffer mode change, or input
grab is involved. Do not enable the hook on-device before visual verification.


Native repaint probe: **PASS** by user confirmation. After the explicit
xrefresh request the screen was fully native, with no remaining chess pixels.
The supervisor's repaint hook is now enabled by default (`xrefresh -display
:0.0`, bounded by `/usr/bin/timeout -k 1 5`). Host tests first failed for the
missing default repaint, then passed after enabling it. An explicitly empty
`KINDLE_CHESS_XREFRESH` disables the hook for host tests. No stock services or
framebuffer modes are changed. Retest automatic restoration on exit, SIGTERM,
and app SIGKILL before treating cleanup as fully verified.


Updated supervisor staged with matching host/device SHA-256:
`87352b4bcb57a2de4d204a383224e3b57dab40e7531125a0a7c6fb9c17eec6b2`.
The binary is unchanged. `scripts/check.sh` passed after the default-repaint
change. Launched app PID 1069; first usable frame 993 ms, initial
submit/completion 97/465 ms. Verified its PID against the supervisor's child
record, then sent SIGKILL only to the app from the recovery SSH shell.
Automatic native display restoration is now under observation. Log:
`/mnt/us/kindle-chess-task07/checkpoint-e-sigkill-repaint.log`.


SIGKILL with automatic repaint: **PASS** by user confirmation. Screen returned
fully to native UI with no chess remnants and no Home/back action needed;
native touch worked. Independently verified no app PID or lock remained,
exit 137, native repaint requested without a logged error, and unchanged
framebuffer geometry/depth. This completes the intentional app-kill checkpoint
for the updated supervisor. Remaining exit types and other checklist items
still need their own verification.


Updated-supervisor SIGTERM retest: process cleanup and automatic repaint
request completed with exit 143, no remaining app or lock. First usable frame
996 ms, initial submit/completion 97/466 ms. User visual recovery confirmation
for this retest is pending. Log: `checkpoint-e-sigterm-repaint.log` in the
Task 07 device staging directory.

Five consecutive normal launcher exit cycles were run with the updated
supervisor. Each launched one app, displayed its initial frame, exited through
Ctrl-C in the controlling SSH PTY, requested native repaint, and logged exit
130. The operator checked no app PID or lock remained after every cycle.
App PIDs: 1119, 1137, 1156, 1172, 1190. Initial submit times 96–98 ms,
completion 465–466 ms. The device was left on the native screen for about
12 seconds after each exit for user menu/display observations. Visual/native
recovery confirmation for all five remains pending. Logs are
`checkpoint-e-normal-1.log` through `checkpoint-e-normal-5.log` in
`/mnt/us/kindle-chess-task07/`.


Updated-supervisor SIGTERM retest and five repeated normal exits: **PASS** by
user confirmation. All six exits returned to a clean native screen with
working native touch. A clean disposable notebook is open for the extended
interaction run. Existing host/process checks and unchanged framebuffer
geometry/depth apply; no service or input-grab transitions have been introduced.

The next app run logs to `checkpoint-e-interactions.log`. Power probes returned
`screenSaverTimeout=600`, `preventScreenSaver=0`, and `state=active`. A passive
`lipc-wait-event -m -t -s 1800 com.lab126.powerd '*'` listener records actual
power events to `checkpoint-e-power-events.log`; it is finite (30 minutes),
does not alter power policy, and its PID is recorded separately as
`power-listener.pid` for cleanup. Extended ghosting/modal and natural
idle/suspend/resume observations remain pending.


Natural 10-minute idle/suspend/resume check: **DEFERRED at the user's request**.
The user asked to perform it later. Stop the passive power-event listener;
do not alter power settings or force suspend as a substitute for this check.
The interaction/ghosting/modal run remains active and its results are still
pending. Task 07 remains in progress; no resume behavior has been established.


### Extended interaction run: responsiveness regression

User reported very slow piece rendering and an unresponsive UI. Retrieved
`checkpoint-e-interactions.log`: alternating Next/Previous submitted **38
partial regions per action**, waiting after every submission. Submission to
last-update completion took 10.36–10.55 seconds; recognized-tap-to-final-submit
was approximately 10.53–10.71 seconds. Selection used 4 regions (~1.16 seconds
to final submit), and a subsequent attempt used 6 (~2.14 seconds). This run
**FAILS responsiveness acceptance** and does not pass the extended visual test.

Stopped app PID 1300 through the supervisor; no app PID or lock remained, and
automatic native repaint was requested. Natural idle/suspend remains deferred.

Host regression tests now require one selection region, at most six navigation
regions at measured Scribe geometry, and one completion wait after all batch
submissions. Renderer compaction stays within board/header/toolbar/navigation/
status groups; it does not replace regional updates with a whole-screen update.
Pixel replay still reconstructs every visible transition. Failed/invalid batches
are covered by mocked platform tests. The upcoming retest will also use an
optimized Rust release build; the original artifact was a development build.
Neither batching reliability nor improved physical latency is claimed verified
until the new device run is observed.


Batching fix prepared and staged: host/device SHA-256
`39c7312e0bcc37c5b1bd940c0a607e2c55a4684d01bff9193a59afd5c0bf177e`.
Local artifact `probe-output/task07/kindle-chess-batched-release`, optimized
ARMv7/glibc-2.35 build. Dynamic loader resolved all dependencies before launch.
`scripts/check.sh`, the standard cross-build, and the additional release
cross-build passed. Supervisor is unchanged from the verified repaint version.
The new run logs to `checkpoint-e-batched-release.log`; physical responsiveness
and visual reliability are pending retest. All natural-idle work stays deferred.


Batching/release responsiveness retest: **PASS** by user confirmation. App
PID 1852; first usable frame 437 ms (FBInk initialization 10 ms). The log
records seven navigation taps and four NOTE toggles so far (11 total).
Navigation now submits **3 regions**, with submit 10–12 ms, completion 390 ms,
and recognized-tap-to-final-submit 48–51 ms (approximately 0.43 seconds to final
completion, versus roughly 10.7 seconds before). NOTE uses two regions,
completion 214–226 ms, and recognized-tap-to-final-submit 33–35 ms. User reports
responsive UI and clean piece redraws. The longer promotion/modal/ghosting
sequence remains pending; the failed old-artifact run is not counted as passing
this checkpoint. No stock-service, input-grab, power-policy, rotation, or
bit-depth changes were introduced.


### Verified input contention and revised Task 07 plan

During the optimized promotion test, user reported tapping a8 also opened a
native Kindle menu. The menu later disappeared, leaving a white area where
the board was not repainted. The app log confirms the a8 action reached
`Square(0)` and promotion/cancel updates were submitted; pixel damage history
cannot account for an external writer. Thus responsiveness passed, but
foreground touch/display isolation **FAILED**.

Stopped app PID 1852; supervisor requested native repaint, no app/lock remains.
Before further promotion testing, the plan is to test exclusive evdev ownership
of the capability-selected finger device only, using EVIOCGRAB and a scoped
owned descriptor. Acquisition errors must fail before presenting the first
frame. Release is explicit on guard drop; process exit/kill closes its owned
file descriptors. Power buttons and pen devices must not be grabbed. Keep
recovery SSH available and use a finite first probe; verify native input after
normal and killed exits before enabling this behavior by default.

Kernel reference for the proposed grab semantics:
https://docs.kernel.org/driver-api/input.html#c.input_grab_device . This is a
reference, not evidence of successful behavior on the Scribe. No stock-service
suspension or shutdown is proposed for this first isolation experiment.


Exclusive-input probe staged (opt-in, not yet accepted): release SHA-256
`3655eccb9a401d50ee804570bb57b7d3bbf8fe9bfd1887832ef1461d224b0f7c`.
Host C ioctl contract covers acquisition/release and EBUSY propagation;
platform error-context test, full host checks and Kindle debug/release builds
pass. Command: supervisor launches `/usr/bin/timeout -k 1 120
/mnt/us/kindle-chess-task07/kindle-chess --exclusive-input`, log
`checkpoint-f-exclusive.log`, with second recovery SSH shell retained.
Physical isolation and native recovery results remain pending user observation.


2026-10-03: User was away for the first exclusive-input probe; its physical
results are unobserved and are not counted as a pass. On reconnect, no app or
lock remained, powerd reported active, and the binary hash matched the staged
exclusive-input release. Restarted the same 120-second probe with recovery SSH
retained; log `checkpoint-f-exclusive-repeat.log`. User observations pending.

2026-10-03 exclusive-input repeat: **PASS** by user confirmation: repeated
promotion/cancel works, no native menu or white patch, native touch/display
works after automatic timeout exit. Log confirms a8 and CancelPromotion taps,
followed by native repaint and supervisor exit 143. Reconnect confirms no app
PID or lock remains. Hard-kill input-release verification is next.

Exclusive-input hard-kill probe: verified app PID 7886 matched the owned
child PID before SIGKILL. Log `checkpoint-f-exclusive-kill.log` records native
repaint and exit 137; no app PID or lock remains. Native finger/pen and display
recovery await user confirmation before making exclusive input the default.

2026-10-03 exclusive-input hard-kill recovery: **PASS** by user confirmation.
Native screen, finger touch and pen work after SIGKILL and automatic repaint.
Exclusive finger ownership is now required on ordinary launches, before the
first frame; acquisition failure aborts startup. The natural suspend checkpoint
remains deferred and is not inferred from these exit tests.

Default-exclusive release built and deployed: SHA-256
`e2a49bbc7169b4ee8b3d23069d5074b7b63bab3ae983189dd54391c2934a2e41`.
Full host checks (fmt, clippy, workspace tests, snapshots, Python contracts),
Kindle debug cross-build and optimized release build pass.

Default-launch smoke: no exclusive-input flag supplied; log
`checkpoint-f-default-smoke.log` confirms the grab before first frame, followed
by timeout, native repaint and exit 143. No app PID or lock remains. The device
is left in its native UI.


## Task 07 completion with deferred sleep checkpoint — 2026-10-03

The user confirmed that **only the sleeping test remains** and instructed
committing/pushing Task 07 now, with that test left for a later session. This
accepts the remaining non-sleep checkpoint E observations, including extended
interaction/ghosting and repeated modal checks. Existing records above retain
the measured exit/recovery, exclusive-input and responsiveness evidence.

Task 07 is marked Implemented with this explicit user-authorized deferral.
Natural suspend/resume, screensaver repaint invalidation and evdev descriptor
survival remain unverified. No new power transition is performed or implied by
this completion record. The release binary remains
`e2a49bbc7169b4ee8b3d23069d5074b7b63bab3ae983189dd54391c2934a2e41`.

## Task 08 automation preparation — 2026-10-03

Task 08 host contracts and a pinned Zig 0.13.0/ARMv7/glibc-2.35 release build
passed in the existing working tree. This is not a clean-checkout physical
checkpoint. The executable SHA-256 is
`e2a49bbc7169b4ee8b3d23069d5074b7b63bab3ae983189dd54391c2934a2e41`, matching
the previously tested default-exclusive Task 07 binary; Task 08 changes its
build/install/launch integration rather than application behavior.

Prepared commands, Scriptlet/KPM payloads, installation/data preservation
policy, and licensing status are documented in [BUILD_DEPLOY.md](../BUILD_DEPLOY.md).
A read-only, five-second BatchMode SSH connection to the recorded
`192.168.1.20:2222` timed out. No new files were transferred, no runtime/package
was installed on the Scribe, and no launch or lifecycle transition occurred.

All HUMAN CHECKPOINT F items remain **pending**, including clean-checkout
execution, deployment, library launch/exit, a second-build update, reboot launch,
and uninstall/reinstall. Scriptlets/KPM availability remains unestablished on
the target Véra setup. The remaining Task 07 lifecycle checkpoint is also
pending. Host package/source/ELF checks do not establish these physical facts.


## Task 08 clean checkout and device installation — 2026-10-03

Reconnected to the awake Scribe at `192.168.1.20:2222`. Reconfirmed ARMv7,
firmware 5.19.6, glibc 2.35, baseline framebuffer geometry/depth, no active app
or supervisor lock. Boot ID before reboot testing:
`7e7361dd-10db-4020-a4ff-d0bacca87d6d`.

KPM is installed at `/var/local/kmc/bin/kpm`: CLI v1.0.0, libkpm v0.2.2,
`kindlehf`. It was absent from the earlier SSH PATH, not absent from the device.
SH_Integration's registered launcher and the existing KOReader library scriptlet
are installed. KOReader's scriptlet uses `# DontUseFBInk`; chess now includes
the same directive to prevent the integration layer from drawing over the app.

Task 07 was independently checked/cross-built from its staged source tree,
committed as `522cf69`, and pushed to origin/main. Natural suspend/resume is
explicitly deferred by the user; packaging checks do not establish that behavior.

For checkpoint F, a separate clean local checkout at candidate commit
`ef60a14c5a502e7d833a92442288921cb054b543` ran exactly:

```sh
git submodule update --init --recursive
scripts/check.sh
scripts/build-kindle.sh
scripts/stage-kindle.sh
```

All passed, including 13 packaging/build contracts. `git status --short` was
empty after generation/build/staging. The candidate contains the Task 08 code
pending the final checkpoint documentation/commit. Its tested source fingerprint
is `42839c315598c42c80c84d65fd4809490486e3d6ba04e94048a7367de27bea6f`.
The clean-checkout release matches the independent working-tree release exactly:
`e2a49bbc7169b4ee8b3d23069d5074b7b63bab3ae983189dd54391c2934a2e41`.

First installation used the documented command:

```sh
scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222
```

It installed runtime at `/mnt/us/kindle-chess/runtime` and the library scriptlet
at `/mnt/us/documents/kindle-chess.sh`. Device loader tracing resolved all
libraries. The second independently built artifact was copied from the clean
checkout and installed through the actual KPM stack:

```sh
scp -O -P 2222 target/kindle/kindle_chess_0.1.0_kindlehf.kpkg root@192.168.1.20:/mnt/us/
/var/local/kmc/bin/kpm -y install file:///mnt/us/kindle_chess_0.1.0_kindlehf.kpkg
```

KPM accepted the manifest-v2 package and installed it under
`/mnt/us/kmc/kpm/packages/kindle_chess`. Both builds are byte-identical;
installation exercised runtime replacement without source edits. No puzzle or
progress bytes changed across either installation.

Tested KPM uninstall/reinstall with the same local package. Uninstall removed
runtime, its package registration/files and the exact matching library scriptlet;
puzzles/progress remained. Reinstall restored runtime/package/scriptlet.
Baseline hashes remained identical at every measured stage:

- `puzzles-endgames.json`: `e6376ecd34dbd6f8a47072c42d191c2c99f5478b2d8c2989aed4f5efcbb28da2`
- `puzzles-promotions.json`: `1df84295fe7b8e657bfd9a4eb05565ded5451c36d1e5970290b828ca84fe62b4`
- `puzzles.json`: `95d9f37c13f9df9ce686a75bee0288c2c54c1f84e53d292a619de4a53e95a9bf`
- `state/progress.json`: `1492a13a56d822be90388ec3601ff36186af143bad306b1c86cf93e89de30baf`

Progress still refers to the promotion collection/current
`promotion-white-queen`, with the prior solved IDs in Endgames and Lichess.
Logs and the pre-install progress copy are retained in ignored
`probe-output/task08/`; clean-checkout check/build/stage logs are in `/tmp`.

Checkpoint F's automated/build/install/data-preservation items pass.
Library launch/exit and post-reboot library launch are pending the user's
physical observations. No reboot or new app launch is claimed by this record.


Task 08 packaged launch smoke: ran the actual installed KPM launch entrypoint
under a 15-second timeout. The app acquired the discovered finger device,
rendered its first usable frame in 440 ms, and completed its first full update
(submit 35 ms, completion 397 ms). Timeout termination propagated through the
supervisor: TERM forwarding, native `xrefresh`, exit 143, and lock removal were
logged. No app PID or lock remained; framebuffer geometry/depth and all
puzzle/progress hashes were unchanged afterward. Logs are in
`probe-output/task08/`. This passes the on-device CLI entrypoint/cleanup smoke;
it does not substitute for the pending library UI and post-reboot observations.


2026-10-03 library regression: user observed that top swipe could not open the
native menu, and a white stock overlay covered part of the app while its controls
still worked. The running app held exclusive finger input, as designed, but had
no in-app exit. Verified supervisor/child PIDs before TERM; supervisor forwarded
TERM, requested xrefresh, and removed its lock. The user confirmed normal stock
UI/touch after stopping the app. No service, rotation or power changes were made.
Library launch/exit checkpoint is therefore failing, not accepted. Planned fix:
explicit Exit accessible through modals, followed by investigation of native
launcher/display ownership and a bounded on-device regression check.


Exit fix build/deploy: host fmt/clippy/workspace tests, rendering snapshots,
Python/C/platform/package contracts and pinned Kindle release cross-build all
passed. Reviewed all 16 before/after visual states: changed pixels are confined
to header rows 38–149; the new X is touch-sized at the header right edge and
FILES/solved badge reserve its area. Exit routing is available through modals;
a binary host test verifies dirty progress is persisted before normal return.
The documented stage/deploy installed release SHA256
`cc6b60a4dbd71c1fc3417959410775f3e7e087421b1e90e9693a91205cd15e52`.
All four prior puzzle/progress hashes remained unchanged afterward. Device
library X exit and white-overpaint regression checks are awaiting observation.
Read-only stock logs show SH Integration 4.1.0 waits for its child before stop,
unload and xrefresh; no verified evidence yet attributes the white overlay to
an early launcher exit. No stock service suspension or power change was added.


## Top-right exit fix deployed — 2026-10-03

At the user's request, stopped the stuck app first: verified child PID 11729
against `/tmp/kindle-chess.lock/child.pid` and `pidof kindle-chess`, then sent
SIGTERM to that child. Supervisor logged native repaint and exit 143; no app
or supervisor lock remained.

Completed the partially prepared Exit action/effect and runtime handling.
The header now contains a touch-sized X at the top right, alongside FILES.
Exit hit testing precedes promotion/collection modal handling. The runtime
retries dirty progress, returns normally, and releases owned display/input
resources; the existing supervisor performs the native repaint.

The top-right position regression test failed before the layout correction.
Core exit/modal tests, runtime progress-flush test, all layout tests, reviewed
render/parity snapshots, fmt, clippy, workspace tests and Python/C/package
contracts passed through `scripts/build-kindle.sh`. Reviewed normal, solved,
promotion and collection-picker frames with the X visible.

Built, staged and deployed using the documented commands:

```sh
scripts/build-kindle.sh
scripts/stage-kindle.sh
scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222
```

Installed runtime SHA-256 matches the local release:
`cc6b60a4dbd71c1fc3417959410775f3e7e087421b1e90e9693a91205cd15e52`.
The device loader resolved all libraries. All three puzzle files and progress
retain the baseline hashes above. No app or lock remains after deployment;
the updated library scriptlet is ready for the user to launch.

Physical X exit: **PASS**, confirmed by the user on 2026-10-03 after
launching the deployed update. This change does not establish a fix for stock
white overpainting, reboot launch, or the deferred suspend/resume checkpoint.


## Task 08 white overlay investigation — 2026-10-03

Read-only SSH inspection found app PID 13202 still responsive. Library launch
at 07:07:40 UTC logged winmgr T0 start; at 07:07:45.587 T0 expired,
`flashTimeoutExpired: window=Active App` named the SH_Integration launcher,
and winmgr submitted a native framebuffer update. The launcher explicitly
logged `Child spawned, waiting to quit`; it had not exited early.
The installed KOReader launcher uses awesome SIGSTOP/SIGCONT.

Bounded probe planned: terminate only the verified chess supervisor, capture
awesome's PID/start time and running status, STOP that PID, launch the existing
chess supervisor under a 20-second timeout, compare visible framebuffer bytes
after startup and beyond the native five-second timeout, then CONT the same
verified PID and xrefresh. EXIT/INT/TERM/HUP traps restore it; second SSH shell
manual recovery is `kill -CONT <verified awesome PID>; xrefresh -display :0.0`.
No service stop, pillow change, rotation/depth or power transition is involved.

Bounded physical probe result: **PASS for framebuffer stability and process
restoration**. Existing supervisor 13187 was terminated normally, awesome PID
3509 was paused, and the installed app ran under a 20-second timeout. Visible
framebuffer captures at 2 and 10 seconds had identical SHA-256
`33e6099c11d70b5d4b80f48de28d2acc3d9b4297db7f646044ea5ed4b63147e6`.
The captured image was inspected and shows the complete promotion puzzle UI.
After timeout, awesome returned to state S, no chess process remained, and
logs recorded input cleanup/native repaint. `/bin/kill` exists on this target.
Evidence is retained under ignored `probe-output/task08-overlay/`.

Regression tests first failed for missing STOP/CONT and already-stopped/PID-reuse
protection, then passed with the scoped handoff implementation. Installed
supervisor validation and user library observations follow below; reboot and
natural suspend/resume are not inferred from this probe.

Installed-supervisor regression uncovered a second writer: the first cycle
retained the chess frame, but a rapid second launch captured the native library
covering the board before the 2-second sample, despite awesome remaining T.
The two second-cycle captures differed only in the top clock rows, but both
were already overwritten. STOP awesome alone is therefore insufficient.
Plan revised before further coding: probe pausing the verified Xorg PID as
well, with EXIT/INT/TERM/HUP cleanup resuming Xorg first, then awesome and
xrefresh, a bounded app lifetime and independent SSH recovery. No service,
rotation, depth or power changes.

Dual-process bounded probe: **PASS**. Awesome 3509 and Xorg 3233 were paused
with cleanup resuming Xorg first. Two consecutive 12-second launches, separated
by native xrefresh and two seconds, each retained the complete chess frame:
all four 2-second/8-second captures matched SHA-256
`33e6099c11d70b5d4b80f48de28d2acc3d9b4297db7f646044ea5ed4b63147e6`.
Both processes resumed afterward. No puzzle/progress data was modified.
New Xorg/order/pre-stopped tests failed against the awesome-only implementation
before adding the second verified process to supervisor ownership.

Final paired-process supervisor deployed with the documented build/stage/deploy
commands. All host checks, 10 lifecycle tests, packaging/FFI/input contracts,
render snapshots and pinned ARMv7/glibc-2.35 release build passed.
Installed-supervisor regression: **PASS** across two consecutive launches.
All four framebuffer samples at 2 and 10 seconds matched the complete chess
frame hash above. SIGTERM to the first supervisor and SIGKILL to the second
app child both resumed Xorg then awesome, released the chess lock/input and
requested native repaint. Both native processes returned to R/S; no app PID or
lock remained. Puzzle and progress hashes were identical to the baseline.
Temporary remote probe scripts/dumps were removed after saving local evidence.
Library visual/tap/X-exit confirmation is requested; post-reboot and deferred
natural suspend/resume checks remain pending.

Library regression checkpoint: **PASS by user confirmation** on 2026-10-03
(“all worked fine”). Library launch at 07:26:38 UTC used the final installed
supervisor SHA-256
`e8139610349633ee770c7fc7ce067c1b6e3d042445316c9f4611cee9567424dd`
and release binary SHA-256
`cc6b60a4dbd71c1fc3417959410775f3e7e087421b1e90e9693a91205cd15e52`.
A framebuffer capture beyond the native overwrite window matched the complete
chess frame hash. Logs recorded board taps, promotion choice and Exit, followed
by Xorg CONT, awesome CONT, native repaint and supervisor exit 0. Both native
processes returned to S; no chess app or SH_Integration launcher remained.
User confirmed the board stayed visible, interaction/exit and native UI worked.
This resolves the reported white-overlay regression and library launch/exit
portion of checkpoint F. Post-reboot library launch and the separately deferred
natural suspend/resume test remain unverified.


---

# Task 09 — phase-one parity/reliability closure — 2026-10-03

Task 09 adds tests and documentation only; it does not change the runtime puzzle,
rendering, input, storage, display, or lifecycle behavior that was physically exercised
in Tasks 04-08. The final installed runtime baseline therefore remains the Task 08
release described above while Task 09 is validated by the host suite and pinned
ARMv7/glibc-2.35 cross-build.

The reliability threshold is supported by cumulative real-device use rather than a new
synthetic session. Task 04 logged 53 accepted finger taps and a repeated-move sequence
without coordinate drift or misses. The nine scripted Task 05 parity groups require more
than 50 additional physical taps before counting their navigation between cases. Tasks
06-08 then add repeated collection browsing, promotion/cancel, navigation, normal and
abnormal exit, package launch, and library launch/exit. The cumulative phase-one record
therefore exceeds 100 physical touch interactions. Later checks also verified exclusive
finger ownership after an observed stock-UI contention bug, and the user accepted the
batched release's responsiveness/ghosting behavior.

Checkpoint G ordinary-use criteria are recorded as **PASS**: reference parity rows are
covered, normal restart preserves progress, no frequent coordinate/input failure remains
in the verified final path, non-sleep ghosting is acceptable, launch/exit/recovery is
repeatable, and the library Scriptlet makes normal use independent of an SSH terminal.
The app is also being used on real puzzles; product improvements from that feedback are
reserved for the next phase rather than folded into parity closure.

Two lifecycle observations remain **UNVERIFIED** and are not converted into passes by
this closure:

- natural idle suspend/resume, including framebuffer invalidation and evdev descriptor
  survival;
- library launch after a full device reboot.

No device power transition or reboot was performed for Task 09. If either behavior
becomes relevant in later real-user testing, repeat the dedicated lifecycle observation
and append the result here.


## Dark-square ghosting follow-up — 2026-10-03

The user tested regional-clean branch `fix/piece-ghosting-regional-clean` at
`00f31cb6714aef03bae67ef51cb3fe9a043c8ba2`. Piece traces were almost resolved,
but faint residue remained on dark squares. Native UI residue was also visible
on dark squares immediately after startup. These observations do not constitute
a complete 30-move acceptance run.

A follow-up build on that branch (local changes to display submission, tests and
refresh documentation) clears clean grayscale regions to white and waits for
each erase before drawing the final batch. Startup/recovery clear the viewport;
ordinary moves retain regional coverage and partial selection remains unchanged.
The policy is described in `docs/PIECE_GHOSTING_FIX.md`.

Commands completed:

```sh
scripts/build-kindle.sh
scripts/stage-kindle.sh
scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222
```

Formatting, clippy, all workspace tests/snapshots, asset verification and all
Python/C/lifecycle/package contracts passed. Nine host presentation tests cover
submission ordering, startup, regional grayscale clearing and failure handling.
The validated release targets ARMv7 hard-float and glibc 2.35 with pinned FBInk
`92e127008145b2a22fba7c59815d810d716310dd`.

Device read-only inspection reconfirmed ARMv7 and firmware 5.19.6. No app or
supervisor lock was present before updating. The installed binary SHA-256 matches
the build: `59ebe68f0696de84fe1f21d27e38fb559a3faa72f77750e4e029200ca5837aac`.
Source receipt SHA-256: `29c8c3e0a842188d013a764847493cac3dbc2d8b33c7bf8016841dc6fce00d5e`.
The device's `/lib/ld-linux-armhf.so.3 --list` resolved all dependencies.

All four uploaded puzzle files and progress retained their before-deployment
SHA-256 values. Progress remained
`64baa78f53812f2a8b222bbe7faa700e35d5617888070fa7848ff9fcaf908014`.
No lifecycle/service/rotation/power changes or reboot were performed.

**Pending:** user visual checks of native residue at startup, light/dark origin
squares after moves, and acceptance of the additional local flashes/latency.
The app was left stopped for normal library launch. Host tests, transfer and
loader checks do not establish elimination of the physical ghosting.


## White-pass rejection and REAGLD experiment — 2026-10-03

The user rejected the preceding white-pass build: native UI ghosts were worse,
including on light squares, and piece ghosts persisted. White appeared briefly,
then the ghost returned with the final redraw. The pending visual acceptance
in the previous record is therefore **FAIL**. Production white clearing and
its tests were removed; a new regression confirms a single final-pixel payload
for regional and startup grayscale updates.

Before stopping that build, captured `/dev/fb0` (1872-byte rows, 2480 rows) while
the user had the app open. The inspected image shows the correct chess frame.
80x80 interiors at (760,500), (770,260) and (100,2350) contain only 238, 184 and
255 respectively. No native UI shapes are stored in those samples. Capture:
ignored `probe-output/ghosting-follow-up/before.raw` and `before.png`. This
supports a physical/controller-history issue, not saved native UI shapes in
the app frame; it does not identify the exact driver cause.

Supervisor PID 29628 was verified by its lock and `/proc` command line before
TERM. It released child/input and resumed Xorg 3233 then awesome 3509 with the
documented native repaint. No new lifecycle commands or state changes were
introduced.

Replacement: flashing REAGLD for Full/Clean requests, mapped by pinned FBInk to
MTK GLD16/FULL; unsupported errors fall back to flashing AUTO. Partial updates
stay AUTO. No white intermediate frame or new screen-wide move refresh exists.
The C mapping/fallback test failed before implementation, then passed; the
no-white-pass regression failed against the rejected build, then passed after
reversion. Full formatting/clippy/workspace/snapshot/asset/C/lifecycle/package
checks and the pinned ARMv7/glibc 2.35 release build passed.

Build/stage/deploy used the same documented scripts and `root@192.168.1.20:2222`.
Installed binary SHA-256:
`69aa4b0aafa65dc2571ff4f67164d9b99ff20bcf5af73d04be533b06e8ca6065`.
Source receipt SHA-256:
`b7b429ea2926055e4cbbbbcbdc599558a2e7a5e72652f459eb022d5f904a3a17`.
Base remains `00f31cb`, with local follow-up changes. All four puzzle hashes and
progress hash from the previous record remained unchanged after deployment
and the startup probe.

A 12-second supervised startup via `/usr/bin/timeout -s TERM -k 3 12 sh
/mnt/us/kindle-chess/runtime/launch.sh` rendered the correct frame and completed
the initial REAGLD submission without unsupported-mode fallback or errors.
Log: first usable frame 1044ms, one region, submit 23ms, completion 934ms.
Capture: ignored `probe-output/ghosting-follow-up/reagld.raw` and `reagld.png`.
On timeout the supervisor forwarded TERM, resumed Xorg then awesome, requested
native repaint, and removed app/lock ownership (exit 143). Native process status
inspection confirmed both were sleeping/running rather than stopped.

**Pending:** visual startup/piece-ghosting acceptance and localized flash/latency
checks. The startup probe verifies submission and recovery, not physical panel
cleanliness. App left stopped for normal library launch; no reboot/suspend test.


## White-screen root cause and baseline restoration — 2026-10-03

The user reported a completely white physical screen on two further launches
of the REAGLD build. While the second reported-white instance was active,
captured `/dev/fb0` again: `probe-output/ghosting-follow-up/white-screen.raw`
(ignored). It matches the earlier complete chess startup capture byte-for-byte,
SHA-256 `fc119182f301fa8377456b43c3add2884478886878964bc296a16ffed85c3fee`.
There are 2,872,651 non-white visible bytes. Logs show successful submission/
completion, so the failure is neither an empty renderer output nor an app crash.

Read-only kernel diagnostics established the specific cause:

```text
[HWTCON ERR]waveform mode[5] not loaded night_mode[0] @wf_lut_get_waveform_mode_slot,608
```

Three errors correspond to the REAGLD launches (kernel timestamps 148437.684387,
148624.639298 and 148727.149787). Pinned FBInk maps REAGLD to MTK GLD16/mode 5.
That day-mode waveform is not loaded on the target firmware. The asynchronous
kernel error is not returned as an ioctl failure, so the errno-based fallback
never runs. Successful submission and framebuffer screenshots were insufficient
physical validation of this experimental mode. Kernel evidence is saved under
ignored `probe-output/ghosting-follow-up/missing-waveform.log`.

Verified supervisor 30876 by its command line and terminated it normally before
updating. Removed REAGLD and reinstated the branch's original C waveform mapping;
white clearing remains removed. The original C mapping contract failed against
REAGLD before reversion and passed after it. The final-pixel/no-white-pass
regression and all build checks passed.

Rebuilt, staged and deployed with the documented scripts. Installed binary
SHA-256 `45994cabb7407d867f3b330e2dce7f06591c09f11d1ba7505c08221bff67ab63`
is identical to the first `00f31cb` regional-clean branch deployment.
The test/documentation-inclusive source receipt is
`4805099deb310ac40723fcf057baf9c67af1d03c35c350458d478934cdaa7579`.

A 12-second supervised baseline startup completed its single full request
(submit 25ms, completion 391ms). Kernel diagnostics retained exactly the three
old missing-mode errors, with no new one. Captured the complete startup frame
under ignored `probe-output/ghosting-follow-up/restored.raw`. Timeout cleanup
forwarded TERM, restored Xorg/awesome to sleeping rather than stopped states,
requested native repaint and removed app ownership. The app is left stopped.
All four uploaded puzzle hashes and progress hash remained unchanged.

The white-screen regression's cause is established and the prior exact binary
is restored. User visual confirmation of the restored board is pending;
residual dark-square ghosting remains unresolved. No reboot, suspend, new power
policy or framebuffer mode change was performed.


### User acceptance of restored regional cleaning — 2026-10-03

After restoring the original regional-clean binary, the user confirmed it is
still better than the pre-fix behavior and requested merging/pushing the fix to
main. This accepts the restored board behavior with the remaining dark-square
ghosting; it does not claim complete ghost elimination or a new 30-move,
reboot, or suspend checkpoint. The rejected white-pass and REAGLD experiments
are absent from the final production change.

### Atkinson Hyperlegible font acceptance — 2026-10-04

Built and deployed the `ui/atkinson-hyperlegible` branch at `3993554` with
uncommitted newline-glyph fixes and their reviewed snapshot updates. Installed
binary SHA-256 `34b55ecdb194b51e6d3aa88c40cb33c715f90fce049e29c793978379cecc6748`;
source receipt `b3f191b760f611e99b8732b0887f922e0aad147f517aa827d76d2e6ac9e0d6b9`.
Formatting, strict clippy, workspace/snapshot tests, Python/C/lifecycle/package
contracts and the ARMv7/glibc-2.35 release build passed. Compared all snapshot
states against the prior bitmap font before updating expected hashes. A failing
regression test demonstrated newline missing-glyph boxes; control glyphs are now
skipped during rasterization while preserving layout line breaks.

The local deployment script pulled the branch upstream, built, staged and
installed through the existing safe installer with empty SSH passwords. All
four uploaded puzzle collections and progress verified unchanged after install.
The installed hash matched; supervised startup completed at 509ms (23ms
submission, 387ms completion). The user reported “fonts look much better” and
requested a squash merge to main. This records visual font acceptance, without
claiming any improvement in physical ghosting or additional lifecycle checks.

## Task 26 — stylus tap preparation (2026-10-04)

At preparation, checkpoint 26A was pending human testing. It subsequently
**passed** by user confirmation below; Task 26 is Implemented.
Read-only SSH reconnected with the recorded `root@192.168.1.20:2222`
transport and an empty password. Firmware remains 5.19.6, architecture armv7l.
Input names/capabilities still expose `WacomDigitizer`, `pt_mt`, and
`stylus-custom`; no chess process was running before deployment preparation.

The prepared adapter selects only `stylus-custom` using its name and ABS_X/Y
plus key-event capabilities, regardless of event-node number. It reuses the
Task-00 inclusive virtual pen ranges and direct portrait axes. The physical
Wacom stream is never opened. Pen input is read-only with no grab; the existing
owned finger grab and supervisor handoff/cleanup are unchanged. Synthetic tests
and the reviewed Task-00 virtual trace cover tap aggregation, hover, movement,
invalid coordinates, repeated contacts, eraser filtering, and contact duration.
The poll bridge is tested with mock readiness, interruption, and error results.

A prerequisite gap was found: no rendered control opened the analysis browser.
The minimal repair adds an ANALYSIS button in a rich puzzle's closed status
area. Existing NOTE/toolbar actions remain intact. Its geometry/action and a
visually reviewed Gray8 snapshot are covered on the host.

Deployment/build identifiers and the human observations are recorded below.
Host checks alone do not establish real pen targeting or native pen recovery.

### Installed checkpoint build

- Implementation commit: `7c6e1e6` on `phase-2` (source unchanged since the build).
- Binary SHA-256: `020fe41c35d2c195579df7b568d00db7d71893b507cf5ffeee0ec8be9b82c17b`.
- Build source SHA-256: `feb2e137c4efd3b9cc02e876900c4909c861cd672d3362e80474886aec124b12`.
- Full `scripts/check.sh` gate passed through `scripts/build-kindle.sh`, including
  fmt, clippy, workspace tests, snapshots, Python/C/lifecycle/package contracts,
  generated assets and ARMv7/glibc-2.35 ELF validation. The declared `chess==1.11.2`
  host-only dependency was installed in `/tmp/kindle-chess-task26-venv` and its
  `bin` directory prepended to PATH for build/staging.
- Staged with `scripts/stage-kindle.sh`; installed with
  `scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222`.
  The installed binary hash matches; `/lib/ld-linux-armhf.so.3 --list`
  resolves its device dependencies.
- All four existing `puzzles-eink-book-*.json` hashes and the progress hash were
  unchanged by installation and checkpoint preparation. The app has been left
  stopped for the human's library launch.

A separate collection, `puzzles-task26-stylus.json`, was generated with:

```sh
python3 tools/pgn_converter.py tests/fixtures/pgn-converter/valid-book.pgn \
  --title 'Stylus checkpoint 26A' -o probe-output/task26/puzzles-task26-stylus.json
```

Its SHA-256 is `3ddf457bbac3472c3f952ed082c46904023e644ddd2c814b9404729bfde27b84`.
It contains three puzzles, including several main-line moves, an alternative,
a nested sideline, inline references, and black promotion. It was copied over
verified SCP to a temporary `/mnt/us` path, then moved to the collection directory
only after checking the destination did not exist. Existing collections were
not edited. SCP to `/tmp` closed the connection on two attempts; no claim about
its cause is made.

Human procedure: launch **Kindle Chess Puzzles**, tap the header title to open
collections, select **Stylus checkpoint 26A**, and tap **ANALYSIS** in the lower
status area. Use the panel's page controls to compare several main-line and
side-line targets with finger and pen. Check hover without contact, then exit
with X and check native notebook pen drawing. The human results are recorded below; installed/loader checks alone are not a
physical-interaction pass.

### HUMAN CHECKPOINT 26A — PASS (2026-10-04)

The user responded “all works” to the installed-build testing checklist. This
records the human's physical-device confirmation for implementation `7c6e1e6`
and the binary SHA-256 listed above on Barolo/firmware 5.19.6.

| Acceptance item | Human result |
| --- | --- |
| Launch the installed app from the Kindle library | Pass |
| Finger taps on multiple main-line and side-line chips | Pass |
| Stylus taps on the same targets | Pass |
| Finger/stylus select matching analysis nodes and board positions | Pass |
| Hover without contact does not activate a target | Pass |
| X exit restores native Kindle pen drawing | Pass |

No failures were reported. Task 26 is now Implemented. This checkpoint does not
add a new crash-exit, reboot, or natural suspend/resume observation.

## HUMAN CHECKPOINT 28A — accepted phase-two closure (2026-10-04)

After using the enriched real book collections, the user confirmed:
“we can close phase 2 now, everything works”, and requested Task 28 completion
and integration/push to main. This is the human's overall physical-device
acceptance of rich browsing and retained solving behavior, alongside the
previously recorded finger/stylus checkpoint 26A. No failures were reported; no
additional per-tap measurements or individual UX observations were supplied.

Tested runtime: implementation `7c6e1e6`, binary SHA-256
`020fe41c35d2c195579df7b568d00db7d71893b507cf5ffeee0ec8be9b82c17b`,
on first-generation Scribe/Barolo firmware 5.19.6. Closure changes only host
acceptance tests and documentation; the runtime remains unchanged.

The four deployed `puzzles-eink-book-001.json` through `004.json` contain 1,615
puzzles. Validated PGNs supplied 1,606 full analysis trees; nine puzzles retain
their legal existing main lines as main-line-only trees. Russian topics were
added to all puzzles. All generated edges were validated offline for legality,
SAN, and exact post-move FEN; all collections passed the Rust runtime parser.

Deployed collection SHA-256 values, verified against local files:

| File | SHA-256 |
| --- | --- |
| `puzzles-eink-book-001.json` | `4a5438b5a1574da45e810dd3680098ff06bd279df4005c77963e66b5570b314e` |
| `puzzles-eink-book-002.json` | `42155a57a7563d550db190f56a7842136adc60157d339738a8544ad0f944547a` |
| `puzzles-eink-book-003.json` | `065be32384bc50f803d2275c98244911b493d0ff306a0852564771712854bc0d` |
| `puzzles-eink-book-004.json` | `cd39b68b170f16c2babdf1224dd5532bf6184ea3c2b5d67f4940b0f3ee53bfa7` |

Progress SHA-256 before and after transfer was identical:
`dc22b8d58666b11fb9694b9248cdf56eeef364595a11cd4433af5b027a9dfcb9`.
Collection filenames, puzzle IDs and order were retained. Five corrected FENs
restore castling rights; 348 grading lines use the validated full solutions,
as explicitly authorized by the user. Local private collection data remains
Git-ignored rather than being published.

The top-left Refresh control remains the accepted residual-ghosting workaround.
No new reboot, natural suspend/resume, crash recovery, or waveform observation
is inferred from the user's overall acceptance.

## Task 29 — Analysis toolbar toggle deployed for testing (2026-10-04)

- Implementation commit: `476abee` on `main`, per explicit user request.
- All repository checks and the ARMv7/glibc-2.35 release build passed. The first
  build attempt lacked the pinned host-only `chess==1.11.2` test dependency;
  after installing it in a temporary virtual environment, an existing lifecycle
  readiness test failed once, passed its focused rerun, and passed the complete
  build checks. No lifecycle code was changed.
- Installed with `scripts/stage-kindle.sh` and
  `scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222`.
- Device binary SHA-256 matches the tested host artifact:
  `dae7826d30e0aa4ec0f67603d3afb1f32a20bbe20367c2a547aa78771bb08fab`.
- Before/after SSH hashes matched for all five uploaded collections and progress
  (`cb51f7f172fb66aad317ee8d3094e19f0aacde1bfa72f4c6ae202b55588fb7bb`).
- App is installed for library launch. Physical readability, finger/stylus
  toggling, preview return, and exit remain pending user testing. Deployment
  and host snapshots do not establish those physical observations.

## Task 30 — Inline PGN analysis deployed for testing (2026-10-04)

- Implementation commit: `e041215` on `phase-2`; `main` remains at `ed26376`.
- Full host gate and ARMv7/glibc-2.35 release build passed before deployment.
- Staged the matching build receipt with `scripts/stage-kindle.sh`, then installed
  using `scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222`.
- Read-only process inspection showed the app stopped before and after installation.
  The installer completed successfully; the app is ready for library launch.
- Installed binary SHA-256 matches the tested release:
  `f1cc7aef1ccd588f6365722aec2040360119af1fee2aded2efb05fb107fdf419`.
- Before/after SSH hashes matched for all five uploaded collections (the four
  book collections and Task 26 stylus fixture) and progress. Progress SHA-256:
  `cb51f7f172fb66aad317ee8d3094e19f0aacde1bfa72f4c6ae202b55588fb7bb`.
- Physical readability, finger/stylus taps, inline nested variations, preview
  return and exit are pending the user's validation. Deployment/hash checks do
  not establish those physical observations. Open the book icon in the toolbar
  to browse the new notation; use the same icon to return to the live board.

### Task 30 human acceptance — PASS (2026-10-04)

The user confirmed “all works” after testing the deployed inline PGN build and
requested merge to main and push. This records overall physical Scribe acceptance
of implementation `e041215`, binary SHA-256
`f1cc7aef1ccd588f6365722aec2040360119af1fee2aded2efb05fb107fdf419`.
No failures or additional individual tap measurements were reported. The previous
pending-validation note describes deployment-time status; Task 30 is now accepted.
No new reboot, suspend/resume or crash-recovery observation is inferred.

## Task 31 — Centered solver side and Cyrillic topics deployed (2026-10-04)

- Implementation commit `50ae2bf` on main, as explicitly requested by the user.
- Side-to-move text is centered in the header; NOTE/solved reveal displays JSON
  topic above the description. Reviewed host snapshots include white/black,
  three-line Russian topics, topic-only, Free Board and long puzzle metadata.
- Embedded Atkinson lacks Cyrillic. Pinned Noto Sans Regular/Bold now provide
  per-character fallback, with both OFL license notices present in the installed
  runtime. Existing Latin analysis fixture pixels below the header are unchanged.
- scripts/check.sh and the complete ARMv7/glibc-2.35 build gate passed. One
  known lifecycle-readiness test race occurred on the first build attempt;
  its focused rerun and the full repeated build passed. No lifecycle changes.
- Staged with scripts/stage-kindle.sh and deployed with
  scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222.
- The app exited during host verification. Read-only process checks showed it
  stopped before installation and afterward, with native Xorg/awesome present.
  No lifecycle/process signal was needed. The existing installer completed.
- Installed binary SHA-256 matches the tested release:
  `7bae259c2c4e5813e737ad85403638d66cc4ea4e4a20f674dee219163c60ec4e`.
- `/lib/ld-linux-armhf.so.3 --list` resolved all runtime dependencies.
- Before/after SHA-256 values matched for all five uploaded collections: the
  four book collection hashes recorded at checkpoint 28A remain unchanged;
  puzzles-task26-stylus.json remains
  `3ddf457bbac3472c3f952ed082c46904023e644ddd2c814b9404729bfde27b84`.
  Progress remains
  `2d2259febd5b28021f9abed6bdff6676080667cc44a4797ffe3e2693866ecd9e`.
- Ready for library launch. Physical Cyrillic readability and the new header's
  on-panel appearance await user review; no physical visual acceptance is inferred
  from installation, host snapshots or dynamic-loader checks.

### Task 31 topic visibility correction deployed — 2026-10-04

The user accepted the header but reported the topic only appeared with NOTE.
Clarified behavior: show the topic by default while solving, replacing the old
side-to-move status; NOTE shows the description alone. The corrected renderer
uses 40px topic text instead of 30px on the Scribe. Cyrillic rendering and the
header remain unchanged.

- Implementation `6d9b269` on main; full host checks and ARMv7/glibc-2.35 build
  passed without a retry. Regression test failed first on absent default topic;
  reviewed snapshots show all three enlarged topic lines and NOTE-only prose.
- Used scripts/stage-kindle.sh and the documented deployment script at
  root@192.168.1.20:2222. Before updating, checked recorded supervisor identity
  and the documented cleanup path; process inspection confirmed the app stopped
  with Xorg/awesome present before installation and afterward.
- Installed binary matches the validated release SHA-256:
  `71d1e225b557776d8b15c1246bbf13eab768bebe1f51d774db2d41de2bd308f5`.
- All five uploaded collection hashes remain unchanged. Progress SHA-256 before
  and after installation remains
  `2d2259febd5b28021f9abed6bdff6676080667cc44a4797ffe3e2693866ecd9e`.
- Ready for library launch. Physical acceptance of the corrected topic behavior
  and larger text awaits user review; installation is not visual acceptance.

## Task 32 — Redundant description and analysis headings removed (2026-10-04)

- User first requested preparation without deployment, then explicitly requested
  deployment. Full host checks, reviewed Gray8 snapshots and the ARMv7/glibc-2.35
  build passed before installation. Implementation is on main.
- Staged with `scripts/stage-kindle.sh` and installed through the existing
  `scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222` installer,
  using documented root/empty-password authentication. Process/lock inspection
  confirmed the app stopped before and after the update. Xorg/awesome remained
  present; no lifecycle signal was needed.
- Installed binary SHA-256 matches the tested artifact:
  `995dca18e2d4c7e8d71b5253a7d2b58941834ec8f285178a49dd47f4951f6d12`.
  `/lib/ld-linux-armhf.so.3 --list` resolves all runtime dependencies.
- The four local book collections remove the first `number - side - difficulty`
  line from both plain and structured descriptions (1,615 puzzles). Filenames,
  IDs/order, FENs, difficulty, topics, solutions, PGN and analysis nodes are
  unchanged. The renderer removes the separate analysis title/page-count row.
- Device collection hashes matched the original local backup copies before the
  update. Uploaded files were checksum-verified before same-filesystem renames,
  with rollback copies during replacement; temporary files were removed on
  success. Installed collection hashes match the local cleaned files:
  - 001: `825f62cd7b7e67aa6230b38066f1492812f072e3fad2b8f8d1219a62ad4152e6`
  - 002: `68ed5918d02f4572dc12d30991d9f564138f9bf0439be70b7ebfdd96f3c848b8`
  - 003: `ddacdeee0ebf3b71187294f64c41c7b705a33cd53648882c085f25fb27b6c563`
  - 004: `38ff9d4e0afd037f643a5ee4af866e988f96b1a404eafde4c4586e1d4b115a07`
- The Task 26 stylus fixture remains
  `3ddf457bbac3472c3f952ed082c46904023e644ddd2c814b9404729bfde27b84`.
  Progress before/after hash is unchanged:
  `816f4cd5927c30ff9320c14998a1c438d16897bfd19a0b16ffa6bbdcba0c0b60`.
- Ready for library launch. Physical readability and the updated on-panel page
  layout await user review; installation/hash checks are not visual acceptance.

## Task 33 — Borderless unselected analysis moves deployed (2026-10-04)

- Unselected tree moves and explicit prose references retain bold text but lose
  their rectangular outlines. Selected black backgrounds and white bold text
  remain identical. Reviewed nine before/after host analysis snapshots; every
  changed pixel was a removed outline, with geometry and pagination unchanged.
- The failing-first border regression and focused render/hit tests passed;
  full scripts/build-kindle.sh checks and ARMv7/glibc-2.35 build passed.
- Verified supervisor 30653 and app child 30684 against lock files, command lines
  and process names, then sent TERM to the supervisor via the documented cleanup
  path. App and lock were absent before installing; native Xorg/awesome present.
- Staged and deployed with the existing scripts to root@192.168.1.20:2222.
  Installed binary SHA-256 matches the tested build:
  `f63264af763fff428560de70b7e11739d80654a0791599e5b72728a4393a316e`.
  The device dynamic loader resolved all dependencies; app and lock remained
  absent after installation, with native Xorg/awesome present.
- All five collection hashes match the Task 32 values above. Progress SHA-256
  before/after is unchanged:
  `61f5aef3786faa56ae8d2396c0447dfb5312272d765787ccff20d648456fd9c6`.
- The user verified the deployed change on the Kindle on 2026-10-04 and
  requested commit and push. Physical visual acceptance is confirmed.

## Task 34 — Redundant book analysis descriptions cleaned (2026-10-04)

- User confirmed JSON cleanup was preferred. No renderer/runtime change remains.
  Offline tools/clean_book_analysis.py removes complete structured main-path
  summaries and Reference lines already in analysis comments from all 1,615
  private book puzzles (500/500/500/115). Nine original no-PGN plain notes remain
  available to NOTE. All analysis trees/comments, IDs/order, FENs, solutions,
  topics, difficulty and PGN/source/reference metadata are unchanged.
- Failing-first synthetic cleanup tests and complete scripts/build-kindle.sh
  gates passed, including unchanged renderer snapshots. The host parser/renderer
  audited all 1,615 cleaned puzzles and every analysis page: PGN starts at the
  top, all tree moves/comment references remain, and preview does not alter
  progress. Reviewed real 135b before/after: two redundant rows are removed.
- Implementation/tools/tests/docs commit `a265f53` was pushed to main before
  uploading the cleaned collection JSON via scp -O / SSH port 2222 to the verified
  root@192.168.1.20. App/process/lock checks showed the app already stopped.
  No lifecycle signal or service change was needed. Temporary install/app locks
  protected replacement; original and uploaded hashes were checked before
  same-filesystem renames, with rollback originals until success.
- Installed hashes match the local cleaned files:
  - 001: `b22b1b1b2bd4b331ee00ce154934b576ce1e31bc59867543c8b833d1e992edf3`
  - 002: `8dfadf965472f5e6d744df85e10435c1bd1af887c125020af39f1042ba4bad0a`
  - 003: `b8c0eaed5e01a19c023db746594d3ca024009a5daf09d8bb3e5fe8d0a65e5357`
  - 004: `7685d3583e42855c80a760454223e9727623bffa881f18444cfe220e78ed5d6e`
- Progress before/after SHA-256 is unchanged:
  `61f5aef3786faa56ae8d2396c0447dfb5312272d765787ccff20d648456fd9c6`.
  Stylus fixture hash remains
  `3ddf457bbac3472c3f952ed082c46904023e644ddd2c814b9404729bfde27b84`.
  The rebuilt release matches the installed Task 33 binary exactly:
  `f63264af763fff428560de70b7e11739d80654a0791599e5b72728a4393a316e`;
  it was not reinstalled for this data-only update.
- Verified app/locks and remote upload files absent after installation, with
  native Xorg/awesome present. Original private collections remain locally under
  target/task34/original-collections/. Ready for library launch; physical visual
  acceptance awaits user review.

## Task 35 — Conventional move annotations deployed (2026-10-04)

- User requested mapping raw PGN codes to conventional notation, a direct main
  commit and device deployment. Renderer maps codes 1–6 to !, ?, !!, ??, !?, ?!,
  attached to the tappable tree-move SAN. Other codes retain $N. Stored JSON,
  reference labels and literal prose are unchanged.
- Failing-first mapping tests passed after implementation. Tests cover all six
  codes, multiple/unknown annotations, narrow wrapping, taps on annotated labels,
  inversion and unchanged solve/progress. Reviewed six symbol snapshots,
  selected/unknown states, three changed existing snapshots and real 135b:
  Qxc1+ $1 becomes Qxc1+!, and Bxb2 $1 becomes Bxb2!. Upper UI is identical.
- Full scripts/build-kindle.sh checks and ARMv7/glibc-2.35 build passed. Commit
  c762853 landed directly on main and was pushed before stage/deploy with the
  documented scripts at root@192.168.1.20:2222.
- App was running on initial inspection but exited during host checks. The
  attempted signal preflight stopped before sending any signal because those
  processes had exited. Fresh read-only checks confirmed app and lock absent
  before installation and afterward; native Xorg/awesome remained present.
- Installed binary SHA-256 matches the validated artifact:
  `211e637917fbdcf4311605deb7da227164e0767cf61fcc114f24933077877adc`.
  The device dynamic loader resolved all dependencies.
- All five collection hashes are unchanged from Task 34. Progress before/after
  remains `77fff6499aae93b3baccccf59d75989d4cf9f2b70074b50daf71e9c034f538ef`.
- Ready for library launch. Physical visual acceptance awaits user review;
  installation and host snapshots do not establish on-panel readability.
