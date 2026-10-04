"""Exercise archived-evidence scope and preservation checks in real Git clones."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[4]
EVIDENCE = Path('docs/research-book/evidence/manifest-provenance-repair')
ARCHIVE = 'd72db7c1bd3fc10b27978c6dcc8ec10709195ace'
SOURCE = 'd4147b311ebfd1b05919251dab703a628eb32316'


class ArchivedEvidence(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.repo = Path(temporary.name) / 'repo'
        subprocess.run(['git', 'clone', '--quiet', '--shared', str(REPO), str(self.repo)],
                       check=True, capture_output=True)
        self.git('checkout', '--quiet', '--detach', ARCHIVE)
        shutil.copyfile(REPO / EVIDENCE / 'verify.py', self.repo / EVIDENCE / 'verify.py')
        self.receipt = (self.repo / EVIDENCE / 'receipt.json').read_bytes()

    def git(self, *args):
        return subprocess.run(['git', *args], cwd=self.repo, check=True,
                              text=True, capture_output=True)

    def verify(self, optimized):
        result = subprocess.run([sys.executable, '-B', *(['-O'] if optimized else []),
                                 str(EVIDENCE / 'verify.py')], cwd=self.repo,
                                text=True, capture_output=True, timeout=15)
        self.assertEqual((self.repo / EVIDENCE / 'receipt.json').read_bytes(), self.receipt)
        return result

    def reject_in_both_modes(self, diagnostic):
        for optimized in (False, True):
            with self.subTest(optimized=optimized):
                result = self.verify(optimized)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(diagnostic, result.stderr)
                self.assertNotIn('PASS archived', result.stdout)

    def test_archived_record_passes_without_rewriting_receipt(self):
        for optimized in (False, True):
            with self.subTest(optimized=optimized):
                result = self.verify(optimized)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn('PASS archived source '+SOURCE+' and evidence '+ARCHIVE, result.stdout)
                self.assertIn('no current-source test qualification', result.stdout)

    def test_untracked_and_ignored_protected_additions_are_rejected(self):
        for name, ignored in (('proofs/Rheon/Unexpected.lean', False),
                              ('proofs/Rheon/unreviewed.log', True),
                              ('docs/research-book/figures/unreviewed.png', True)):
            with self.subTest(path=name):
                path = self.repo / name
                path.write_bytes(b'unreviewed addition\n')
                check = subprocess.run(['git', 'check-ignore', name], cwd=self.repo,
                                       capture_output=True)
                self.assertEqual(check.returncode == 0, ignored)
                self.reject_in_both_modes('Untracked or ignored protected path added')
                path.unlink()

    def test_tracked_protected_changes_are_rejected(self):
        (self.repo / 'proofs/Rheon/Discrete.lean').write_text('-- altered proof\n')
        self.reject_in_both_modes('Protected dependency/scientific source changed')

    def test_altered_historical_log_is_rejected(self):
        path = self.repo / EVIDENCE / 'tests.log'
        # Keep the old superficial success markers while changing the record.
        path.write_text(path.read_text().replace('test_', 'changed_test_', 1))
        self.reject_in_both_modes('Historical evidence changed: tests.log')

    def test_altered_historical_receipt_is_rejected(self):
        path = self.repo / EVIDENCE / 'receipt.json'
        path.write_text(path.read_text().replace(SOURCE, '0' * 40))
        self.receipt = path.read_bytes()
        self.reject_in_both_modes('Historical evidence changed: receipt.json')

    def test_later_failing_source_is_never_qualified_by_historical_logs(self):
        gate = self.repo / 'proofs/scripts/check_sources.py'
        gate.write_text(gate.read_text()+"\nraise ValueError('later source intentionally fails')\n")
        self.git('add', 'proofs/scripts/check_sources.py')
        self.git('-c', 'user.name=Evidence test', '-c', 'user.email=evidence-test@example.invalid',
                 'commit', '--quiet', '-m', 'Test later failing source')
        current = self.git('rev-parse', 'HEAD').stdout.strip()
        for optimized in (False, True):
            with self.subTest(optimized=optimized):
                gate_result = subprocess.run([sys.executable, '-B', *(['-O'] if optimized else []),
                                              str(gate)], cwd=self.repo, capture_output=True,
                                             text=True, timeout=10,
                                             env={**os.environ, 'PYTHONDONTWRITEBYTECODE': '1'})
                self.assertNotEqual(gate_result.returncode, 0)
                self.assertIn('later source intentionally fails', gate_result.stderr)
                result = self.verify(optimized)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn('PASS archived source '+SOURCE, result.stdout)
                self.assertNotIn(current, result.stdout)
                self.assertIn('no current-source test qualification', result.stdout)


if __name__ == '__main__':
    unittest.main()
