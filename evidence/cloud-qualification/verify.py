#!/usr/bin/env python3
"""Independently validate retained benchmark outputs, without rerunning timings."""
import csv
import hashlib
import json
import math
from pathlib import Path

root = Path(__file__).resolve().parent
for entry in json.loads((root / 'qualified-source-manifest.json').read_text())['files']:
    assert hashlib.sha256(Path(entry['path']).read_bytes()).hexdigest() == entry['sha256']

for size, steps in ((16, 12), (32, 6), (64, 3)):
    for accuracy, relative, pressure in (('standard', 1e-9, 1e-7), ('tight', 1e-11, 1e-9)):
        case = root / 'benchmarks' / f'{size}-{accuracy}'
        comparison = json.loads((case / 'comparison.json').read_text())
        assert len(comparison['runs']) == 8
        assert comparison['equal_accepted_time']
        times, budgets = set(), set()
        for method in ('jacobi-pcg-v1', 'sgs-pcg-v1'):
            runs = [r for r in comparison['runs'] if r['implementation'] == method]
            assert sorted(r['repeat'] for r in runs) == [-1, 0, 1, 2]
            outputs = set()
            iterations = set()
            for run in runs:
                path = case / run['path']
                manifest = json.loads((path / 'run.json').read_text())
                assert manifest['implementation'] == method
                assert manifest['size'] == size and manifest['steps'] == steps
                assert manifest['source_off_at'] == steps // 2
                assert manifest['requested_dt'] == 0.02
                assert manifest['pressure_relative_residual'] == relative
                assert manifest['pressure_absolute_residual'] == 1e-12
                assert manifest['pressure_divergence_limit'] == pressure
                assert manifest['actual_divergence_limit'] == 1e-5
                assert manifest['pressure_iteration_limit'] == 2000
                assert 0 < manifest['measured_step_seconds'] < math.inf
                series = list(csv.DictReader((path / 'steps.csv').read_text().splitlines()))
                assert len(series) == steps
                for row in series:
                    assert all(math.isfinite(float(v)) for v in row.values())
                    assert float(row['full_residual_max']) * float(row['dt']) <= pressure
                    assert float(row['actual_divergence_max']) <= 1e-5
                    assert float(row['courant']) <= 1.0
                    assert int(row['pressure_iterations']) <= 2000
                times.add(manifest['accepted_time'])
                budgets.add(manifest['managed_simulation_bytes'])
                outputs.add(tuple(hashlib.sha256((path / name).read_bytes()).hexdigest()
                                  for name in ('steps.csv', 'opacity.png')))
                iterations.add(sum(int(r['pressure_iterations']) for r in series))
            assert len(outputs) == 1, 'Per-method replay must be byte deterministic'
            assert len(iterations) == 1
        assert len(times) == len(budgets) == 1
        print(f'PASS {size}-{accuracy}: all 8 runs, explicit settings, residual/divergence/Courant gates, equal horizons/budgets, deterministic per-method PNG/CSV')

for entry in json.loads((root / 'replay' / 'receipt.json').read_text()):
    for name, digest in entry['byte_identical_sha256'].items():
        fixture = Path(entry['fixture']) / name
        replay = root / 'replay' / fixture.parent.name / name
        assert hashlib.sha256(fixture.read_bytes()).hexdigest() == digest
        assert fixture.read_bytes() == replay.read_bytes()
print('PASS original fixture bytes and qualified source hashes')
