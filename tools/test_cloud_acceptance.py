"""Adversarial gates remain active under python -O."""
from pathlib import Path
import sys
import tempfile
import unittest

from cloud_acceptance import measure, parse_sample

VALID = ('steps=12 accepted_time=0.24 divergence=1e-8 step_seconds=1.0 '
         'capture_seconds=0.2 frame_seconds=1.21 managed_bytes=500 capture_bytes=100 '
         'fingerprint=0123456789abcdef')


class AcceptanceTests(unittest.TestCase):
    def test_valid_record(self):
        self.assertEqual(parse_sample(VALID, 12)['accepted_time'], 0.24)

    def test_rejects_corrupt_and_incomplete_measurements(self):
        for text in ('', VALID + '\n' + VALID, VALID.replace('steps=12', 'steps=11'),
                     VALID.replace('divergence=1e-8', 'divergence=nan'),
                     VALID.replace('divergence=1e-8', 'divergence=1e-3'),
                     VALID.replace('frame_seconds=1.21', 'frame_seconds=0.5'),
                     VALID.replace('capture_seconds=0.2', 'capture_seconds=-1'),
                     VALID.replace('accepted_time=0.24', 'accepted_time=0'),
                     VALID.replace('accepted_time=0.24', 'accepted_time=1'),
                     VALID.replace('0123456789abcdef', 'bad'), VALID + ' unknown=1', VALID + ' steps=12',
                     VALID.replace('managed_bytes=500', 'managed_bytes=0'),
                     VALID.replace('capture_bytes=100', 'capture_bytes=0.5')):
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_sample(text, 12)

    def test_child_failure_and_timeout_are_not_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / 'run.log'
            with self.assertRaises(ValueError):
                measure([sys.executable, '-c', 'raise SystemExit(7)'], log)
            with self.assertRaises(TimeoutError):
                measure([sys.executable, '-c', 'import time; time.sleep(5)'], log, timeout=0.02)
            metrics = measure([sys.executable, '-c', 'print("complete")'], log)
            self.assertGreater(metrics['peak_rss_kib'], 0)
            self.assertEqual(log.read_text().strip(), 'complete')

    def test_timeout_stops_descendants(self):
        with tempfile.TemporaryDirectory() as tmp:
            marker = Path(tmp) / 'survived'
            grandchild = f'import time; from pathlib import Path; time.sleep(0.4); Path({str(marker)!r}).touch()'
            parent = f'import subprocess,sys,time; subprocess.Popen([sys.executable,"-c",{grandchild!r}]); time.sleep(5)'
            with self.assertRaises(TimeoutError):
                measure([sys.executable, '-c', parent], Path(tmp) / 'run.log', timeout=0.15)
            # A surviving descendant would create the marker after the timeout.
            import time
            time.sleep(0.5)
            self.assertFalse(marker.exists())


if __name__ == '__main__':
    unittest.main()
