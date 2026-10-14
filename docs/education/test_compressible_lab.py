"""Focused publication rejection probes for computed compressible fixtures."""
from pathlib import Path
import copy
import json
import subprocess
import sys
import tempfile
import unittest

from compressible_browser import validate_receipt
from compressible_lab import publish, validate_packet, verify_published


ROOT = Path(__file__).resolve().parents[2]


class CompressiblePublicationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory(prefix='rheon-compressible-publication-')
        cls.directory = Path(cls.temporary.name)
        cls.packet_dir = cls.directory / 'packet'
        subprocess.run([sys.executable, str(ROOT / 'docs/research-book/compressible/generate.py'), '--output', str(cls.packet_dir)], check=True, capture_output=True)
        cls.packet = json.loads((cls.packet_dir / 'results.json').read_text())

    @classmethod
    def tearDownClass(cls):
        cls.temporary.cleanup()

    def reject(self, change, pattern):
        packet = copy.deepcopy(self.packet)
        change(packet)
        path = self.directory / 'mutated'
        path.mkdir(exist_ok=True)
        (path / 'results.json').write_text(json.dumps(packet))
        with self.assertRaisesRegex(RuntimeError, pattern):
            validate_packet(ROOT, path)

    def test_source_only_has_no_controls_or_invented_field_checks(self):
        output = self.directory / 'source-only'
        output.mkdir()
        body = publish(ROOT, output)
        self.assertIn('fields are absent', body)
        self.assertNotIn('id="compressible-case"', body)
        self.assertIsNone(verify_published(ROOT, output))
        receipt = {'schema': 'rheon-compressible-browser-v1', 'included': False, 'browser_fluid_solves': 0,
                   'dynamic_states': 1, 'field_comparisons': 0, 'metric_comparisons': 0, 'camera_checks': 0,
                   'mobile_no_overflow': True, 'screenshots': [], 'records_sha256': None, 'source_sha256': None}
        with self.assertRaisesRegex(RuntimeError, 'dynamic_states'):
            validate_receipt(receipt, None)
        (output / 'compressible-records.json').write_text('{}')
        with self.assertRaisesRegex(RuntimeError, 'source-only'):
            verify_published(ROOT, output)

    def test_stale_source_and_missing_source_binding_are_rejected(self):
        self.reject(lambda p: p['provenance']['source_sha256'].__setitem__('reference.py', '0' * 64), 'source binding')
        self.reject(lambda p: p.pop('provenance'), 'source binding')

    def test_nonfinite_and_inadmissible_fields_are_rejected(self):
        self.reject(lambda p: p['cases'][0]['frames'][1]['density'].__setitem__(0, float('nan')), 'density field')
        self.reject(lambda p: p['cases'][0]['frames'][1]['pressure'].__setitem__(0, -1), 'pressure field')
        self.reject(lambda p: p['cases'][0]['frames'][1]['conserved'][0].__setitem__(4, float('inf')), 'conserved field shape')

    def test_case_frame_and_vector_contract_is_required(self):
        self.reject(lambda p: p['cases'].pop(), 'two qualified cases')
        self.reject(lambda p: p['cases'][0]['frames'].pop(), 'seven saved')
        self.reject(lambda p: p['cases'][0]['frames'][0]['velocity'].pop(), 'three-component velocity')
        self.reject(lambda p: p['cases'][0]['frames'][0]['conserved'][0].pop(), 'conserved field shape')

    def test_data_or_ledger_changes_do_not_preserve_qualification(self):
        self.reject(lambda p: p['cases'][0]['frames'][1]['density'].__setitem__(0, .77), 'conserved density')
        self.reject(lambda p: p['cases'][0]['frames'][1]['diagnostics'].__setitem__('mass', 3), 'mass ledger')
        self.reject(lambda p: p['cases'][0]['frames'][1]['diagnostics']['momentum'].__setitem__(0, 2), 'momentum ledger')
        self.reject(lambda p: p['cases'][0]['frames'][1]['diagnostics'].__setitem__('energy', 20), 'energy ledger')
        self.reject(lambda p: p['cases'][0]['frames'][1]['diagnostics'].__setitem__('density_l1_reference_error', 0), 'density reference error mismatch')

    def test_range_and_time_forgery_is_rejected(self):
        self.reject(lambda p: p['cases'][1]['field_ranges']['speed'].__setitem__(1, 100), 'field range speed')
        self.reject(lambda p: p['cases'][1]['frames'][1].__setitem__('time', float('nan')), 'saved time order|finite saved time')
        self.reject(lambda p: p['cases'][1]['parameters'].__setitem__('gamma', 1.5), 'physical parameters')

    def test_qualification_numbers_are_recomputed(self):
        self.reject(lambda p: p['qualification']['acoustic']['normalized_density_l1_errors'].__setitem__(0, 0), 'numerical producer verification failed')
        self.reject(lambda p: p['qualification']['sod']['pressure_l1_errors'].__setitem__(-1, 0), 'numerical producer verification failed')

    def test_published_fields_and_sources_are_bound_and_revalidated(self):
        output = self.directory / 'published'
        output.mkdir()
        body = publish(ROOT, output, self.packet_dir)
        self.assertIn('id="compressible-case"', body)
        self.assertIsNotNone(verify_published(ROOT, output))
        records = output / 'compressible-records.json'
        altered = copy.deepcopy(self.packet)
        altered['cases'][0]['frames'][1]['diagnostics']['mass'] = 3
        records.write_text(json.dumps(altered))
        receipt_path = output / 'compressible-presentation.json'
        presentation = json.loads(receipt_path.read_text())
        import hashlib
        presentation['qualification']['records_sha256'] = hashlib.sha256(records.read_bytes()).hexdigest()
        receipt_path.write_text(json.dumps(presentation))
        with self.assertRaisesRegex(RuntimeError, 'total|mass'):
            verify_published(ROOT, output)


if __name__ == '__main__':
    unittest.main()
