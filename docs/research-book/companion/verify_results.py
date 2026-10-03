#!/usr/bin/env python3
"""Compare regenerated numerical evidence with the committed JSON baselines.

Only top-level environment metadata is excluded. Integers, strings, booleans,
container shape and keys must match exactly. Finite floats permit relative
1e-8 and absolute 1e-12 error; these are evidence-reproduction tolerances, not
solver stopping thresholds or permissions to relax fixture assertions.
"""
from __future__ import annotations
import argparse
import json
import math
from pathlib import Path

RELATIVE_TOLERANCE = 1e-8
ABSOLUTE_TOLERANCE = 1e-12
RESULT_FILES = ("results.json", "depth-results.json")


def compare(expected: object, actual: object, path: str = "$") -> None:
    if type(expected) is not type(actual):
        raise ValueError(f"{path}: JSON type mismatch")
    if isinstance(expected, dict):
        if expected.keys() != actual.keys():
            raise ValueError(f"{path}: JSON keys differ")
        for key in expected:
            if path == "$" and key == "environment":
                continue
            compare(expected[key], actual[key], f"{path}.{key}")
    elif isinstance(expected, list):
        if len(expected) != len(actual):
            raise ValueError(f"{path}: array lengths differ")
        for index, (left, right) in enumerate(zip(expected, actual)):
            compare(left, right, f"{path}[{index}]")
    elif isinstance(expected, float):
        if not (math.isfinite(expected) and math.isfinite(actual)):
            raise ValueError(f"{path}: nonfinite numerical evidence")
        if not math.isclose(expected, actual, rel_tol=RELATIVE_TOLERANCE,
                            abs_tol=ABSOLUTE_TOLERANCE):
            raise ValueError(f"{path}: numerical evidence differs: {expected!r} != {actual!r}")
    elif expected != actual:
        raise ValueError(f"{path}: exact evidence differs: {expected!r} != {actual!r}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline-dir", required=True, type=Path)
    parser.add_argument("--actual-dir", type=Path, default=Path(__file__).resolve().parent)
    args = parser.parse_args()
    for name in RESULT_FILES:
        expected = json.loads((args.baseline_dir / name).read_text(encoding="utf-8"))
        actual = json.loads((args.actual_dir / name).read_text(encoding="utf-8"))
        compare(expected, actual)
        print(f"PASS committed evidence comparison: {name}")


if __name__ == "__main__":
    main()
