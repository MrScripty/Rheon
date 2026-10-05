"""Successor fresh-output gate; frozen liquid-step verifier/packets are unchanged.

X-directed slab translation on a 3D grid, not multidirectional/free-surface
accuracy. Ledger checks use explicit finiteness, native reduction convention,
independent carry-forward and recorded boundary/source accounting.
"""
import csv
import importlib.util
import json
import math
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("frozen_liquid_numerics", ROOT / "evidence/liquid-step/verify_numerics.py")
legacy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(legacy)
require = legacy.require


def verify(directory):
    directory = Path(directory)
    ledgers = []
    for case in sorted(p for p in directory.iterdir() if p.is_dir()):
        with (case / "steps.csv").open() as stream:
            steps = list(csv.DictReader(stream))
        previous = 0.125
        expected = 0.125
        accumulated_boundary = []
        for index, row in enumerate(steps, 1):
            values = {key: float(value) for key, value in row.items()}
            require(all(math.isfinite(v) for v in values.values()), "nonfinite ledger field")
            before, after = values["volume_before"], values["volume_after"]
            inward, outward, source = values["inward"], values["outward"], values["source"]
            require(before == previous, "volume carry-forward")
            require(inward == 0.0 and source == 0.0 and outward >= 0.0, "scenario boundary/source ledger")
            require(before >= 0.0 and after >= 0.0, "negative represented volume")
            balance = after - before + outward - inward - source
            budget = (64.0 * sys.float_info.epsilon) * (abs(before) + abs(after) + outward + inward + abs(source))
            require(values["balance"] == balance, "independent balance recomputation")
            require(values["rounding_budget"] == budget, "independent budget recomputation")
            require(abs(balance) <= budget, "independent ledger acceptance")
            expected += inward - outward + source
            accumulated_boundary.append(outward - inward - source)
            require(abs(after - expected) <= index * budget, "independent accumulated ledger")
            previous = after
        # These are finite before the legacy maximum, which can hide NaN.
        for filename in ["first-pressure.csv", "cells.csv", "faces.csv"]:
            with (case / filename).open() as stream:
                for row in csv.DictReader(stream):
                    require(all(math.isfinite(float(value)) for value in row.values()), "nonfinite field: " + filename)
        ledgers.append({"scenario": case.name, "steps": len(steps),
                        "recorded_net_outflow": math.fsum(accumulated_boundary),
                        "final_recorded_volume": previous,
                        "independent_balance_budget_carry_forward": "passed"})
    result = legacy.verify(directory)
    return {"numerics": result, "ledgers": ledgers,
            "scope": "X-directed slab translation stored in 3D; conservation within rounding, including recorded tiny outflow tail. No multidirectional or free-surface accuracy claim."}


if __name__ == "__main__":
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "evidence/liquid-step/demo"
    print(json.dumps(verify(path), indent=2))
