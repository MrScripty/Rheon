"""Discriminating source/evidence admission tests, independent of pass counts."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('step_qualification', Path(__file__).with_name('qualify_aligned_stokes.py'))
Q = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(Q)


class QualificationGateTests(unittest.TestCase):
    def test_existing_evidence_cannot_be_reused(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, 'must be new'):
                Q.external_new_directory(Path(directory))

    def test_repository_output_refused(self):
        with self.assertRaisesRegex(ValueError, 'outside Git'):
            Q.external_new_directory(Q.ROOT / 'discarded-step-evidence')

    def test_failed_command_never_becomes_success(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(RuntimeError, 'failed'):
                Q.run(['python3', '-c', 'raise SystemExit(3)'], Path(directory), 'deliberate-refusal')
            self.assertTrue((Path(directory) / 'deliberate-refusal.log').is_file())

    def test_source_snapshot_tracks_same_commit_byte_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / 'source.rs'
            path.write_text('before')
            responses = {'ls-files': 'source.rs', 'rev-parse': 'stable-id', 'status': ''}
            with patch.object(Q, 'ROOT', root), patch.object(Q, 'git', side_effect=lambda *args: responses[args[0]]):
                before = Q.source_snapshot()
                path.write_text('after')
                after = Q.source_snapshot()
            self.assertEqual(before['head'], after['head'])
            self.assertNotEqual(before['sha256'], after['sha256'])

    def test_source_snapshot_tracks_removed_file_and_new_path(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / 'source.rs'
            path.write_text('source')
            responses = {'ls-files': 'source.rs', 'rev-parse': 'stable-id', 'status': ''}
            with patch.object(Q, 'ROOT', root), patch.object(Q, 'git', side_effect=lambda *args: responses[args[0]]):
                before = Q.source_snapshot()
                path.unlink()
                after = Q.source_snapshot()
                responses['ls-files'] = 'source.rs\nnew.rs'
                added = Q.source_snapshot()
            self.assertNotEqual(before, after)
            self.assertNotEqual(after, added)


if __name__ == '__main__':
    unittest.main()
