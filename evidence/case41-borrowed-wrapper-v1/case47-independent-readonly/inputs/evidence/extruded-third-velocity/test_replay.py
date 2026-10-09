"""Mandatory actual topology/identity inputs, including renderer index safety."""
import copy
import json
from pathlib import Path
import unittest
import replay


class PublishedSchema(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.rows = [json.loads(x) for x in (Path(__file__).parent / 'native-trials/mapped-native.jsonl').read_text().splitlines()]

    def rejects(self, mutate):
        rows = copy.deepcopy(self.rows)
        mutate(rows)
        with self.assertRaises(ValueError):
            replay.schema(rows)

    def test_all_actual_cases(self):
        self.assertEqual(len(replay.schema(self.rows)), 20)

    def test_missing_actual_periodic_identification(self):
        self.rejects(lambda rows: rows[1].pop('periodic_indices'))

    def test_float_indices_cannot_be_native_indices(self):
        self.rejects(lambda rows: rows[1]['periodic_indices'].__setitem__(0, 0.0))
        self.rejects(lambda rows: rows[1]['triangles'][0].__setitem__(0, float(rows[1]['triangles'][0][0])))

    def test_bool_indices_cannot_be_native_indices(self):
        self.rejects(lambda rows: rows[1]['periodic_indices'].__setitem__(0, False))

    def test_false_owner_identity(self):
        self.rejects(lambda rows: rows[1]['stamp'].__setitem__('id', 71))
        self.rejects(lambda rows: rows[1]['stamp'].__setitem__('version', 2))

    def test_missing_full_report(self):
        self.rejects(lambda rows: rows[1]['report'].pop('total'))

    def test_duplicated_refinement(self):
        self.rejects(lambda rows: rows.__setitem__(slice(3, 8), copy.deepcopy(rows[:5])))


if __name__ == '__main__':
    unittest.main()
