"""Refusal probes use real exported frames when explicitly supplied."""
import copy
import json
import os
from pathlib import Path
import tempfile
import unittest

from verify_session_flow import verify


@unittest.skipUnless(os.environ.get('RHEON_SESSION_FRAMES'), 'requires actual API headless_session output')
class RealSessionFlow(unittest.TestCase):
    def setUp(self):
        self.source = Path(os.environ['RHEON_SESSION_FRAMES'])
        self.records = [json.loads(line) for line in self.source.read_text().splitlines()]
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.path = Path(self.tmp.name) / 'frames.jsonl'

    def test_real_discrete_reference(self):
        self.assertEqual(verify(self.source)['accepted_time_s'], 0.5)

    def test_corrupted_flow_cannot_pass(self):
        def frame_gap(rows): rows.pop(3)
        def wrong_time(rows): rows[3]['time_s'] += 0.01
        def rounded_identity(rows): rows[3]['carrier_stamp']['id'] = float(2**64 - 2)
        def false_pressure(rows): rows[1]['fields']['pressure'][2] = 0
        def wrong_transport(rows): rows[5]['fields']['fraction'][2] += 0.01
        def wrong_velocity(rows): rows[3]['fields']['velocity_y'][4] = 0.1
        def false_ledger(rows): rows[-1]['diagnostics']['outward_m3'] = 0
        def false_diagnostic(rows): rows[3]['diagnostics']['carrier_divergence_s_inv'] = 1
        def nonfinite(rows): rows[2]['fields']['tracer'][0] = float('nan')
        for mutation in (frame_gap, wrong_time, rounded_identity, false_pressure,
                         wrong_transport, wrong_velocity, false_ledger, false_diagnostic, nonfinite):
            with self.subTest(mutation=mutation.__name__):
                records = copy.deepcopy(self.records)
                mutation(records)
                self.path.write_text(''.join(json.dumps(row) + '\n' for row in records))
                with self.assertRaises(ValueError): verify(self.path)

    def test_duplicate_keys_are_rejected(self):
        self.path.write_text(self.source.read_text().replace('"frame":', '"frame":999,"frame":', 1))
        with self.assertRaises(ValueError): verify(self.path)


if __name__ == '__main__':
    unittest.main()
