"""Negative records must never qualify timing evidence, including under python -O."""
import csv
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from measurement_checks import read_rows


def trajectory():
    return [dict(step=str(i), accepted='true', startup_s='0.1', step_s='1',
                 pressure_s='0.9', capture_s='0.01', iterations='1',
                 time=str((i+1)/64), dt=str(1/64), residual_l2='0', residual_max='0',
                 divergence='0', mass_error='0', energy='0', pressure_error='0',
                 max_speed='0', bytes='4096', fingerprint='0000000000000000', error='')
            for i in range(16)]


def encode(rows):
    text = io.StringIO()
    writer = csv.DictWriter(text, fieldnames=list(trajectory()[0]))
    writer.writeheader()
    writer.writerows(rows)
    return text.getvalue()


class MeasurementEvidenceTests(unittest.TestCase):
    def test_rejects_nonfinite_negative_and_incomplete_costs(self):
        self.assertEqual(len(read_rows(encode(trajectory()), 16, 'steady')), 16)
        for value in ['NaN', 'inf', '-1']:
            rows = trajectory()
            rows[3]['step_s'] = value
            with self.subTest(value=value), self.assertRaises(ValueError):
                read_rows(encode(rows), 16, 'steady')
        with self.assertRaises(ValueError):
            read_rows(encode(trajectory()[:15]), 16, 'steady')

    def test_rejects_changed_horizon_and_sequence(self):
        for field, value in [('step', '14'), ('dt', '0.001'), ('time', '0.25')]:
            rows = trajectory()
            rows[3][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                read_rows(encode(rows), 16, 'steady')

    def test_failed_attempt_must_preserve_published_state(self):
        rows = trajectory()[:2]
        rows[1].update(accepted='false', time=str(1/64), dt='0', error='bounded failure')
        self.assertEqual(len(read_rows(encode(rows), 16, 'steady')), 2)
        rows[1]['fingerprint'] = '0000000000000001'
        with self.assertRaises(ValueError):
            read_rows(encode(rows), 16, 'steady')

    def test_progress_is_not_final_qualification_even_under_optimization(self):
        script = Path(__file__).with_name('analyze_matrix.py')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prefix = '16-16-16-w1-steady-jacobi-r0'
            manifest = dict(group='primary-small', steps=16, repetitions=6, seed=20261013,
                            baseline='/base', candidate='/candidate',
                            baseline_sha256='0'*64, candidate_sha256='1'*64)
            (root/'primary-small-manifest-v2.json').write_text(json.dumps(manifest))
            receipts = []
            for variant in ['base', 'cold', 'warm']:
                (root/f'{prefix}-{variant}.csv').write_text(encode(trajectory()))
                (root/f'{prefix}-{variant}.stderr').write_text('')
                (root/f'{prefix}-{variant}.rss').write_text('4096\n')
                receipts.append(dict(variant=variant,
                                     argv=['/base' if variant=='base' else '/candidate',
                                           '16','16','16','1','16',
                                           'warm' if variant=='warm' else 'cold','steady','jacobi'],
                                     full_s=sum(1.01 for _ in range(16)), accepted=16))
            (root/f'{prefix}-receipt.json').write_text(json.dumps(receipts))
            progress = subprocess.run([sys.executable,'-O',str(script),directory,'--progress'],
                                      capture_output=True, text=True)
            self.assertEqual(progress.returncode, 0, progress.stdout+progress.stderr)
            self.assertFalse(json.loads((root/'summary.json').read_text())['qualified'])
            final = subprocess.run([sys.executable,'-O',str(script),directory],
                                   capture_output=True, text=True)
            self.assertNotEqual(final.returncode, 0)
            self.assertFalse(json.loads((root/'summary.json').read_text())['qualified'])
            rows = trajectory()
            rows[0]['step_s'] = 'NaN'
            (root/f'{prefix}-warm.csv').write_text(encode(rows))
            invalid = subprocess.run([sys.executable,'-O',str(script),directory,'--progress'],
                                     capture_output=True, text=True)
            self.assertNotEqual(invalid.returncode, 0)
            self.assertFalse(json.loads((root/'summary.json').read_text())['qualified'])


if __name__ == '__main__':
    unittest.main()
