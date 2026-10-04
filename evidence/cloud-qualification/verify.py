#!/usr/bin/env python3
"""Validate historical retained evidence, without rerunning or relabeling timings."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import statistics

METHODS = ('jacobi-pcg-v1', 'sgs-pcg-v1')
METADATA = {'repeat', 'warmup', 'path', 'pressure_iterations', 'maximum_step_divergence'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def number(value, field, *, integer=False, positive=False):
    require(type(value) is int if integer else type(value) in (int, float),
            f'{field}: expected numeric type (not bool or string)')
    require(math.isfinite(value) and (value > 0 if positive else value >= 0),
            f'{field}: expected finite {"positive" if positive else "nonnegative"} value')
    return value


def same(actual, expected, field):
    # Python numeric equality alone would accept True as 1 and 3.0 as a count.
    require(type(actual) is type(expected), f'{field}: differs in type')
    if type(expected) is dict:
        require(set(actual) == set(expected), f'{field}: fields differ')
        for key, value in expected.items():
            same(actual[key], value, f'{field}.{key}')
    elif type(expected) is list:
        require(len(actual) == len(expected), f'{field}: length differs')
        for index, value in enumerate(expected):
            same(actual[index], value, f'{field}[{index}]')
    else:
        require(actual == expected, f'{field}: differs')


def summary_from_runs(samples, method):
    durations = [r['measured_step_seconds'] for r in samples]
    return {'implementation': method, 'median_seconds': statistics.median(durations),
            'min_seconds': min(durations), 'max_seconds': max(durations), 'samples': len(samples),
            'managed_simulation_bytes': samples[0]['managed_simulation_bytes'],
            'max_divergence': max(r['maximum_step_divergence'] for r in samples),
            'pressure_iterations': samples[0]['pressure_iterations']}


def check_summary(actual, expected, context):
    require(type(actual) is list and len(actual) == len(METHODS), f'{context}: summary count')
    require([s['implementation'] for s in actual] == list(METHODS), f'{context}: summary methods')
    for stored, computed in zip(actual, expected):
        require(set(stored) == set(computed), f'{context}: summary fields')
        for key, value in computed.items():
            if type(value) in (int, float):
                number(stored[key], f'{context}: summary {key}', integer=type(value) is int)
            same(stored[key], value, f'{context}: summary {key}')


def verify(root, source_root, fixture_root):
    for entry in json.loads((root / 'qualified-source-manifest.json').read_text())['files']:
        digest = hashlib.sha256((source_root / entry['path']).read_bytes()).hexdigest()
        require(digest == entry['sha256'], f"Historical source hash differs: {entry['path']}")

    replays = json.loads((root / 'replay/receipt.json').read_text())
    require(type(replays) is list and len(replays) == 3, 'Original replay fixture count differs')
    require(all(type(entry) is dict and type(entry.get('fixture')) is str for entry in replays),
            'Original replay fixture entries differ')
    require(sorted(entry['fixture'] for entry in replays) ==
            sorted(('evidence/demo-16', 'evidence/demo-plume', 'evidence/demo-64')),
            'Original replay fixture inventory differs')
    for entry in replays:
        hashes = entry.get('byte_identical_sha256')
        require(type(hashes) is dict and set(hashes) == {'opacity.png', 'steps.csv'},
                'Original replay artifact hash inventory differs')
    passed = []

    receipt = json.loads((root / 'benchmarks/receipt.json').read_text())
    require(len(receipt['cases']) == 6, 'Benchmark receipt case count')
    receipt_cases = {(c['size'], c['accuracy']): c for c in receipt['cases']}
    require(len(receipt_cases) == 6, 'Duplicate benchmark receipt case')
    for size, steps in ((16, 12), (32, 6), (64, 3)):
        for accuracy, relative, pressure in (('standard', 1e-9, 1e-7), ('tight', 1e-11, 1e-9)):
            label = f'{size}-{accuracy}'
            case = root / 'benchmarks' / label
            comparison = json.loads((case / 'comparison.json').read_text())
            require(len(comparison['runs']) == 8, f'{label}: retained run count')
            require(comparison['equal_accepted_time'] is True, f'{label}: unequal accepted times')
            same(comparison['workload'], {'size': size, 'steps': steps, 'repeats': 3,
                 'accuracy': accuracy, 'requested_dt': 0.02, 'source_off_at': steps // 2},
                 f'{label}: historical workload')
            times, budgets, expected_summary, paths = set(), set(), [], set()
            for method in METHODS:
                runs = [r for r in comparison['runs'] if r['implementation'] == method]
                require(all(type(r['repeat']) is int for r in runs), f'{label}: repeat type')
                require(sorted(r['repeat'] for r in runs) == [-1, 0, 1, 2], f'{label}: repeat indices')
                outputs, iterations, measured = set(), set(), []
                for run in runs:
                    same(run['warmup'], run['repeat'] == -1, f'{label}: warmup flag')
                    require(type(run['path']) is str and Path(run['path']).name == run['path']
                            and run['path'] not in ('', '.', '..') and run['path'] not in paths,
                            f'{label}: unique retained run directory')
                    paths.add(run['path'])
                    path = case / run['path']
                    manifest = json.loads((path / 'run.json').read_text())
                    require(set(run) == set(manifest) | METADATA, f'{label}: embedded run fields')
                    for key, value in manifest.items():
                        same(run[key], value, f'{label}/{run["path"]}: embedded {key}')
                    controls = {'implementation': method, 'size': size, 'steps': steps,
                                'source_off_at': steps // 2, 'requested_dt': 0.02,
                                'pressure_relative_residual': relative, 'pressure_absolute_residual': 1e-12,
                                'pressure_divergence_limit': pressure, 'actual_divergence_limit': 1e-5,
                                'pressure_iteration_limit': 2000}
                    for key, value in controls.items():
                        same(manifest[key], value, f'{label}: control {key}')
                    for key in ('accepted_time', 'measured_step_seconds', 'last_divergence_max'):
                        number(manifest[key], f'{label}: {key}', positive=key != 'last_divergence_max')
                    for key in ('managed_simulation_bytes', 'managed_budget_bytes', 'raw_export_pixel_bytes'):
                        number(manifest[key], f'{label}: {key}', integer=True, positive=True)
                    series = list(csv.DictReader((path / 'steps.csv').read_text().splitlines()))
                    require(len(series) == steps, f'{label}: step count')
                    elapsed = 0.0
                    for row in series:
                        require(all(math.isfinite(float(v)) for v in row.values()), f'{label}: finite diagnostics')
                        # These are norms, nonnegative smoke totals/energy, and
                        # counts. Do not apply an unsigned rule to other fields.
                        for field in ('full_residual_max', 'actual_divergence_max',
                                      'courant', 'tracer_integral', 'kinetic_energy'):
                            number(float(row[field]), f'{label}: {field}')
                        for field in ('step', 'pressure_iterations'):
                            number(int(row[field]), f'{label}: {field}', integer=True)
                        # Preserve the original historical gate (not the later repaired harness contract).
                        require(float(row['full_residual_max']) * float(row['dt']) <= pressure,
                                f'{label}: historical residual gate')
                        require(float(row['actual_divergence_max']) <= 1e-5, f'{label}: divergence gate')
                        require(float(row['courant']) <= 1.0, f'{label}: Courant gate')
                        require(int(row['pressure_iterations']) <= 2000, f'{label}: iteration gate')
                        elapsed += number(float(row['dt']), f'{label}: accepted dt', positive=True)
                        same(float(row['time']), elapsed, f'{label}: cumulative accepted time')
                    same(manifest['accepted_time'], elapsed, f'{label}: accepted horizon')
                    same(manifest['last_divergence_max'], float(series[-1]['actual_divergence_max']),
                         f'{label}: final divergence')
                    count = sum(int(r['pressure_iterations']) for r in series)
                    maximum = max(float(r['actual_divergence_max']) for r in series)
                    number(run['pressure_iterations'], f'{label}: embedded iterations', integer=True)
                    number(run['maximum_step_divergence'], f'{label}: embedded maximum divergence')
                    same(run['pressure_iterations'], count, f'{label}: CSV iteration total')
                    same(run['maximum_step_divergence'], maximum, f'{label}: CSV maximum divergence')
                    times.add(manifest['accepted_time'])
                    budgets.add(manifest['managed_simulation_bytes'])
                    outputs.add(tuple(hashlib.sha256((path / name).read_bytes()).hexdigest()
                                      for name in ('steps.csv', 'opacity.png')))
                    iterations.add(count)
                    if not run['warmup']:
                        # Independently reconstruct measurements from manifests/CSV, not embedded rows.
                        measured.append(manifest | {'pressure_iterations': count, 'maximum_step_divergence': maximum})
                require(len(outputs) == 1, f'{label}: per-method PNG/CSV must be byte deterministic')
                require(len(iterations) == 1, f'{label}: deterministic iteration totals')
                expected_summary.append(summary_from_runs(measured, method))
            require(len(times) == len(budgets) == 1, f'{label}: equal horizons and array budgets')
            check_summary(comparison['summary'], expected_summary, label)
            recorded = receipt_cases[(size, accuracy)]
            same(recorded['steps'], steps, f'{label}: receipt steps')
            same(recorded['status'], 'completed', f'{label}: receipt status')
            same(recorded['error'], None, f'{label}: receipt error')
            check_summary(recorded['summary'], expected_summary, f'{label} receipt')
            passed.append(f'PASS {label}: all 8 historical runs, controls/diagnostics, horizons/budgets, deterministic bytes and recomputed summaries')

    for entry in replays:
        fixture = fixture_root / entry['fixture']
        replay = root / 'replay' / fixture.name
        original = json.loads((fixture / 'run.json').read_text())
        result = json.loads((replay / 'run.json').read_text())
        fields = [k for k in original if k != 'measured_step_seconds']
        same(entry['equal_manifest_fields'], fields, f'{fixture.name}: replay manifest field list')
        for key in fields:
            same(result[key], original[key], f'{fixture.name}: replay manifest {key}')
        for name, digest in entry['byte_identical_sha256'].items():
            expected, actual = (fixture / name).read_bytes(), (replay / name).read_bytes()
            require(hashlib.sha256(expected).hexdigest() == digest, f'{fixture.name}/{name}: fixture hash')
            require(expected == actual, f'{fixture.name}/{name}: replay bytes')
    for message in passed:
        print(message)
    print('PASS original fixture bytes/manifests and historical qualified source hashes')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-root', type=Path, default=Path.cwd(),
                        help='historical qualification checkout 7e1a76dd487549a496e04b9305fa531fcc7ab2f9')
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    try:
        verify(root, args.source_root, root.parents[1])
    except (OSError, ValueError, KeyError, TypeError, OverflowError, csv.Error) as exc:
        parser.exit(1, f'FAIL historical qualification: {exc}\n')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
