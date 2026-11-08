#!/usr/bin/env python3
"""Prepare a private Kindle config locally; never connects to a board or device."""
import argparse
import json
import os
from pathlib import Path
import re
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def prepare(source, target):
    data = json.loads(Path(source).read_text())
    address = data.get('ble', {}).get('address')
    if not isinstance(address, str) or not re.fullmatch(r'(?:[0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}', address):
        raise ValueError('source needs a valid ble.address; no config written')
    target = Path(target)
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.is_symlink(): raise ValueError('refusing to replace symlink')
    fd, name = tempfile.mkstemp(prefix='.chessnut-', dir=target.parent)
    try:
        with os.fdopen(fd, 'w') as output:
            json.dump({'address': address}, output)
            output.write('\n')
            output.flush()
            os.fsync(output.fileno())
        os.replace(name, target)
    finally:
        Path(name).unlink(missing_ok=True)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=ROOT/'local-kindle/chessnut-move/devices.private.json')
    parser.add_argument('--output', type=Path, default=ROOT/'local-kindle/chessnut-move/runtime.private.json')
    args = parser.parse_args()
    try:
        prepare(args.source, args.output)
    except (OSError, ValueError, TypeError, AttributeError):
        raise SystemExit('Cannot prepare private config; check source ble.address and output permissions.') from None
    print('Private board config prepared (mode 600). No connection attempted.')
