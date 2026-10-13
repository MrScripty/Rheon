"""Artifact and receipt rejection tests; real replay requires explicit local inputs."""
import os
from pathlib import Path
import shutil
import tempfile
import unittest

from export_ai_pressure import export


class ArtifactExport(unittest.TestCase):
    def test_wrong_checkpoint_is_rejected_before_deserialization(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'bad.pt'
            path.write_bytes(b'not a checkpoint')
            with self.assertRaisesRegex(ValueError, 'SHA256'):
                export(path)


@unittest.skipUnless(os.environ.get('RHEON_AI_CHECKPOINT') and os.environ.get('RHEON_AI_RUN'),
                     'requires actual checkpoint and native comparison output')
class ActualReplay(unittest.TestCase):
    def setUp(self):
        from verify_ai_pressure import verify
        self.verify = verify
        self.checkpoint = os.environ['RHEON_AI_CHECKPOINT']
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)/'run'
        shutil.copytree(os.environ['RHEON_AI_RUN'], self.directory)

    def test_actual_forward_pass(self):
        self.assertEqual(self.verify(self.checkpoint, self.directory)['cases'], 36)
        self.assertEqual(len(export(self.checkpoint)), 270336)

    def test_mutated_native_prediction_fails(self):
        path = self.directory/'native-pressure.f64'
        path.write_bytes(bytes(path.stat().st_size))
        with self.assertRaisesRegex(ValueError, 'discrepancy'):
            self.verify(self.checkpoint, self.directory)

    def test_unreported_fallback_fails(self):
        path = self.directory/'comparison.csv'
        path.write_text(path.read_text().replace('lab-modal-pressure-v1', 'jacobi-pcg-v1'))
        with self.assertRaisesRegex(ValueError, 'fallback'):
            self.verify(self.checkpoint, self.directory)

    def test_duplicate_case_fails(self):
        path = self.directory/'comparison.csv'
        path.write_text(path.read_text().replace('901,', '900,'))
        with self.assertRaisesRegex(ValueError, 'incomplete or duplicate'):
            self.verify(self.checkpoint, self.directory)


if __name__ == '__main__':
    unittest.main()
