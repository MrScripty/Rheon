"""Real Git/evidence mutations in disposable historical checkouts and fresh replay."""
from contextlib import contextmanager
import json
import os
from pathlib import Path
import shutil
import statistics
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
HISTORICAL = {'terminal': '8bc0f54604d98d838b3a1ab2f6fcdfff9af2ac51',
              'result': '416744700a45a8e74de27ca4e25cd00b0931926c',
              'integration': '2d36096f17c494b25c46e64335521cea5e19db5d'}
SCRIPTS = {'terminal': ROOT / 'evidence/publication-terminal-repair/verify.py',
           'result': ROOT / 'evidence/result-qualification/verify.py',
           'integration': ROOT / 'evidence/result-integration-qualification/verify.py'}
FRESH = ROOT / 'evidence/result-integration-qualification/fresh_replay.py'
FRESH_DATA = Path(os.environ.get('RHEON_FRESH_REPLAY', str(ROOT / 'evidence/pr5-evidence-repair/fresh-replay')))
BINARY = Path(os.environ.get('RHEON_BINARY', str(ROOT / 'target/release/rheon')))


def change_json(path, change):
    data = json.loads(path.read_text())
    change(data)
    path.write_text(json.dumps(data))


class EvidenceIntegrity(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if not (FRESH_DATA / 'receipt.json').is_file():
            raise RuntimeError('Produce the real fresh replay first, as documented in README.md')
        cls.pool = tempfile.TemporaryDirectory()
        cls.seeds = {}
        for name, commit in HISTORICAL.items():
            seed = Path(cls.pool.name) / name
            subprocess.run(['git', 'clone', '--shared', '--no-checkout', '--quiet', str(ROOT), str(seed)], check=True)
            subprocess.run(['git', '-C', str(seed), 'checkout', '--quiet', '--detach', commit], check=True)
            cls.seeds[name] = seed

    @classmethod
    def tearDownClass(cls):
        cls.pool.cleanup()

    @contextmanager
    def workspace(self, scope):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'checkout'
            shutil.copytree(self.seeds[scope], root)
            yield root

    def command(self, script, args, cwd):
        flags = ['-O'] if sys.flags.optimize else []
        return subprocess.run([sys.executable, *flags, str(script), *map(str, args)],
                              cwd=cwd, capture_output=True, text=True, timeout=20)

    def check(self, scope, root, *, fail=None, extra=()):
        destination = root / 'qualification.json'
        result = self.command(SCRIPTS[scope], ['--receipt-output', destination, *extra], root)
        if fail:
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIn(fail, result.stderr)
            self.assertNotIn('PASS', result.stdout)
            self.assertFalse(destination.exists(), 'Failed gates must publish no successful receipt')
        else:
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(destination.is_file())
        return result

    def test_clean_historical_result_and_integration_pass_without_relabeling_binary(self):
        for scope in ('result', 'integration'):
            with self.subTest(scope=scope), self.workspace(scope) as root:
                self.check(scope, root)
                receipt = json.loads((root / 'qualification.json').read_text())
                if scope == 'integration':
                    self.assertIsNone(receipt['current_release_sha256'])
                    self.assertIsNone(receipt['fresh_replay_provenance'])
                    self.assertIn('lack producing-binary', receipt['historical_replay_provenance'])

    def test_terminal_guard_rejects_untracked_and_ignored_protected_inventory(self):
        for name in ('src/new.rs', 'tests/new_test.rs', 'src/ignored.rs.bk'):
            with self.subTest(path=name), self.workspace('terminal') as root:
                (root / name).write_text('protected addition')
                if name.endswith('.bk'):
                    result = subprocess.run(['git', 'check-ignore', name], cwd=root, capture_output=True)
                    self.assertEqual(result.returncode, 0)
                self.check('terminal', root, fail='Untracked or ignored protected path additions')

    def test_terminal_guard_retains_tracked_check_and_accepts_clean_inventory(self):
        with self.workspace('terminal') as root:
            # Exercise the actual inventory gate separately: the historical e100
            # executable is unavailable, so no whole historical binary gate is claimed.
            code = ('import importlib.util; '
                    f's=importlib.util.spec_from_file_location("terminal", {str(SCRIPTS["terminal"])!r}); '
                    'm=importlib.util.module_from_spec(s); s.loader.exec_module(m); '
                    'm.check_protected_inventory()')
            flags = ['-O'] if sys.flags.optimize else []
            result = subprocess.run([sys.executable, *flags, '-c', code], cwd=root, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            (root / 'Cargo.toml').write_text('tracked mutation')
            self.check('terminal', root, fail='Protected source/evidence changed')

    def test_integration_stable_manifest_corruption_rejected(self):
        for field, value in [('accepted_time', 5.0), ('managed_simulation_bytes', 1),
                             ('last_divergence_max', 0.3), ('source_off_at', 1),
                             ('real_time_performance_claimed', True)]:
            with self.subTest(field=field), self.workspace('integration') as root:
                change_json(root / 'evidence/result-integration-qualification/replay/demo-16/run.json',
                            lambda d: d.update({field: value}))
                self.check('integration', root, fail='stable replay manifest differs')

    def test_integration_excludes_only_documented_timing_field(self):
        with self.workspace('integration') as root:
            change_json(root / 'evidence/result-integration-qualification/replay/demo-16/run.json',
                        lambda d: d.update(measured_step_seconds=123.0))
            self.check('integration', root)

    def test_integration_png_csv_and_named_inventory_rejected(self):
        for file in ('steps.csv', 'opacity.png'):
            with self.subTest(file=file), self.workspace('integration') as root:
                path = root / 'evidence/result-integration-qualification/replay/demo-16' / file
                path.write_bytes(path.read_bytes() + b'corrupted bytes')
                self.check('integration', root, fail='replay bytes differ')
        with self.workspace('integration') as root:
            change_json(root / 'evidence/result-integration-qualification/replay/receipt.json',
                        lambda d: d[1].update(fixture='evidence/demo-16'))
            self.check('integration', root, fail='fixture inventory differs')

    def test_executable_alone_and_archived_replay_cannot_gain_current_sha_receipt(self):
        with self.workspace('integration') as root:
            self.check('integration', root, extra=['--binary', BINARY],
                       fail='An executable alone cannot qualify retained replay outputs')
        with self.workspace('integration') as root:
            self.check('integration', root, extra=['--binary', BINARY, '--fresh-replay',
                       root / 'evidence/result-integration-qualification/replay'],
                       fail='Fresh producing-binary provenance required')

    def test_separate_fresh_producer_replay_passes_on_fixed_historical_source(self):
        with self.workspace('integration') as root:
            self.check('integration', root, extra=['--binary', BINARY, '--fresh-replay', FRESH_DATA])
            receipt = json.loads((root / 'qualification.json').read_text())
            producer = json.loads((FRESH_DATA / 'receipt.json').read_text())
            self.assertEqual(receipt['current_release_sha256'], producer['producing_binary_sha256'])
            self.assertNotEqual(receipt['current_release_sha256'], receipt['historical_measured_release_sha256'])

    def test_different_real_release_executable_rejected_before_receipt(self):
        with self.workspace('integration') as root:
            other = root / 'other-rheon'
            other.write_bytes(BINARY.read_bytes() + b'\0different-local-binary')
            other.chmod(0o755)
            self.assertEqual(subprocess.check_output([str(other), '--build-info']),
                             subprocess.check_output([str(BINARY), '--build-info']))
            self.check('integration', root, extra=['--binary', other, '--fresh-replay', FRESH_DATA],
                       fail='Replay producing binary differs')

    def test_fresh_output_and_provenance_mutations_rejected_without_qualification_receipt(self):
        for file in ('run.json', 'steps.csv', 'opacity.png', 'receipt.json'):
            with self.subTest(file=file), tempfile.TemporaryDirectory() as tmp:
                output = Path(tmp) / 'fresh'
                shutil.copytree(FRESH_DATA, output)
                if file == 'receipt.json':
                    change_json(output / file, lambda d: d.update(producing_binary_sha256='0'*64))
                elif file == 'run.json':
                    change_json(output / 'demo-16' / file, lambda d: d.update(measured_step_seconds=0.0))
                else:
                    path = output / 'demo-16' / file
                    path.write_bytes(path.read_bytes() + b'corruption')
                destination = Path(tmp) / 'checked.json'
                result = self.command(FRESH, ['check', BINARY, output, '--receipt-output', destination], ROOT)
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn('PASS', result.stdout)
                self.assertFalse(destination.exists())

    def test_coordinated_duration_and_local_summary_changes_cannot_rewrite_history(self):
        with self.workspace('result') as root:
            case = root / 'evidence/result-qualification/benchmarks/16-standard'
            comparison = json.loads((case / 'comparison.json').read_text())
            for run in comparison['runs']:
                run['measured_step_seconds'] *= 2
                change_json(case / run['path'] / 'run.json',
                            lambda d: d.update(measured_step_seconds=run['measured_step_seconds']))
            for summary in comparison['summary']:
                durations = [r['measured_step_seconds'] for r in comparison['runs']
                             if r['implementation'] == summary['implementation'] and not r['warmup']]
                summary.update(median_seconds=statistics.median(durations),
                               min_seconds=min(durations), max_seconds=max(durations))
            (case / 'comparison.json').write_text(json.dumps(comparison))
            self.check('result', root, fail='Timing summary differs from historical case receipt')

    def test_receipt_summary_fields_must_match_by_method_and_type(self):
        for field, value in [('samples', 4), ('samples', 3.0), ('median_seconds', 0.0),
                             ('min_seconds', 0.0), ('max_seconds', 0.0)]:
            with self.subTest(field=field, value=value), self.workspace('result') as root:
                change_json(root / 'evidence/result-qualification/benchmarks/receipt.json',
                            lambda d: d['cases'][0]['summary'][0].update({field: value}))
                self.check('result', root, fail='Timing summary')
        with self.workspace('result') as root:
            change_json(root / 'evidence/result-qualification/benchmarks/receipt.json',
                        lambda d: d['cases'][0]['summary'].reverse())
            self.check('result', root)  # Bind by implementation, not receipt row position.

    def test_historical_result_controls_and_finite_guards_survive_optimization(self):
        for field, value in [('pressure_iteration_limit', 1), ('measured_step_seconds', float('nan'))]:
            with self.subTest(field=field), self.workspace('result') as root:
                change_json(root / 'evidence/result-qualification/benchmarks/16-standard/000-jacobi-pcg-v1/run.json',
                            lambda d: d.update({field: value}))
                self.check('result', root, fail='Historical qualification failed')


if __name__ == '__main__':
    unittest.main()
