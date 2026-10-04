"""Corrupt disposable copies of real retained evidence; never replace measurements."""
from contextlib import contextmanager, redirect_stdout
import csv
import importlib.util
import io
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / 'evidence/cloud-qualification'
HISTORICAL = '7e1a76dd487549a496e04b9305fa531fcc7ab2f9'


def load(name):
    spec = importlib.util.spec_from_file_location('qualification_' + name, EVIDENCE / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


VERIFY, REPLAY, BENCHMARK = (load(name) for name in ('verify', 'replay', 'benchmark'))


def change_json(path, change):
    data = json.loads(path.read_text())
    change(data)
    path.write_text(json.dumps(data))


class QualificationGates(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.seed_temp = tempfile.TemporaryDirectory()
        cls.seed = Path(cls.seed_temp.name)
        target = cls.seed / 'evidence/cloud-qualification'
        shutil.copytree(EVIDENCE, target, ignore=shutil.ignore_patterns('__pycache__'))
        for entry in json.loads((EVIDENCE / 'qualified-source-manifest.json').read_text())['files']:
            path = cls.seed / entry['path']
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(subprocess.check_output(['git', 'show', f'{HISTORICAL}:{entry["path"]}'], cwd=ROOT))
        for name in ('demo-16', 'demo-plume', 'demo-64'):
            shutil.copytree(ROOT / 'evidence' / name, cls.seed / 'evidence' / name)

    @classmethod
    def tearDownClass(cls):
        cls.seed_temp.cleanup()

    @contextmanager
    def workspace(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'checkout'
            shutil.copytree(self.seed, root)
            yield root, root / 'evidence/cloud-qualification'

    def verify(self, root, evidence):
        with redirect_stdout(io.StringIO()):
            VERIFY.verify(evidence, root, root)

    def reject(self, mutation):
        with self.workspace() as (root, evidence):
            mutation(root, evidence)
            with self.assertRaises((ValueError, KeyError, TypeError, OSError)):
                self.verify(root, evidence)

    @staticmethod
    def run_field(evidence, field, value):
        case = evidence / 'benchmarks/16-standard'
        comparison = json.loads((case / 'comparison.json').read_text())
        comparison['runs'][0][field] = value
        path = case / comparison['runs'][0]['path'] / 'run.json'
        change_json(path, lambda d: d.update({field: value}))
        (case / 'comparison.json').write_text(json.dumps(comparison))

    def test_unchanged_retained_artifacts_and_source_pass(self):
        with self.workspace() as (root, evidence):
            self.verify(root, evidence)

    def test_corrupted_source_rejected(self):
        self.reject(lambda root, _: (root / 'Cargo.toml').write_text('corrupted source'))

    def test_current_checkout_cannot_be_labeled_historical(self):
        args = [sys.executable] + (['-O'] if sys.flags.optimize else [])
        result = subprocess.run(args + [str(EVIDENCE / 'verify.py'), '--source-root', str(ROOT)],
                                capture_output=True, text=True, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Historical source hash differs', result.stderr)
        self.assertNotIn('PASS', result.stdout)

    def test_requested_controls_preserved_even_when_embedded_values_match(self):
        changes = {'implementation': 'other-method', 'size': 17, 'steps': 11,
                   'source_off_at': 5, 'requested_dt': 0.03, 'pressure_relative_residual': 1e-8,
                   'pressure_absolute_residual': 1e-10, 'pressure_divergence_limit': 1e-6,
                   'actual_divergence_limit': 1e-4, 'pressure_iteration_limit': 1999}
        for field, value in changes.items():
            with self.subTest(field=field):
                self.reject(lambda _, e: self.run_field(e, field, value))

    def test_nonfinite_untyped_horizon_and_budget_measurements_rejected(self):
        for field, value in [('measured_step_seconds', float('nan')),
                             ('measured_step_seconds', float('inf')),
                             ('measured_step_seconds', 0), ('measured_step_seconds', '0.2'),
                             ('accepted_time', True), ('accepted_time', 0.2),
                             ('last_divergence_max', float('inf')),
                             ('managed_simulation_bytes', True), ('managed_simulation_bytes', 333825),
                             ('managed_budget_bytes', '67108864')]:
            with self.subTest(field=field, value=value):
                self.reject(lambda _, e: self.run_field(e, field, value))

    def test_csv_diagnostic_gates_reject_corruption(self):
        for field, value in [('courant', '1.1'), ('actual_divergence_max', '0.001'),
                             ('full_residual_max', '1'), ('pressure_iterations', '2001'),
                             ('kinetic_energy', 'nan'), ('time', '0.03')]:
            with self.subTest(field=field):
                def mutate(_, evidence):
                    path = evidence / 'benchmarks/16-standard/000-jacobi-pcg-v1/steps.csv'
                    reader = csv.DictReader(io.StringIO(path.read_text()))
                    fields, rows = reader.fieldnames, list(reader)
                    rows[0][field] = value
                    with path.open('w', newline='') as stream:
                        writer = csv.DictWriter(stream, fieldnames=fields)
                        writer.writeheader()
                        writer.writerows(rows)
                self.reject(mutate)

    def test_missing_step_and_unequal_horizon_flag_rejected(self):
        self.reject(lambda _, e: change_json(e / 'benchmarks/16-standard/comparison.json',
                                            lambda d: d.update(equal_accepted_time=False)))
        self.reject(lambda _, e: (e / 'benchmarks/16-standard/000-jacobi-pcg-v1/steps.csv').write_text(
            (e / 'benchmarks/16-standard/000-jacobi-pcg-v1/steps.csv').read_text().rsplit('\n', 2)[0] + '\n'))

    def test_embedded_manifest_and_csv_measurements_must_match(self):
        for field, value in [('measured_step_seconds', 0.1), ('accepted_time', 0.1),
                             ('managed_simulation_bytes', 1), ('last_divergence_max', 0.1),
                             ('pressure_iterations', 1), ('maximum_step_divergence', 0.1)]:
            with self.subTest(field=field):
                self.reject(lambda _, e: change_json(e / 'benchmarks/16-standard/comparison.json',
                    lambda d: d['runs'][0].update({field: value})))

    def test_all_published_summary_fields_recomputed(self):
        for field in ('median_seconds', 'min_seconds', 'max_seconds', 'samples',
                      'managed_simulation_bytes', 'max_divergence', 'pressure_iterations'):
            with self.subTest(field=field):
                self.reject(lambda _, e: change_json(e / 'benchmarks/16-standard/comparison.json',
                    lambda d: d['summary'][0].update({field: 0.0 if field.endswith('seconds')
                                                    or field == 'max_divergence' else 0})))
        self.reject(lambda _, e: change_json(e / 'benchmarks/receipt.json',
                    lambda d: d['cases'][0]['summary'][0].update(median_seconds=0.0)))
        for value in (3.0, True, '3', float('nan')):
            with self.subTest(sample_type=value):
                self.reject(lambda _, e: change_json(e / 'benchmarks/16-standard/comparison.json',
                    lambda d: d['summary'][0].update(samples=value)))

    def test_cli_exits_unsuccessfully_for_corrupted_published_summary(self):
        with self.workspace() as (root, evidence):
            args = [sys.executable] + (['-O'] if sys.flags.optimize else [])
            command = args + [str(evidence / 'verify.py'), '--source-root', str(root)]
            result = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            change_json(evidence / 'benchmarks/16-standard/comparison.json',
                        lambda d: d['summary'][0].update(median_seconds=0.0))
            result = subprocess.run(command, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 1)
            self.assertIn('summary median_seconds', result.stderr)

    def test_warmup_is_excluded_and_methods_cannot_be_regrouped(self):
        with self.workspace() as (root, evidence):
            self.run_field(evidence, 'measured_step_seconds', 999.0)
            self.verify(root, evidence)  # Both warmup copies changed; measured aggregates stay valid.
        for field, value in [('warmup', False), ('repeat', 0), ('repeat', True)]:
            with self.subTest(field=field):
                self.reject(lambda _, e: change_json(e / 'benchmarks/16-standard/comparison.json',
                    lambda d: d['runs'][0].update({field: value})))
        self.reject(lambda _, e: change_json(e / 'benchmarks/16-standard/comparison.json',
                    lambda d: d['summary'].reverse()))

    def test_benchmark_repeat_and_original_replay_bytes_checked(self):
        for file in ('opacity.png', 'steps.csv'):
            for directory in ('benchmarks/16-standard/000-jacobi-pcg-v1', 'replay/demo-16'):
                with self.subTest(file=file, directory=directory):
                    self.reject(lambda _, e: (e / directory / file).write_bytes(
                        (e / directory / file).read_bytes() + b'corruption'))
        self.reject(lambda root, _: (root / 'evidence/demo-16/opacity.png').write_bytes(b'wrong fixture'))
        self.reject(lambda _, e: change_json(e / 'replay/demo-16/run.json', lambda d: d.update(accepted_time=1)))

    def test_fresh_replay_manifest_and_bytes_checked(self):
        for field in ('accepted_time', 'managed_simulation_bytes'):
            with self.workspace() as (root, _):
                fixture = root / 'evidence/demo-16'
                run = root / 'fresh'
                shutil.copytree(fixture, run)
                change_json(run / 'run.json', lambda d: d.update({field: 1}))
                with self.assertRaisesRegex(ValueError, 'stable manifest field differs'):
                    REPLAY.check_run(fixture, run)
        for file in ('opacity.png', 'steps.csv'):
            with self.workspace() as (root, _):
                fixture = root / 'evidence/demo-16'
                run = root / 'fresh'
                shutil.copytree(fixture, run)
                REPLAY.check_run(fixture, run)
                (run / file).write_bytes(b'bad bytes')
                with self.assertRaisesRegex(ValueError, 'fixture bytes differ'):
                    REPLAY.check_run(fixture, run)

    def test_benchmark_unequal_horizons_fail_receipt_and_exit(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / 'fresh'
            job = SimpleNamespace(status='completed', error=None,
                                  result={'summary': [], 'equal_accepted_time': False},
                                  poll=lambda: 'completed')
            with mock.patch.object(BENCHMARK, 'Comparison', return_value=job), \
                 mock.patch.object(BENCHMARK, 'implementations', return_value=[{'id': 'jacobi-pcg-v1'}]), \
                 mock.patch.object(BENCHMARK, 'snapshot', return_value={}), \
                 mock.patch.object(sys, 'argv', ['benchmark.py', str(ROOT / 'Cargo.toml'), str(output)]), \
                 redirect_stdout(io.StringIO()):
                self.assertEqual(BENCHMARK.main(), 1)
            cases = json.loads((output / 'receipt.json').read_text())['cases']
            self.assertEqual(len(cases), 1)
            self.assertEqual(cases[0]['status'], 'failed')
            self.assertEqual(cases[0]['error'], 'Methods accepted different times')


if __name__ == '__main__':
    unittest.main()
