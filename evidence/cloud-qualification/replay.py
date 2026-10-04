#!/usr/bin/env python3
"""Check every original demo's deterministic outputs against frozen fixtures.

Usage from repository root: python3 evidence/cloud-qualification/replay.py BINARY FRESH_OUTPUT
Completion manifests contain timing and new schema fields, so their whole-file
bytes cannot match the historical manifests. Compare all old non-timing fields.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def check_run(fixture, run):
    original = json.loads((fixture / 'run.json').read_text())
    result = json.loads((run / 'run.json').read_text())
    checked_fields = [k for k in original if k != 'measured_step_seconds']
    for key in checked_fields:
        if original[key] != result[key]:
            raise ValueError(f'{fixture.name}: stable manifest field differs: {key}')
    hashes = {}
    for file in ('opacity.png', 'steps.csv'):
        expected = (fixture / file).read_bytes()
        actual = (run / file).read_bytes()
        if actual != expected:
            raise ValueError(f'{fixture.name}: fixture bytes differ: {file}')
        hashes[file] = hashlib.sha256(actual).hexdigest()
    return hashes, checked_fields


def main():
    binary = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2])
    output.mkdir(exist_ok=False)
    receipt = []
    for name in ('demo-16', 'demo-plume', 'demo-64'):
        fixture = Path('evidence') / name
        original = json.loads((fixture / 'run.json').read_text())
        run = output / name
        command = [str(binary), '--size', str(original['size']),
                   '--steps', str(original['steps']), '--source-off-at', str(original['source_off_at']),
                   '--dt', str(original['requested_dt']), '--output', str(run)]
        subprocess.run(command, check=True)
        hashes, checked_fields = check_run(fixture, run)
        receipt.append({'fixture': str(fixture), 'command': command,
                        'byte_identical_sha256': hashes, 'equal_manifest_fields': checked_fields})
        print('PASS', name, 'PNG/CSV byte-identical; stable manifest fields equal', flush=True)
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as exc:
        raise SystemExit(f'FAIL fixture replay: {exc}') from exc
