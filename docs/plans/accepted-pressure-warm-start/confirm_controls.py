#!/usr/bin/env python3
"""One final twelve-pair confirmation of the frozen adverse screening list.
Usage: confirm_controls.py baseline candidate cases.json new_output_directory
cases.json: [{"counts":[16,16,16],"workers":1,"mode":"localized","method":"jacobi"}]
Each case retains its original16-step horizon, without pooling or extension.
"""
import hashlib
import json
from pathlib import Path
import random
import statistics
import subprocess
import sys
from measurement_checks import read_rows, require

base, candidate, cases_path, directory = sys.argv[1:]
cases = json.loads(Path(cases_path).read_text())
require(isinstance(cases, list) and bool(cases), 'empty confirmation case list')
labels = []
for case in cases:
    require(case['counts'] in [[16,16,16],[32,32,32],[64,64,64],[48,24,12]] and
            case['workers'] in [1,2,4] and case['mode'] in ['steady','localized','switch','rest'] and
            case['method'] in ['jacobi','sgs'], 'case outside screening protocol')
    labels.append((tuple(case['counts']), case['workers'], case['mode'], case['method']))
require(len(set(labels)) == len(labels), 'duplicate confirmation case')
out = Path(directory)
out.mkdir(parents=True, exist_ok=False)
manifest = {'cases':cases,'pairs':12,'steps':16,'seed':20261013,'binaries':{name:{'path':path,'sha256':hashlib.sha256(Path(path).read_bytes()).hexdigest()} for name,path in [('base',base),('candidate',candidate)]},'runner_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
results = []
ignored = {'startup_s','step_s','pressure_s','capture_s'}
for case in cases:
    label = '-'.join(map(str,case['counts']))+f"-w{case['workers']}-{case['mode']}-{case['method']}"
    orders = [['base','candidate'],['candidate','base']]*6
    random.Random(20261013).shuffle(orders)
    trials = []
    for repeat, order in enumerate(orders):
        data = {}
        commands = {}
        for variant in order:
            args = [base if variant=='base' else candidate,*map(str,case['counts']),str(case['workers']),'16','cold',case['mode'],case['method']]
            proc = subprocess.run(args,text=True,capture_output=True,check=False)
            (out/f'{label}-r{repeat}-{variant}.csv').write_text(proc.stdout)
            (out/f'{label}-r{repeat}-{variant}.stderr').write_text(proc.stderr)
            require(proc.returncode == 0, f'process exit {proc.returncode}: {args}')
            data[variant] = read_rows(proc.stdout, 16, case['mode'])
            commands[variant] = args
        complete = all(len(rows)==16 and all(r['accepted']=='true' for r in rows) for rows in data.values())
        exact = [{k:v for k,v in r.items() if k not in ignored} for r in data['base']] == [{k:v for k,v in r.items() if k not in ignored} for r in data['candidate']]
        timing = {v:{'full':sum(float(r['step_s'])+float(r['capture_s']) for r in rows),'pressure':sum(float(r['pressure_s']) for r in rows),'capture':sum(float(r['capture_s']) for r in rows),'startup':float(rows[0]['startup_s']),'step0':float(rows[0]['step_s'])+float(rows[0]['capture_s'])} for v,rows in data.items()}
        delta = 100*(timing['candidate']['full']/timing['base']['full']-1)
        trial = {'repeat':repeat,'order':order,'commands':commands,'complete':complete,'exact':exact,'seconds':timing,'delta_pct':delta}
        trials.append(trial)
        (out/f'{label}-r{repeat}-receipt.json').write_text(json.dumps(trial,indent=2)+'\n')
        print(label,repeat,complete,exact,f'{delta:+.2f}%',flush=True)
    deltas = [t['delta_pct'] for t in trials]
    results.append({'case':case,'median_delta_pct':statistics.median(deltas),'min_delta_pct':min(deltas),'max_delta_pct':max(deltas),'adverse_pairs':sum(d>0 for d in deltas),'trials':trials})
    (out/'summary.json').write_text(json.dumps(results,indent=2)+'\n')
require(all(t['complete'] and t['exact'] for c in results for t in c['trials']), 'cold confirmation failed')
