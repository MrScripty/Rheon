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
    result = json.loads((run / 'run.json').read_text())
    checked_fields = [k for k in original if k != 'measured_step_seconds']
    assert all(original[k] == result[k] for k in checked_fields)
    hashes = {}
    for file in ('opacity.png', 'steps.csv'):
        expected = (fixture / file).read_bytes()
        actual = (run / file).read_bytes()
        assert actual == expected, (name, file)
        hashes[file] = hashlib.sha256(actual).hexdigest()
    receipt.append({'fixture': str(fixture), 'command': command,
                    'byte_identical_sha256': hashes, 'equal_manifest_fields': checked_fields})
    print('PASS', name, 'PNG/CSV byte-identical; stable manifest fields equal', flush=True)
(output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
