#!/usr/bin/env python3
"""Compare regenerated numerical evidence with immutable historical baselines.

Only top-level environment metadata is excluded. Integers, strings, booleans,
container shape and keys must match exactly. Finite floats permit relative
1e-8 and absolute 1e-12 error; these are evidence-reproduction tolerances, not
solver stopping thresholds or permissions to relax fixture assertions.
An explicitly selected backend profile supplies only a separately qualified CG
history after checking historical fixture/source hashes and the loaded runtime.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import math
import sys
from pathlib import Path
from historical_baselines import historical_baselines

RELATIVE_TOLERANCE = 1e-8
ABSOLUTE_TOLERANCE = 1e-12
RESULT_FILES = ("results.json", "depth-results.json")
COMPANION = Path(__file__).resolve().parent
CG_PROFILE = "haswell-openblas-0.3.30"


def check_profile_runtime(runtime: dict) -> None:
    """Require the qualified wheel/backend configuration, not just an env hint."""
    import platform
    import numpy
    import scipy.linalg  # Load SciPy's BLAS as well as NumPy's.
    from threadpoolctl import threadpool_info

    if (platform.system(), platform.machine(), f"{sys.version_info.major}.{sys.version_info.minor}",
            numpy.__version__, scipy.__version__) != (
            "Linux", "x86_64", runtime["python"], runtime["numpy"], runtime["scipy"]):
        raise ValueError("Unqualified numerical reproduction runtime")
    pools = [p for p in threadpool_info() if p["user_api"] == "blas"]
    if len(pools) != 2 or any(
            p["internal_api"] != "openblas"
            or p["version"] != runtime["openblas"]
            or p["architecture"] != runtime["architecture"]
            or p["num_threads"] != runtime["threads"] for p in pools):
        raise ValueError("CG profile requires both qualified single-threaded Haswell BLAS libraries")


def profile_baselines(baseline_dir: Path, profile: dict) -> dict:
    """Keep historical evidence immutable; replace only the qualified CG trace."""
    for name, digest in profile["source_sha256"].items():
        if hashlib.sha256((COMPANION / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"{name}: source changed; CG profile requires requalification")
    baselines = {}
    for name in RESULT_FILES:
        raw = (baseline_dir / name).read_bytes()
        if hashlib.sha256(raw).hexdigest() != profile["historical_sha256"][name]:
            raise ValueError(f"{name}: historical evidence changed; CG profile requires review")
        baselines[name] = json.loads(raw)
    projection = baselines["results.json"]["projection"]
    history = profile["cg_history"]
    if len(history) != len(projection["histories"]["cg"]) or len(history) != projection["cg_iterations"]:
        raise ValueError("CG profile changes the historical iteration count")
    projection["histories"]["cg"] = history
    return baselines


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
    parser.add_argument("--cg-profile", choices=[CG_PROFILE],
                        help="Explicit backend-qualified CG history; historical fixtures stay hash-locked")
    args = parser.parse_args()
    baselines = None
    if args.cg_profile:
        profile = json.loads((historical_baselines() / "reproduction" / f"{args.cg_profile}.json").read_text(encoding="utf-8"))
        check_profile_runtime(profile["runtime"])
        baselines = profile_baselines(args.baseline_dir, profile)
    for name in RESULT_FILES:
        expected = baselines[name] if baselines is not None else json.loads(
            (args.baseline_dir / name).read_text(encoding="utf-8"))
        actual = json.loads((args.actual_dir / name).read_text(encoding="utf-8"))
        compare(expected, actual)
        print(f"PASS immutable historical evidence comparison: {name}")


if __name__ == "__main__":
    main()
