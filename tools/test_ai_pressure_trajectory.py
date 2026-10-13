"""Adversarial qualification of real full-field trajectories, never fake AI data.

Set RHEON_AI_TRAJECTORY to the completed native output directory. Hard-linked
fixtures avoid copying 300 MB; mutations always replace links with new files.
"""
import csv
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest


@unittest.skipUnless(os.environ.get('RHEON_AI_TRAJECTORY'), 'requires completed actual native trajectories')
class TrajectoryQualification(unittest.TestCase):
    def setUp(self):
        from verify_ai_pressure_trajectory import verify
        self.verify = verify
        self.temporary = tempfile.TemporaryDirectory(dir=Path(os.environ['RHEON_AI_TRAJECTORY']).resolve().parent)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)/'run'
        shutil.copytree(os.environ['RHEON_AI_TRAJECTORY'],self.root,copy_function=os.link)

    def replace(self, path, data):
        path.unlink()
        path.write_bytes(data)

    def mutate_rows(self, name, mutate):
        path = self.root/name
        with path.open() as file:
            reader = csv.DictReader(file)
            fields, rows = reader.fieldnames, list(reader)
        mutate(rows)
        path.unlink()
        with path.open('w') as file:
            writer = csv.DictWriter(file,fieldnames=fields)
            writer.writeheader()
            writer.writerows(rows)

    def test_actual_complete_rollouts(self):
        result = self.verify(self.root)
        self.assertEqual(result['captured_frames'],288)
        self.assertEqual(result['timed_frames'],864)

    def test_incomplete_run_is_rejected(self):
        (self.root/'complete.json').unlink()
        with self.assertRaises(FileNotFoundError):
            self.verify(self.root)

    def test_omitted_frame_is_rejected(self):
        self.mutate_rows('frames.csv',lambda rows: rows.pop())
        with self.assertRaisesRegex(ValueError,'incomplete/duplicate frames'):
            self.verify(self.root)

    def test_hidden_fault_fallback_is_rejected(self):
        def mutate(rows):
            for row in rows:
                if row['mode']=='injected-fallback' and row['frame']=='9':
                    row['selected']='lab-modal-pressure-v1'
                    row['rejection']='none'
        self.mutate_rows('frames.csv',mutate)
        with self.assertRaisesRegex(ValueError,'provider selection/rejection'):
            self.verify(self.root)

    def test_underreported_whole_rollout_time_is_rejected(self):
        self.mutate_rows('runs.csv',lambda rows: rows[0].update(rollout_wall_ms='0'))
        with self.assertRaisesRegex(ValueError,'run timing scope|nonfinite/negative run timing'):
            self.verify(self.root)

    def test_falsified_predicted_divergence_receipt_is_rejected(self):
        def mutate(rows):
            row = next(r for r in rows if r['phase']=='capture' and r['seed']=='900' and r['mode']=='jacobi' and r['frame']=='1')
            row['predicted_divergence']='0'
        self.mutate_rows('frames.csv',mutate)
        with self.assertRaisesRegex(ValueError,'predicted divergence receipt|residual norm/divergence consistency'):
            self.verify(self.root)

    def test_nonfinite_native_velocity_is_rejected(self):
        import struct
        path = self.root/'900-learned'/'001.bin'
        raw = bytearray(path.read_bytes())
        struct.pack_into('<f',raw,16*32768,float('nan'))
        self.replace(path,raw)
        with self.assertRaisesRegex(ValueError,'nonfinite field'):
            self.verify(self.root)

    def zero_process(self):
        path = self.root/'complete.json'
        data = json.loads(path.read_text())
        data.update(process_ms=0,model_load_ms=0)
        self.replace(path,json.dumps(data).encode())

    def test_joint_zero_timings_are_rejected(self):
        def zero_frames(rows):
            for row in rows:
                for key in ('step_ms','provider_ms','inference_ms','qualification_ms','fallback_ms'):
                    row[key]='0'
        def zero_runs(rows):
            for row in rows:
                for key in ('initialization_ms','steps_ms','rollout_wall_ms'):
                    row[key]='0'
        self.mutate_rows('frames.csv',zero_frames)
        self.mutate_rows('runs.csv',zero_runs)
        self.zero_process()
        with self.assertRaisesRegex(ValueError,'process timing'):
            self.verify(self.root)

    def test_zero_process_timings_are_rejected(self):
        self.zero_process()
        with self.assertRaisesRegex(ValueError,'process timing'):
            self.verify(self.root)

    def test_underreported_positive_process_time_is_rejected(self):
        path=self.root/'complete.json'
        data=json.loads(path.read_text())
        data['process_ms']=1
        self.replace(path,json.dumps(data).encode())
        with self.assertRaisesRegex(ValueError,'process timing scope'):
            self.verify(self.root)

    def test_enormous_timed_residuals_are_rejected(self):
        def mutate(rows):
            for row in rows:
                if row['phase']=='timed':
                    row['residual_l2']='1e100'
        self.mutate_rows('frames.csv',mutate)
        with self.assertRaisesRegex(ValueError,'residual norm/divergence consistency'):
            self.verify(self.root)

    def test_coordinated_duplicate_fields_and_receipts_are_rejected(self):
        # Duplicate a plausible prior frame in every backend and update all its
        # diagnostics, preserving cross-method comparisons and per-frame gates.
        for mode in ('jacobi','sgs','learned','injected-fallback','shifted-jacobi','unsupported-fallback'):
            directory=self.root/f'900-{mode}'
            self.replace(directory/'002.bin',(directory/'001.bin').read_bytes())
        def mutate(rows):
            for mode in ('jacobi','sgs','learned','injected-fallback','shifted-jacobi','unsupported-fallback'):
                src=next(r for r in rows if r['phase']=='capture' and r['seed']=='900' and r['mode']==mode and r['frame']=='1')
                dst=next(r for r in rows if r['phase']=='capture' and r['seed']=='900' and r['mode']==mode and r['frame']=='2')
                for key in ('residual_l2','predicted_divergence','actual_divergence','courant','tracer_integral'):
                    dst[key]=src[key]
        self.mutate_rows('frames.csv',mutate)
        with self.assertRaisesRegex(ValueError,'transition'):
            self.verify(self.root)

    def test_wrong_frame_fields_are_rejected(self):
        path = self.root/'900-learned'/'002.bin'
        self.replace(path,(self.root/'900-learned'/'001.bin').read_bytes())
        with self.assertRaises(ValueError):
            self.verify(self.root)

    def test_valid_but_different_fallback_fields_are_rejected(self):
        # Direct independent equality guard; keep the fields' own diagnostics so
        # failure cannot hide behind only a stale numerical receipt.
        source = self.root/'900-injected-fallback'/'001.bin'
        target = self.root/'900-unsupported-fallback'/'001.bin'
        self.replace(target,source.read_bytes())
        def mutate(rows):
            src = next(r for r in rows if r['phase']=='capture' and r['seed']=='900' and r['mode']=='injected-fallback' and r['frame']=='1')
            dst = next(r for r in rows if r['phase']=='capture' and r['seed']=='900' and r['mode']=='unsupported-fallback' and r['frame']=='1')
            for key in ('residual_l2','predicted_divergence','actual_divergence','courant','tracer_integral'):
                dst[key]=src[key]
        self.mutate_rows('frames.csv',mutate)
        with self.assertRaisesRegex(ValueError,'unsupported fallback differs'):
            self.verify(self.root)


if __name__ == '__main__':
    unittest.main()
