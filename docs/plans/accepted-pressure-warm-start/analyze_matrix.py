#!/usr/bin/env python3
"""Summarize complete triplets; reject cold numerical differences and lost horizons."""
import argparse
import itertools
import re
import json
import math
from pathlib import Path
import statistics
import sys

from measurement_checks import validate_receipt

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('directory', type=Path)
parser.add_argument('--progress', action='store_true', help='incomplete/rejected evidence; never qualifies final coverage')
args = parser.parse_args()
root = args.directory
(root / "summary.json").unlink(missing_ok=True)
orders = {}
repeats = {}
small = [(16,16,16), (32,32,32), (48,24,12)]
expected = {
    'primary-small': itertools.product(small, [1,2,4], ['steady','localized'], ['jacobi']),
    'primary-64': itertools.product([(64,64,64)], [1,2,4], ['steady','localized'], ['jacobi']),
    'controls': itertools.product(small+[(64,64,64)], [1], ['switch','rest','saturated'], ['jacobi']),
    'sgs': itertools.product(small[:2], [1], ['steady','localized'], ['sgs']),
}
expected = {group:{'-'.join(map(str,n))+f'-w{w}-{mode}-{method}' for n,w,mode,method in cases} for group,cases in expected.items()}
manifest_files = list(root.glob('*-manifest*.json'))
manifests = [json.loads(path.read_text()) for path in manifest_files]
reference_manifest = manifests[0] if manifests else None
groups = {}
violations = []
replay = {}
ignored = {'startup_s', 'step_s', 'pressure_s', 'capture_s'}
for receipt in sorted(root.glob('*-receipt.json')):
    name = receipt.name.removesuffix('-receipt.json')
    case = name.rsplit('-r', 1)[0]
    try:
        match = re.fullmatch(r'(\d+)-(\d+)-(\d+)-w([124])-(steady|localized|switch|rest|saturated)-(jacobi|sgs)-r(\d+)', name)
        if match is None:
            raise ValueError('unrecognized case name')
        nx,ny,nz,workers,mode,method,repeat = match.groups()
        records = json.loads(receipt.read_text())
        rows = validate_receipt(root, name, records, (nx,ny,nz), workers, mode, method,
                                reference_manifest['baseline'] if reference_manifest else None,
                                reference_manifest['candidate'] if reference_manifest else None)
        orders.setdefault(case, []).append(tuple(r['variant'] for r in records))
        repeats.setdefault(case, []).append(int(repeat))
    except (ValueError, KeyError, TypeError, OSError) as error:
        violations.append(name + ': invalid receipt/data: ' + str(error))
        continue
    reference = [{k:v for k,v in r.items() if k not in ignored} for r in rows['base']]
    cold = [{k:v for k,v in r.items() if k not in ignored} for r in rows['cold']]
    if reference != cold:
        violations.append(name + ': candidate cold differs from baseline')
    accepted = {variant: sum(r['accepted'] == 'true' for r in values) for variant,values in rows.items()}
    if mode != 'saturated' and any(n != 16 for n in accepted.values()):
        violations.append(name + ': required full horizon incomplete')
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
if not reference_manifest:
    violations.append('missing measurement manifest')
for manifest in manifests:
    if manifest.get('group') not in expected or manifest.get('steps') != 16:
        violations.append('invalid manifest group or horizon')
    if reference_manifest and any(manifest.get(k) != reference_manifest.get(k) for k in ['baseline_sha256','candidate_sha256','baseline','candidate','seed']):
        violations.append('inconsistent frozen binaries/protocol across manifests')
all_expected = set.union(*expected.values())
if set(groups) - all_expected:
    violations.append('unexpected case set')
for case in groups:
    if len(set(orders[case])) != len(orders[case]) or len(orders[case]) > 6:
        violations.append(case + ': repeated or excessive orderings')
    if sorted(repeats[case]) != list(range(len(repeats[case]))):
        violations.append(case + ': repeat gaps')
    if not args.progress and (len(orders[case]) != 6 or set(orders[case]) != set(itertools.permutations(['base','cold','warm']))):
        violations.append(case + ': incomplete six-permutation coverage')
if not args.progress:
    if set(groups) != all_expected:
        violations.append('incomplete final case coverage: ' + str(sorted(all_expected-set(groups))))
    if {m.get('group') for m in manifests} != set(expected):
        violations.append('incomplete final manifest coverage')
    if any(m.get('repetitions') != 6 for m in manifests):
        violations.append('final manifest is not six-repetition protocol')
    prefixes = {p.name.removesuffix('-receipt.json') for p in root.glob('*-receipt.json')}
    for path in root.glob('*-r*-*.csv'):
        if path.name.rsplit('-',1)[0] not in prefixes:
            violations.append('unreceipted raw attempt: ' + path.name)
summary = {}
for case,items in sorted(groups.items()):
    metrics = ['warm_vs_base_pct','cold_vs_base_pct','warm_vs_cold_pct']
    complete = all(all(n == 16 for n in i['accepted'].values()) for i in items)
    values = {m:[i[m] for i in items if i[m] is not None] if complete else [] for m in metrics}
    summary[case] = {'repetitions':len(items), **{m:{'median':statistics.median(v),'min':min(v),'max':max(v)} for m,v in values.items() if v}, 'trials':items}
    print(case, 'n='+str(len(items)), ' '.join(f'{m}={statistics.median(v):+.2f}% [{min(v):+.2f},{max(v):+.2f}]' for m,v in values.items() if v))
print('VIOLATIONS', json.dumps(violations))
(root/'summary.json').write_text(json.dumps({'final_coverage_requested':not args.progress,'qualified':not args.progress and not violations,'cases':summary,'violations':violations},indent=2)+'\n')

if violations:
    raise SystemExit(1)
