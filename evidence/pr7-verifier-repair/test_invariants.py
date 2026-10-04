"""Scoped before/after probes using real historical Git and retained packets."""
from contextlib import contextmanager, redirect_stdout
import csv
import importlib.util
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
BASE = '3b71007bfcc25f5c752dd2911ae7ed3f698dbc61'
BEFORE = os.environ.get('RHEON_PROBE_BEFORE') == '1'
SCOPES = {'cloud': ('7e1a76dd487549a496e04b9305fa531fcc7ab2f9', 'cloud-qualification'),
          'terminal': ('8bc0f54604d98d838b3a1ab2f6fcdfff9af2ac51', 'publication-terminal-repair'),
          'result': ('416744700a45a8e74de27ca4e25cd00b0931926c', 'result-qualification')}


def change_json(path, change):
    data = json.loads(path.read_text())
    change(data)
    path.write_text(json.dumps(data))


class Invariants(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Each Git subprocess must use its temporary checkout, including verifier calls.
        git_env = subprocess.check_output(['git', 'rev-parse', '--local-env-vars'], text=True).splitlines()
        saved_git_env = {name: os.environ[name] for name in git_env if name in os.environ}
        cls.addClassCleanup(os.environ.update, saved_git_env)
        for name in git_env:
            os.environ.pop(name, None)
        cls.pool = tempfile.TemporaryDirectory()
        cls.seeds, cls.modules = {}, {}
        for scope, (commit, directory) in SCOPES.items():
            seed = Path(cls.pool.name) / scope
            subprocess.run(['git', 'clone', '--shared', '--no-checkout', '--quiet', str(ROOT), str(seed)], check=True)
            subprocess.run(['git', '-C', str(seed), 'checkout', '--quiet', '--detach', commit], check=True)
            cls.seeds[scope] = seed
            script = ROOT / 'evidence' / directory / 'verify.py'
            if BEFORE:
                path = Path(cls.pool.name) / (scope + '_before.py')
                path.write_bytes(subprocess.check_output(['git', 'show', f'{BASE}:evidence/{directory}/verify.py'], cwd=ROOT))
                script = path
            spec = importlib.util.spec_from_file_location('audit_' + scope, script)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            cls.modules[scope] = module

    @classmethod
    def tearDownClass(cls):
        cls.pool.cleanup()

    @contextmanager
    def workspace(self, scope):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'checkout'
            shutil.copytree(self.seeds[scope], root)
            original = Path.cwd()
            os.chdir(root)
            try:
                yield root
            finally:
                os.chdir(original)

    def reject_or_confirm_gap(self, call, label):
        stdout = io.StringIO()
        with redirect_stdout(stdout):
            if BEFORE:
                call()
            else:
                with self.assertRaises(ValueError):
                    call()
        if BEFORE:
            print('CONFIRMED BEFORE:', label)
        else:
            self.assertNotIn('PASS', stdout.getvalue())

    def cloud(self, root):
        self.modules['cloud'].verify(root / 'evidence/cloud-qualification', root, root)

    def mutate_diagnostic(self, root, field, value):
        case = root / 'evidence/cloud-qualification/benchmarks/16-standard'
        comparison = json.loads((case / 'comparison.json').read_text())
        for run in comparison['runs']:
            if run['implementation'] != 'jacobi-pcg-v1':
                continue
            path = case / run['path'] / 'steps.csv'
            reader = csv.DictReader(io.StringIO(path.read_text()))
            fields, rows = reader.fieldnames, list(reader)
            rows[0][field] = str(value)
            with path.open('w', newline='') as stream:
                writer = csv.DictWriter(stream, fieldnames=fields)
                writer.writeheader()
                writer.writerows(rows)
            if field == 'pressure_iterations':
                run['pressure_iterations'] = sum(int(row[field]) for row in rows)
        if field == 'pressure_iterations':
            count = comparison['runs'][0]['pressure_iterations']
            comparison['summary'][0]['pressure_iterations'] = count
            change_json(root / 'evidence/cloud-qualification/benchmarks/receipt.json',
                        lambda d: d['cases'][0]['summary'][0].update(pressure_iterations=count))
        (case / 'comparison.json').write_text(json.dumps(comparison))

    def test_01_consistent_negative_diagnostics(self):
        for field in ('courant', 'actual_divergence_max', 'pressure_iterations',
                      'full_residual_max', 'tracer_integral', 'kinetic_energy'):
            with self.subTest(field=field), self.workspace('cloud') as root:
                self.mutate_diagnostic(root, field, -1)
                self.reject_or_confirm_gap(lambda: self.cloud(root), 'same negative '+field+' in all four method repeats accepted')

    def test_02_incomplete_replay_receipt(self):
        for mutation in (lambda d: d.clear(), lambda d: d[0]['byte_identical_sha256'].clear()):
            with self.workspace('cloud') as root:
                change_json(root / 'evidence/cloud-qualification/replay/receipt.json', mutation)
                self.reject_or_confirm_gap(lambda: self.cloud(root), 'empty fixture inventory or artifact hash map accepted')

    def test_03_staged_protected_change_hidden_by_restored_worktree(self):
        for scope in ('terminal', 'result'):
            with self.subTest(scope=scope), self.workspace(scope) as root:
                original = (root / 'Cargo.toml').read_bytes()
                (root / 'Cargo.toml').write_bytes(original+b'\n# staged mutation\n')
                subprocess.run(['git', 'add', 'Cargo.toml'], check=True)
                (root / 'Cargo.toml').write_bytes(original)
                self.assertEqual(subprocess.check_output(['git', 'diff', self.modules[scope].BASE, '--', 'Cargo.toml']), b'')
                call = self.modules[scope].check_protected_inventory if scope == 'terminal' else self.modules[scope].verify
                self.reject_or_confirm_gap(call, scope+' accepted staged protected content with BASE bytes restored')

    def test_04_terminal_historical_checkout_identity(self):
        with self.workspace('terminal') as root:
            with (root / 'docs/COMPARISON.md').open('a') as stream:
                stream.write('\nLocal wrong-source qualification probe.\n')
            subprocess.run(['git', 'add', 'docs/COMPARISON.md'], check=True)
            subprocess.run(['git', '-c', 'user.name=Local Probe', '-c', 'user.email=probe@example.invalid',
                            'commit', '--quiet', '-m', 'temporary wrong repair identity'], check=True)
            module = self.modules['terminal']
            original_read = Path.read_bytes
            class BinaryReached(Exception):
                pass
            def read(path):
                if path == Path('target/release/rheon'):
                    raise BinaryReached('Stopped before unavailable historical binary; no full gate claimed')
                return original_read(path)
            with mock.patch.object(Path, 'read_bytes', read):
                if BEFORE:
                    with self.assertRaises(BinaryReached):
                        module.verify()
                    print('CONFIRMED BEFORE: different committed repair sources pass source phase; stopped before binary gate')
                else:
                    with self.assertRaisesRegex(ValueError, 'Historical checkout'):
                        module.verify()

    def main(self, scope, output, *, receipt=None):
        module = self.modules[scope]
        with mock.patch.object(sys, 'argv', ['verify.py', '--receipt-output', str(output)]):
            if receipt is None:
                module.main()
            else:
                with mock.patch.object(module, 'verify', return_value=receipt):
                    module.main()

    def test_05_protected_receipt_output(self):
        for scope in ('terminal', 'result'):
            with self.subTest(scope=scope), self.workspace(scope) as root:
                output = root / 'proofs/receipt-probe.json'
                # Terminal publication is isolated with its actual archived receipt;
                # this is not a fabricated pass of its unavailable e100 binary gate.
                receipt = json.loads((root / 'evidence/publication-terminal-repair/receipt.json').read_text()) if scope == 'terminal' else None
                self.reject_or_confirm_gap(lambda: self.main(scope, output, receipt=receipt), scope+' publisher created protected addition while claiming unchanged paths')
                self.assertEqual(output.exists(), BEFORE)

    def test_06_untracked_result_protected_addition(self):
        with self.workspace('result') as root:
            (root / 'src/new-untracked.rs').write_text('untracked protected addition')
            self.reject_or_confirm_gap(self.modules['result'].verify, 'result verifier qualified an untracked protected addition')

    def test_07_nonfinite_serialization_residue(self):
        with self.workspace('result') as root:
            change_json(root / 'evidence/result-qualification/benchmarks/16-standard/comparison.json',
                        lambda d: d['summary'][0].update(extra=float('nan')))
            output = root / 'receipt-probe.json'
            with redirect_stdout(io.StringIO()), self.assertRaises(ValueError):
                self.main('result', output)
            if BEFORE:
                self.assertTrue(output.exists())
                self.assertEqual(output.read_bytes(), b'')
                print('CONFIRMED BEFORE: serialization failure left empty exclusive destination; availability defect, no false PASS')
            else:
                self.assertFalse(output.exists())
                change_json(root / 'evidence/result-qualification/benchmarks/16-standard/comparison.json',
                            lambda d: d['summary'][0].pop('extra'))
                with redirect_stdout(io.StringIO()):
                    self.main('result', output)
                self.assertTrue(json.loads(output.read_text())['checks'])

    @unittest.skipIf(BEFORE, 'additional after-repair coverage')
    def test_replay_missing_duplicate_unexpected_fixtures_and_hash_keys(self):
        mutations = [lambda d: d.pop(), lambda d: d.append(d[0]),
                     lambda d: d[1].update(fixture=d[0]['fixture']),
                     lambda d: d[0].update(fixture='evidence/unexpected'),
                     lambda d: d[0]['byte_identical_sha256'].pop('opacity.png'),
                     lambda d: d[0]['byte_identical_sha256'].update(extra='0'*64)]
        for mutation in mutations:
            with self.workspace('cloud') as root:
                change_json(root / 'evidence/cloud-qualification/replay/receipt.json', mutation)
                self.reject_or_confirm_gap(lambda: self.cloud(root), '')
        with self.workspace('cloud') as root:
            change_json(root / 'evidence/cloud-qualification/replay/receipt.json', lambda d: d.reverse())
            with redirect_stdout(io.StringIO()):
                self.cloud(root)

    @unittest.skipIf(BEFORE, 'additional after-repair coverage')
    def test_zero_magnitude_diagnostics_remain_admissible(self):
        for field in ('courant', 'actual_divergence_max', 'full_residual_max', 'tracer_integral', 'kinetic_energy'):
            with self.workspace('cloud') as root:
                self.mutate_diagnostic(root, field, 0)
                with redirect_stdout(io.StringIO()):
                    self.cloud(root)

    @unittest.skipIf(BEFORE, 'additional after-repair coverage')
    def test_untracked_ignored_and_tracked_result_inventory(self):
        for name in ('tests/new.rs', 'src/ignored.rs.bk', 'proofs/new.lean'):
            with self.workspace('result') as root:
                (root / name).write_text('protected addition')
                self.reject_or_confirm_gap(self.modules['result'].verify, '')
        with self.workspace('result') as root:
            (root / 'Cargo.toml').write_text('tracked modification')
            self.reject_or_confirm_gap(self.modules['result'].verify, '')

    @unittest.skipIf(BEFORE, 'additional after-repair coverage')
    def test_output_symlink_aliases_rejected_before_qualification(self):
        for scope in ('terminal', 'result'):
            with self.workspace(scope) as root:
                (root / 'alias').symlink_to(root / 'proofs', target_is_directory=True)
                module = self.modules[scope]
                with mock.patch.object(module, 'verify') as verify:
                    with self.assertRaises(ValueError):
                        self.main(scope, root / 'alias/output.json')
                    verify.assert_not_called()
                self.assertFalse((root / 'proofs/output.json').exists())

    @unittest.skipIf(BEFORE, 'additional after-repair coverage')
    def test_failed_write_cleanup_and_existing_operator_files(self):
        for scope in ('terminal', 'result'):
            with self.workspace(scope) as root:
                module = self.modules[scope]
                output = root / 'receipt.json'
                payload = {'scope': 'isolated receipt writer test; not historical gate evidence'}
                original_open = Path.open
                class FailedWriter:
                    def __init__(self, stream): self.stream = stream
                    def fileno(self): return self.stream.fileno()
                    def __enter__(self): return self
                    def __exit__(self, *args): return self.stream.__exit__(*args)
                    def write(self, data):
                        self.stream.write(data[:10])
                        self.stream.flush()
                        raise OSError('injected partial write failure')
                def open_file(path, *args, **kwargs):
                    stream = original_open(path, *args, **kwargs)
                    return FailedWriter(stream) if path == output and args == ('x',) else stream
                with mock.patch.object(Path, 'open', open_file), self.assertRaises(OSError):
                    module.publish_receipt(output, payload)
                self.assertFalse(output.exists())
                module.publish_receipt(output, payload)
                original = output.read_bytes()
                with self.assertRaises(FileExistsError):
                    module.publish_receipt(output, payload)
                self.assertEqual(output.read_bytes(), original)
                link = root / 'operator-symlink.json'
                link.symlink_to(root / 'absent-target.json')
                with self.assertRaises(FileExistsError):
                    module.publish_receipt(link, payload)
                self.assertTrue(link.is_symlink())
                self.assertFalse((root / 'absent-target.json').exists())


if __name__ == '__main__':
    unittest.main()
