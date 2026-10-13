#!/usr/bin/env python3
"""Run the pre-recorded matrix serially. Never run builds/tests concurrently.
Usage: run_matrix.py baseline_binary candidate_binary output_directory group
Groups: primary-small, primary-64, controls, sgs. Resume only completed triplets.
"""
import csv
import hashlib
import io
import itertools
import json
import os
from pathlib import Path
import random
import subprocess
import sys

baseline, candidate, destination, group = sys.argv[1:]
out = Path(destination)
out.mkdir(parents=True, exist_ok=True)
small = [(16, 16, 16), (32, 32, 32), (48, 24, 12)]
if group == 'primary-small':
    cases = list(itertools.product(small, [1, 2, 4], ['steady', 'localized'], ['jacobi']))
elif group == 'primary-64':
    cases = list(itertools.product([(64, 64, 64)], [1, 2, 4], ['steady', 'localized'], ['jacobi']))
elif group == 'controls':
    cases = list(itertools.product(small + [(64, 64, 64)], [1], ['switch', 'rest', 'saturated'], ['jacobi']))
elif group == 'sgs':
    cases = list(itertools.product(small[:2], [1], ['steady', 'localized'], ['sgs']))
else:
    raise SystemExit('unknown group')
manifest = {
    'group': group, 'seed': 20261013, 'steps': 16, 'repetitions': 6,
    'baseline_sha256': hashlib.sha256(Path(baseline).read_bytes()).hexdigest(),
    'candidate_sha256': hashlib.sha256(Path(candidate).read_bytes()).hexdigest(),
    'baseline': str(Path(baseline).resolve()), 'candidate': str(Path(candidate).resolve()),
    'runner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
}
for previous in out.glob(f'{group}-manifest*.json'):
    prior = json.loads(previous.read_text())
    for key in ['baseline_sha256', 'candidate_sha256', 'steps', 'seed']:
        if prior[key] != manifest[key]:
            raise SystemExit(f'incompatible resume manifest: {previous} {key}')
(out / f'{group}-manifest-v2.json').write_text(json.dumps(manifest, indent=2) + '\n')
orders = list(itertools.permutations(['base', 'cold', 'warm']))
rng = random.Random(20261013)
rng.shuffle(cases)
for case in cases:
    counts, workers, mode, method = case
    label = '-'.join(map(str, counts)) + f'-w{workers}-{mode}-{method}'
    # All six permutations, each variant exactly twice per position. Preserve
    # the first four screening trials and add precisely their missing orders.
    existing = sorted(out.glob(f'{label}-r*-receipt.json'))
    used = [tuple(r['variant'] for r in json.loads(p.read_text())) for p in existing]
    if len(set(used)) != len(used):
        raise SystemExit(f'duplicate ordering in {label}')
    selected = [order for order in orders if order not in used]
    rng.shuffle(selected)
    for repeat, order in enumerate(selected, start=len(existing)):
        prefix = f'{label}-r{repeat}'
        receipt = out / f'{prefix}-receipt.json'
        if list(out.glob(f'{prefix}-*.csv')):
            raise SystemExit(f'incomplete attempt preserved: {prefix}; archive explicitly before retry')
        run_receipt = []
        for variant in order:
            executable = baseline if variant == 'base' else candidate
            args = [executable, *map(str, counts), str(workers), '16', 'warm' if variant == 'warm' else 'cold', mode, method]
            rss_path = out / f'{prefix}-{variant}.rss'
            stdout_path = out / f'{prefix}-{variant}.csv'
            with stdout_path.open('w') as stdout, (out / f'{prefix}-{variant}.stderr').open('w') as stderr:
                proc = subprocess.Popen(args, stdout=stdout, stderr=stderr)
                _, status, usage = os.wait4(proc.pid, 0)
                proc.returncode = os.waitstatus_to_exitcode(status)
                if proc.returncode:
                    raise subprocess.CalledProcessError(proc.returncode, args)
            rss_path.write_text(str(usage.ru_maxrss) + '\n')
            rows = list(csv.DictReader(io.StringIO(stdout_path.read_text())))
            total = sum(float(row['step_s']) + float(row['capture_s']) for row in rows)
            accepted = sum(row['accepted'] == 'true' for row in rows)
            run_receipt.append({'variant': variant, 'argv': args, 'full_s': total, 'accepted': accepted})
        receipt.write_text(json.dumps(run_receipt, indent=2) + '\n')
        print(prefix, ' '.join(f"{r['variant']}={r['full_s']:.4f}s/{r['accepted']}" for r in run_receipt), flush=True)
