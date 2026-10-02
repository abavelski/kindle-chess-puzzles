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
