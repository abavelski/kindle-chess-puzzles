#!/usr/bin/env python3
"""Pinned build, verified stage and scp deployment. Requires only Python stdlib."""
import argparse
import importlib.util
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import struct
import subprocess
import tarfile
import tempfile
import tomllib
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[1]
TARGET = 'armv7-unknown-linux-gnueabihf'
FBINK = '92e127008145b2a22fba7c59815d810d716310dd'
# All tracked sources, recursively including the pinned FBInk submodules.
FBINK_SOURCE_SHA256 = '57d05ce7d8b5b4621e4bb17d547daf9980a89f16a7e97217f1a7c33e488cb49c'
ZIG_VERSION = '0.13.0'
ZIG_HASHES = {
    ('Darwin', 'arm64'): ('macos-aarch64', '46fae219656545dfaf4dce12fb4e8685cec5b51d721beee9389ab4194d43394c'),
    ('Darwin', 'x86_64'): ('macos-x86_64', '8b06ed1091b2269b700b3b07f8e3be3b833000841bae5aa6a09b1a8b4773effd'),
    ('Linux', 'x86_64'): ('linux-x86_64', 'd45312e61ebcc48032b77bc4cf7fd6915c11fa16e4aad116b66c9468211230ea'),
    ('Linux', 'aarch64'): ('linux-aarch64', '041ac42323837eb5624068acd8b00cd5777dac4cf91179e8dad7a7e90dd0c556'),
}
BUILD = ROOT / 'target/kindle'

PRODUCTS = {
    'puzzles': {'cargo': 'kindle-puzzles', 'cargo_binary': 'kindle-puzzles',
                'binary': 'kindle-chess', 'package_id': 'kindle_chess',
                'runtime_root': '/mnt/us/kindle-chess', 'scriptlet': 'kindle-chess.sh',
                'manifest': 'packaging/kindle/manifest.json'},
    'review': {'cargo': 'kindle-game-review', 'cargo_binary': 'kindle-game-review',
               'binary': 'kindle-game-review', 'package_id': 'kindle_game_review',
               'runtime_root': '/mnt/us/kindle-game-review', 'scriptlet': 'kindle-game-review.sh',
               'manifest': 'packaging/kindle-review/manifest.json'},
}

def product(app):
    if app not in PRODUCTS:
        raise ValueError('unknown product')
    return PRODUCTS[app]

def output_dir(app):
    product(app)
    return BUILD / app

def validate_product_elf(path, app):
    validate_elf(path)
    data = Path(path).read_bytes()
    for candidate in PRODUCTS:
        marker = f'KINDLE_CHESS_PRODUCT:{candidate}:v1'.encode()
        if (marker in data) != (candidate == app):
            raise ValueError('ELF product identity does not match selected app')

def validate_receipt(receipt, app):
    spec = product(app)
    expected = {'app': app, **{key: spec[key] for key in ('binary', 'package_id', 'runtime_root')}}
    if any(receipt.get(key) != value for key, value in expected.items()):
        raise ValueError('receipt product identity mismatch')
    if (app == 'review') and 'chessnut' in receipt:
        raise ValueError('Review receipt must not include a board worker')


def board_build():
    spec = importlib.util.spec_from_file_location('chessnut_build', ROOT / 'tools/chessnut_build.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module



def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(args, **kwargs):
    return subprocess.run([str(x) for x in args], cwd=ROOT, check=True, **kwargs)


def source_files():
    # Include local changes, including new task files, but no generated build output.
    directories = ['support/chessnut-move/src', 'support/chessnut-move/native', 'support/chessnut-move/scripts', 'support/chessnut-move/licenses', 'support/chessnut-move/tests', 'app', 'apps', 'crates', 'assets', 'scripts', 'tools', 'tests', 'packaging', '.github']
    paths = [ROOT / name for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'LICENSE', 'support/chessnut-move/Cargo.toml', 'support/chessnut-move/Cargo.lock', 'support/chessnut-move/BUILD_PINS.json', 'support/chessnut-move/SOURCE_PINS.json']]
    for directory in directories:
        paths.extend(p for p in (ROOT / directory).rglob('*')
                     if p.is_file() and '__pycache__' not in p.parts and p.suffix != '.pyc')
    return sorted(p for p in paths if p.exists())


def source_digest(app='puzzles'):
    spec = product(app)
    digest = hashlib.sha256()
    for path in source_files():
        digest.update(str(path.relative_to(ROOT)).encode() + b'\0' + path.read_bytes() + b'\0')
    # Verify pinned vendor separately, bind the revision into the receipt.
    digest.update(FBINK.encode())
    digest.update(json.dumps({"app": app, **spec}, sort_keys=True).encode())
    return digest.hexdigest()


def validate_elf(path):
    data = Path(path).read_bytes()
    if len(data) < 52 or data[:7] != b'\x7fELF\x01\x01\x01':
        raise ValueError('binary must be ELF32 little-endian')
    if struct.unpack_from('<H', data, 18)[0] != 40:
        raise ValueError('binary must target ARM')
    flags = struct.unpack_from('<I', data, 36)[0]
    if flags & 0xff000000 != 0x05000000 or not flags & 0x400 or flags & 0x200:
        raise ValueError('binary must use EABI5 hard-float')
    if b'/lib/ld-linux-armhf.so.3\0' not in data:
        raise ValueError('expected Scribe hard-float dynamic loader')
    versions = re.findall(rb'GLIBC_(\d+)\.(\d+)(?:\.(\d+))?', data)
    if not versions or any(tuple(int(v or 0) for v in version) > (2, 35, 0) for version in versions):
        raise ValueError('binary requires unsupported or unverified glibc (Scribe baseline 2.35)')


def zig_tools():
    key = (platform.system(), platform.machine())
    if key not in ZIG_HASHES:
        raise ValueError(f'unsupported build host: {key}')
    name, checksum = ZIG_HASHES[key]
    folder = f'zig-{name}-{ZIG_VERSION}'
    cache = ROOT / 'target/toolchains'
    cache.mkdir(parents=True, exist_ok=True)
    archive_path = cache / f'{folder}.tar.xz'
    if not archive_path.exists() or sha256(archive_path) != checksum:
        temporary = archive_path.with_suffix('.download')
        print(f'Downloading verified Zig {ZIG_VERSION}', flush=True)
        with urllib.request.urlopen(f'https://ziglang.org/download/{ZIG_VERSION}/{folder}.tar.xz', timeout=60) as response:
            with temporary.open('wb') as output:
                shutil.copyfileobj(response, output)
        if sha256(temporary) != checksum:
            temporary.unlink()
            raise ValueError('Zig archive checksum mismatch')
        temporary.replace(archive_path)
    destination = cache / folder
    # Extract anew from the verified archive, so cache edits cannot change the build.
    if destination.exists():
        shutil.rmtree(destination)
    with tarfile.open(archive_path) as archive_file:
        for member in archive_file.getmembers():
            if member.name.startswith('/') or '..' in Path(member.name).parts:
                raise ValueError('unsafe Zig archive path')
        archive_file.extractall(cache, filter='data')
    wrappers = cache / 'bin'
    wrappers.mkdir(exist_ok=True)
    for tool, arguments in [('cc', 'cc -target arm-linux-gnueabihf.2.35'), ('ar', 'ar'), ('ranlib', 'ranlib')]:
        wrapper = wrappers / f'kindle-{tool}'
        wrapper.write_text(f'#!/bin/sh\nexec {shlex.quote(str(destination / "zig"))} {arguments} "$@"\n')
        wrapper.chmod(0o755)
    return wrappers


def vendor_files():
    vendor = ROOT / 'vendor/FBInk'
    if (vendor / '.git').exists():
        names = run(['git', '-C', vendor, 'ls-files', '--recurse-submodules', '-z'],
                    capture_output=True, text=True).stdout.rstrip('\0').split('\0')
        return sorted((vendor / name for name in names), key=str)
    manifest = vendor / '.kcp-source-files.json'
    names = json.loads(manifest.read_text())
    if any(Path(name).is_absolute() or '..' in Path(name).parts for name in names) or len(names) != len(set(names)):
        raise ValueError('unsafe FBInk source manifest')
    return sorted((vendor / name for name in names), key=str)


def verify_vendor():
    vendor = ROOT / 'vendor/FBInk'
    digest = hashlib.sha256()
    for path in vendor_files():
        digest.update(str(path.relative_to(vendor)).encode() + b'\0' + path.read_bytes() + b'\0')
    if digest.hexdigest() != FBINK_SOURCE_SHA256:
        raise ValueError('FBInk sources must match the pinned revision and nested submodules')
    if (vendor / '.git').exists():
        revision = run(['git', '-C', vendor, 'rev-parse', 'HEAD'], capture_output=True, text=True).stdout.strip()
        dirty = run(['git', '-C', vendor, 'status', '--porcelain', '--untracked-files=no'], capture_output=True, text=True).stdout
        if revision != FBINK or dirty:
            raise ValueError('FBInk must be clean at pinned revision; initialize submodules')


def build(app='puzzles'):
    spec = product(app)
    destination = output_dir(app)
    verify_vendor()
    # No receipt survives a failed build or failed checks.
    destination.mkdir(parents=True, exist_ok=True)
    receipt = destination / 'build.json'
    receipt.unlink(missing_ok=True)
    run(['sh', 'scripts/check.sh'])
    run(['python3', 'scripts/generate_sashite.py'])
    before = source_digest(app)
    wrappers = zig_tools()
    chessnut = board_build().build(wrappers) if app == 'puzzles' else None
    environment = dict(os.environ)
    # Keep Cargo artifacts in a controlled directory regardless of host settings.
    environment['CARGO_TARGET_DIR'] = str(ROOT / 'target')
    for variable, tool in [('KINDLE_CC', 'cc'), ('KINDLE_AR', 'ar'), ('KINDLE_RANLIB', 'ranlib'),
                           ('CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER', 'cc')]:
        environment[variable] = str(wrappers / f'kindle-{tool}')
    run(['cargo', 'build', '--locked', '--release', '-p', spec['cargo'], '--target', TARGET], env=environment)
    binary = ROOT / 'target' / TARGET / 'release' / spec['cargo_binary']
    validate_product_elf(binary, app)
    if source_digest(app) != before:
        raise ValueError('source changed during build; repeat build')
    shutil.copyfile(binary, destination / spec['binary'])
    if chessnut is not None:
        bundle = destination / 'chessnut'
        if bundle.exists(): shutil.rmtree(bundle)
        shutil.copytree(BUILD / 'chessnut', bundle)
    receipt_data = {'app': app, **{key: spec[key] for key in ('binary', 'package_id', 'runtime_root')},
                    'binary_sha256': sha256(binary), 'source_sha256': before,
                    'target': TARGET, 'checks': 'passed', 'rust': '1.85.1',
                    'zig': ZIG_VERSION, 'glibc': '2.35', 'fbink': FBINK}
    if chessnut is not None: receipt_data['chessnut'] = chessnut
    receipt.write_text(json.dumps(receipt_data, indent=2) + '\n')
    print(f"Validated {app}: {destination / spec['binary']} ({binary.stat().st_size} bytes)")


def stage(binary, receipt_path, destination, app='puzzles'):
    spec = product(app)
    receipt = json.loads(Path(receipt_path).read_text())
    validate_receipt(receipt, app)
    if (receipt.get('checks') != 'passed' or receipt.get('target') != TARGET
            or receipt.get('binary_sha256') != sha256(binary)
            or receipt.get('source_sha256') != source_digest(app)):
        raise ValueError('build receipt does not match tested source/binary; run build-kindle.sh')
    validate_product_elf(binary, app)
    verify_vendor()
    destination = Path(destination)
    if destination.is_symlink() or (destination.exists() and not (destination / 'SHA256SUMS').is_file()):
        raise ValueError('refusing to replace a directory that is not an app-owned stage')
    if destination.exists():
        verify_stage(destination, app=app)
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=destination.parent) as temporary:
        contents = Path(temporary)
        runtime = contents / 'runtime'
        runtime.mkdir()
        shutil.copyfile(binary, runtime / spec['binary'])
        shutil.copyfile(receipt_path, runtime / 'build.json')
        shutil.copyfile(ROOT / 'scripts/kindle_launch.sh', runtime / 'kindle_launch.sh')
        for name in ['launch.sh', 'install.sh', 'uninstall.sh', 'kindle-chess.sh', 'package-launch.sh']:
            target = (runtime / name if name == 'launch.sh' else
                      contents / ('launch.sh' if name == 'package-launch.sh' else
                                  spec['scriptlet'] if name == 'kindle-chess.sh' else name))
            text = (ROOT / 'packaging/kindle' / name).read_text()
            if app == 'review':
                text = text.replace('kindle-chess', 'kindle-game-review').replace('Kindle Chess Puzzles', 'Kindle Game Review')
                if name in ('launch.sh', 'install.sh', 'uninstall.sh'):
                    text = '#!/bin/sh\necho "Review install/launch awaits SPLIT-11" >&2\nexit 1\n'
            target.write_text(text)
        shutil.copyfile(ROOT / spec['manifest'], contents / 'manifest.json')
        notices = runtime / 'licenses'
        notices.mkdir()
        for path, name in [(ROOT / 'vendor/FBInk/LICENSE', 'FBInk-GPL-3.0.txt'),
                           (ROOT / 'assets/sashite-western/README.md', 'Sashite.md'),
                           (ROOT / 'packaging/kindle/NOTICE.md', 'NOTICE.md'),
                           (ROOT / 'docs/PROVENANCE.md', 'PROVENANCE.md'),
                           (ROOT / 'assets/fonts/OFL.txt', 'Atkinson-OFL.txt'),
                           (ROOT / 'assets/fonts/NotoSans-OFL.txt', 'NotoSans-OFL.txt')]:
            shutil.copyfile(path, notices / name)
        if app == 'review':
            notice = notices / 'NOTICE.md'
            notice.write_text(notice.read_text().split('\nThe optional Chessnut Move integration')[0] + '\n')
        if app == 'puzzles':
            if 'chessnut' not in receipt:
                raise ValueError('build receipt lacks the board worker bundle; rebuild')
            board_build().stage(runtime, receipt['chessnut'], Path(binary).parent / 'chessnut')
        metadata = json.loads(run(['cargo', 'metadata', '--locked', '--format-version=1'],
                                  capture_output=True, text=True).stdout)
        dependencies = [package for package in metadata['packages'] if package['source']]
        # Project, dependency and pinned FBInk sources accompany the executable.
        with tarfile.open(runtime / 'source.tar', 'w', format=tarfile.USTAR_FORMAT) as source:
            pinned_files = vendor_files()
            paths = source_files() + pinned_files
            paths += [ROOT / 'README.md', ROOT / 'AGENTS.md', ROOT / '.gitmodules']
            paths += sorted(p for p in (ROOT / 'docs').rglob('*') if p.is_file())
            for path in sorted(paths):
                add_tar_file(source, path, str(path.relative_to(ROOT)))
            data = json.dumps([str(path.relative_to(ROOT / 'vendor/FBInk')) for path in pinned_files]).encode()
            manifest_info = tarfile.TarInfo('vendor/FBInk/.kcp-source-files.json')
            manifest_info.size = len(data)
            manifest_info.mode = 0o644
            source.addfile(manifest_info, io.BytesIO(data))
            lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
            checksums = {(p['name'], p['version']): p['checksum'] for p in lock['package'] if 'checksum' in p}
            for package in sorted(dependencies, key=lambda item: item['name']):
                directory = Path(package['manifest_path']).parent
                files = {}
                for path in sorted(directory.rglob('*')):
                    if path.is_file() and path.name not in ['.cargo-checksum.json', '.cargo-ok']:
                        files[str(path.relative_to(directory))] = sha256(path)
                        add_tar_file(source, path, f'vendor/rust/{directory.name}/{path.relative_to(directory)}')
                data = json.dumps({'files': files, 'package': checksums[(package['name'], package['version'])]}, sort_keys=True).encode()
                checksum_info = tarfile.TarInfo(f'vendor/rust/{directory.name}/.cargo-checksum.json')
                checksum_info.size = len(data)
                checksum_info.mode = 0o644
                source.addfile(checksum_info, io.BytesIO(data))
            config = tarfile.TarInfo('.cargo/config.toml')
            data = b'[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = "vendor/rust"\n'
            config.size = len(data)
            config.mode = 0o644
            source.addfile(config, io.BytesIO(data))
        for path in contents.rglob('*'):
            if path.is_file():
                path.chmod(0o755 if path.suffix == '.sh' or path.name in ('kindle-chess', 'kindle-game-review', 'kindlebt-worker') else 0o644)
        sums = ''.join(f'{sha256(p)}  {p.relative_to(contents)}\n'
                       for p in sorted(contents.rglob('*')) if p.is_file())
        (contents / 'SHA256SUMS').write_text(sums)
        if destination.exists():
            shutil.rmtree(destination)
        shutil.copytree(contents, destination)
    print(f'Staged {destination}')


def verify_stage(directory, app=None):
    directory = Path(directory)
    receipt = json.loads((directory / 'runtime/build.json').read_text())
    app = receipt.get('app') if app is None else app
    spec = product(app)
    validate_receipt(receipt, app)
    expected = {}
    for line in (directory / 'SHA256SUMS').read_text().splitlines():
        checksum, name = line.split('  ', 1)
        if Path(name).is_absolute() or '..' in Path(name).parts or name in expected:
            raise ValueError('invalid staged manifest')
        expected[name] = checksum
    files = {str(p.relative_to(directory)) for p in directory.rglob('*') if p.is_file()}
    if files != set(expected) | {'SHA256SUMS'}:
        raise ValueError('staged file set changed')
    for name, checksum in expected.items():
        path = directory / name
        if path.is_symlink() or sha256(path) != checksum:
            raise ValueError(f'staged checksum mismatch: {name}')
    validate_product_elf(directory / 'runtime' / spec['binary'], app)
    manifest = json.loads((directory / 'manifest.json').read_text())
    if manifest.get('id') != spec['package_id']:
        raise ValueError('stage manifest product mismatch')
    if app == 'review' and any((directory / 'runtime' / name).exists() for name in ('kindle-chess', 'kindlebt-worker', 'lib/libkindlebt.so')):
        raise ValueError('Review stage contains puzzle/board assets')
    # Previously tested stable stages remain valid rollback packages. New stages
    # always require the board bundle in stage() above.
    if 'chessnut' in receipt:
        board_build().verify_stage(directory / 'runtime', receipt['chessnut'])
    if receipt.get('checks') != 'passed' or receipt.get('binary_sha256') != sha256(directory / 'runtime' / spec['binary']):
        raise ValueError('stage has no valid tested-build receipt')


def add_tar_file(archive_file, path, name):
    data = path.read_bytes()
    info = tarfile.TarInfo(name)
    info.size = len(data)
    info.mode = 0o755 if path.suffix == '.sh' or path.name in ('kindle-chess', 'kindle-game-review', 'kindlebt-worker') else 0o644
    info.mtime = 0
    archive_file.addfile(info, io.BytesIO(data))


def archive(directory, output):
    verify_stage(directory)
    with tarfile.open(output, 'w', format=tarfile.USTAR_FORMAT) as archive_file:
        for path in sorted(Path(directory).rglob('*')):
            if path.is_file():
                add_tar_file(archive_file, path, str(path.relative_to(directory)))


def kpkg(directory, output):
    # KPM manifest v2 uses gzip-compressed tar; normalize gzip/tar metadata.
    verify_stage(directory)
    with Path(output).open('wb') as file:
        with gzip.GzipFile(filename='', mode='wb', fileobj=file, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode='w', format=tarfile.USTAR_FORMAT) as package:
                for path in sorted(Path(directory).rglob('*')):
                    if path.is_file():
                        add_tar_file(package, path, str(path.relative_to(directory)))


def deploy(directory, host, port, app='puzzles'):
    if app == 'review':
        raise ValueError('Review deployment awaits SPLIT-11')
    if not re.fullmatch(r'(?:[a-zA-Z0-9_]+@)?[a-zA-Z0-9][a-zA-Z0-9.-]*', host) or not 1 <= port <= 65535:
        raise ValueError('use user@hostname or user@IPv4 and a valid SSH port')
    verify_stage(directory)
    token = uuid.uuid4().hex
    remote_tar = f'/mnt/us/kindle-chess-upload-{token}.tar'
    incoming = f'/mnt/us/kindle-chess-incoming-{token}'
    with tempfile.TemporaryDirectory() as temporary:
        tar_path = Path(temporary) / 'install.tar'
        archive(directory, tar_path)
        run(['scp', '-O', '-P', str(port), tar_path, f'{host}:{remote_tar}'])
        # Fixed paths + generated hex token only. No user text in remote shell code.
        command = (f"set -eu; trap 'rm -rf {incoming}; rm -f {remote_tar}' EXIT; "
                   f'mkdir {incoming}; cd {incoming}; tar xf {remote_tar}; '
                   'sh ./install.sh /mnt/us/kindle-chess')
        run(['ssh', '-p', str(port), host, command])
    print('Runtime and library scriptlet installed; user data preserved.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['build', 'stage', 'deploy'])
    parser.add_argument('--app', choices=list(PRODUCTS), default='puzzles')
    parser.add_argument('--stage-dir', type=Path)
    parser.add_argument('--host')
    parser.add_argument('--port', type=int, default=2222)
    args = parser.parse_args()
    spec = product(args.app)
    destination = output_dir(args.app)
    stage_dir = args.stage_dir or destination / 'stage'
    if args.command == 'build':
        build() if args.app == 'puzzles' else build(args.app)
    elif args.command == 'stage':
        stage(destination / spec['binary'], destination / 'build.json', stage_dir, app=args.app)
        archive(stage_dir, destination / f"{spec['binary']}.tar")
        kpkg(stage_dir, destination / f"{spec['package_id']}_0.1.0_kindlehf.kpkg")
    elif args.command == 'deploy':
        if args.app == 'review': parser.error('Review deployment awaits SPLIT-11')
        if not args.host: parser.error('deploy requires --host user@host')
        deploy(stage_dir, args.host, args.port)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f'kindle-build: {error}') from error
