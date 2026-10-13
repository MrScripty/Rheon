#!/usr/bin/env python3
"""Independent dense-rational checks of actual native plate/shear records.

Fixture geometry is reconstructed from inputs. A dense full-system elimination
(including prescribed rows) is independent of Rust's reduced Thomas solve.
No saved records or historical receipts substitute for a fresh native run.
"""
from fractions import Fraction as F
from pathlib import Path
import argparse
import copy
import hashlib
import json
import math
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
LIMIT = F(1, 2**39)  # 8192 * binary64 epsilon, no dimensional floor


def require(condition, message):
    if not condition:
        raise ValueError(message)


def number(x):
    require(type(x) in (int, float) and math.isfinite(x), 'finite scalar')
    return F(x)


def enclosure(raw, exact, name):
    require(type(raw) is list and len(raw) == 2, name + ' interval shape')
    lo, hi = map(number, raw)
    require(lo <= exact <= hi, name + ' interval containment')
    return lo, hi


def dense_solve(a, b):
    a = [list(row) + [rhs] for row, rhs in zip(a, b)]
    n = len(b)
    for col in range(n):
        pivot = next(i for i in range(col, n) if a[i][col])
        a[col], a[pivot] = a[pivot], a[col]
        divisor = a[col][col]
        a[col] = [v / divisor for v in a[col]]
        for row in range(n):
            if row != col:
                factor = a[row][col]
                a[row] = [x - factor*y for x, y in zip(a[row], a[col])]
    return [row[-1] for row in a]


def verify(record):
    n = record['n']
    require(type(n) is int and 1 <= n <= 4, 'fixture layer count')
    cut, rho, mu, dt = [number(record[k]) for k in ('cut', 'rho', 'mu', 'dt')]
    volumes = [F(i+1, n) - max(cut, F(i, n)) for i in range(n)]
    centers = [(F(i+1, n) + max(cut, F(i, n)))/2 for i in range(n)]
    require(all(v > 0 for v in volumes), 'positive layer volumes')
    beta = None if record['beta'] is None else number(record['beta'])
    lower = mu/(centers[0]-cut) if beta is None else mu*beta/(mu+beta*(centers[0]-cut))
    physical_k = [lower] + [mu/(b-a) for a, b in zip(centers, centers[1:])] + [mu/(1-centers[-1])]
    # Admission matches represented coefficients after the independently derived
    # geometry formula is rounded to binary64. Solve/accounting then uses those
    # stored binary64 values as exact rationals, not unrounded geometry claims.
    k = [F(float(v)) for v in physical_k]
    require(list(map(number, record['volumes'])) == volumes, 'geometry volumes')
    require(list(map(number, record['centers'])) == centers, 'geometry centroids')
    require(list(map(number, record['interior_k'])) == k[1:-1], 'interior conductances')
    require(list(map(number, record['wall_k'])) == [k[0], k[-1]], 'wall conductances')
    plates = list(map(number, record['plate_masses']))
    require(len(plates) == 2 and all(m >= 0 for m in plates), 'plate masses')
    masses = [plates[0]] + [F(float(rho*v)) for v in volumes] + [plates[1]]
    old = [number(record['old_walls'][0])] + list(map(number, record['old'])) + [number(record['old_walls'][1])]
    new = [number(record['new_walls'][0])] + list(map(number, record['new'])) + [number(record['new_walls'][1])]
    require(len(old) == len(new) == n+2 and len(record['new_walls']) == len(record['old_walls']) == 2, 'state shapes')
    fixed = {i for i in (0, n+1) if masses[i] == 0}
    a = [[F(0) for _ in old] for _ in old]
    b = [F(0) for _ in old]
    worst = F(0)
    for i in range(n+2):
        if i in fixed:
            a[i][i], b[i] = F(1), old[i]
            require(new[i] == old[i], 'prescribed wall invariant')
            continue
        a[i][i] = masses[i]
        b[i] = masses[i]*old[i]
        inertia = masses[i]*(new[i]-old[i])
        defect, scale = inertia, abs(inertia)
        for other, edge in ((i-1, i-1), (i+1, i)):
            if 0 <= other < n+2:
                a[i][i] += dt*k[edge]
                a[i][other] -= dt*k[edge]
                traction = dt*k[edge]*(new[i]-new[other])
                defect += traction
                scale += abs(traction)
        require(abs(defect) <= LIMIT*scale, 'actual node momentum equation')
        worst = max(worst, abs(defect))
    ideal = dense_solve(a, b)
    require(all(abs(x-y) <= F(1, 10**12)*max(1, abs(y)) for x, y in zip(new, ideal)), 'independent dense velocity')
    require(number(record['equation_max']) >= worst, 'reported equation maximum')
    before = sum(m*u*u/2 for m, u in zip(masses, old))
    after = sum(m*v*v/2 for m, v in zip(masses, new))
    increment = sum(m*(v-u)**2/2 for m, u, v in zip(masses, old, new))
    loss = dt*sum(c*(right-left)**2 for c, left, right in zip(k, new, new[1:]))
    impulses = [dt*k[0]*(new[0]-new[1]), dt*k[-1]*(new[-1]-new[-2])]
    work = sum(impulses[end]*new[i] for end, i in enumerate((0, n+1)) if i in fixed)
    external = sum(impulses[end] for end, i in enumerate((0, n+1)) if i in fixed)
    momentum = sum(m*(v-u) for m, u, v in zip(masses, old, new))
    edef = after-before+increment+loss-work
    pdef = momentum-external
    escale = before+after+increment+loss+abs(work)
    pscale = sum(abs(m*u)+abs(m*v) for m, u, v in zip(masses, old, new)) + sum(abs(impulses[end]) for end, i in enumerate((0, n+1)) if i in fixed)
    require(abs(edef) <= LIMIT*escale, 'actual energy identity')
    require(abs(pdef) <= LIMIT*pscale, 'actual momentum identity')
    if not work:
        require(after <= before, 'zero-work kinetic nonincrease')
    values = dict(before=before, after=after, change=after-before, increment=increment,
                  dissipation=loss, work=work, energy_defect=edef, momentum=momentum,
                  external_impulse=external, momentum_defect=pdef,
                  energy_budget=LIMIT*escale, momentum_budget=LIMIT*pscale)
    for name, exact in values.items():
        lo, hi = enclosure(record[name], exact, name)
        if name.endswith('_budget'):
            require(hi-lo <= abs(exact)*F(1, 10**10), name+' width')
    for defect, budget in [('energy_defect', 'energy_budget'), ('momentum_defect', 'momentum_budget')]:
        require(max(map(abs, map(number, record[defect]))) <= number(record[budget][0]),
                defect+' certificate acceptance')
    if record['work'] == [0, 0]:
        require(number(record['change'][1]) <= 0, 'zero-work energy certificate')
    require(len(record['fluid_impulses']) == 2, 'two boundary impulses')
    for raw, exact in zip(record['fluid_impulses'], impulses):
        enclosure(raw, exact, 'fluid impulse')
    return {'energy_before': str(before), 'energy_after': str(after),
            'dense_error_max': float(max(abs(x-y) for x, y in zip(new, ideal)))}


def verify_all(records):
    require(type(records) is list and len(records) == 18, 'native case roster')
    summaries = []
    for case in range(6):
        previous = None
        for step in range(3):
            r = records[3*case+step]
            expected = dict(case=case, step=step, n=1 if case < 3 else 4,
                            normal=case % 3, tangent=(case+1) % 3,
                            cut=0 if case < 3 else .125, rho=1 if case < 3 else 2,
                            mu=1 if case < 3 else .5, beta={4: 2, 5: 0}.get(case), dt=.25,
                            plate_masses=[[0, 0], [1, 0], [1, 1]][case] if case < 3 else [2, 3])
            require(all(r.get(k) == v for k, v in expected.items()), 'declared specimen identity')
            old = ([0] if case < 3 else [0, .25, -.5, .75]) if previous is None else previous['new']
            walls = ([[0, 1], [1, 0], [1, -1]][case] if case < 3 else [1, -1]) if previous is None else previous['new_walls']
            require(r['old'] == old and r['old_walls'] == walls, 'specimen initial/continuity')
            summaries.append(verify(r))
            previous = r
    return summaries


def corruption_checks(records):
    changes = [lambda r: r[0]['wall_k'].__setitem__(0, 4),
               lambda r: r[0]['new'].__setitem__(0, .5),
               lambda r: r[3]['plate_masses'].__setitem__(0, 2),
               lambda r: r[0].__setitem__('work', [0, 0]),
               lambda r: r[0].__setitem__('energy_budget', [0, 1]),
               lambda r: r[1]['old'].__setitem__(0, 0),
               lambda r: r[6]['fluid_impulses'].__setitem__(0, [-1, -1]),
               lambda r: r[0].__setitem__('energy_defect', [-1, 1]),
               lambda r: r[0].__setitem__('momentum_defect', [-1, 1]),
               lambda r: r[6].__setitem__('change', [-1, 1]),
               lambda r: r.pop()]
    for mutate in changes:
        bad = copy.deepcopy(records)
        mutate(bad)
        try:
            verify_all(bad)
        except ValueError:
            continue
        raise ValueError('corrupted native record accepted')
    return len(changes)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_snapshot():
    paths = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard'], cwd=ROOT, text=True).splitlines()
    return {'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
            'worktree_status': subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True),
            'source_sha256': {p: digest(ROOT/p) for p in paths if (ROOT/p).is_file()}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--allow-dirty', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    require(not output.exists(), 'new output directory required')
    parent = output.parent
    require(parent.exists(), 'output parent must exist')
    probe = subprocess.run(['git', '-C', str(parent), 'rev-parse', '--is-inside-work-tree'], capture_output=True)
    require(probe.returncode != 0, 'evidence must be outside Git')
    source = source_snapshot()
    require(args.allow_dirty or not source['worktree_status'], 'clean committed source required')
    executable = args.native.resolve()
    binary_hash = digest(executable)
    result = subprocess.run([str(executable)], capture_output=True, text=True, check=True, timeout=60)
    records = json.loads(result.stdout)
    summaries = verify_all(records)
    negative = corruption_checks(records)
    require(digest(executable) == binary_hash, 'binary changed during qualification')
    output.mkdir()
    (output/'records.json').write_text(result.stdout)
    require(source_snapshot() == source, 'source changed during qualification')
    receipt = {'status': 'passed', 'schema': 'rheon-plate-coupling-qualification-v1',
               **source,
               'native_sha256': binary_hash, 'records_sha256': digest(output/'records.json'),
               'native_steps': len(records), 'corruptions_rejected': negative,
               'python_optimization': sys.flags.optimize, 'summaries': summaries,
               'scope': 'bounded tangential plate/shear; stored-coefficient discrete controls'}
    (output/'qualification.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print(json.dumps({k: receipt[k] for k in ('status', 'head', 'native_steps', 'corruptions_rejected', 'python_optimization')}))


if __name__ == '__main__':
    main()
