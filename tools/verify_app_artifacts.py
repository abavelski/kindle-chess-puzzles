#!/usr/bin/env python3
"""Fail CI before publishing incomplete, substituted or contaminated app artifacts."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import tarfile

ROOT = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location('artifact_build', ROOT / 'tools/kindle_build.py')
build = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(build)


def verify_artifacts(app, directory=None):
    product = build.product(app)
    directory = Path(directory) if directory is not None else build.output_dir(app)
    binary = directory / product['binary']
    receipt_path = directory / 'build.json'
    receipt = json.loads(receipt_path.read_text())
    build.validate_receipt(receipt, app)
    for key, expected in {'checks': 'passed', 'target': build.TARGET, 'rust': '1.85.1',
                          'zig': build.ZIG_VERSION, 'glibc': '2.35', 'fbink': build.FBINK,
                          'source_sha256': build.source_digest(app),
                          'binary_sha256': build.sha256(binary)}.items():
        if receipt.get(key) != expected:
            raise ValueError(f'{app} receipt mismatch: {key}')
    build.validate_product_elf(binary, app)
    stage = directory / 'stage'
    build.verify_stage(stage, app=app)
    if (stage / 'runtime/build.json').read_bytes() != receipt_path.read_bytes():
        raise ValueError('staged receipt differs from selected build')
    if build.sha256(stage / 'runtime' / product['binary']) != receipt['binary_sha256']:
        raise ValueError('staged binary differs from selected build')
    if json.loads((stage / 'manifest.json').read_text()) != json.loads((ROOT / product['manifest']).read_text()):
        raise ValueError('staged manifest differs from selected product')
    if app in ('review', 'play'):
        label = 'Review' if app == 'review' else 'Play'
        for path in (stage / 'runtime').rglob('*'):
            if path.name.startswith(('puzzles', 'games', 'kindlebt', 'libkindlebt', 'chessnut-')):
                raise ValueError(f'{label} includes puzzle/board runtime assets')
        if b'promotion-white-queen' in binary.read_bytes():
            raise ValueError(f'{label} includes bundled puzzle examples')
    expected_files = {str(path.relative_to(stage)): build.sha256(path)
                      for path in stage.rglob('*') if path.is_file()}
    result = {'app': app, 'binary_sha256': receipt['binary_sha256'],
              'source_sha256': receipt['source_sha256'], 'binary_bytes': binary.stat().st_size}
    for kind, name in [('tar', product['binary'] + '.tar'),
                       ('kpkg', product['package_id'] + '_0.1.0_kindlehf.kpkg')]:
        archive = directory / name
        with tarfile.open(archive, 'r:*') as packed:
            members = packed.getmembers()
            if len(members) != len(expected_files) or {m.name for m in members} != set(expected_files):
                raise ValueError(f'{app} {kind} file set differs from stage')
            for member in members:
                if not member.isfile():
                    raise ValueError('archive contains a non-file member')
                data = packed.extractfile(member).read()
                if hashlib.sha256(data).hexdigest() != expected_files[member.name]:
                    raise ValueError(f'{app} {kind} payload differs from stage: {member.name}')
        result[kind + '_sha256'] = build.sha256(archive)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', choices=list(build.PRODUCTS), required=True)
    parser.add_argument('--directory', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify_artifacts(args.app, args.directory), sort_keys=True))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, tarfile.TarError) as error:
        raise SystemExit(f'artifact verification: {error}') from error
