#!/usr/bin/env python3
"""Predeclared twelve balanced frozen-binary cold pairs,128 equal steps."""
import hashlib
import json
import random
import statistics
import subprocess
import sys
from measurement_checks import read_rows, require
from pathlib import Path

base, candidate, directory = sys.argv[1:]
out = Path(directory)
out.mkdir(parents=True, exist_ok=False)
orders = [['base', 'candidate'], ['candidate', 'base']] * 6
random.Random(20261013).shuffle(orders)
manifest = {'orders': orders, 'binaries': {k: {'path': v, 'sha256': hashlib.sha256(Path(v).read_bytes()).hexdigest()} for k,v in [('base',base),('candidate',candidate)]}}
(out/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
trials = []
ignored = {'startup_s', 'step_s', 'pressure_s', 'capture_s'}
for repeat, order in enumerate(orders):
    data = {}
    for variant in order:
        args = [base if variant == 'base' else candidate, '16','16','16','4','128','cold','steady','jacobi']
        proc = subprocess.run(args, text=True, capture_output=True, check=False)
        (out/f'r{repeat}-{variant}.csv').write_text(proc.stdout)
        (out/f'r{repeat}-{variant}.stderr').write_text(proc.stderr)
        require(proc.returncode == 0, f'process exit {proc.returncode}: {args}')
        data[variant] = read_rows(proc.stdout, 128, 'steady')
    valid = all(len(rows) == 128 and all(r['accepted']=='true' for r in rows) and float(rows[-1]['time']) == 2.0 for rows in data.values())
    exact = [{k:v for k,v in r.items() if k not in ignored} for r in data['base']] == [{k:v for k,v in r.items() if k not in ignored} for r in data['candidate']]
    timing = {v: {'startup':float(rows[0]['startup_s']), 'step0':float(rows[0]['step_s'])+float(rows[0]['capture_s']), 'sustained':sum(float(r['step_s'])+float(r['capture_s']) for r in rows[1:]), 'full':sum(float(r['step_s'])+float(r['capture_s']) for r in rows)} for v,rows in data.items()}
    delta = {k:100*(timing['candidate'][k]/timing['base'][k]-1) for k in timing['base']}
    trials.append({'repeat':repeat,'order':order,'complete_horizons':valid,'exact':exact,'seconds':timing,'delta_pct':delta})
    summary = {'trials':trials,'median_delta_pct':{k:statistics.median(t['delta_pct'][k] for t in trials) for k in delta}}
    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(repeat,valid,exact,delta,flush=True)

require(all(t['complete_horizons'] and t['exact'] for t in trials), 'cold confirmation failed')
