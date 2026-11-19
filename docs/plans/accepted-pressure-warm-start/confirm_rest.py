#!/usr/bin/env python3
"""One bounded 32³ rest confirmation: twelve balanced three-variant triplets.

Usage: confirm_rest.py baseline_binary candidate_binary new_output_directory
This partition remains separate from both screening and cold-only confirmations.
"""
import hashlib
import itertools
import json
from pathlib import Path
import random
import statistics
import subprocess
import sys

from measurement_checks import read_rows, require

baseline, candidate, directory = sys.argv[1:]
out = Path(directory)
out.mkdir(parents=True, exist_ok=False)
orders = list(itertools.permutations(['base', 'cold', 'warm'])) * 2
random.Random(20261013).shuffle(orders)
manifest = {
    'counts': [32, 32, 32], 'workers': 1, 'steps': 16,
    'mode': 'rest', 'method': 'jacobi', 'triplets': 12, 'seed': 20261013,
    'orders': orders,
    'binaries': {name: {'path': path, 'sha256': hashlib.sha256(Path(path).read_bytes()).hexdigest()}
                 for name, path in [('base', baseline), ('candidate', candidate)]},
    'runner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'validation_sha256': hashlib.sha256(Path(__file__).with_name('measurement_checks.py').read_bytes()).hexdigest(),
}
(out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
trials = []
timing_fields = {'startup_s', 'step_s', 'pressure_s', 'capture_s'}
for repeat, order in enumerate(orders):
    data, commands = {}, {}
    for variant in order:
        args = [baseline if variant == 'base' else candidate, '32', '32', '32',
                '1', '16', 'warm' if variant == 'warm' else 'cold', 'rest', 'jacobi']
        proc = subprocess.run(args, text=True, capture_output=True, check=False)
        (out / f'r{repeat}-{variant}.csv').write_text(proc.stdout)
        (out / f'r{repeat}-{variant}.stderr').write_text(proc.stderr)
        require(proc.returncode == 0, f'process exit {proc.returncode}: {args}')
        data[variant] = read_rows(proc.stdout, 16, 'rest')
        commands[variant] = args
    complete = all(len(rows) == 16 and all(r['accepted'] == 'true' for r in rows)
                   for rows in data.values())
    fields = {v: [{k: value for k, value in row.items() if k not in timing_fields}
                  for row in rows] for v, rows in data.items()}
    exact = fields['base'] == fields['cold'] == fields['warm']
    seconds = {v: {
        'full': sum(float(r['step_s']) + float(r['capture_s']) for r in rows),
        'pressure': sum(float(r['pressure_s']) for r in rows),
        'capture': sum(float(r['capture_s']) for r in rows),
        'startup': float(rows[0]['startup_s']),
        'step0': float(rows[0]['step_s']) + float(rows[0]['capture_s']),
    } for v, rows in data.items()}
    deltas = {f'{v}_vs_{reference}_pct': 100 * (seconds[v]['full'] / seconds[reference]['full'] - 1)
              for v, reference in [('cold', 'base'), ('warm', 'base'), ('warm', 'cold')]}
    trial = {'repeat': repeat, 'order': order, 'commands': commands,
             'complete': complete, 'exact': exact, 'seconds': seconds, **deltas}
    trials.append(trial)
    (out / f'r{repeat}-receipt.json').write_text(json.dumps(trial, indent=2) + '\n')
    print(repeat, complete, exact, deltas, flush=True)
summary = {'trials': trials}
for key in ['cold_vs_base_pct', 'warm_vs_base_pct', 'warm_vs_cold_pct']:
    values = [trial[key] for trial in trials]
    summary[key] = {'median': statistics.median(values), 'min': min(values),
                    'max': max(values), 'adverse_pairs': sum(value > 0 for value in values)}
(out / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
require(all(t['complete'] and t['exact'] for t in trials), 'rest confirmation failed')
