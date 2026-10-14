#!/usr/bin/env python3
"""Execute the fixed CPU suite and publish a claim-scoped, exact-source receipt."""
import argparse
import csv
import hashlib
import io
import json
import math
from pathlib import Path
import platform
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
METHODS = ("jacobi-pcg-v1", "sgs-pcg-v1")
COLUMNS = (
    "case method n steps dt_s continuum_linf time_linf residual_max "
    "divergence_max_per_s net_flux_m3_per_s energy_growth_j "
    "wall_normal_max_m_per_s energy_j iterations"
).split()


def evaluate(text):
    """Fixed-suite policy. Failures identify the measurement that blocks acceptance."""
    reader = csv.DictReader(io.StringIO(text))
    if reader.fieldnames != COLUMNS:
        raise ValueError("invalid CSV columns")
    rows = list(reader)
    expected = {( "pressure_mesh", m, n, 0) for m in METHODS for n in (6, 12, 24)}
    expected |= {("viscosity_mesh", "symmetric-strain-explicit-v1", n, n*n) for n in (8, 16, 32)}
    expected |= {("viscosity_time", "symmetric-strain-explicit-v1", 12, s) for s in (16, 32, 64)}
    seen = set()
    for row in rows:
        if None in row or any(v is None for v in row.values()):
            raise ValueError("incomplete CSV row")
        for key in COLUMNS[2:]:
            row[key] = float(row[key])
            if not math.isfinite(row[key]):
                raise ValueError(f"nonfinite {key}")
        key = (row["case"], row["method"], row["n"], row["steps"])
        if key not in expected or key in seen:
            raise ValueError(f"unexpected or duplicate case {key}")
        seen.add(key)
        for name in ("continuum_linf", "time_linf", "residual_max", "divergence_max_per_s",
                     "energy_growth_j", "wall_normal_max_m_per_s", "energy_j", "iterations"):
            if row[name] < 0:
                raise ValueError(f"negative {name}")
        if row["wall_normal_max_m_per_s"] != 0 or abs(row["net_flux_m3_per_s"]) > 1e-12:
            raise ValueError(f"wall/flux gate {key}")
        if row["divergence_max_per_s"] > (1e-7 if row["case"] == "pressure_mesh" else 1e-4):
            raise ValueError(f"divergence gate {key}")
        if row["energy_growth_j"] > 1e-10:
            raise ValueError(f"energy growth gate {key}")
        if row["case"] == "pressure_mesh":
            if row["dt_s"] != 0 or row["residual_max"] > 1e-10 or row["iterations"] < 1:
                raise ValueError(f"pressure solve gate {key}")
        else:
            if abs(row["dt_s"]*row["steps"] - 0.1) > 1e-14 or row["energy_j"] <= 0:
                raise ValueError(f"time/energy gate {key}")
            if row["case"] == "viscosity_mesh" and row["time_linf"] > 0.1*row["continuum_linf"]:
                raise ValueError(f"mesh study temporal contamination {key}")
    if seen != expected:
        raise ValueError("missing suite cases")
    orders = {}
    groups = [("pressure_mesh", m, "continuum_linf", 1.8, 2.2) for m in METHODS]
    groups += [("viscosity_mesh", "symmetric-strain-explicit-v1", "continuum_linf", 1.8, 2.2),
               ("viscosity_time", "symmetric-strain-explicit-v1", "time_linf", 0.75, 1.25)]
    for case, method, norm, lower, upper in groups:
        subset = [r for r in rows if r["case"] == case and r["method"] == method]
        subset.sort(key=lambda r: r["steps"] if case == "viscosity_time" else r["n"])
        if any(r[norm] <= 0 for r in subset):
            raise ValueError(f"zero error has no observable refinement order {case}")
        slopes = [math.log2(a[norm]/b[norm]) for a, b in zip(subset, subset[1:])]
        if not all(lower <= p <= upper for p in slopes):
            raise ValueError(f"refinement order gate {case}/{method}: {slopes}")
        orders[f"{case}/{method}"] = slopes
    return {"rows": rows, "observed_orders": orders}


def command(args):
    return subprocess.check_output(args, cwd=REPO, text=True).strip()


def source_identity():
    paths = command(["git", "ls-files", "--cached", "--others", "--exclude-standard", "--",
                     "src", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "validation/cpu-cfd"])
    return {
        "commit": command(["git", "rev-parse", "HEAD"]),
        "dirty_paths": command(["git", "status", "--porcelain"]),
        "sha256": {p: hashlib.sha256((REPO/p).read_bytes()).hexdigest()
                   for p in sorted(set(paths.splitlines()))},
    }


def execute(output, cargo="cargo", timeout=600):
    output = Path(output).resolve()
    if output == REPO or REPO in output.parents:
        raise ValueError("receipt directory must be outside the source checkout")
    output.mkdir(parents=True, exist_ok=False)
    receipt = {"schema_version": 1, "status": "failed", "backend": "local-cpu",
               "experimental_agreement": "not_assessed", "training_label_quality": "not_assessed",
               "body_forces_moments": "not_assessed", "full_flow_capability": "not_assessed",
               "platform": platform.platform(),
               "python": sys.version, "reference": "validation/cpu-cfd/README.md"}
    args = [cargo, "run", "--release", "--locked", "--manifest-path",
            "validation/cpu-cfd/Cargo.toml"]
    receipt["command"] = args
    started = time.monotonic()
    try:
        receipt["source"] = source_identity()
        receipt["rustc"] = command(["rustc", "--version", "--verbose"])
        receipt["cargo"] = command([cargo, "--version"])
        with (output/"measurements.csv").open("w") as stdout, (output/"cargo.log").open("w") as stderr:
            result = subprocess.run(args, cwd=REPO, stdout=stdout, stderr=stderr, timeout=timeout, check=False)
        receipt["exit_code"] = result.returncode
        if result.returncode != 0:
            raise ValueError(f"CPU benchmark exited {result.returncode}; see cargo.log")
        receipt["measurements_sha256"] = hashlib.sha256((output/"measurements.csv").read_bytes()).hexdigest()
        receipt["assessment"] = evaluate((output/"measurements.csv").read_text())
        if source_identity() != receipt["source"]:
            raise ValueError("source changed during run")
        receipt["status"] = "accepted_elementary_numerics"
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        receipt["failure"] = str(error)
        raise
    finally:
        receipt["build_and_run_elapsed_s"] = time.monotonic()-started
        name = "receipt.json" if receipt["status"] == "accepted_elementary_numerics" else "failed.json"
        (output/name).write_text(json.dumps(receipt, indent=2, allow_nan=False)+"\n")
    return receipt


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path, help="new directory outside checkout")
    options = parser.parse_args()
    try:
        result = execute(options.output)
        print(json.dumps({"status": result["status"], "observed_orders": result["assessment"]["observed_orders"]}, indent=2))
    except (OSError, subprocess.SubprocessError, ValueError) as failure:
        parser.exit(1, f"CPU validation failed: {failure}\n")
