# Dark-square ghosting: framebuffer and physical-screen findings

Recorded on 2026-10-03; operational decision updated on 2026-10-04. The
regional-clean fix improves rendering on the Scribe, but faint physical piece
ghosts can remain. Their exact mechanism is unresolved. Further investigation
is archived because the existing top-left **Refresh** control is an acceptable
manual workaround when a ghost becomes visible.

## Accepted operational workaround

If a recognizable residual ghost is visible during normal use, tap the app's top-left
**Refresh** button. The manual refresh is accepted as the product workaround; no additional
waveform/controller investigation is currently planned. This does not claim the underlying
panel/controller behavior has been fixed.

## Target and tested build

- Kindle Scribe first generation (Barolo), firmware 5.19.6.
- Visible framebuffer: 1860×2480, 8-bit Y8, 1872-byte scanline stride.
- FBInk revision: `92e127008145b2a22fba7c59815d810d716310dd`.
- Installed binary SHA-256:
  `45994cabb7407d867f3b330e2dce7f06591c09f11d1ba7505c08221bff67ab63`.
- Production behavior is the regional-clean branch at `00f31cb`, merged into
  main at `8accb0f`, plus regression tests and device documentation. The merged
  production code produces the same installed binary.

Startup uses a flashing AUTO refresh over the full frame. Committed piece
changes use flashing AUTO over the affected complete squares, or the board for
large/reoriented changes. On the pinned Kindle MTK path, flashing AUTO maps to
GC16 with FULL update strength. Selection and other ordinary UI changes remain
partial AUTO. There is no white intermediate frame or periodic whole-screen
cleaning. See [the implementation notes](EINK_LIFECYCLE.md).

## Controlled reproduction: queen c7 → d6

The app was started through the verified supervisor, and its startup buffer
was saved before any taps. The user switched to Free Board, moved the black
queen from c7 to d6, and reported a very light queen ghost on c7. The board
was black-facing. Logs confirm origin `Square(10)` and destination `Square(19)`.

After the report, the visible framebuffer was captured without triggering
another app refresh. The full c7 square was compared with its startup pixels
and with the empty dark square e7 in the same post-move frame.

| Measurement | Result |
| --- | --- |
| c7 rectangle, display pixels | x=1142, y=1496, width=212, height=212 |
| Total c7 pixels | 44,944 |
| Before move, gray value 0 | 10,594 pixels |
| Before move, gray value 51 | 6,615 pixels |
| Before move, gray value 184 | 27,735 pixels |
| After move, gray value 184 | All 44,944 pixels |
| After move, pixels differing from 184 | Zero |
| After move, c7 compared with empty e7 | Byte-for-byte identical |

The destination d6 has the same pixel histogram as the original queen on c7.
The comparison below shows c7 before the move on the left and afterward on
the right, enlarged twofold using nearest-neighbor sampling.

![Framebuffer c7 before and after the queen move](images/ghosting-c7-framebuffer.png)

The user supplied `IMG_9676.HEIC`. Its physical-screen photograph shows a faint
queen silhouette on c7, directly above the bishop on c8, while the queen is
visible on d6. The photograph and framebuffer measurements therefore agree
on the position but differ on the visible residue.

## What this establishes, and what it does not

For this captured position, no queen-shaped pixels remain in the visible
framebuffer on c7. The renderer and framebuffer write have cleared the entire
origin square. The visible ghost is a real physical-screen observation, not
an image of the queen preserved in those framebuffer bytes.

The evidence places the remaining issue downstream of the visible framebuffer:
the display refresh path, controller history/state, or panel response. It does
not distinguish those mechanisms, verify every submitted refresh rectangle,
or establish an unavoidable Kindle hardware limitation. A framebuffer screenshot
cannot measure the physical panel's residual image.

## Experiments already tried

| Experiment | Physical result | Final disposition |
| --- | --- | --- |
| Regional flashing AUTO over changed complete squares | User reported substantial improvement, with faint dark-square ghosts remaining | Kept; user accepted for merge |
| White clearing pass, waited on before final pixels | Native UI ghosting became worse, including on light squares; piece ghosts remained | Reverted |
| Flashing REAGLD for startup and regional cleaning | Completely white physical screen on repeated launches despite correct framebuffer pixels | Reverted |

For the white-pass experiment, the user saw a briefly white square followed by
the ghost returning with the final gray redraw. This observation alone does
not mean the app saved or copied the ghost: later c7 measurement shows that the
final visible framebuffer contains a uniform gray square.

The REAGLD failure has a specific kernel diagnostic:

```text
[HWTCON ERR]waveform mode[5] not loaded night_mode[0] @wf_lut_get_waveform_mode_slot,608
```

Pinned FBInk maps REAGLD to MTK GLD16, mode 5. That day-mode waveform was not
loaded on this device. Submission and completion still returned success, so
errno-based unsupported-mode fallback did not activate. Correct framebuffer
captures and successful API calls were insufficient to validate that mode.
Restoring the original exact binary removed the experimental mode; its startup
probe produced no new missing-waveform errors.

## Why the Kobo observation does not establish a hardware limit

The user reports no comparable ghosting in the Kobo app, but clarified that
it uses Cobalt. This Kindle app uses a project-owned grayscale renderer and a
direct FBInk adapter. These are different runtime/display paths, not the same
application backend running on two devices.

Cobalt's refresh policy, timing, region handling and cleaning behavior have not
been inspected in this investigation. They are possible comparison points,
not established explanations. The Kobo result is useful evidence that a similar
chess interface can look clean, but does not isolate hardware from software.

## Next tests, not yet performed

1. Preserve the exact post-move frame and refresh it once using the known-working
   flashing AUTO/GC16 path over the whole screen. Obtain a physical observation
   or photo of c7 before and after, rather than relying on another buffer dump.
2. If that clears c7, compare targeted square, whole-board and whole-screen
   coverage using the same final pixels and supported waveform. Measure flash
   acceptability and completion timing.
3. Inspect the actual Kobo/Cobalt display path to compare refresh mode, update
   sequencing and cleaning cadence with the Kindle adapter.

If a larger cleaning refresh does not clear the ghost, investigate controller
state and panel response further. Do not assume that a mode listed in a header
is loaded by the target firmware, or treat successful ioctls as visual acceptance.

The executable follow-up is tracked as [Task 10](../tasks/10-scribe-ghosting-follow-up.md). It preserves the accepted regional-clean build as the control and tests repeated final-pixel cleaning, coverage, explicit GC16, serialized waits, Kindle MTK fast mode, and only then a direct-ioctl A/B probe.

## Evidence and reproducibility

The small framebuffer comparison is committed with this document. Raw dumps,
full screenshots, the analysis JSON and the user's photo remain local; they
are not present in a fresh checkout.

| Local evidence | SHA-256 |
| --- | --- |
| `probe-output/dark-square-test/launch.raw` | `fc119182f301fa8377456b43c3add2884478886878964bc296a16ffed85c3fee` |
| `probe-output/dark-square-test/after-c7-d6.raw` | `09ec2936dd9f44a4d4e838b62e26888f1a980762503de3a6c716141a25d54f02` |
| User photo `IMG_9676.HEIC` | `d4ab19d8a5f89406e02fdefe4d168226275df14cb786561f5e3662a4288ce322` |

The capture command reads the measured visible rows, including stride padding:

```sh
dd if=/dev/fb0 bs=1872 count=2480
```

Analyze each c7 row as 212 bytes starting at
`(1496 + row) * 1872 + 1142`, for rows 0 through 211. Exclude framebuffer
padding when creating the full 1860-pixel-wide screenshot. The histogram uses
raw bytes, without image contrast enhancement or compression loss.

For the earlier device probes, recovery and build checks, see
[the device record](device/ks1-barolo.md) and
[the original fix plan](PIECE_GHOSTING_FIX.md). This document adds measured
findings; it changes no app behavior, task status or lifecycle checkpoint.
