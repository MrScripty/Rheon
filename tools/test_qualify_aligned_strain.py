"""Executable refusal probes; synthetic sources never qualify native physics."""
from pathlib import Path
import argparse
import json
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import qualify_aligned_strain as q


class QualificationGates(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='rheon-strain-gates-')
        self.addCleanup(self.temp.cleanup)
        self.parent = Path(self.temp.name)
        self.source = self.parent / 'source'
        self.source.mkdir()
        subprocess.run(['git', 'init', '-q', str(self.source)], check=True)
        for name in ['tests/aligned_strain_contract.rs', 'tools/test_aligned_strain_oracle.py',
                     'tools/test_qualify_aligned_strain.py', 'examples/aligned_strain.rs', 'marker']:
            p = self.source / name
            p.parent.mkdir(exist_ok=True)
            p.write_text('gate fixture only\n')
        subprocess.run(['git', 'add', '.'], cwd=self.source, check=True)
        subprocess.run(['git', '-c', 'user.name=Gate Probe', '-c',
                        'user.email=gate@example.invalid', 'commit', '-qm', 'gate fixture'],
                       cwd=self.source, check=True)
        self.root_patch = patch.object(q, 'ROOT', self.source)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)
        self.base_patch = patch.object(q, 'BASE', q.git('rev-parse', 'HEAD'))
        self.base_patch.start()
        self.addCleanup(self.base_patch.stop)

    def args(self, output):
        return argparse.Namespace(output=output, allow_dirty=False, lean=False)

    def test_git_destinations_refuse_before_creation(self):
        other = self.parent / 'other'
        subprocess.run(['git', 'init', '-q', str(other)], check=True)
        for root in [self.source, other]:
            output = root / 'evidence'
            with self.assertRaisesRegex(ValueError, 'outside Git'):
                q.qualify(self.args(output))
            self.assertFalse(output.exists())

    def test_dirty_source_requires_explicit_intermediate_mode(self):
        (self.source / 'marker').write_text('changed\n')
        output = self.parent / 'evidence'
        with self.assertRaisesRegex(ValueError, 'clean committed'):
            q.qualify(self.args(output))
        self.assertFalse(output.exists())

    def test_existing_evidence_is_never_overwritten(self):
        output = self.parent / 'evidence'
        output.mkdir()
        (output / 'marker').write_text('retained\n')
        with self.assertRaises(FileExistsError):
            q.qualify(self.args(output))
        self.assertEqual((output / 'marker').read_text(), 'retained\n')

    def test_failed_command_retains_failed_receipt(self):
        output = self.parent / 'evidence'
        original_run = q.run

        def fail(command, directory, name, cwd=q.ROOT):
            original_run([sys.executable, '-c', 'raise SystemExit(7)'], directory, name, cwd)

        with patch.object(q, 'run', fail), self.assertRaises(RuntimeError):
            q.qualify(self.args(output))
        receipt = json.loads((output / 'qualification.json').read_text())
        self.assertEqual(receipt['status'], 'failed')
        self.assertNotIn('executable_sha256', receipt)

    def test_source_addition_deletion_and_byte_changes_break_binding(self):
        head = q.git('rev-parse', 'HEAD')
        status = q.git('status', '--porcelain')
        paths = q.git('ls-files', '--cached', '--others', '--exclude-standard').splitlines()
        hashes = {p: q.digest(self.source / p) for p in paths}
        self.assertTrue(q.unchanged(head, status, paths, hashes))
        added = self.source / 'new-source'
        added.write_text('new\n')
        self.assertFalse(q.unchanged(head, status, paths, hashes))
        added.unlink()
        marker = self.source / 'marker'
        marker.unlink()
        self.assertFalse(q.unchanged(head, status, paths, hashes))
        marker.write_text('changed\n')
        self.assertFalse(q.unchanged(head, status, paths, hashes))
        dirty_status = q.git('status', '--porcelain')
        dirty_hashes = {p: q.digest(self.source / p) for p in paths}
        self.assertTrue(q.unchanged(head, dirty_status, paths, dirty_hashes))
        marker.write_text('changed again\n')
        self.assertEqual(q.git('status', '--porcelain'), dirty_status)
        self.assertFalse(q.unchanged(head, dirty_status, paths, dirty_hashes))


if __name__ == '__main__':
    unittest.main()
