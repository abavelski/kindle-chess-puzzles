import importlib.util
import pathlib
import struct
import sys
import unittest

MODULE_PATH = pathlib.Path(__file__).parents[1] / "tools" / "decode_evdev.py"
spec = importlib.util.spec_from_file_location("decode_evdev", MODULE_PATH)
decode_evdev = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = decode_evdev
spec.loader.exec_module(decode_evdev)


class DecodeEvdevTests(unittest.TestCase):
    def test_decodes_32_bit_input_event_records(self):
        data = b"".join(
            [
                struct.pack("<IIHHi", 10, 100, 3, 0x35, 42),
                struct.pack("<IIHHi", 10, 200, 3, 0x36, 99),
                struct.pack("<IIHHi", 10, 300, 0, 0, 0),
            ]
        )
        self.assertEqual(decode_evdev.choose_layout(data), "32")
        events, leftover = decode_evdev.unpack_events(data, "32")
        self.assertEqual(leftover, 0)
        self.assertEqual(events[0].value, 42)
        self.assertEqual(decode_evdev.code_name(events[0]), "ABS_MT_POSITION_X")
        self.assertEqual(decode_evdev.code_name(events[1]), "ABS_MT_POSITION_Y")
        self.assertEqual(decode_evdev.code_name(events[2]), "SYN_REPORT")

    def test_decodes_64_bit_input_event_records(self):
        data = b"".join(
            [
                struct.pack("<QQHHi", 1000, 10, 1, 0x14A, 1),
                struct.pack("<QQHHi", 1000, 20, 1, 0x14A, 0),
                struct.pack("<QQHHi", 1000, 30, 0, 0, 0),
            ]
        )
        self.assertEqual(decode_evdev.choose_layout(data), "64")
        events, leftover = decode_evdev.unpack_events(data, "64")
        self.assertEqual(leftover, 0)
        self.assertEqual(decode_evdev.code_name(events[0]), "BTN_TOUCH")

    def test_extracts_multitouch_contacts(self):
        records = []
        usec = 0
        for tracking, x, y in [(10, 100, 200), (11, 900, 800)]:
            for type_, code, value in [
                (3, 0x39, tracking),
                (3, 0x35, x),
                (3, 0x36, y),
                (0, 0, 0),
                (3, 0x39, -1),
                (0, 0, 0),
            ]:
                usec += 100
                records.append(struct.pack("<IIHHi", 10, usec, type_, code, value))
        events, _ = decode_evdev.unpack_events(b"".join(records), "32")
        contacts = decode_evdev.extract_contacts(events)
        self.assertEqual([(c.x, c.y) for c in contacts], [(100, 200), (900, 800)])

    def test_partial_trailing_record_is_reported_not_crashed(self):
        data = struct.pack("<IIHHi", 10, 100, 3, 0x35, 42) + b"abc"
        events, leftover = decode_evdev.unpack_events(data, "32")
        self.assertEqual(len(events), 1)
        self.assertEqual(leftover, 3)

    def test_parse_screen(self):
        self.assertEqual(decode_evdev.parse_screen("1860x2480"), (1860, 2480))

    def test_rejects_too_short_capture(self):
        with self.assertRaises(ValueError):
            decode_evdev.choose_layout(b"short")


if __name__ == "__main__":
    unittest.main()
