#!/usr/bin/env python3
"""Adversarial tests for the committed-evidence acceptance gate."""
import unittest
from verify_results import compare


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


if __name__ == "__main__":
    unittest.main()
