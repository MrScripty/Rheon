#!/usr/bin/env python3
"""Adversarial tests for the committed-evidence acceptance gate."""
import unittest
import copy
import json
from pathlib import Path
import shutil
import tempfile
from unittest import mock
from verify_results import compare, profile_baselines, check_profile_runtime, COMPANION, CG_PROFILE


class EvidenceComparison(unittest.TestCase):
    def test_platform_metadata_and_small_float_rounding_are_allowed(self):
        compare({"environment": {"python": "old"}, "x": [1.0, 2, True, "ok"]},
                {"environment": {"python": "new"}, "x": [1.0+1e-10, 2, True, "ok"]})

    def test_changed_numerical_evidence_is_rejected(self):
        with self.assertRaises(ValueError):
            compare({"energy": 4.0}, {"energy": 5.0})

    def test_changed_count_boolean_string_and_type_are_rejected(self):
        for a, b in [(2, 3), (True, False), ("1/3", "1/2"), (1, 1.0), (True, 1)]:
            with self.subTest(a=a, b=b), self.assertRaises(ValueError):
                compare(a, b)

    def test_missing_keys_and_changed_array_lengths_are_rejected(self):
        for a, b in [({"x": 1}, {}), ([1], [1, 2])]:
            with self.subTest(a=a, b=b), self.assertRaises(ValueError):
                compare(a, b)

    def test_nonfinite_values_are_rejected_even_when_identical(self):
        for value in [float("nan"), float("inf"), -float("inf")]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                compare(value, value)

    def test_near_zero_outside_absolute_tolerance_is_rejected(self):
        with self.assertRaises(ValueError):
            compare(0.0, 2e-12)


class BackendProfile(unittest.TestCase):
    def setUp(self):
        self.profile=json.loads((COMPANION/'reproduction'/f'{CG_PROFILE}.json').read_text(encoding='utf-8'))

    def test_profile_changes_only_the_explicit_cg_trace(self):
        expected=profile_baselines(COMPANION,self.profile)
        original=json.loads((COMPANION/'results.json').read_text(encoding='utf-8'))
        with self.assertRaises(ValueError):
            compare(original,expected['results.json'])
        original['projection']['histories']['cg']=self.profile['cg_history']
        self.assertEqual(original,expected['results.json'])
        self.assertEqual(json.loads((COMPANION/'depth-results.json').read_text(encoding='utf-8')),
                         expected['depth-results.json'])

    def test_profile_still_rejects_changed_history_and_other_numerical_fields(self):
        expected=profile_baselines(COMPANION,self.profile)['results.json']
        for history in (True,False):
            with self.subTest(history=history):
                actual=copy.deepcopy(expected)
                if history: actual['projection']['histories']['cg'][53]*=1.01
                else: actual['projection']['energy_after']*=1.01
                with self.assertRaises(ValueError): compare(expected,actual)

    def test_changed_historical_fixture_is_rejected_before_profile_substitution(self):
        with tempfile.TemporaryDirectory() as root:
            root=Path(root)
            for name in ('results.json','depth-results.json'):
                shutil.copyfile(COMPANION/name,root/name)
            # Even whitespace changes require explicit historical-evidence review.
            with (root/'results.json').open('a',encoding='utf-8') as f: f.write('\n')
            with self.assertRaisesRegex(ValueError,'historical evidence changed'):
                profile_baselines(root,self.profile)

    def test_changed_experiment_requires_requalification(self):
        with tempfile.TemporaryDirectory() as root:
            root=Path(root)
            for name in self.profile['source_sha256']:
                shutil.copyfile(COMPANION/name,root/name)
            with (root/'experiments.py').open('a',encoding='utf-8') as f: f.write('\n')
            with mock.patch('verify_results.COMPANION',root):
                with self.assertRaisesRegex(ValueError,'source changed'):
                    profile_baselines(COMPANION,self.profile)

    def test_changed_profile_iteration_count_is_rejected(self):
        self.profile['cg_history'].pop()
        with self.assertRaisesRegex(ValueError,'iteration count'):
            profile_baselines(COMPANION,self.profile)

    def test_unqualified_blas_runtime_is_rejected(self):
        pool={'user_api':'blas','internal_api':'openblas','version':'0.3.30',
              'architecture':'Haswell','num_threads':1}
        variants=[[],[pool],[pool,pool|{'version':'unknown'}],
                  [pool,pool|{'architecture':'SkylakeX'}],[pool,pool|{'num_threads':4}]]
        for pools in variants:
            with self.subTest(pools=pools), mock.patch('threadpoolctl.threadpool_info',return_value=pools):
                with self.assertRaises(ValueError): check_profile_runtime(self.profile['runtime'])


if __name__ == "__main__":
    unittest.main()
