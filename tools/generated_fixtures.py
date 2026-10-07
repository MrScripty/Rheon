"""Regenerate verifier inputs with the real Rust examples into ignored storage.

No committed simulation output, historical-data copy or synthetic replacement.
Caches are source/executable-bound. Partial directories are retained for diagnosis;
this module never removes unrelated files or silently resumes a failed example.
"""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
EXAMPLES = {'column_interface', 'column_mac', 'column_momentum', 'column_shear',
            'free_surface', 'liquid_step', 'viscosity'}


def run(command):
    return subprocess.run(command, cwd=ROOT, check=True, capture_output=True,
                          text=True, timeout=600).stdout


def fingerprint():
    paths = sorted((ROOT / 'src').rglob('*.rs')) + sorted((ROOT / 'examples').glob('*.rs'))
    paths += [ROOT / name for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml']]
    digest = hashlib.sha256()
    for path in paths:
        digest.update(str(path.relative_to(ROOT)).encode())
        digest.update(path.read_bytes())
    return digest.hexdigest()


def fixture(example):
    if example not in EXAMPLES:
        raise ValueError('unknown real-example fixture: ' + example)
    sources = fingerprint()
    parent = ROOT / '.generated/verifier-fixtures'
    directory = parent / (example + '-' + sources[:16])
    # Keep cache metadata outside the native output root: verifiers deliberately
    # reject extra files/cases there, and that inventory contract stays intact.
    marker = parent / (directory.name + '.json')
    if directory.exists():
        if not marker.exists():
            raise ValueError('partial fixture retained; move it aside before explicit regeneration: ' + str(directory))
        receipt = json.loads(marker.read_text())
        if receipt.get('source_sha256') != sources or receipt.get('example') != example:
            raise ValueError('fixture source identity mismatch')
        return directory
    parent.mkdir(parents=True, exist_ok=True)
    metadata = json.loads(run(['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1']))
    package = next(p for p in metadata['packages'] if p['manifest_path'] == str(ROOT / 'Cargo.toml'))
    command = ['cargo', 'build', '--locked', '--release', '--example', example, '--message-format=json']
    messages = [json.loads(line) for line in run(command).splitlines()]
    artifacts = [m for m in messages if m.get('reason') == 'compiler-artifact'
                 and m.get('package_id') == package['id'] and m.get('target', {}).get('name') == example
                 and m['target'].get('kind') == ['example'] and m.get('executable')]
    if len(artifacts) != 1:
        raise ValueError('one exact Cargo example executable required')
    executable = Path(artifacts[0]['executable'])
    executable_hash = hashlib.sha256(executable.read_bytes()).hexdigest()
    print('Regenerating real verifier fixture:', example, '->', directory, flush=True)
    run([str(executable), str(directory)])
    if fingerprint() != sources or hashlib.sha256(executable.read_bytes()).hexdigest() != executable_hash:
        raise ValueError('example/source changed during regeneration')
    receipt = {'example': example, 'source_sha256': sources, 'executable_sha256': executable_hash,
               'build_command': command, 'synthetic': False}
    with marker.open('x') as stream:
        json.dump(receipt, stream, indent=2)
        stream.write('\n')
    return directory


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('example', choices=sorted(EXAMPLES))
    print(fixture(parser.parse_args().example))
