#!/usr/bin/env python3
"""Run the pre-recorded matrix serially. Never run builds/tests concurrently.
Usage: run_matrix.py baseline_binary candidate_binary output_directory group
Groups: primary-small, primary-64, controls, sgs. Resume only completed triplets.
"""
import hashlib
import itertools
import json
import os
from pathlib import Path
import random
import subprocess
import sys
from measurement_checks import read_rows, require, validate_receipt

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
    'validation_sha256': hashlib.sha256(Path(__file__).with_name('measurement_checks.py').read_bytes()).hexdigest(),
}
for previous in out.glob(f'{group}-manifest*.json'):
    prior = json.loads(previous.read_text())
    for key in ['baseline_sha256', 'candidate_sha256', 'steps', 'seed', 'runner_sha256', 'validation_sha256']:
        if prior[key] != manifest[key]:
            raise SystemExit(f'incompatible resume manifest: {previous} {key}')
manifest_path = out / f'{group}-manifest-v2.json'
if not manifest_path.exists():
    with manifest_path.open('x') as file:
        file.write(json.dumps(manifest, indent=2) + '\n')
orders = list(itertools.permutations(['base', 'cold', 'warm']))
rng = random.Random(20261013)
rng.shuffle(cases)
for case in cases:
    counts, workers, mode, method = case
    label = '-'.join(map(str, counts)) + f'-w{workers}-{mode}-{method}'
    # All six permutations, each variant exactly twice per position. Preserve
    # the first four screening trials and add precisely their missing orders.
    existing = sorted(out.glob(f'{label}-r*-receipt.json'), key=lambda p:int(p.name.split('-r')[-1].split('-')[0]))
    require(len(existing) <= 6, 'too many existing repetitions')
    used = []
    for index, path in enumerate(existing):
        require(path.name == f'{label}-r{index}-receipt.json', 'repeat gaps in resume')
        records = json.loads(path.read_text())
        validate_receipt(out, path.name.removesuffix('-receipt.json'), records,
                         counts, workers, mode, method, baseline, candidate)
        ordering = tuple(r['variant'] for r in records)
        require(ordering in orders, 'foreign ordering in resume')
        used.append(ordering)
    if len(set(used)) != len(used):
        raise SystemExit(f'duplicate ordering in {label}')
    selected = [order for order in orders if order not in used]
    rng.shuffle(selected)
    for repeat, order in enumerate(selected, start=len(existing)):
        prefix = f'{label}-r{repeat}'
        receipt = out / f'{prefix}-receipt.json'
        if list(out.glob(f'{prefix}-*')):
            raise SystemExit(f'incomplete attempt preserved: {prefix}; archive explicitly before retry')
        run_receipt = []
        for variant in order:
            executable = baseline if variant == 'base' else candidate
            args = [executable, *map(str, counts), str(workers), '16', 'warm' if variant == 'warm' else 'cold', mode, method]
            rss_path = out / f'{prefix}-{variant}.rss'
            stdout_path = out / f'{prefix}-{variant}.csv'
            with stdout_path.open('x') as stdout, (out / f'{prefix}-{variant}.stderr').open('x') as stderr:
                proc = subprocess.Popen(args, stdout=stdout, stderr=stderr)
                _, status, usage = os.wait4(proc.pid, 0)
                proc.returncode = os.waitstatus_to_exitcode(status)
                if proc.returncode:
                    raise subprocess.CalledProcessError(proc.returncode, args)
            with rss_path.open('x') as file:
                file.write(str(usage.ru_maxrss) + '\n')
            rows = read_rows(stdout_path.read_text(), 16, mode)
            total = sum(float(row['step_s']) + float(row['capture_s']) for row in rows)
            accepted = sum(row['accepted'] == 'true' for row in rows)
            run_receipt.append({'variant': variant, 'argv': args, 'full_s': total, 'accepted': accepted})
        with receipt.open('x') as file:
            file.write(json.dumps(run_receipt, indent=2) + '\n')
        print(prefix, ' '.join(f"{r['variant']}={r['full_s']:.4f}s/{r['accepted']}" for r in run_receipt), flush=True)
