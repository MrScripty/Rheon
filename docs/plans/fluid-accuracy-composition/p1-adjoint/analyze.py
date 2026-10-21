"""Frozen static gate evaluation, stdlib only; ROOT is a completed native run."""
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

def rows(name):
    with (root / name).open() as handle:
        return list(csv.DictReader(handle))

summary = rows('summary.csv')
sites = rows('sites.csv')
derivatives = rows('derivatives.csv')
require(len(summary) == 16 and len(sites) == 768 and len(derivatives) == 48, 'p1-adjoint: input contract at original line 17')

def key(row):
    return tuple(row[k] for k in ('map', 'n', 'perturbed', 'hydro'))

require(len({key(row) for row in summary}) == 16, 'p1-adjoint: input contract at original line 22')
report = []
for row in summary:
    group = [r for r in derivatives if key(r) == key(row)]
    fields = [r for r in sites if key(r) == key(row)]
    require(len(group) == 3 and len(fields) == int(row['n']), 'p1-adjoint: input contract at original line 27')
    require([float(r['eps']) for r in group] == [1e-5, 1e-6, 1e-7], 'p1-adjoint: input contract at original line 28')
    for field in fields:
        require(all(math.isfinite(float(field[k])) for k in field if k not in ('map', 'perturbed', 'hydro')), 'p1-adjoint: input contract at original line 30')
        require(all(float(field[k]) > 0 for k in ('y', 'mass', 'rho', 'volume')), 'p1-adjoint: input contract at original line 31')
    for derivative in group:
        require(all(math.isfinite(float(derivative[k])) for k in derivative if k not in ('map', 'perturbed', 'hydro')), 'p1-adjoint: input contract at original line 33')
    fd_ok = [all(float(r[k]) <= 1e-6 for k in ('energy_error', 'volume_error', 'density_error')) for r in group]
    entry = dict(row)
    entry['work_pass'] = float(row['work_normalized']) <= 1e-12
    entry['momentum_pass'] = float(row['momentum_normalized']) <= 1e-12
    entry['derivative_pass'] = any(a and b for a, b in zip(fd_ok, fd_ok[1:]))
    entry['derivative_eps_pass'] = fd_ok
    entry['e0_pass'] = None if row['hydro'] == 'true' else float(row['e0_force_over_pressure']) <= 1e-10
    entry['max_fd_energy_error'] = max(float(r['energy_error']) for r in group)
    entry['max_fd_volume_error'] = max(float(r['volume_error']) for r in group)
    entry['max_fd_density_error'] = max(float(r['density_error']) for r in group)
    if row['hydro'] == 'false':
        pressure = float(fields[0]['pressure'])
        entry['force_volume_identity_max_absolute'] = max(abs(float(r['force']) / pressure - float(r['volume_gradient'])) for r in fields)
    report.append(entry)
print(json.dumps(report, indent=2, allow_nan=False))
