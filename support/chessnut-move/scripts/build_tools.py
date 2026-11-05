"""Pinned probe build primitives. No downloads, SSH, installation or application integration."""
import hashlib
from pathlib import Path
import re
import struct

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def inspect_elf(data, executable, device_reference=False):
    """Read ELF tables, not incidental strings, including actual version needs."""
    def unpack(fmt, offset):
        if offset < 0 or offset + struct.calcsize(fmt) > len(data):
            raise ValueError('truncated ELF table')
        return struct.unpack_from(fmt, data, offset)
    if len(data) < 52 or data[:7] != b'\x7fELF\x01\x01\x01':
        raise ValueError('expected ELF32 little-endian')
    kind, machine, version, _, phoff, _, flags, _, phsize, phnum, _, _, _ = unpack('<HHIIIIIHHHHHH', 16)
    if machine != 40 or version != 1 or kind not in (2, 3) or phsize != 32:
        raise ValueError('expected ARM executable/shared object')
    if executable and kind != 3:
        raise ValueError('native executables must use PIE to avoid imported-data copy relocation hazards')
    if flags & 0xff000000 != 0x05000000 or not flags & 0x400 or flags & 0x200:
        raise ValueError('expected EABI5 hard-float')
    headers = [unpack('<IIIIIIII', phoff + i * phsize) for i in range(phnum)]
    def mapped(address, length=1):
        for typ, off, va, _, size, _, _, _ in headers:
            if typ == 1 and va <= address and address + length <= va + size:
                pos = off + address - va
                if pos + length <= len(data): return pos
        raise ValueError('unmapped ELF address')
    interpreter = None
    dynamic = {}
    for typ, offset, _, _, size, _, _, _ in headers:
        if offset + size > len(data): raise ValueError('truncated ELF segment')
        if typ == 3:
            interpreter = data[offset:offset+size].rstrip(b'\0').decode('ascii')
        if typ == 2:
            for pos in range(offset, offset + size, 8):
                tag, value = unpack('<II', pos)
                if tag == 0: break
                dynamic.setdefault(tag, []).append(value)
    if executable and interpreter != '/lib/ld-linux-armhf.so.3':
        raise ValueError('wrong/missing dynamic loader')
    if not executable and interpreter is not None and not device_reference:
        raise ValueError('shared object unexpectedly has interpreter')
    if 5 not in dynamic or 10 not in dynamic:
        raise ValueError('missing dynamic string table')
    string_size = dynamic[10][0]
    strings = mapped(dynamic[5][0], string_size)
    def string(index):
        if index >= string_size: raise ValueError('bad dynamic string index')
        end = data.find(b'\0', strings + index, strings + string_size)
        if end < 0: raise ValueError('unterminated dynamic string')
        return data[strings+index:end].decode('ascii')
    needed = [string(i) for i in dynamic.get(1, [])]
    if any(not re.fullmatch(r'[A-Za-z0-9_.+-]+', n) for n in needed):
        raise ValueError('unsafe or absolute NEEDED entry')
    if 15 in dynamic or 29 in dynamic: raise ValueError('unexpected RPATH/RUNPATH')
    # Zig 0.13 libc object stubs alias distinct globals. A COPY relocation
    # can promote those aliases into the executable and corrupt libc startup.
    # Reject COPY in the actual relocation tables, even if the ELF is PIE.
    for address_tag, size_tag, entry_tag, entry_size in [(17, 18, 19, 8), (7, 8, 9, 12)]:
        if address_tag not in dynamic and size_tag not in dynamic: continue
        if address_tag not in dynamic or size_tag not in dynamic:
            raise ValueError('incomplete ELF relocation table')
        size = dynamic[size_tag][0]
        if dynamic.get(entry_tag, [entry_size])[0] != entry_size or size % entry_size:
            raise ValueError('invalid ELF relocation entry size')
        if size == 0: continue
        start = mapped(dynamic[address_tag][0], size)
        for pos in range(start, start+size, entry_size):
            _, info = unpack('<II', pos)
            if info & 0xff == 20:  # R_ARM_COPY
                raise ValueError('unsafe R_ARM_COPY relocation; compile native code with -fPIC -pie')
    versions = []
    if 0x6ffffffe in dynamic:
        pos = mapped(dynamic[0x6ffffffe][0], 16)
        for _ in range(dynamic.get(0x6fffffff, [0])[0]):
            _, count, _, aux, nxt = unpack('<HHIII', pos)
            at = pos + aux
            for _ in range(count):
                _, _, _, name, following = unpack('<IHHII', at)
                value = string(name)
                if value == 'GLIBC_PRIVATE' and device_reference: continue
                if value.startswith('GLIBC_'):
                    parsed = re.fullmatch(r'GLIBC_(\d+)\.(\d+)(?:\.(\d+))?', value)
                    if not parsed or tuple(int(v or 0) for v in parsed.groups()) > (2, 35, 0):
                        raise ValueError(f'unsupported GLIBC requirement: {value}')
                    versions.append(value)
                at += following
            pos += nxt
    return {'needed': sorted(needed), 'interpreter': interpreter, 'glibc_versions': sorted(set(versions)),
            'machine': 'ARM', 'class': 'ELF32', 'abi': 'EABI5 hard-float',
            'pie': executable and kind == 3, 'copy_relocations': 0}

def localize_needed(data, allowed_file):
    """Point an absolute SDK NEEDED entry at its existing basename suffix.

    ACE DSOs lack SONAME. This is the narrow DT_NEEDED correction that upstream
    Meson performs with patchelf, without editing SDK/vendor binaries.
    """
    result = bytearray(data)
    if data[:7] != b'\x7fELF\x01\x01\x01': raise ValueError('expected ELF32')
    phoff = struct.unpack_from('<I', data, 28)[0]
    phsize, phnum = struct.unpack_from('<HH', data, 42)
    headers = [struct.unpack_from('<IIIIIIII', data, phoff+i*phsize) for i in range(phnum)]
    entries = {}
    for typ, offset, _, _, size, _, _, _ in headers:
        if typ != 2: continue
        for pos in range(offset, offset+size, 8):
            tag, value = struct.unpack_from('<II', data, pos)
            if tag == 0: break
            entries.setdefault(tag, []).append((pos+4, value))
    va = entries[5][0][1]
    start = next(off+va-base for typ, off, base, _, size, _, _, _ in headers if typ == 1 and base <= va < base+size)
    for position, index in entries.get(1, []):
        end = data.index(b'\0', start+index)
        name = data[start+index:end].decode('ascii')
        if '/' not in name: continue
        if name != str(allowed_file): raise ValueError('unexpected absolute library dependency')
        basename = Path(name).name
        struct.pack_into('<I', result, position, index+len(name)-len(basename))
    return bytes(result)

def write_manifest(root):
    paths = sorted(p for p in root.rglob('*') if p.is_file() and p.name != 'SHA256SUMS')
    (root / 'SHA256SUMS').write_text(''.join(f'{sha(p)}  {p.relative_to(root)}\n' for p in paths))

def verify_manifest(root):
    expected = {}
    for line in (root / 'SHA256SUMS').read_text().splitlines():
        match = re.fullmatch(r'([0-9a-f]{64})  ([A-Za-z0-9_./+-]+)', line)
        if not match: raise ValueError('invalid manifest line')
        digest, name = match.groups()
        if Path(name).is_absolute() or '..' in Path(name).parts or name in expected or name == 'SHA256SUMS':
            raise ValueError('unsafe/duplicate manifest path')
        path = root / name
        if path.is_symlink() or not path.is_file() or sha(path) != digest:
            raise ValueError(f'checksum/type mismatch: {name}')
        expected[name] = digest
    actual = set()
    for p in root.rglob('*'):
        if p.is_symlink(): raise ValueError('symlinks forbidden in payload')
        if p.is_file() and p.name != 'SHA256SUMS': actual.add(str(p.relative_to(root)))
    if not expected or actual != set(expected): raise ValueError('manifest file inventory mismatch')

def runtime_dependencies(elf_reports):
    # The device loader validates the transitive closure of its own vendor
    # library. An older SDK's transitive library names are not runtime facts.
    return sorted({name for report in elf_reports.values() for name in report['needed']}
                  - {'libkindlebt.so'})

def descriptor_bridge_source(source):
    """Add only the missing public descriptor callback forwarding at this pin."""
    include = '#include "kindlebt_callbacks.c"'
    entry = '    .on_ble_gattc_write_characteristics_cb = bleGattcWriteCharsCallbackWrapper,\n'
    if source.count(include) != 1 or source.count(entry) != 1:
        raise ValueError('pinned KindleBT callback anchors changed')
    return source.replace(include, include + '\n#include "kindlebt_descriptor_bridge.h"').replace(
        entry, entry + '    .on_ble_gattc_write_descriptor_cb = probe_descriptor_callback_bridge,\n')
