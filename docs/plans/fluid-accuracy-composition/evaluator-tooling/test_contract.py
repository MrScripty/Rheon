"""Tooling-only replay; never executes a native numerical campaign."""
import argparse
import ast
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys

parser = argparse.ArgumentParser(description=__doc__)
for key in ('adjoint', 'material', 'q1', 'q2_v1', 'q2_v2', 'output'):
    parser.add_argument('--' + key.replace('_', '-'), type=Path, required=True)
args = parser.parse_args()
args.output.mkdir(exist_ok=False)
logs = args.output / 'logs'
logs.mkdir()
base = Path(__file__).resolve().parent.parent
hashes = json.loads((base / 'evaluator-tooling/evaluator-hashes.json').read_text())
files = {
    'p1-adjoint': ['summary.csv', 'sites.csv', 'derivatives.csv'],
    'p1-material': ['summary.csv', 'probes.csv', 'nodes.csv', 'elements.csv'],
    'q1': ['channel-summary.csv', 'transport-summary.csv', 'geometry.csv', 'constant-controls.csv'],
    'q2': ['summary.csv'],
}
roots = {'p1-adjoint': args.adjoint, 'p1-material': args.material, 'q1': args.q1, 'q2': args.q2_v2}
records = []

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def invoke(label, owner, root, mode='', frozen=False):
    script = base / ('evaluator-tooling/frozen/' + owner + '.py') if frozen else base / owner / 'analyze.py'
    command = [sys.executable] + ([mode] if mode else []) + [str(script), str(root)]
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    (logs / (label + '.stdout')).write_bytes(result.stdout)
    (logs / (label + '.stderr')).write_bytes(result.stderr)
    records.append(dict(label=label, command=command, exit_code=result.returncode,
                        stdout_sha256=hashlib.sha256(result.stdout).hexdigest()))
    return result

# The promoted input predicates must be structurally identical to the frozen ones.
for owner, metadata in hashes.items():
    frozen = base / 'evaluator-tooling/frozen' / (owner + '.py')
    active = base / owner / 'analyze.py'
    require(digest(frozen) == metadata['original_sha256'], owner + ': frozen bytes changed')
    require(digest(active) == metadata['repaired_sha256'], owner + ': new evaluator hash mismatch')
    old_tree = ast.parse(frozen.read_text())
    tree = ast.parse(active.read_text())
    original = sorted((n for n in ast.walk(old_tree) if isinstance(n, ast.Assert)), key=lambda n:n.lineno)
    calls = sorted((n for n in ast.walk(tree) if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == 'require'), key=lambda n:n.lineno)
    require(len(original) == len(calls) == metadata['always_on_guards'], owner + ': guard count')
    require(not any(isinstance(n, ast.Assert) for n in ast.walk(tree)), owner + ': active assertion remains')
    require(all(ast.dump(a.test) == ast.dump(b.args[0]) for a,b in zip(original,calls)), owner + ': changed scientific/input predicate')

input_before = {str(root / name): digest(root / name) for owner, root in roots.items() for name in files[owner]}
input_before[str(args.q2_v1 / 'summary.csv')] = digest(args.q2_v1 / 'summary.csv')

# Real retained complete campaigns: byte-identical JSON, including negative findings.
for owner in ('p1-adjoint', 'p1-material', 'q2'):
    baseline = invoke(owner + '-frozen-normal', owner, roots[owner], frozen=True)
    require(baseline.returncode == 0, owner + ': retained baseline unexpectedly refused')
    json.loads(baseline.stdout)
    for mode in ('', '-O', '-OO'):
        result = invoke(owner + '-retained-' + (mode or 'normal'), owner, roots[owner], mode)
        require(result.returncode == 0 and result.stdout == baseline.stdout, owner + ': retained output changed ' + mode)

# Incomplete retained native matrices remain refused; no substitute full Q1 run.
for owner, root, label in [('q1', args.q1, 'q1-incomplete'), ('q2', args.q2_v1, 'q2-v1-empty')]:
    require(invoke(label + '-frozen-normal', owner, root, frozen=True).returncode != 0, label + ': old refusal lost')
    for mode in ('', '-O', '-OO'):
        result = invoke(label + '-' + (mode or 'normal'), owner, root, mode)
        require(result.returncode != 0 and b'evaluator input refused:' in result.stderr and not result.stdout, label + ': incomplete matrix accepted')

malformed = []
def fixture(owner, label, change):
    root = args.output / 'fixtures' / label
    root.mkdir(parents=True)
    tables = {}
    headers = {}
    for name in files[owner]:
        with (roots[owner] / name).open(newline='') as handle:
            reader = csv.DictReader(handle)
            headers[name] = reader.fieldnames
            tables[name] = list(reader)
    change(tables)
    for name in files[owner]:
        with (root / name).open('w', newline='') as handle:
            writer = csv.DictWriter(handle, fieldnames=headers[name])
            writer.writeheader()
            writer.writerows(tables[name])
    malformed.append((owner, label, root))
    return root

for owner in files:
    fixture(owner, owner + '-empty', lambda t: [v.clear() for v in t.values()])
    fixture(owner, owner + '-truncated', lambda t: t[next(iter(t))].pop())

def bad_adjoint_eps(t):
    t['derivatives.csv'][0]['eps'] = '9e-6'
fixture('p1-adjoint', 'adjoint-epsilon', bad_adjoint_eps)
def bad_adjoint_field(t):
    t['sites.csv'][0]['mass'] = 'nan'
fixture('p1-adjoint', 'adjoint-nonfinite', bad_adjoint_field)
def bad_material_eps(t):
    t['probes.csv'][0]['eps'] = '9e-6'
fixture('p1-material', 'material-epsilon', bad_material_eps)
def bad_material_groups(t):
    t['summary.csv'][1] = t['summary.csv'][0].copy()
fixture('p1-material', 'material-duplicate', bad_material_groups)
def bad_adjoint_groups(t):
    t['summary.csv'][1] = t['summary.csv'][0].copy()
fixture('p1-adjoint', 'adjoint-duplicate', bad_adjoint_groups)
def bad_q2_group(t):
    t['summary.csv'][0]['mu'] = '0.2'
fixture('q2', 'q2-refinement-group', bad_q2_group)

# Independently exercise nested guards with top-level row counts preserved.
def reassign(table, field):
    table[0][field] = 'false' if table[0][field] == 'true' else 'true'
fixture('p1-adjoint', 'adjoint-site-group', lambda t: reassign(t['sites.csv'], 'perturbed'))
fixture('p1-adjoint', 'adjoint-derivative-group', lambda t: reassign(t['derivatives.csv'], 'perturbed'))
fixture('p1-material', 'material-probe-group', lambda t: reassign(t['probes.csv'], 'perturbed'))
for value in ('nan', 'inf', '-inf'):
    fixture('p1-adjoint', 'adjoint-derivative-' + value, lambda t, v=value: t['derivatives.csv'][0].__setitem__('energy_error',v))
    fixture('p1-material', 'material-probe-' + value, lambda t, v=value: t['probes.csv'][0].__setitem__('energy_fd',v))
for field in ('y', 'mass', 'rho', 'volume'):
    for value in ('0', '-1'):
        fixture('p1-adjoint', 'adjoint-positive-' + field + value, lambda t,k=field,v=value:t['sites.csv'][0].__setitem__(k,v))

def reorder_eps(t, filename):
    key_fields = ('map','n','perturbed','hydro') if filename=='derivatives.csv' else ('perturbed','hydro','deformed','translation')
    first = t[filename][0]
    ids = [i for i,r in enumerate(t[filename]) if all(r[k] == first[k] for k in key_fields)]
    i,j=ids[:2]
    t[filename][i],t[filename][j]=t[filename][j],t[filename][i]
fixture('p1-adjoint','adjoint-epsilon-order',lambda t:reorder_eps(t,'derivatives.csv'))
fixture('p1-material','material-epsilon-order',lambda t:reorder_eps(t,'probes.csv'))

def q1_bad_group(t):
    # Synthetic table cardinality ONLY: copied coarse summary rows are explicitly
    # not native64 measurements and cannot serve a scientific gate or result.
    for row in list(t['transport-summary.csv']):
        if row['n']=='32':
            copy=row.copy();copy['n']='64';t['transport-summary.csv'].append(copy)
    for name in ('translation-final','vortex-initial'):
        row=next(r for r in t['geometry.csv'] if r['name']==name and r['n']=='32').copy()
        row['n']='64';t['geometry.csv'].append(row)
    for row in list(t['constant-controls.csv']):
        if row['n']=='32':
            copy=row.copy();copy['n']='64';t['constant-controls.csv'].append(copy)
    require(len(t['transport-summary.csv'])==18 and len(t['geometry.csv'])==9 and len(t['constant-controls.csv'])==18,'synthetic Q1 cardinality helper')
    t['transport-summary.csv'][0]['requested_courant']='0.2'
fixture('q1','q1-synthetic-cardinality-only-bad-group',q1_bad_group)

# Reproduce optimization bypass of the ORIGINAL evaluators as tooling evidence.
def material_subset(t):
    row = next(r for r in t['summary.csv'] if r['hydro']=='true' and r['deformed']=='false')
    keys = ('perturbed', 'hydro', 'deformed')
    t['summary.csv'] = [row]
    t['probes.csv'] = [p for p in t['probes.csv'] if all(p[k] == row[k] for k in keys)]
    t['nodes.csv'] = t['nodes.csv'][:1]
    t['elements.csv'] = t['elements.csv'][:1]
material_bad = fixture('p1-material', 'material-subset', material_subset)
adjoint_bad = next(root for owner,label,root in malformed if label == 'p1-adjoint-empty')
q2_bad = next(root for owner,label,root in malformed if label == 'q2-truncated')
for owner, root in [('p1-adjoint',adjoint_bad),('p1-material',material_bad),('q1',args.q1),('q2',q2_bad)]:
    bypass = invoke(owner + '-original-O-bypass', owner, root, '-O', frozen=True)
    require(bypass.returncode == 0, owner + ': expected old optimization bypass not reproduced')
    json.loads(bypass.stdout)

for owner, label, root in malformed:
    for mode in ('', '-O', '-OO'):
        result = invoke(label + '-refusal-' + (mode or 'normal'), owner, root, mode)
        require(result.returncode != 0 and b'evaluator input refused:' in result.stderr and not result.stdout, label + ': malformed input accepted or incidental failure')

require(all(digest(Path(p)) == value for p,value in input_before.items()), 'retained CSV bytes changed')
report = dict(evaluator_hashes=hashes, subprocess_checks=len(records), malformed_fixtures=len(malformed),
              original_optimized_bypasses_reproduced=4, retained_complete_outputs_identical=True,
              retained_incomplete_matrices_refused=True, normal_O_OO_malformed_refusals=True,
              retained_input_sha256=input_before, no_native_campaign_executed=True, records=records)
(args.output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k:v for k,v in report.items() if k not in ('records','evaluator_hashes','retained_input_sha256')}, indent=2))
