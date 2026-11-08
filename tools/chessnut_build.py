"""Pinned Chessnut worker build. Vendor SDK is build-only, never installed or shipped."""
import importlib.util
import json
from pathlib import Path
import posixpath
import re
import shutil
import subprocess
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = ROOT / 'support/chessnut-move'
DOWNLOADS = ROOT / 'target/chessnut-downloads'
OUTPUT = ROOT / 'target/kindle/chessnut'
PINS = json.loads((REFERENCE / 'BUILD_PINS.json').read_text())
SOURCES = {name: (pin['url'], pin['sha256']) for name, pin in PINS['sources'].items()}
spec = importlib.util.spec_from_file_location('chessnut_build_tools', REFERENCE / 'scripts/build_tools.py')
helpers = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helpers)
sha = helpers.sha

def run(args, **kwargs):
    return subprocess.run(list(map(str, args)), cwd=ROOT, check=True, **kwargs)

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
    return sources

def build(wrappers):
    OUTPUT.mkdir(parents=True, exist_ok=True)
    sources = prepare()
    cc = [wrappers / 'kindle-cc', '-O2', '-fno-omit-frame-pointer']
    includes = [sources['kindlebt'] / 'include', sources['logc'] / 'src', sources['darkroot'] / 'ace/sdk/include']
    if not includes[2].exists():
        includes[2] = next(sources['darkroot'].rglob('bluetooth_ble_api.h')).parents[1]
    args = [arg for inc in includes for arg in ['-I', inc]]
    # Exercise the real pinned header layouts on the build host, separately
    # from portable lifecycle tests. This executable is never staged.
    adapter_test = ROOT / 'target/native-adapter-test'
    run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-pthread'] + args
        + ['-I', REFERENCE / 'native', REFERENCE / 'tests/native_adapter_test.c', '-ldl', '-o', adapter_test])
    run([adapter_test])
    upstream = sources['kindlebt'] / 'src/kindlebt.c'
    if sha(upstream) != '950dc7fd601e8bdb4f2af46e66cd211c1cfcb249c137086ba4f350814167c7ea':
        raise ValueError('unexpected pinned KindleBT translation unit')
    generated = OUTPUT / 'kindlebt-descriptor.c'
    generated.write_text(helpers.descriptor_bridge_source(upstream.read_text()))
    native_sources = ['kindlebt.c', 'kindlebt_log.c', 'compat_ace.c', 'compat_ace_implementations.c',
                      'compat_ace_shims.c', 'compat_ace_utils.c', 'compat_ace_handler.c', 'compat_acealloc.c']
    run(cc + ['-shared', '-fPIC', '-pthread', '-Wl,-soname,libkindlebt.so', '-Wl,--allow-shlib-undefined'] + args
        + ['-I', REFERENCE / 'native', '-I', sources['kindlebt'] / 'src']
        + [generated if n == 'kindlebt.c' else sources['kindlebt'] / 'src' / n for n in native_sources]
        + [sources['logc'] / 'src/log.c', '-L', sources['sdk'], '-lace_bt', '-ldl', '-o', OUTPUT / 'libkindlebt.so'])
    library = OUTPUT / 'libkindlebt.so'
    library.write_bytes(helpers.localize_needed(library.read_bytes(), sources['sdk'] / 'libace_bt.so'))
    run(cc + ['-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-pie'] + args
        + [REFERENCE / 'native/kindlebt_worker.c', '-pthread', '-ldl', '-o', OUTPUT / 'kindlebt-worker'])

    reports = {name: helpers.inspect_elf((OUTPUT / name).read_bytes(), name == 'kindlebt-worker')
               for name in ('kindlebt-worker', 'libkindlebt.so')}
    allowed = {'libc.so.6', 'libpthread.so.0', 'libdl.so.2', 'libace_bt.so', 'ld-linux-armhf.so.3'}
    if any(set(report['needed']) - allowed for report in reports.values()):
        raise ValueError('unexpected worker dependency')
    required = helpers.runtime_dependencies(reports)
    (OUTPUT / 'chessnut-required-libs.txt').write_text(''.join(name+'\n' for name in required))
    artifacts = {name: sha(OUTPUT / name) for name in reports}
    artifacts['chessnut-required-libs.txt'] = sha(OUTPUT / 'chessnut-required-libs.txt')
    return {'artifacts': artifacts, 'elf': reports, 'sources': PINS['sources'],
            'kindlebt_generated_source_sha256': sha(generated),
            'descriptor_bridge_sha256': sha(REFERENCE / 'native/kindlebt_descriptor_bridge.h')}


def verify(folder, receipt):
    if set(receipt['artifacts']) != {'kindlebt-worker', 'libkindlebt.so', 'chessnut-required-libs.txt'}:
        raise ValueError('incomplete board worker bundle')
    for name, digest in receipt['artifacts'].items():
        path = folder / name
        if path.is_symlink() or sha(path) != digest:
            raise ValueError('board worker bundle checksum mismatch')
        if name != 'chessnut-required-libs.txt':
            report = helpers.inspect_elf(path.read_bytes(), name == 'kindlebt-worker')
            if report != receipt['elf'][name]: raise ValueError('board ELF receipt mismatch')
    required = helpers.runtime_dependencies(receipt['elf'])
    if (folder / 'chessnut-required-libs.txt').read_text() != ''.join(n+'\n' for n in required):
        raise ValueError('board dependency list mismatch')


def stage(runtime, receipt, folder=OUTPUT):
    verify(folder, receipt)
    (runtime / 'lib').mkdir()
    for name in receipt['artifacts']:
        shutil.copyfile(folder / name, runtime / ('lib/libkindlebt.so' if name == 'libkindlebt.so' else name))
    (runtime / 'kindlebt-worker').chmod(0o755)
    for notice in (REFERENCE / 'licenses').iterdir():
        shutil.copyfile(notice, runtime / 'licenses' / notice.name)


def verify_stage(runtime, receipt):
    # Staging moves the sole bundled DSO into a private library directory.
    for name, digest in receipt['artifacts'].items():
        path = runtime / ('lib/libkindlebt.so' if name == 'libkindlebt.so' else name)
        if path.is_symlink() or sha(path) != digest: raise ValueError('staged worker checksum mismatch')
        if name != 'chessnut-required-libs.txt':
            if helpers.inspect_elf(path.read_bytes(), name == 'kindlebt-worker') != receipt['elf'][name]:
                raise ValueError('staged worker ELF mismatch')
