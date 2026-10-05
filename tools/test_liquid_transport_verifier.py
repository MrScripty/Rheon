import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from verify_liquid_transport import ROOT, verify


class LiquidVerifierTest(unittest.TestCase):
    def test_frozen_committed_data_passes_stronger_gate(self):
        result = verify(ROOT / "evidence/liquid-step/demo")
        self.assertEqual(sum(row["steps"] for row in result["ledgers"]), 240)

    def test_adversarial_pressure_and_ledger_changes_are_rejected(self):
        changes = ["nan_first_pressure", "nan_final_pressure", "infinite_velocity",
                   "before_999", "inward_999", "outward_999", "balance", "budget", "carry_forward"]
        for change in changes:
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                demo = Path(temporary) / "demo"
                shutil.copytree(ROOT / "evidence/liquid-step/demo", demo)
                case = demo / "jacobi-pcg-v1-n16-c025"
                if change == "nan_first_pressure":
                    filename, field, index, value = "first-pressure.csv", "pressure", 0, "nan"
                elif change == "nan_final_pressure":
                    filename, field, index, value = "cells.csv", "pressure", 0, "nan"
                elif change == "infinite_velocity":
                    filename, field, index, value = "faces.csv", "velocity", 0, "inf"
                else:
                    filename, index, value = "steps.csv", 0, "999"
                    field = {"before_999": "volume_before", "inward_999": "inward",
                             "outward_999": "outward", "balance": "balance",
                             "budget": "rounding_budget", "carry_forward": "volume_before"}[change]
                    if change == "carry_forward":
                        index = 1
                path = case / filename
                with path.open() as stream:
                    reader = csv.DictReader(stream)
                    fields, rows = reader.fieldnames, list(reader)
                rows[index][field] = value
                with path.open("w", newline="") as stream:
                    writer = csv.DictWriter(stream, fieldnames=fields)
                    writer.writeheader()
                    writer.writerows(rows)
                with self.assertRaises(ValueError):
                    verify(demo)


if __name__ == "__main__":
    unittest.main()
