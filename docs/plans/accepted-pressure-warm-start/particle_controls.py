#!/usr/bin/env python3
"""Unaffected particle control: six balanced pairs,64 steps at dt0.001.
Usage: particle_controls.py baseline_binary candidate_binary output_directory [--confirm-single-worker]
Confirmation uses exactly twelve balanced pairs per one-worker case; no extension.
"""
import hashlib
import json
import re
import random
import statistics
import subprocess
from pathlib import Path
import sys
from measurement_checks import finite, require

require(len(sys.argv) in [4,5] and (len(sys.argv) == 4 or sys.argv[4] == '--confirm-single-worker'), 'unexpected arguments')
baseline,candidate,directory=sys.argv[1:4]
confirmation = len(sys.argv) == 5
pair_count = 12 if confirmation else 6
worker_counts = [1] if confirmation else [1,4]
out=Path(directory);out.mkdir(parents=True,exist_ok=False)
manifest = {'baseline_sha256':hashlib.sha256(Path(baseline).read_bytes()).hexdigest(),
            'candidate_sha256':hashlib.sha256(Path(candidate).read_bytes()).hexdigest(),
            'runner_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            'counts':[1000,4096], 'workers':worker_counts, 'steps':64, 'dt':0.001, 'pairs':pair_count,
            'protocol':'bounded-confirmation' if confirmation else 'screening', 'seed':20261013}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
results=[]
for samples in [10,16]:
    for workers in worker_counts:
        pairs=[]
        orders = [['base','candidate'],['candidate','base']] * (pair_count//2)
        if confirmation:
            random.Random(20261013).shuffle(orders)
        for repeat, order in enumerate(orders):
            pair={}
            for variant in order:
                binary=baseline if variant=='base' else candidate
                args=[binary,str(samples),str(workers),'64']
                proc=subprocess.run(args,text=True,capture_output=True,check=False)
                name=f'n{samples**3}-w{workers}-r{repeat}-{variant}'
                (out/f'{name}.txt').write_text(proc.stdout)
                (out/f'{name}.stderr').write_text(proc.stderr)
                require(proc.returncode == 0, f'process exit {proc.returncode}: {args}')
                samples_s = json.loads(proc.stdout.splitlines()[0].split('samples=',1)[1])
                require(len(samples_s) == 64, 'missing particle step distribution')
                for value in samples_s:
                    finite(value, 'particle step time', nonnegative=True)
                fields=dict(re.findall(r'([a-z_]+)=([^ ]+)',proc.stdout.splitlines()[0]))
                pair[variant]={'command':args,'frame_s':finite(fields['frame_seconds'], 'particle frame time', nonnegative=True),'capture_s':finite(fields['capture_seconds'], 'particle capture time', nonnegative=True),'time':fields['accepted_time'],'fingerprint':fields['fingerprint'],'bytes':fields['managed_bytes'],'particles':fields['particles'],'samples_s':samples_s,'phases':[dict(re.findall(r'([a-z_]+)=([^ ]+)', line)) for line in proc.stdout.splitlines()[1:]]}
            require(all(float(p['time']) > 0.063999999999 and float(p['time']) < 0.064000000001 and int(p['particles']) == samples**3 and p['frame_s'] > 0 for p in pair.values()), 'particle horizon/count/time failed')
            for key in ['time','fingerprint','bytes','particles']:
                require(pair['base'][key]==pair['candidate'][key], str((samples,workers,repeat,key)))
            pair['order'] = order
            pair['delta_pct']=100*(pair['candidate']['frame_s']/pair['base']['frame_s']-1)
            pairs.append(pair)
            print(samples**3,workers,repeat,f"{pair['delta_pct']:+.2f}%",flush=True)
        results.append({'particles':samples**3,'workers':workers,'median_delta_pct':statistics.median(p['delta_pct'] for p in pairs),'pairs':pairs})
        (out/'particle-summary.json').write_text(json.dumps({'baseline_sha256':hashlib.sha256(Path(baseline).read_bytes()).hexdigest(),'candidate_sha256':hashlib.sha256(Path(candidate).read_bytes()).hexdigest(),'cases':results},indent=2)+'\n')
