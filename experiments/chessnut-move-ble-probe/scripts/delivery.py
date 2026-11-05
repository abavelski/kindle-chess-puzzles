#!/usr/bin/env python3
"""Independent, fail-closed ARM build/stage/deploy. Python 3.12+, stdlib only."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import posixpath
import re
import shlex
import shutil
import struct
import subprocess
import tarfile
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'target/kindle'
DOWNLOADS = ROOT / 'target/downloads'
TARGET = 'armv7-unknown-linux-gnueabihf'
INSTALL = '/mnt/us/chessnut-ble-probe'
KINDLEBT = '59e1d8e69a52e3cd58cee6fcc416d1774ad55791'
SOURCES = {
    'kindlebt': (f'https://codeload.github.com/Sighery/kindlebt/tar.gz/{KINDLEBT}', '88cc29547e150444900cbbbac7a27c7252f5537b916da85467f2fce26a841dc2'),
    'logc': ('https://codeload.github.com/rxi/log.c/tar.gz/f9ea34994bd58ed342d2245cd4110bb5c6790153', '0e1b70940986c16a8cb7ca322680b3c7614f769bc477c5b90e68a42aecc77ff6'),
    'darkroot': ('https://dl.espressif.com/dl/acs/acs_rtos_reference_ffs_1.15.0.tar.gz', 'e70a82ab242ce9d1412949bb035cd972b785b2627179aa934912fcd751b3b545'),
    'sdk': ('https://github.com/Sighery/kindle-sdk/releases/download/build-20260325/scribe1-5.17.3.tar.gz', '8d80e019cc64826822c002712bf44e5131e8f64853a647c0cbb8a2fea82b42dd'),
}
ZIG = {
    ('Darwin', 'arm64'): ('macos-aarch64', '46fae219656545dfaf4dce12fb4e8685cec5b51d721beee9389ab4194d43394c'),
    ('Darwin', 'x86_64'): ('macos-x86_64', '8b06ed1091b2269b700b3b07f8e3be3b833000841bae5aa6a09b1a8b4773effd'),
    ('Linux', 'x86_64'): ('linux-x86_64', 'd45312e61ebcc48032b77bc4cf7fd6915c11fa16e4aad116b66c9468211230ea'),
    ('Linux', 'aarch64'): ('linux-aarch64', '041ac42323837eb5624068acd8b00cd5777dac4cf91179e8dad7a7e90dd0c556'),
}

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def run(args, **kwargs):
    return subprocess.run(list(map(str, args)), cwd=ROOT, check=True, timeout=kwargs.pop('timeout', 600), **kwargs)

def source_files():
    return sorted(p for p in ROOT.rglob('*') if p.is_file() and not p.is_symlink()
                  and 'target' not in p.relative_to(ROOT).parts and 'stage' not in p.relative_to(ROOT).parts
                  and '__pycache__' not in p.parts and p.suffix != '.pyc')

def source_digest():
    h = hashlib.sha256()
    for p in source_files():
        h.update(str(p.relative_to(ROOT)).encode() + b'\0' + p.read_bytes() + b'\0')
    return h.hexdigest()

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

def fetch(name, url, digest):
    DOWNLOADS.mkdir(parents=True, exist_ok=True)
    archive = DOWNLOADS / name
    if not archive.exists() or sha(archive) != digest:
        temporary = archive.with_suffix('.partial')
        print(f'Downloading {name}', flush=True)
        with urllib.request.urlopen(url, timeout=60) as response, temporary.open('wb') as out:
            shutil.copyfileobj(response, out)
        if sha(temporary) != digest:
            temporary.unlink()
            raise ValueError(f'{name}: checksum mismatch')
        temporary.replace(archive)
    return archive

def extract(archive, folder):
    if folder.exists(): shutil.rmtree(folder)
    folder.mkdir(parents=True)
    with tarfile.open(archive) as tar:
        tar.extractall(folder, filter='data')
    return folder

def prepare():
    sources = {}
    for name, (url, digest) in SOURCES.items():
        archive = fetch(name+'.tar.gz', url, digest)
        if name == 'sdk':
            # Only regular sysroot library files, never execute Linux SDK tools
            # on a macOS host, and never extract its absolute sysroot symlinks.
            folder = ROOT / 'target/toolchains/sdk-libs'
            if folder.exists(): shutil.rmtree(folder)
            folder.mkdir(parents=True)
            entries = {}
            # Streaming extraction avoids repeatedly seeking/decompressing the
            # large SDK. Copy regular files only; resolve links as data later.
            with tarfile.open(archive, 'r|gz') as tar:
                for member in tar:
                    path = posixpath.normpath(member.name)
                    if '/sysroot/' not in path: continue
                    relative = path.split('/sysroot/', 1)[1]
                    if posixpath.dirname(relative) not in ('lib', 'usr/lib'): continue
                    if not re.fullmatch(r'.+\.so(?:\.[0-9]+)*', Path(relative).name): continue
                    if member.isfile():
                        dest = folder / 'reference' / relative
                        dest.parent.mkdir(parents=True, exist_ok=True)
                        dest.write_bytes(tar.extractfile(member).read())
                        entries[relative] = ('file', dest)
                    elif member.issym():
                        target = posixpath.normpath(member.linkname.lstrip('/') if member.linkname.startswith('/') else posixpath.join(posixpath.dirname(relative), member.linkname))
                        if target.startswith('../'): raise ValueError('SDK link escapes sysroot')
                        entries[relative] = ('link', target)
            # Firmware usr/lib precedes the GCC runtime overlay in lib.
            for relative in sorted(entries, key=lambda n: (n.startswith('usr/lib/'), n)):
                entry = relative
                for _ in range(16):
                    if entry not in entries: break
                    kind, value = entries[entry]
                    if kind == 'file':
                        shutil.copyfile(value, folder / Path(relative).name)
                        break
                    entry = value
            sources[name] = folder
        else:
            folder = extract(archive, ROOT / 'target/sources' / name)
            sources[name] = next(p for p in folder.iterdir() if p.is_dir())
    host = (platform.system(), platform.machine())
    if host not in ZIG: raise ValueError(f'unsupported host: {host}')
    name, digest = ZIG[host]
    archive = fetch(f'zig-{name}-0.13.0.tar.xz', f'https://ziglang.org/download/0.13.0/zig-{name}-0.13.0.tar.xz', digest)
    sources['zig'] = next(extract(archive, ROOT / 'target/toolchains/zig').iterdir()) / 'zig'
    return sources

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

def build():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / 'build.json').unlink(missing_ok=True)
    run(['sh', ROOT / 'scripts/check.sh'])
    before = source_digest()
    sources = prepare()
    zig = sources['zig']
    cc = [zig, 'cc', '-target', 'arm-linux-gnueabihf.2.35', '-O2', '-fno-omit-frame-pointer']
    includes = [sources['kindlebt'] / 'include', sources['logc'] / 'src', sources['darkroot'] / 'ace/sdk/include']
    if not includes[2].exists():
        includes[2] = next(sources['darkroot'].rglob('bluetooth_ble_api.h')).parents[1]
    args = [arg for inc in includes for arg in ['-I', inc]]
    # Exercise the real pinned header layouts on the build host, separately
    # from portable lifecycle tests. This executable is never staged.
    adapter_test = ROOT / 'target/native-adapter-test'
    run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-pthread'] + args
        + ['-I', ROOT / 'native', ROOT / 'tests/native_adapter_test.c', '-ldl', '-o', adapter_test])
    run([adapter_test])
    upstream = sources['kindlebt'] / 'src/kindlebt.c'
    if sha(upstream) != '950dc7fd601e8bdb4f2af46e66cd211c1cfcb249c137086ba4f350814167c7ea':
        raise ValueError('unexpected pinned KindleBT translation unit')
    generated = OUTPUT / 'kindlebt-descriptor.c'
    generated.write_text(descriptor_bridge_source(upstream.read_text()))
    native_sources = ['kindlebt.c', 'kindlebt_log.c', 'compat_ace.c', 'compat_ace_implementations.c',
                      'compat_ace_shims.c', 'compat_ace_utils.c', 'compat_ace_handler.c', 'compat_acealloc.c']
    run(cc + ['-shared', '-fPIC', '-pthread', '-Wl,-soname,libkindlebt.so', '-Wl,--allow-shlib-undefined'] + args
        + ['-I', ROOT / 'native', '-I', sources['kindlebt'] / 'src']
        + [generated if n == 'kindlebt.c' else sources['kindlebt'] / 'src' / n for n in native_sources]
        + [sources['logc'] / 'src/log.c', '-L', sources['sdk'], '-lace_bt', '-ldl', '-o', OUTPUT / 'libkindlebt.so'])
    library = OUTPUT / 'libkindlebt.so'
    library.write_bytes(localize_needed(library.read_bytes(), sources['sdk'] / 'libace_bt.so'))
    run(cc + ['-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-pie'] + args
        + [ROOT / 'native/kindlebt_worker.c', '-pthread', '-ldl', '-o', OUTPUT / 'kindlebt-worker'])
    linker = ROOT / 'target/toolchains/kindle-cc'
    linker.write_text('#!/bin/sh\nexec ' + shlex.quote(str(zig)) + ' cc -target arm-linux-gnueabihf.2.35 "$@"\n')
    linker.chmod(0o755)
    env = dict(os.environ, RUSTUP_TOOLCHAIN='1.85.1', RUSTFLAGS='', CARGO_ENCODED_RUSTFLAGS='',
               CARGO_TARGET_DIR=str(ROOT / 'target'),
               CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=str(linker))
    run(['cargo', 'build', '--locked', '--release', '--bin', 'chessnut-ble-probe', '--target', TARGET], env=env)
    shutil.copy2(ROOT / 'target' / TARGET / 'release/chessnut-ble-probe', OUTPUT / 'chessnut-ble-probe')
    elf = {}
    for name in ['chessnut-ble-probe', 'kindlebt-worker', 'libkindlebt.so']:
        elf[name] = inspect_elf((OUTPUT / name).read_bytes(), name != 'libkindlebt.so')
    # Build-only SDK closure for reference; runtime checks use actual artifact imports.
    required = set()
    pending = set(n for report in elf.values() for n in report['needed']) - {'libkindlebt.so'}
    while pending:
        name = pending.pop()
        if name in required: continue
        required.add(name)
        library = sources['sdk'] / name
        if not library.exists(): raise ValueError(f'SDK lacks required library {name}')
        report = inspect_elf(library.read_bytes(), False, device_reference=True)
        pending.update(set(report['needed']) - required)
    (OUTPUT / 'required-libs.txt').write_text(''.join(n+'\n' for n in runtime_dependencies(elf)))
    notices = OUTPUT / 'notices'
    notices.mkdir(exist_ok=True)
    shutil.copy2(sources['kindlebt'] / 'LICENSE', notices / 'KindleBT-MIT.txt')
    shutil.copy2(sources['logc'] / 'LICENSE', notices / 'logc-MIT.txt')
    # Preserve license text for the protocol crate included in the Rust binary.
    vendor = ROOT / 'target/license-vendor'
    if vendor.exists(): shutil.rmtree(vendor)
    run(['cargo', 'vendor', '--locked', '--versioned-dirs', str(vendor)], stdout=subprocess.DEVNULL)
    for package in vendor.iterdir():
        for license_file in package.glob('*'):
            if license_file.is_file() and license_file.name.lower().startswith(('license', 'copying', 'notice')):
                shutil.copy2(license_file, notices / (package.name + '-' + license_file.name))
    (notices / 'README.txt').write_text('KindleBT/log.c: MIT. Rust dependency license texts included.\nBundled KindleBT includes a local public write-descriptor callback forwarding bridge.\nAmazon SDK libraries and vendor headers are build references only and are not distributed.\nProject code has no declared distribution license; this is a local-install experiment.\n')
    if source_digest() != before: raise ValueError('source changed during build; rebuild')
    receipt = {'milestone': 4, 'source_sha256': before, 'kindlebt_commit': KINDLEBT,
               'kindlebt_local_changes': ['forward public write-descriptor callback'],
               'kindlebt_generated_source_sha256': sha(generated),
               'kindlebt_descriptor_bridge_sha256': sha(ROOT / 'native/kindlebt_descriptor_bridge.h'),
               'target': TARGET, 'rust': '1.85.1', 'zig': '0.13.0', 'glibc_max': '2.35',
               'host': {'system': platform.system(), 'machine': platform.machine()},
               'zig_archive': ZIG[(platform.system(), platform.machine())],
               'native_flags': ['-target arm-linux-gnueabihf.2.35', '-O2', '-fno-omit-frame-pointer'],
               'kindlebt_link': ['-shared', '-fPIC', '-pthread', '-soname=libkindlebt.so', '--allow-shlib-undefined', '-lace_bt', '-ldl'],
               'worker_flags': ['-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-pie', '-pthread', '-ldl'],
               'cargo': ['build', '--locked', '--release', '--bin', 'chessnut-ble-probe', '--target', TARGET],
               'sources': SOURCES, 'elf': elf, 'device_libraries': runtime_dependencies(elf),
               'sdk_reference_libraries': sorted(required), 'host_checks': 'PASS',
               'artifacts': {n: sha(OUTPUT / n) for n in elf}}
    (OUTPUT / 'build.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print('Build verified:', OUTPUT)

def stage():
    receipt = json.loads((OUTPUT / 'build.json').read_text())
    if receipt['source_sha256'] != source_digest(): raise ValueError('stale build receipt; rebuild')
    folder = OUTPUT / 'stage'
    if folder.exists(): shutil.rmtree(folder)
    runtime = folder / 'runtime'
    runtime.mkdir(parents=True)
    for name, digest in receipt['artifacts'].items():
        if sha(OUTPUT / name) != digest: raise ValueError(f'stale binary {name}')
        inspect_elf((OUTPUT / name).read_bytes(), name != 'libkindlebt.so')
        shutil.copy2(OUTPUT / name, runtime / name)
    for name in ['build.json', 'required-libs.txt']: shutil.copy2(OUTPUT / name, runtime / name)
    shutil.copytree(OUTPUT / 'notices', runtime / 'notices')
    for name in ['launch.sh', 'uninstall-device.sh', 'read-board.sh', 'move-board.sh']: shutil.copy2(ROOT / 'scripts' / name, runtime / name)
    shutil.copy2(ROOT / 'scripts/install-device.sh', folder / 'install.sh')
    write_manifest(runtime)
    write_manifest(folder)
    verify_stage(folder)
    archive = OUTPUT / 'chessnut-ble-probe.tar'
    with tarfile.open(archive, 'w') as tar:
        for path in sorted(folder.rglob('*')):
            if path.is_file(): tar.add(path, arcname=str(path.relative_to(folder)), recursive=False)
    print('Stage verified:', folder)

def verify_stage(folder):
    verify_manifest(folder)
    runtime = folder / 'runtime'
    verify_manifest(runtime)
    receipt = json.loads((runtime / 'build.json').read_text())
    if receipt['target'] != TARGET or receipt['kindlebt_commit'] != KINDLEBT or receipt['host_checks'] != 'PASS':
        raise ValueError('invalid build receipt')
    if set(receipt['artifacts']) != {'chessnut-ble-probe', 'kindlebt-worker', 'libkindlebt.so'}:
        raise ValueError('unexpected artifact receipt inventory')
    for name, digest in receipt['artifacts'].items():
        if sha(runtime / name) != digest: raise ValueError('artifact receipt mismatch')
        if inspect_elf((runtime / name).read_bytes(), name != 'libkindlebt.so') != receipt['elf'][name]:
            raise ValueError('ELF receipt mismatch')
    if (runtime / 'required-libs.txt').read_text().splitlines() != receipt['device_libraries']:
        raise ValueError('device dependency receipt mismatch')
    for name in ['launch.sh', 'uninstall-device.sh', 'read-board.sh', 'move-board.sh']:
        if sha(runtime / name) != sha(ROOT / 'scripts' / name): raise ValueError('stale runtime script')
    if sha(folder / 'install.sh') != sha(ROOT / 'scripts/install-device.sh'): raise ValueError('stale installer')
    allowed = {'chessnut-ble-probe', 'kindlebt-worker', 'libkindlebt.so', 'build.json', 'required-libs.txt',
               'launch.sh', 'uninstall-device.sh', 'read-board.sh', 'move-board.sh', 'SHA256SUMS'}
    if any(p.name not in allowed and p.name != 'notices' for p in runtime.iterdir()):
        raise ValueError('unexpected runtime payload')
    if any(p.name not in {'runtime', 'install.sh', 'SHA256SUMS'} for p in folder.iterdir()):
        raise ValueError('unexpected stage payload')

def remote_options(args):
    if not re.fullmatch(r'(?:[A-Za-z0-9_][A-Za-z0-9_.-]*@)?[A-Za-z0-9][A-Za-z0-9_.-]*', args.host):
        raise ValueError('host must be a hostname or IPv4, optionally user@host')
    if not 1 <= args.port <= 65535: raise ValueError('invalid SSH port')
    return ['-o', 'ConnectTimeout=15', '-o', 'ServerAliveInterval=15', '-o', 'ServerAliveCountMax=2']

def deploy(args):
    folder = OUTPUT / 'stage'
    verify_stage(folder)
    if json.loads((folder / 'runtime/build.json').read_text())['source_sha256'] != source_digest():
        raise ValueError('stage does not match current sources; rebuild and restage')
    token = uuid.uuid4().hex
    incoming = INSTALL + '/incoming-' + token
    ssh = ['ssh'] + remote_options(args) + ['-p', str(args.port), args.host]
    # No persistent changes outside the dedicated experiment install directory.
    setup = f'''set -eu
root={INSTALL}
[ ! -L "$root" ]
if [ -d "$root" ]; then
  [ -f "$root/.probe-owner" ] && [ "$(cat "$root/.probe-owner")" = chessnut-ble-probe-v1 ]
else
  mkdir "$root"
  printf '%s\\n' chessnut-ble-probe-v1 > "$root/.probe-owner"
fi
mkdir {incoming}
'''
    run(ssh + ['sh -s'], input=setup, text=True, timeout=90)
    try:
        # Individual payload upload avoids remote extraction of an unverified tar.
        payload = [str(folder / name) for name in ['install.sh', 'SHA256SUMS', 'runtime']]
        run(['scp', '-O'] + remote_options(args) + ['-P', str(args.port), '-r'] + payload + [args.host + ':' + incoming + '/'], timeout=180)
        script = f'cd {incoming}\nsha256sum -c SHA256SUMS\nsh ./install.sh\n'
        run(ssh + ['sh -se'], input=script, text=True, timeout=120)
    finally:
        try:
            run(ssh + ['sh -s'], input=f'rm -rf -- {incoming}\n', text=True, timeout=60)
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
            print('Upload cleanup incomplete; inspect the experiment incoming directory through SSH.')
    print('Installed only; NOT launched. Run preflight, then stop for human H4 motor verification with the board.')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['build', 'stage', 'verify-stage', 'deploy', 'uninstall'])
    parser.add_argument('--host')
    parser.add_argument('--port', type=int, default=2222)
    args = parser.parse_args()
    if args.action == 'build': build()
    elif args.action == 'stage': stage()
    elif args.action == 'verify-stage': verify_stage(OUTPUT / 'stage'); print('Stage PASS')
    elif args.action == 'deploy':
        if not args.host: parser.error('--host required')
        deploy(args)
    else:
        if not args.host: parser.error('--host required')
        run(['ssh'] + remote_options(args) + ['-p', str(args.port), args.host, 'sh', INSTALL + '/runtime/uninstall-device.sh'], timeout=90)
if __name__ == '__main__': main()
