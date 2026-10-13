#!/usr/bin/env python3
"""Summarize complete triplets; reject cold numerical differences and lost horizons."""
import csv
import json
import math
from pathlib import Path
import statistics
import sys

root = Path(sys.argv[1])
groups = {}
violations = []
replay = {}
ignored = {'startup_s', 'step_s', 'pressure_s', 'capture_s'}
for receipt in sorted(root.glob('*-receipt.json')):
    name = receipt.name.removesuffix('-receipt.json')
    case = name.rsplit('-r', 1)[0]
    rows = {}
    for variant in ['base', 'cold', 'warm']:
        rows[variant] = list(csv.DictReader((root / f'{name}-{variant}.csv').open()))
    reference = [{k:v for k,v in r.items() if k not in ignored} for r in rows['base']]
    cold = [{k:v for k,v in r.items() if k not in ignored} for r in rows['cold']]
    if reference != cold:
        violations.append(name + ': candidate cold differs from baseline')
    accepted = {variant: sum(r['accepted'] == 'true' for r in values) for variant,values in rows.items()}
    if len(set(accepted.values())) != 1:
        violations.append(name + ': acceptance differs ' + str(accepted))
    for variant, values in rows.items():
        numeric = [{k:v for k,v in r.items() if k not in ignored} for r in values]
        replay_key = (case, variant)
        if replay_key in replay and replay[replay_key] != numeric:
            violations.append(name + ': same-policy replay differs ' + variant)
        replay[replay_key] = numeric
        if any(r['bytes'] != rows['base'][0]['bytes'] for r in values):
            violations.append(name + ': managed memory differs ' + variant)
        for r in values:
            if r['accepted'] != 'true':
                continue
            for metric in ['residual_l2','residual_max','divergence','mass_error','energy','max_speed']:
                if not math.isfinite(float(r[metric])):
                    violations.append(name + ': nonfinite ' + variant + ' ' + metric)
            if float(r['divergence']) > 1e-5 or float(r['energy']) < 0:
                violations.append(name + ': physical gate ' + variant)
            if abs(float(r['mass_error'])) > 64*sys.float_info.epsilon*0.5:
                violations.append(name + ': mass balance ' + variant)
            if 'localized' not in name:
                if not math.isfinite(float(r['pressure_error'])) or float(r['pressure_error']) > 1e-6 or float(r['max_speed']) > 1e-7:
                    violations.append(name + ': independent hydrostatic oracle ' + variant)
    warm_first = {k:v for k,v in rows['warm'][0].items() if k not in ignored}
    if warm_first != cold[0]:
        violations.append(name + ': first warm step differs from cold')
    if accepted['base'] == 16:
        for variant,values in rows.items():
            if values[-1]['time'] != rows['base'][-1]['time'] or [r['dt'] for r in values] != [r['dt'] for r in rows['base']]:
                violations.append(name + ': horizon differs ' + variant)
    sums = {v: sum(float(r['step_s']) + float(r['capture_s']) for r in rs) for v,rs in rows.items()}
    sustained = {v: sum(float(r['step_s']) + float(r['capture_s']) for r in rs[1:]) for v,rs in rows.items()}
    item = {'repeat':name, 'accepted':accepted, 'full_s':sums,
            'warm_vs_base_pct':100*(sums['warm']/sums['base']-1) if all(n == 16 for n in accepted.values()) else None,
            'cold_vs_base_pct':100*(sums['cold']/sums['base']-1) if all(n == 16 for n in accepted.values()) else None,
            'warm_vs_cold_pct':100*(sums['warm']/sums['cold']-1) if all(n == 16 for n in accepted.values()) else None,
            'sustained_s':sustained,
            'startup_s':{v:float(rs[0]['startup_s']) for v,rs in rows.items()},
            'step0_s':{v:float(rs[0]['step_s'])+float(rs[0]['capture_s']) for v,rs in rows.items()},
            'pressure_phase_s':{v:sum(float(r['pressure_s']) for r in rs) for v,rs in rows.items()},
            'capture_s':{v:sum(float(r['capture_s']) for r in rs) for v,rs in rows.items()},
            'managed_bytes':{v:int(rs[0]['bytes']) for v,rs in rows.items()},
            'iterations':{v:sum(int(r['iterations']) for r in rs if r['accepted'] == 'true') for v,rs in rows.items()},
            'rss_kib':{v:int((root/f'{name}-{v}.rss').read_text()) for v in rows}}
    groups.setdefault(case,[]).append(item)
summary = {}
for case,items in sorted(groups.items()):
    metrics = ['warm_vs_base_pct','cold_vs_base_pct','warm_vs_cold_pct']
    values = {m:[i[m] for i in items if i[m] is not None] for m in metrics}
    summary[case] = {'repetitions':len(items), **{m:{'median':statistics.median(v),'min':min(v),'max':max(v)} for m,v in values.items() if v}, 'trials':items}
    print(case, 'n='+str(len(items)), ' '.join(f'{m}={statistics.median(v):+.2f}% [{min(v):+.2f},{max(v):+.2f}]' for m,v in values.items() if v))
print('VIOLATIONS', json.dumps(violations))
(root/'summary.json').write_text(json.dumps({'cases':summary,'violations':violations},indent=2)+'\n')
