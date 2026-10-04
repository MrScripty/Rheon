#!/usr/bin/env python3
"""Independently check new qualification evidence against archived run bytes.

Run from the repository root after serial benchmark.py completes. This reads
evidence, does not rerun timing measurements or reuse harness acceptance helpers.
"""
import csv
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess

BASE = '0de6a857adb17a6ee0312e889aff50e2cd7aa6dd'
ROOT = Path('evidence/result-qualification')
BINARY_SHA256 = 'e10000f4fb61ba09765d7e9cbc76cac0463550cc7aecbcd72a66d458e0a1a584'
METHODS = ('jacobi-pcg-v1', 'sgs-pcg-v1')


def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    receipt = json.loads((ROOT / 'benchmarks/receipt.json').read_text())
    assert receipt['binary_sha256'] == BINARY_SHA256
    source = receipt['source_commit']
    protected = ('src', 'tests', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
                 'proofs', 'docs/research-book', 'evidence/cloud-qualification',
                 'evidence/demo-16', 'evidence/demo-plume', 'evidence/demo-64')
    assert git('diff', BASE, '--', *protected) == '', 'Protected source/evidence changed'
    hashes = {}
    for name in ('tools/rheon_compare.py', 'tools/test_result_qualification.py',
                 'tools/test_rheon_compare.py', '.github/workflows/rust-rheon.yml'):
        committed = subprocess.check_output(['git', 'show', f'{source}:{name}'])
        assert Path(name).read_bytes() == committed, name
        hashes[name] = sha256(Path(name))
    checks, maximum_predicted, maximum_actual = [], 0.0, 0.0
    assert [(c['size'], c['steps'], c['accuracy']) for c in receipt['cases']] == [
        (n, steps, accuracy) for n, steps in ((16, 12), (32, 6), (64, 3))
        for accuracy in ('standard', 'tight')]
    for case in receipt['cases']:
        assert case['status'] == 'completed' and case['error'] is None
        n, steps, accuracy = case['size'], case['steps'], case['accuracy']
        folder = f'{n}-{accuracy}'
        comparison = json.loads((ROOT / 'benchmarks' / folder / 'comparison.json').read_text())
        assert comparison['environment']['executable_sha256'] == BINARY_SHA256
        assert comparison['environment']['build']['debug_assertions'] is False
        assert comparison['equal_accepted_time'] is True
        assert comparison['real_time_claim'] is False
        assert len(comparison['runs']) == 8
        assert (ROOT / 'benchmarks' / folder / 'status.json').exists()
        assert json.loads((ROOT / 'benchmarks' / folder / 'status.json').read_text())['status'] == 'completed'
        relative, pressure = (1e-9, 1e-7) if accuracy == 'standard' else (1e-11, 1e-9)
        settings = {'schema_version': 2, 'size': n, 'steps': steps,
                    'source_off_at': steps // 2, 'requested_dt': .02,
                    'pressure_relative_residual': relative, 'pressure_absolute_residual': 1e-12,
                    'pressure_divergence_limit': pressure, 'actual_divergence_limit': 1e-5,
                    'pressure_iteration_limit': 2000, 'managed_budget_bytes': 64 * 1024 * 1024}
        payload = 8 * 3 * n * n * (n + 1) + 56 * n ** 3
        for run in comparison['runs']:
            path = ROOT / 'benchmarks' / folder / run['path']
            archived = Path('evidence/cloud-qualification/benchmarks') / folder / run['path']
            manifest = json.loads((path / 'run.json').read_text())
            assert all(type(manifest[key]) is type(value) and manifest[key] == value
                       for key, value in settings.items())
            assert manifest['implementation'] in METHODS
            assert manifest['managed_simulation_bytes'] == payload
            assert manifest['raw_export_pixel_bytes'] == n * n
            assert manifest['whole_process_memory_cap_claimed'] is False
            assert manifest['real_time_performance_claimed'] is False
            for artifact in ('steps.csv', 'opacity.png'):
                assert (path / artifact).read_bytes() == (archived / artifact).read_bytes(), str(path / artifact)
            old_manifest = json.loads((archived / 'run.json').read_text())
            assert {k: v for k, v in manifest.items() if k != 'measured_step_seconds'} == {
                k: v for k, v in old_manifest.items() if k != 'measured_step_seconds'}
            with (path / 'steps.csv').open(newline='') as stream:
                rows = list(csv.DictReader(stream))
            assert len(rows) == steps
            elapsed, count, divergence = 0.0, 0, 0.0
            h = 1.0 / n
            for index, row in enumerate(rows, 1):
                assert int(row['step']) == index
                values = {key: float(value) for key, value in row.items()}
                assert all(math.isfinite(v) and v >= 0 for v in values.values())
                assert 0 < values['dt'] <= .02
                elapsed += values['dt']
                assert values['time'] == elapsed
                predicted = values['full_residual_max'] * values['dt'] / (h * h * h)
                assert predicted <= pressure
                assert values['actual_divergence_max'] <= 1e-5 and values['courant'] <= 1
                assert 0 <= int(row['pressure_iterations']) <= 2000
                count += int(row['pressure_iterations'])
                divergence = max(divergence, values['actual_divergence_max'])
                maximum_predicted = max(maximum_predicted, predicted)
                maximum_actual = max(maximum_actual, values['actual_divergence_max'])
            assert manifest['accepted_time'] == elapsed
            assert manifest['last_divergence_max'] == values['actual_divergence_max']
            assert run['pressure_iterations'] == count and run['maximum_step_divergence'] == divergence
            assert math.isfinite(manifest['measured_step_seconds']) and manifest['measured_step_seconds'] >= 0
        for summary in comparison['summary']:
            samples = [r for r in comparison['runs'] if r['implementation'] == summary['implementation'] and not r['warmup']]
            durations = [r['measured_step_seconds'] for r in samples]
            assert summary['samples'] == len(samples) == 3
            assert summary['median_seconds'] == statistics.median(durations)
            assert summary['min_seconds'] == min(durations) and summary['max_seconds'] == max(durations)
        checks.append({'case': folder, 'runs': 8, 'png_csv_pairs_byte_identical_to_archive': 8,
                       'summary': comparison['summary']})
        print('PASS', folder, '8 complete runs; all-step gates; archived PNG/CSV bytes and stable manifests')
    result = {'benchmark_source_commit': source, 'benchmark_source_tree': git('rev-parse', f'{source}^{{tree}}'),
              'reviewed_base': BASE, 'harness_source_sha256': hashes, 'binary_sha256': BINARY_SHA256,
              'protected_paths_unchanged_from_reviewed_base': list(protected),
              'maximum_predicted_divergence': maximum_predicted, 'maximum_actual_divergence': maximum_actual,
              'checks': checks}
    (ROOT / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print('PASS unchanged protected source/evidence, compiler binary, 48 deterministic run pairs')


if __name__ == '__main__':
    main()
