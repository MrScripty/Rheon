"""Real production captures required; never synthesize a passing baseline."""
import csv
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

import numpy as np


@unittest.skipUnless(os.environ.get('RHEON_PRODUCTION_CAPTURE') and os.environ.get('RHEON_CANDIDATE'),
                     'requires external production capture and exact candidate source')
class ProductionTransition(unittest.TestCase):
    def verify(self, directory):
        command = [sys.executable] + (['-O'] if sys.flags.optimize else [])
        command += [str(Path(__file__).with_name('verify_production_transition.py')), str(directory),
                    str(Path(os.environ['RHEON_CANDIDATE']) / 'tools')]
        return subprocess.run(command, capture_output=True, text=True, check=False)

    def test_valid_and_corrupted_captures(self):
        source = Path(os.environ['RHEON_PRODUCTION_CAPTURE'])
        valid = self.verify(source)
        self.assertEqual(valid.returncode, 0, valid.stderr)
        for mutation in ('duplicate_frame', 'false_time', 'false_divergence', 'false_provider', 'pressure_gauge'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary) / 'capture'
                shutil.copytree(source, root, copy_function=os.link)
                if mutation == 'duplicate_frame':
                    data = (root / '001.bin').read_bytes()
                    (root / '002.bin').unlink()
                    (root / '002.bin').write_bytes(data)
                elif mutation == 'pressure_gauge':
                    path = root / '002.bin'
                    values = np.fromfile(path, dtype='<f8')
                    values[:32**3] += 1.0
                    path.unlink()
                    values.tofile(path)
                else:
                    path = root / 'frames.csv'
                    with path.open() as stream:
                        reader = csv.DictReader(stream)
                        fields, rows = reader.fieldnames, list(reader)
                    key = {'false_time': 'time', 'false_divergence': 'divergence',
                           'false_provider': 'selected'}[mutation]
                    rows[1][key] = '123'
                    path.unlink()
                    with path.open('w') as stream:
                        writer = csv.DictWriter(stream, fieldnames=fields)
                        writer.writeheader()
                        writer.writerows(rows)
                result = self.verify(root)
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertIn('ValueError', result.stderr)


if __name__ == '__main__':
    unittest.main()
