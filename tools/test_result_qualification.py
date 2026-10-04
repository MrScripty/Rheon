"""Adversarial result packets produced by the real, unchanged release CLI."""
import csv
import json
import math
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock
import zlib

from rheon_compare import Comparison

BINARY = Path('target/release/rheon').resolve()


class ResultQualification(unittest.TestCase):
    def completed_child(self, output, *, accuracy='standard'):
        job = Comparison(BINARY, output, ['jacobi-pcg-v1'], size=4, steps=3,
                         repeats=1, accuracy=accuracy)
        job.poll()
        job.process.wait(timeout=10)
        return job

    def cleanup(self, job):
        if job.process is not None:
            if job.process.poll() is None:
                job.process.kill()
            job.process.wait(timeout=5)
        if job.log is not None:
            job.log.close()

    def reject(self, *, manifest=None, rows=None, image=None, csv_text=None):
        with tempfile.TemporaryDirectory() as root:
            output = Path(root) / 'comparison'
            job = self.completed_child(output)
            try:
                if manifest is not None:
                    path = job.run_dir / 'run.json'
                    data = json.loads(path.read_text())
                    manifest(data)
                    path.write_text(json.dumps(data))
                if rows is not None:
                    path = job.run_dir / 'steps.csv'
                    with path.open(newline='') as f:
                        reader = csv.DictReader(f)
                        fields, series = reader.fieldnames, list(reader)
                    rows(series)
                    with path.open('w', newline='') as f:
                        writer = csv.DictWriter(f, fieldnames=fields)
                        writer.writeheader()
                        writer.writerows(series)
                if csv_text is not None:
                    path = job.run_dir / 'steps.csv'
                    path.write_text(csv_text(path.read_text()))
                if image is not None:
                    image(job.run_dir / 'opacity.png')
                self.assertEqual(job.poll(), 'failed')
                self.assertEqual(job.poll(), 'failed')
                self.assertEqual(job.index, 1)
                self.assertEqual(job.rows, [])
                self.assertIsNone(job.result)
                self.assertIsNone(job.process)
                self.assertIsNone(job.log)
                self.assertFalse((output / 'comparison.json').exists())
                status = json.loads((output / 'status.json').read_text())
                self.assertEqual(status['status'], 'failed')
                self.assertTrue(status['error'])
                self.assertTrue((job.run_dir / 'steps.csv').exists())
            finally:
                self.cleanup(job)

    def test_changed_workload_settings_are_rejected(self):
        for key, value in (
            ('schema_version', 3), ('schema_version', 2.0),
            ('model', 'different forcing model'), ('source_off_at', 0),
            ('pressure_absolute_residual', 1.0), ('pressure_iteration_limit', 1),
            ('managed_budget_bytes', 32 * 1024 * 1024), ('size', 4.0), ('steps', 3.0),
        ):
            with self.subTest(key=key, value=value):
                self.reject(manifest=lambda data: data.__setitem__(key, value))

    def test_missing_workload_settings_are_rejected(self):
        for key in ('schema_version', 'model', 'source_off_at', 'pressure_absolute_residual',
                    'pressure_iteration_limit', 'managed_budget_bytes'):
            with self.subTest(key=key):
                self.reject(manifest=lambda data: data.pop(key))

    def test_child_command_pins_iteration_and_memory_limits(self):
        with tempfile.TemporaryDirectory() as root:
            with mock.patch('rheon_compare.subprocess.Popen', wraps=subprocess.Popen) as spawn:
                job = self.completed_child(Path(root) / 'comparison')
            try:
                command = spawn.call_args.args[0]
                for option, value in (('--max-iterations', '2000'), ('--memory-mib', '64')):
                    self.assertIn(option, command)
                    self.assertEqual(command[command.index(option) + 1], value)
            finally:
                self.cleanup(job)

    def test_nonfinite_or_negative_step_diagnostics_are_rejected(self):
        for field in ('time', 'dt', 'full_residual_max', 'actual_divergence_max',
                      'courant', 'tracer_integral', 'kinetic_energy'):
            for value in ('NaN', 'inf', '-inf', '-0.01'):
                with self.subTest(field=field, value=value):
                    self.reject(rows=lambda rows: rows[0].__setitem__(field, value))

    def test_step_order_time_and_admission_limits_are_checked(self):
        for field, value in (('step', '0'), ('step', '1.0'), ('time', '0.03'),
                             ('dt', '0'), ('dt', '0.04'), ('pressure_iterations', '-1'),
                             ('pressure_iterations', '2001'), ('pressure_iterations', '1.5'),
                             ('full_residual_max', '1e-6'),
                             ('actual_divergence_max', '1.01e-5'), ('courant', '1.01')):
            with self.subTest(field=field, value=value):
                self.reject(rows=lambda rows: rows[0].__setitem__(field, value))
        self.reject(rows=lambda rows: rows.reverse())
        self.reject(rows=lambda rows: rows[0].__setitem__('kinetic_energy', ''))

    def test_manifest_diagnostics_match_the_accepted_steps(self):
        for field, value in (
            ('accepted_time', 0.07), ('last_divergence_max', 0.5),
            ('accepted_time', math.nan), ('last_divergence_max', math.inf),
            ('measured_step_seconds', -1), ('measured_step_seconds', math.nan),
            ('measured_step_seconds', True),
            ('managed_simulation_bytes', -1), ('managed_simulation_bytes', 0),
            ('managed_simulation_bytes', 64 * 1024 * 1024 + 1),
            ('managed_simulation_bytes', 4096.0), ('raw_export_pixel_bytes', 0),
            ('raw_export_pixel_bytes', 16 * 1024 * 1024 + 1),
            ('whole_process_memory_cap_claimed', True), ('real_time_performance_claimed', True),
        ):
            with self.subTest(field=field, value=value):
                self.reject(manifest=lambda data: data.__setitem__(field, value))
        for field in ('accepted_time', 'last_divergence_max', 'measured_step_seconds',
                      'managed_simulation_bytes', 'raw_export_pixel_bytes',
                      'whole_process_memory_cap_claimed', 'real_time_performance_claimed'):
            with self.subTest(missing=field):
                self.reject(manifest=lambda data: data.pop(field))

    def test_unrepresentable_manifest_number_has_terminal_failure(self):
        self.reject(manifest=lambda data: data.__setitem__('measured_step_seconds', 10 ** 400))

    def test_missing_or_incomplete_png_is_rejected(self):
        self.reject(image=lambda path: path.unlink())
        self.reject(image=lambda path: path.write_bytes(b'not a PNG'))
        self.reject(image=lambda path: path.write_bytes(path.read_bytes()[:33]))
        def wrong_dimensions(path):
            image = bytearray(path.read_bytes())
            image[16:20] = (8).to_bytes(4, 'big')
            image[29:33] = zlib.crc32(image[12:29]).to_bytes(4, 'big')
            path.write_bytes(image)
        self.reject(image=wrong_dimensions)
        self.reject(image=lambda path: path.write_bytes(path.read_bytes()[:-13]
                                                      + b'\x00' + path.read_bytes()[-12:]))

    def test_malformed_csv_has_terminal_failure(self):
        for rewrite in (
            lambda text: text.replace('kinetic_energy', 'unknown_diagnostic', 1),
            lambda text: '\n'.join(text.splitlines()[:-1]) + '\n',
            lambda text: text.replace('\n', ',extra\n', 2),
            lambda text: text.replace('step,', '"step,', 1),
        ):
            with self.subTest(rewrite=rewrite):
                self.reject(csv_text=rewrite)

    def test_valid_zero_and_active_source_runs_for_both_methods(self):
        with tempfile.TemporaryDirectory() as root:
            for size, steps in ((1, 1), (4, 20)):
                for accuracy in ('standard', 'tight'):
                    with self.subTest(size=size, accuracy=accuracy):
                        job = Comparison(BINARY, Path(root) / f'{size}-{accuracy}',
                                         ['jacobi-pcg-v1', 'sgs-pcg-v1'], size=size,
                                         steps=steps, repeats=1, accuracy=accuracy)
                        try:
                            while job.poll() == 'running':
                                job.process.wait(timeout=10)
                            self.assertEqual(job.status, 'completed', job.error)
                            self.assertTrue(job.result['equal_accepted_time'])
                            self.assertEqual(len(job.rows), 4)
                        finally:
                            self.cleanup(job)

    def test_publication_failure_has_no_owned_success_or_summary(self):
        original_write, original_replace = Path.write_text, Path.replace
        for operation in ('write', 'replace', 'status'):
            with self.subTest(operation=operation), tempfile.TemporaryDirectory() as root:
                output = Path(root) / 'comparison'
                job = self.completed_child(output)
                try:
                    job.poll()  # Accept warmup and start the measured child.
                    job.process.wait(timeout=10)
                    def write(path, *args, **kwargs):
                        if path.name == 'comparison.tmp' and operation == 'write':
                            raise OSError('injected summary write failure')
                        if path.name == 'status.tmp' and operation == 'status' and job.status == 'completed':
                            raise OSError('injected completion status write failure')
                        return original_write(path, *args, **kwargs)
                    def replace(path, *args, **kwargs):
                        if path.name == 'comparison.tmp' and operation == 'replace':
                            raise OSError('injected summary rename failure')
                        return original_replace(path, *args, **kwargs)
                    with mock.patch.object(Path, 'write_text', write), mock.patch.object(Path, 'replace', replace):
                        self.assertEqual(job.poll(), 'failed')
                    self.assertEqual(job.poll(), 'failed')
                    self.assertIsNone(job.result)
                    self.assertIsNone(job.process)
                    self.assertIsNone(job.log)
                    self.assertEqual(len(job.rows), 2)  # Qualified diagnostics remain.
                    self.assertEqual(job.index, 2)
                    self.assertFalse((output / 'comparison.json').exists())
                    self.assertEqual(json.loads((output / 'status.json').read_text())['status'], 'failed')
                finally:
                    self.cleanup(job)

    def test_combined_publication_cleanup_failures_stay_terminal(self):
        original_write, original_unlink = Path.write_text, Path.unlink
        for status_also_fails in (False, True):
            with self.subTest(status_also_fails=status_also_fails), tempfile.TemporaryDirectory() as root:
                output = Path(root) / 'comparison'
                job = self.completed_child(output)
                try:
                    job.poll()
                    job.process.wait(timeout=10)
                    diagnostics = {name: (job.run_dir / name).read_bytes()
                                   for name in ('steps.csv', 'run.json', 'opacity.png')}
                    writes, cleanup_states = [], []
                    def write(path, *args, **kwargs):
                        if path.name == 'status.tmp':
                            writes.append(job.status)
                            if job.status == 'completed':
                                raise OSError('injected primary completion write failure')
                            if status_also_fails and job.status == 'failed':
                                raise OSError('injected failure-status persistence failure')
                        return original_write(path, *args, **kwargs)
                    def unlink(path, *args, **kwargs):
                        if path.name == 'comparison.json':
                            cleanup_states.append(job.status)
                            raise OSError('injected summary cleanup failure')
                        return original_unlink(path, *args, **kwargs)
                    with mock.patch.object(Path, 'write_text', write), mock.patch.object(Path, 'unlink', unlink):
                        self.assertEqual(job.poll(), 'failed')
                        self.assertEqual(job.poll(), 'failed')
                        self.assertEqual(job.poll(), 'failed')
                    self.assertEqual(cleanup_states, ['failed'])
                    self.assertEqual(writes, ['completed', 'failed'])
                    self.assertIn('injected primary completion write failure', job.error)
                    self.assertIn('injected summary cleanup failure', job.error)
                    if status_also_fails:
                        self.assertIn('injected failure-status persistence failure', job.error)
                    self.assertIsNone(job.result)
                    self.assertIsNone(job.process)
                    self.assertIsNone(job.log)
                    self.assertEqual(len(job.rows), 2)
                    self.assertEqual(job.index, 2)
                    self.assertTrue((output / 'comparison.json').exists())
                    for name, data in diagnostics.items():
                        self.assertEqual((job.run_dir / name).read_bytes(), data)
                    status = json.loads((output / 'status.json').read_text())
                    self.assertEqual(status['status'], 'running' if status_also_fails else 'failed')
                    if not status_also_fails:
                        self.assertEqual(status['error'], job.error)
                finally:
                    self.cleanup(job)


if __name__ == '__main__':
    unittest.main()
