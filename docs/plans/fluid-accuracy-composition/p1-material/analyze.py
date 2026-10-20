"""Frozen necessary criteria for the ONE n8 material reference. Stdlib only."""
import csv
import json
import math
import pathlib
import sys
# Evaluator tooling v2. Frozen scientific criteria and original bytes are
# retained under ../evaluator-tooling/. Guards remain active under -O / -OO.
def require(condition, message):
    if not condition:
        raise SystemExit("evaluator input refused: " + message)

root = pathlib.Path(sys.argv[1])
def read(name):
    with (root/name).open() as f:
        return list(csv.DictReader(f))
s = read('summary.csv'); p = read('probes.csv')
require(len(s)==8 and len(p)==48 and len(read('nodes.csv'))==72 and len(read('elements.csv'))==64, 'p1-material: input contract at original line 12')
keys = ('perturbed','hydro','deformed')
def key(r): return tuple(r[k] for k in keys)
require(len({key(r) for r in s})==8, 'p1-material: input contract at original line 15')
report=[]
for r in s:
    normalized = ('volume_error','mass_error','moment_error','work_error','momentum_error','affine_error','quadratic_error')
    algebra = all(math.isfinite(float(r[k])) and float(r[k])<=1e-12 for k in normalized)
    matrix = float(r['cholesky_min'])>0 and math.isfinite(float(r['row_bound'])) and float(r['row_bound'])>0
    alternate = float(r['alternating_max'])==20.0
    equilibrium = None if r['deformed']=='true' else all(float(r[k])<=1e-10 for k in ('equilibrium_error','bottom_load_error','top_load_error'))
    groups=[]
    for translation in ('false','true'):
        g=[x for x in p if key(x)==key(r) and x['translation']==translation]
        require(len(g)==3 and [float(x['eps']) for x in g]==[1e-5,1e-6,1e-7], 'p1-material: input contract at original line 26')
        require(all(math.isfinite(float(x[k])) for x in g for k in ('energy_fd','energy_analytic','energy_error','volume_error','hessian_error')), 'p1-material: input contract at original line 27')
        ok=[float(x['energy_error'])<=1e-6 and float(x['hessian_error'])<=1e-6 and float(x['volume_error'])<=1e-12 for x in g]
        groups.append({'translation':translation,'eps_pass':ok,'pass':any(a and b for a,b in zip(ok,ok[1:]))})
    report.append({'case':key(r),'algebra_pass':algebra,'matrix_pass':matrix,'alternating_pass':alternate,'equilibrium_pass':equilibrium,'probes':groups})
base_hydro=[r for r in s if r['hydro']=='true' and r['deformed']=='false']
controls={k:max(float(r[k]) for r in base_hydro) for k in ('wrong_gravity_error','wrong_wall_error')}
controls_pass=all(v>1e-8 for v in controls.values())
print(json.dumps({'cases':report,'control_maxima':controls,'controls_discriminate':controls_pass,
 'all_native_necessary_reference_gates_pass':controls_pass and all(r['algebra_pass'] and r['matrix_pass'] and r['alternating_pass'] and r['equilibrium_pass'] is not False and all(g['pass'] for g in r['probes']) for r in report),
 'production_admitted':False,'nonlinear_or_production_dt_admitted':False},indent=2))
