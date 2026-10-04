#!/usr/bin/env python3
"""Check evidence independently, then requalify with the final harness.

Run from the repository root after serial benchmark.py completes. This reads
evidence and does not rerun timing measurements. Independent byte/numeric checks
are followed by a separate pass through the current harness's acceptance helpers.
"""
import argparse
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import statistics
import subprocess
import sys

sys.path.insert(0, str(Path('tools').resolve()))
from rheon_compare import _qualify_png, _qualify_series, _qualify_settings

BASE = '0de6a857adb17a6ee0312e889aff50e2cd7aa6dd'
ROOT = Path('evidence/result-qualification')
BINARY_SHA256 = 'e10000f4fb61ba09765d7e9cbc76cac0463550cc7aecbcd72a66d458e0a1a584'
METHODS = ('jacobi-pcg-v1', 'sgs-pcg-v1')

PROTECTED = ('src', 'tests', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
                 'proofs', 'docs/research-book', 'evidence/cloud-qualification',
                 'evidence/demo-16', 'evidence/demo-plume', 'evidence/demo-64')


def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate_receipt_output(path):
    root = Path.cwd().resolve()
    path = path.absolute()
    lexical = Path(os.path.abspath(path))
    physical = path.resolve()
    for name in PROTECTED:
        protected = root / name
        for target in (protected, protected.resolve()):
            require(not lexical.is_relative_to(target) and not physical.is_relative_to(target),
                    'Receipt output is inside a protected path: ' + str(path))
    return path


def publish_receipt(path, receipt):
    path = validate_receipt_output(path)
    payload = json.dumps(receipt, indent=2, allow_nan=False) + '\n'
    stream = path.open('x')  # Existing operator files never enter cleanup.
    identity = None
    try:
        with stream:
            created = os.fstat(stream.fileno())
            identity = (created.st_dev, created.st_ino)
            if stream.write(payload) != len(payload):
                raise OSError('Short receipt write')
    except BaseException as error:
        try:
            current = path.lstat()
            if identity == (current.st_dev, current.st_ino):
                path.unlink()
        except FileNotFoundError:
            pass
        except OSError as cleanup:
            raise OSError(f'Receipt write failed: {error}; cleanup failed: {cleanup}') from error
        raise


def verify():
    receipt = json.loads((ROOT / 'benchmarks/receipt.json').read_text())
    require((receipt['binary_sha256'] == BINARY_SHA256), "Historical qualification failed: receipt['binary_sha256'] == BINARY_SHA256")
    source = receipt['source_commit']
    require((git('diff', BASE, '--', *PROTECTED) == ''), 'Protected source/evidence changed')
    require(git('diff', '--cached', BASE, '--', *PROTECTED) == '', 'Staged protected source/evidence changed')
    additions = subprocess.check_output(['git', 'ls-files', '--others', '-z', '--', *PROTECTED])
    require(not additions, 'Untracked or ignored protected path additions')
    hashes, qualified_hashes = {}, {}
    qualified_source = git('log', '-1', '--format=%H', '--', 'tools/rheon_compare.py',
                           'tools/test_result_qualification.py')
    for name in ('tools/rheon_compare.py', 'tools/test_result_qualification.py',
                 'tools/test_rheon_compare.py', '.github/workflows/rust-rheon.yml'):
        committed = subprocess.check_output(['git', 'show', f'{source}:{name}'])
        hashes[name] = hashlib.sha256(committed).hexdigest()
        qualified = subprocess.check_output(['git', 'show', f'{qualified_source}:{name}'])
        require((Path(name).read_bytes() == qualified), name)
        qualified_hashes[name] = sha256(Path(name))
    checks, maximum_predicted, maximum_actual = [], 0.0, 0.0
    require(([(c['size'], c['steps'], c['accuracy']) for c in receipt['cases']] == [
        (n, steps, accuracy) for n, steps in ((16, 12), (32, 6), (64, 3))
        for accuracy in ('standard', 'tight')]), "Historical qualification failed: [(c['size'], c['steps'], c['accuracy']) for c in receipt['cases']] == [(n, steps, accuracy) for n, steps in ((16, 12), (32, 6), (64, 3)) for accuracy in ('standard', 'tight')]")
    for case in receipt['cases']:
        require((case['status'] == 'completed' and case['error'] is None), "Historical qualification failed: case['status'] == 'completed' and case['error'] is None")
        n, steps, accuracy = case['size'], case['steps'], case['accuracy']
        folder = f'{n}-{accuracy}'
        comparison = json.loads((ROOT / 'benchmarks' / folder / 'comparison.json').read_text())
        require((comparison['environment']['executable_sha256'] == BINARY_SHA256), "Historical qualification failed: comparison['environment']['executable_sha256'] == BINARY_SHA256")
        require((comparison['environment']['build']['debug_assertions'] is False), "Historical qualification failed: comparison['environment']['build']['debug_assertions'] is False")
        require((comparison['equal_accepted_time'] is True), "Historical qualification failed: comparison['equal_accepted_time'] is True")
        require((comparison['real_time_claim'] is False), "Historical qualification failed: comparison['real_time_claim'] is False")
        require((len(comparison['runs']) == 8), "Historical qualification failed: len(comparison['runs']) == 8")
        require(((ROOT / 'benchmarks' / folder / 'status.json').exists()), "Historical qualification failed: (ROOT / 'benchmarks' / folder / 'status.json').exists()")
        require((json.loads((ROOT / 'benchmarks' / folder / 'status.json').read_text())['status'] == 'completed'), "Historical qualification failed: json.loads((ROOT / 'benchmarks' / folder / 'status.json').read_text())['status'] == 'completed'")
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
            require((all(type(manifest[key]) is type(value) and manifest[key] == value
                       for key, value in settings.items())), 'Historical qualification failed: all((type(manifest[key]) is type(value) and manifest[key] == value for key, value in settings.items()))')
            require((manifest['implementation'] in METHODS), "Historical qualification failed: manifest['implementation'] in METHODS")
            require((manifest['managed_simulation_bytes'] == payload), "Historical qualification failed: manifest['managed_simulation_bytes'] == payload")
            require((manifest['raw_export_pixel_bytes'] == n * n), "Historical qualification failed: manifest['raw_export_pixel_bytes'] == n * n")
            require((manifest['whole_process_memory_cap_claimed'] is False), "Historical qualification failed: manifest['whole_process_memory_cap_claimed'] is False")
            require((manifest['real_time_performance_claimed'] is False), "Historical qualification failed: manifest['real_time_performance_claimed'] is False")
            for artifact in ('steps.csv', 'opacity.png'):
                require(((path / artifact).read_bytes() == (archived / artifact).read_bytes()), str(path / artifact))
            old_manifest = json.loads((archived / 'run.json').read_text())
            require(({k: v for k, v in manifest.items() if k != 'measured_step_seconds'} == {
                k: v for k, v in old_manifest.items() if k != 'measured_step_seconds'}), "Historical qualification failed: {k: v for k, v in manifest.items() if k != 'measured_step_seconds'} == {k: v for k, v in old_manifest.items() if k != 'measured_step_seconds'}")
            with (path / 'steps.csv').open(newline='') as stream:
                rows = list(csv.DictReader(stream))
            require((len(rows) == steps), 'Historical qualification failed: len(rows) == steps')
            elapsed, count, divergence = 0.0, 0, 0.0
            h = 1.0 / n
            for index, row in enumerate(rows, 1):
                require((int(row['step']) == index), "Historical qualification failed: int(row['step']) == index")
                values = {key: float(value) for key, value in row.items()}
                require((all(math.isfinite(v) and v >= 0 for v in values.values())), 'Historical qualification failed: all((math.isfinite(v) and v >= 0 for v in values.values()))')
                require((0 < values['dt'] <= .02), "Historical qualification failed: 0 < values['dt'] <= 0.02")
                elapsed += values['dt']
                require((values['time'] == elapsed), "Historical qualification failed: values['time'] == elapsed")
                predicted = values['full_residual_max'] * values['dt'] / (h * h * h)
                require((predicted <= pressure), 'Historical qualification failed: predicted <= pressure')
                require((values['actual_divergence_max'] <= 1e-5 and values['courant'] <= 1), "Historical qualification failed: values['actual_divergence_max'] <= 1e-05 and values['courant'] <= 1")
                require((0 <= int(row['pressure_iterations']) <= 2000), "Historical qualification failed: 0 <= int(row['pressure_iterations']) <= 2000")
                count += int(row['pressure_iterations'])
                divergence = max(divergence, values['actual_divergence_max'])
                maximum_predicted = max(maximum_predicted, predicted)
                maximum_actual = max(maximum_actual, values['actual_divergence_max'])
            require((manifest['accepted_time'] == elapsed), "Historical qualification failed: manifest['accepted_time'] == elapsed")
            require((manifest['last_divergence_max'] == values['actual_divergence_max']), "Historical qualification failed: manifest['last_divergence_max'] == values['actual_divergence_max']")
            require((run['pressure_iterations'] == count and run['maximum_step_divergence'] == divergence), "Historical qualification failed: run['pressure_iterations'] == count and run['maximum_step_divergence'] == divergence")
            require((math.isfinite(manifest['measured_step_seconds']) and manifest['measured_step_seconds'] >= 0), "Historical qualification failed: math.isfinite(manifest['measured_step_seconds']) and manifest['measured_step_seconds'] >= 0")
            # Additional final-validator pass, separate from the independent
            # archived-byte, physical-unit and summary checks above.
            _qualify_settings(manifest, manifest['implementation'], n, steps, accuracy)
            require((_qualify_series(manifest, path / 'steps.csv', n, steps, accuracy) == (count, divergence)), "Historical qualification failed: _qualify_series(manifest, path / 'steps.csv', n, steps, accuracy) == (count, divergence)")
            _qualify_png(path / 'opacity.png', n)
        for summary in comparison['summary']:
            samples = [r for r in comparison['runs'] if r['implementation'] == summary['implementation'] and not r['warmup']]
            durations = [r['measured_step_seconds'] for r in samples]
            require((summary['samples'] == len(samples) == 3), "Historical qualification failed: summary['samples'] == len(samples) == 3")
            require((summary['median_seconds'] == statistics.median(durations)), "Historical qualification failed: summary['median_seconds'] == statistics.median(durations)")
            require((summary['min_seconds'] == min(durations) and summary['max_seconds'] == max(durations)), "Historical qualification failed: summary['min_seconds'] == min(durations) and summary['max_seconds'] == max(durations)")
        require(sorted(s['implementation'] for s in comparison['summary']) == sorted(METHODS),
                'Comparison summary implementation inventory differs')
        require(sorted(s['implementation'] for s in case['summary']) == sorted(METHODS),
                'Receipt summary implementation inventory differs')
        for summary in comparison['summary']:
            recorded = [s for s in case['summary'] if s['implementation'] == summary['implementation']]
            require(len(recorded) == 1, 'Receipt must have one summary per implementation')
            for field in ('samples', 'median_seconds', 'min_seconds', 'max_seconds'):
                value = summary[field]
                expected_type = int if field == 'samples' else float
                require(type(value) is expected_type and type(recorded[0][field]) is expected_type,
                        'Timing summary field type differs: ' + field)
                require(math.isfinite(value) and value >= 0 and value == recorded[0][field],
                        'Timing summary differs from historical case receipt: ' + field)
        checks.append({'case': folder, 'runs': 8, 'png_csv_pairs_byte_identical_to_archive': 8,
                       'summary': comparison['summary']})

    result = {'benchmark_source_commit': source, 'benchmark_source_tree': git('rev-parse', f'{source}^{{tree}}'),
              'reviewed_base': BASE, 'benchmark_source_sha256': hashes, 'binary_sha256': BINARY_SHA256,
              'qualification_source_commit': qualified_source,
              'qualification_source_sha256': qualified_hashes,
              'retained_packets_requalified_with_final_validator': 48,
              'protected_paths_unchanged_from_reviewed_base': list(PROTECTED),
              'maximum_predicted_divergence': maximum_predicted, 'maximum_actual_divergence': maximum_actual,
              'checks': checks}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--receipt-output', type=Path, default=ROOT / 'successor-verification.json')
    args = parser.parse_args()
    output = validate_receipt_output(args.receipt_output)
    result = verify()
    result['verifier_sha256'] = sha256(Path(__file__))
    publish_receipt(output, result)
    print('PASS historical result qualification: 48 retained packets and receipt-bound summaries')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError, OverflowError, csv.Error) as exc:
        raise SystemExit(f'FAIL historical result qualification: {exc}') from exc
