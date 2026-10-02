#!/usr/bin/env python3
"""Decode short raw Linux evdev captures made by kindle_capture_input.sh.

Supports the two common struct input_event layouts:
- 16 bytes: 32-bit timeval (common on armv7 userspace)
- 24 bytes: 64-bit timeval

The decoder is host-side only and never touches a device. With --five-point,
it expects contacts in this order: top-left, top-right, center, bottom-left,
bottom-right, and prints a raw-axis orientation/scale hint.
"""

from __future__ import annotations

import argparse
import collections
import pathlib
import statistics
import struct
import sys
from dataclasses import dataclass
from typing import Iterable

EVENT_LAYOUTS = {
    "32": ("<IIHHi", 16),
    "64": ("<QQHHi", 24),
}

EV_NAMES = {
    0x00: "EV_SYN",
    0x01: "EV_KEY",
    0x02: "EV_REL",
    0x03: "EV_ABS",
    0x04: "EV_MSC",
    0x05: "EV_SW",
    0x11: "EV_LED",
    0x12: "EV_SND",
    0x14: "EV_REP",
    0x15: "EV_FF",
    0x16: "EV_PWR",
    0x17: "EV_FF_STATUS",
}

SYN_NAMES = {
    0x00: "SYN_REPORT",
    0x01: "SYN_CONFIG",
    0x02: "SYN_MT_REPORT",
    0x03: "SYN_DROPPED",
}

ABS_NAMES = {
    0x00: "ABS_X",
    0x01: "ABS_Y",
    0x18: "ABS_PRESSURE",
    0x19: "ABS_DISTANCE",
    0x1A: "ABS_TILT_X",
    0x1B: "ABS_TILT_Y",
    0x2F: "ABS_MT_SLOT",
    0x30: "ABS_MT_TOUCH_MAJOR",
    0x31: "ABS_MT_TOUCH_MINOR",
    0x32: "ABS_MT_WIDTH_MAJOR",
    0x33: "ABS_MT_WIDTH_MINOR",
    0x34: "ABS_MT_ORIENTATION",
    0x35: "ABS_MT_POSITION_X",
    0x36: "ABS_MT_POSITION_Y",
    0x37: "ABS_MT_TOOL_TYPE",
    0x38: "ABS_MT_BLOB_ID",
    0x39: "ABS_MT_TRACKING_ID",
    0x3A: "ABS_MT_PRESSURE",
    0x3B: "ABS_MT_DISTANCE",
}

KEY_NAMES = {
    0x140: "BTN_TOOL_PEN",
    0x141: "BTN_TOOL_RUBBER",
    0x142: "BTN_TOOL_BRUSH",
    0x143: "BTN_TOOL_PENCIL",
    0x144: "BTN_TOOL_AIRBRUSH",
    0x145: "BTN_TOOL_FINGER",
    0x146: "BTN_TOOL_MOUSE",
    0x147: "BTN_TOOL_LENS",
    0x14A: "BTN_TOUCH",
    0x14B: "BTN_STYLUS",
    0x14C: "BTN_STYLUS2",
}


@dataclass(frozen=True)
class Event:
    sec: int
    usec: int
    type: int
    code: int
    value: int


@dataclass(frozen=True)
class Contact:
    source: str
    x: int
    y: int
    samples: int


def unpack_events(data: bytes, layout: str) -> tuple[list[Event], int]:
    fmt, size = EVENT_LAYOUTS[layout]
    count = len(data) // size
    events = [Event(*struct.unpack_from(fmt, data, offset)) for offset in range(0, count * size, size)]
    return events, len(data) - count * size


def plausibility(events: Iterable[Event]) -> float:
    events = list(events)
    if not events:
        return -1.0
    score = 0.0
    for event in events[:200]:
        if 0 <= event.type <= 0x1F:
            score += 3.0
        else:
            score -= 8.0
        if 0 <= event.code <= 0x3FF:
            score += 1.0
        else:
            score -= 2.0
        if 0 <= event.usec < 1_000_000:
            score += 1.0
        else:
            score -= 1.0
        if event.type == 0 and event.code <= 3:
            score += 1.0
        if event.type == 3 and event.code <= 0x3F:
            score += 1.0
    return score / min(len(events), 200)


def choose_layout(data: bytes) -> str:
    candidates: list[tuple[float, str]] = []
    for layout in EVENT_LAYOUTS:
        events, leftover = unpack_events(data, layout)
        if not events:
            continue
        score = plausibility(events)
        score -= leftover / EVENT_LAYOUTS[layout][1]
        candidates.append((score, layout))
    if not candidates:
        raise ValueError("capture is too short to contain an input_event record")
    candidates.sort(reverse=True)
    best_score, best_layout = candidates[0]
    if best_score < 1.0:
        raise ValueError(
            "could not recognize input_event layout; try --layout 32 or --layout 64 explicitly"
        )
    return best_layout


def code_name(event: Event) -> str:
    if event.type == 0:
        return SYN_NAMES.get(event.code, f"SYN_0x{event.code:02x}")
    if event.type == 1:
        return KEY_NAMES.get(event.code, f"KEY_0x{event.code:03x}")
    if event.type == 3:
        return ABS_NAMES.get(event.code, f"ABS_0x{event.code:02x}")
    return f"CODE_0x{event.code:03x}"


def event_type_name(event: Event) -> str:
    return EV_NAMES.get(event.type, f"EV_0x{event.type:02x}")


def extract_contacts(events: list[Event]) -> list[Contact]:
    """Extract sequential one-pointer contacts from common MT-B or ABS_X/Y traces."""
    contacts: list[Contact] = []

    has_mt = any(event.type == 3 and event.code in (0x35, 0x36, 0x39) for event in events)
    if has_mt:
        active = False
        current_x = None
        current_y = None
        points: list[tuple[int, int]] = []

        def finish_mt() -> None:
            nonlocal points
            if points:
                xs = [p[0] for p in points]
                ys = [p[1] for p in points]
                contacts.append(
                    Contact("mt", round(statistics.median(xs)), round(statistics.median(ys)), len(points))
                )
            points = []

        for event in events:
            if event.type == 3 and event.code == 0x39:
                if event.value >= 0:
                    if active:
                        finish_mt()
                    active = True
                    current_x = None
                    current_y = None
                else:
                    if active:
                        finish_mt()
                    active = False
                    current_x = None
                    current_y = None
            elif event.type == 3 and event.code == 0x35:
                current_x = event.value
            elif event.type == 3 and event.code == 0x36:
                current_y = event.value
            elif event.type == 0 and event.code == 0 and active:
                if current_x is not None and current_y is not None:
                    points.append((current_x, current_y))
        if active:
            finish_mt()
        if contacts:
            return contacts

    active = False
    current_x = None
    current_y = None
    points: list[tuple[int, int]] = []

    def finish_st() -> None:
        nonlocal points
        if points:
            xs = [p[0] for p in points]
            ys = [p[1] for p in points]
            contacts.append(
                Contact("single", round(statistics.median(xs)), round(statistics.median(ys)), len(points))
            )
        points = []

    for event in events:
        if event.type == 1 and event.code == 0x14A:
            if event.value:
                if active:
                    finish_st()
                active = True
                current_x = None
                current_y = None
            else:
                if active:
                    finish_st()
                active = False
        elif event.type == 3 and event.code == 0x00:
            current_x = event.value
        elif event.type == 3 and event.code == 0x01:
            current_y = event.value
        elif event.type == 0 and event.code == 0 and active:
            if current_x is not None and current_y is not None:
                points.append((current_x, current_y))
    if active:
        finish_st()
    return contacts


def parse_screen(value: str) -> tuple[int, int]:
    try:
        width_s, height_s = value.lower().split("x", 1)
        width, height = int(width_s), int(height_s)
    except (ValueError, TypeError) as exc:
        raise argparse.ArgumentTypeError("screen must be WIDTHxHEIGHT, e.g. 1860x2480") from exc
    if width <= 0 or height <= 0:
        raise argparse.ArgumentTypeError("screen dimensions must be positive")
    return width, height


def print_five_point_hint(contacts: list[Contact], screen: tuple[int, int] | None) -> None:
    print("\nfive_point_transform_hint:")
    if len(contacts) < 5:
        print(f"  need at least 5 contacts; found {len(contacts)}")
        return
    tl, tr, center, bl, br = contacts[:5]
    print("  expected_order=top-left,top-right,center,bottom-left,bottom-right")
    print(f"  raw_top_left={tl.x},{tl.y}")
    print(f"  raw_top_right={tr.x},{tr.y}")
    print(f"  raw_center={center.x},{center.y}")
    print(f"  raw_bottom_left={bl.x},{bl.y}")
    print(f"  raw_bottom_right={br.x},{br.y}")

    hx, hy = tr.x - tl.x, tr.y - tl.y
    vx, vy = bl.x - tl.x, bl.y - tl.y
    if abs(hx) >= abs(hy) and abs(vy) >= abs(vx):
        mapping = "display_x<-raw_x, display_y<-raw_y"
        x_start = (tl.x + bl.x) / 2
        x_end = (tr.x + br.x) / 2
        y_start = (tl.y + tr.y) / 2
        y_end = (bl.y + br.y) / 2
    elif abs(hy) > abs(hx) and abs(vx) > abs(vy):
        mapping = "display_x<-raw_y, display_y<-raw_x (axes swapped)"
        x_start = (tl.y + bl.y) / 2
        x_end = (tr.y + br.y) / 2
        y_start = (tl.x + tr.x) / 2
        y_end = (bl.x + br.x) / 2
    else:
        print("  mapping=ambiguous; inspect full event sequence and repeat taps farther from bezels")
        return

    print(f"  mapping={mapping}")
    print(f"  horizontal_direction={'increasing' if x_end > x_start else 'decreasing'}")
    print(f"  vertical_direction={'increasing' if y_end > y_start else 'decreasing'}")
    print(f"  observed_raw_horizontal_edges={x_start:.1f},{x_end:.1f}")
    print(f"  observed_raw_vertical_edges={y_start:.1f},{y_end:.1f}")

    if screen is not None and x_end != x_start and y_end != y_start:
        width, height = screen
        print(f"  screen={width}x{height}")
        print(
            "  approximate_normalized_formula: "
            f"x=(raw_h-{x_start:.1f})/({x_end - x_start:.1f}), "
            f"y=(raw_v-{y_start:.1f})/({y_end - y_start:.1f})"
        )
        print(
            "  approximate_pixel_formula: "
            f"px=round(x*{width - 1}), py=round(y*{height - 1}); clamp to screen bounds"
        )
        print("  note=corner taps give observed edges, not necessarily kernel-declared ABS minima/maxima")


def print_summary(events: list[Event], layout: str, leftover: int) -> list[Contact]:
    print(f"layout={layout}-bit-timeval")
    print(f"events={len(events)}")
    print(f"leftover_bytes={leftover}")

    counts = collections.Counter(event.type for event in events)
    print("event_types=" + ", ".join(f"{EV_NAMES.get(t, hex(t))}:{n}" for t, n in sorted(counts.items())))

    abs_values: dict[int, list[int]] = collections.defaultdict(list)
    key_values: dict[int, list[int]] = collections.defaultdict(list)
    for event in events:
        if event.type == 3:
            abs_values[event.code].append(event.value)
        elif event.type == 1:
            key_values[event.code].append(event.value)

    print("absolute_observed_ranges:")
    if not abs_values:
        print("  [none]")
    else:
        for code, values in sorted(abs_values.items()):
            name = ABS_NAMES.get(code, f"ABS_0x{code:02x}")
            print(f"  {name}: min={min(values)} max={max(values)} samples={len(values)}")

    interesting = []
    for code in (0x35, 0x36, 0x00, 0x01, 0x39, 0x3A, 0x18):
        if code in abs_values:
            interesting.append(ABS_NAMES.get(code, hex(code)))
    if interesting:
        print("coordinate_candidates=" + ",".join(interesting))

    print("key/button_activity:")
    if not key_values:
        print("  [none]")
    else:
        for code, values in sorted(key_values.items()):
            name = KEY_NAMES.get(code, f"KEY_0x{code:03x}")
            print(f"  {name}: values={sorted(set(values))} samples={len(values)}")

    contacts = extract_contacts(events)
    print("contacts:")
    if not contacts:
        print("  [none extracted]")
    else:
        for index, contact in enumerate(contacts, 1):
            print(
                f"  {index}: source={contact.source} x={contact.x} y={contact.y} samples={contact.samples}"
            )
    return contacts


def print_events(events: list[Event]) -> None:
    if not events:
        return
    base = events[0].sec + events[0].usec / 1_000_000.0
    print("\nevent_sequence:")
    for index, event in enumerate(events):
        when = event.sec + event.usec / 1_000_000.0 - base
        print(
            f"{index:05d} +{when:9.6f}s "
            f"{event_type_name(event):10s} {code_name(event):20s} value={event.value}"
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture", type=pathlib.Path, help="raw .bin capture")
    parser.add_argument(
        "--layout",
        choices=("auto", "32", "64"),
        default="auto",
        help="struct input_event timeval width; default: auto-detect",
    )
    parser.add_argument(
        "--summary-only",
        action="store_true",
        help="print observed ranges/buttons/contacts without the full event sequence",
    )
    parser.add_argument(
        "--five-point",
        action="store_true",
        help="interpret first five contacts as TL, TR, center, BL, BR and infer orientation",
    )
    parser.add_argument(
        "--screen",
        type=parse_screen,
        help="screen pixel size WIDTHxHEIGHT; combines with --five-point for an approximate transform",
    )
    args = parser.parse_args(argv)

    data = args.capture.read_bytes()
    try:
        layout = choose_layout(data) if args.layout == "auto" else args.layout
        events, leftover = unpack_events(data, layout)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    print(f"capture={args.capture}")
    print(f"bytes={len(data)}")
    contacts = print_summary(events, layout, leftover)
    if args.five_point:
        print_five_point_hint(contacts, args.screen)
    if not args.summary_only:
        print_events(events)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
