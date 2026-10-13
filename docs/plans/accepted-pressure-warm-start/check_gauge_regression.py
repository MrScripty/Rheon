#!/usr/bin/env python3
"""Actual64³ two-step regression; old must fail with accepted state intact.
Usage: check_gauge_regression.py old_binary corrected_binary output_directory
This correctness diagnostic is excluded from performance summaries.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from measurement_checks import read_rows, require

old, corrected, directory = sys.argv[1:]
out = Path(directory)
out.mkdir(parents=True, exist_ok=False)
results = {}
for name, binary in [('old', old), ('corrected', corrected)]:
    args = [binary, '64', '64', '64', '4', '2', 'warm', 'localized', 'jacobi']
    proc = subprocess.run(args, text=True, capture_output=True, check=False)
    (out/f'{name}.csv').write_text(proc.stdout)
    (out/f'{name}.stderr').write_text(proc.stderr)
    require(proc.returncode == 0, f'process exit {proc.returncode}: {args}')
    rows = read_rows(proc.stdout, 2, 'localized')
    require(len(rows) == 2 and rows[0]['accepted'] == 'true', 'missing initial accepted step')
    if name == 'old':
        require(rows[1]['accepted'] == 'false' and 'IterationLimit' in rows[1]['error'], 'old failure did not reproduce')
        for field in ['time', 'fingerprint', 'bytes']:
            require(rows[0][field] == rows[1][field], 'failed step changed '+field)
    else:
        require(rows[1]['accepted'] == 'true', 'corrected step rejected')
        require(float(rows[1]['time']) == 2/64, 'changed regression horizon')
        # The unchanged solver enforces current-RHS relative/full residual gates;
        # independently check the accepted physical divergence and conservation.
        require(all(float(r['divergence']) <= 1e-5 and abs(float(r['mass_error'])) <= 32*sys.float_info.epsilon for r in rows), 'physical regression gate')
    results[name] = {'command':args,'sha256':hashlib.sha256(Path(binary).read_bytes()).hexdigest(),'accepted':sum(r['accepted']=='true' for r in rows)}
(out/'result.json').write_text(json.dumps(results,indent=2)+'\n')
