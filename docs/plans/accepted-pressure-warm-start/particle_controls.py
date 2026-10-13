#!/usr/bin/env python3
"""Unaffected particle control: six balanced pairs,64 steps at dt0.001.
Usage: particle_controls.py baseline_binary candidate_binary output_directory
"""
import hashlib
import json
import re
import statistics
import subprocess
from pathlib import Path
import sys

baseline,candidate,directory=sys.argv[1:]
out=Path(directory);out.mkdir(parents=True,exist_ok=True)
results=[]
for samples in [10,16]:
    for workers in [1,4]:
        pairs=[]
        for repeat in range(6):
            pair={}
            for variant in (['base','candidate'] if repeat%2==0 else ['candidate','base']):
                binary=baseline if variant=='base' else candidate
                args=[binary,str(samples),str(workers),'64']
                proc=subprocess.run(args,text=True,capture_output=True,check=True)
                name=f'n{samples**3}-w{workers}-r{repeat}-{variant}'
                (out/f'{name}.txt').write_text(proc.stdout)
                (out/f'{name}.stderr').write_text(proc.stderr)
                fields=dict(re.findall(r'([a-z_]+)=([^ ]+)',proc.stdout.splitlines()[0]))
                pair[variant]={'command':args,'frame_s':float(fields['frame_seconds']),'capture_s':float(fields['capture_seconds']),'time':fields['accepted_time'],'fingerprint':fields['fingerprint'],'bytes':fields['managed_bytes'],'particles':fields['particles']}
            for key in ['time','fingerprint','bytes','particles']:
                assert pair['base'][key]==pair['candidate'][key],(samples,workers,repeat,key)
            pair['delta_pct']=100*(pair['candidate']['frame_s']/pair['base']['frame_s']-1)
            pairs.append(pair)
            print(samples**3,workers,repeat,f"{pair['delta_pct']:+.2f}%",flush=True)
        results.append({'particles':samples**3,'workers':workers,'median_delta_pct':statistics.median(p['delta_pct'] for p in pairs),'pairs':pairs})
        (out/'particle-summary.json').write_text(json.dumps({'baseline_sha256':hashlib.sha256(Path(baseline).read_bytes()).hexdigest(),'candidate_sha256':hashlib.sha256(Path(candidate).read_bytes()).hexdigest(),'cases':results},indent=2)+'\n')
