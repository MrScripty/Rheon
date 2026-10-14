#!/usr/bin/env python3
"""Capture or cold-recheck the frozen bounded X/Y Stokes diagnostic family."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import copy
from fractions import Fraction
import math
from pathlib import Path
import re
import subprocess
import sys
import json

import oracle
import run

HERE = Path(__file__).resolve().parent
FIXTURES = HERE / "directional_fixtures.json"
PROFILE = "rheon-i1-directional-diagnostic-v1"
LEVELS = {
    "base": ([8, 5, 5], ["0.25", "0.25", "0.25"]),
    "x-medium": ([16, 5, 5], ["0.125", "0.25", "0.25"]),
    "x-fine": ([32, 5, 5], ["0.0625", "0.25", "0.25"]),
    "y-medium": ([8, 10, 5], ["0.25", "0.125", "0.25"]),
    "y-fine": ([8, 20, 5], ["0.25", "0.0625", "0.25"]),
}
CASES = tuple(f"{kind}-{level}" for kind in ("straight", "mms") for level in LEVELS)
SOURCE_FILES = (
    "directional.py", "test_directional.py", "directional_fixtures.json",
    "run.py", "oracle.py", "fixtures.json",
)


def pin(value, claim):
    oracle.require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value),
                   claim, "explicit lowercase SHA256 pin required")
    return value


def freeze_family(data, baseline):
    """Accept exactly the prospective family, including unchanged physical controls."""
    expected = copy.deepcopy(baseline)
    expected["contract_status"] = "reviewed-directional-n3-diagnostic"
    expected["study_profile"] = PROFILE
    expected["cases"] = {}
    for kind, original in (("straight", "tiny-straight"), ("mms", "tiny-mms")):
        for level, (counts, spacing) in LEVELS.items():
            fixture = copy.deepcopy(baseline["cases"][original])
            fixture.update(counts=counts, spacing=spacing)
            expected["cases"][f"{kind}-{level}"] = fixture
    oracle.require(data == expected, "directional-family",
                   "family differs from frozen counts, spacing, domain, body, controls or profile")
    return data


def load_family(fixtures_sha256):
    expected = pin(fixtures_sha256, "directional-fixture-pin")
    oracle.require(oracle.digest(FIXTURES) == expected, "directional-fixture-pin",
                   "directional fixture bytes differ from the admitted pin")
    return freeze_family(oracle.read_json(FIXTURES), oracle.read_json(HERE / "fixtures.json")), expected


def source_bindings(repo, source):
    """Bind wrapper, tests and unchanged verifier inputs to the exact source archive."""
    source = run.full_oid(source)
    oracle.require(run.git(repo, "rev-parse", source + "^{commit}") == source,
                   "source-identity", "source is not the pinned commit")
    bindings = {}
    for name in SOURCE_FILES:
        relative = "validation/submarine/" + name
        result = subprocess.run(["git", "-C", str(repo), "show", source + ":" + relative],
                                capture_output=True, check=False)
        oracle.require(result.returncode == 0 and result.stdout == (HERE / name).read_bytes(),
                       "directional-source", "current file differs from pinned source: " + relative)
        bindings[relative] = oracle.digest(HERE / name)
    return bindings


@contextmanager
def selected_fixtures():
    # Oracle's default fixtures remain authoritative for the unchanged profile/controls.
    previous = run.FIXTURES
    run.FIXTURES = FIXTURES
    try:
        yield
    finally:
        run.FIXTURES = previous


def observed_order(values):
    """Descriptive order only; the resolution heuristic never accepts a native row."""
    oracle.require(isinstance(values, list) and len(values) == 3 and
                   all(type(v) in (float, int) and math.isfinite(v) for v in values),
                   "directional-series", "three finite numeric levels required")
    q = [Fraction(v) for v in values]
    coarse, fine = q[1] - q[0], q[2] - q[1]
    noise = Fraction(4096) * Fraction(oracle.EPS) * sum(map(abs, q))
    result = {"values": values, "coarse_difference": float(coarse),
              "fine_difference": float(fine), "resolution_heuristic": float(noise),
              "descriptive_order": None}
    if coarse == 0 and fine == 0:
        classification = "constant"
    elif abs(coarse) <= noise or abs(fine) <= noise:
        classification = "noise-limited"
    elif coarse * fine < 0:
        classification = "oscillatory"
    elif abs(fine) >= abs(coarse):
        classification = "nonpositive-order"
    else:
        classification = "resolved-monotone-positive-order"
        def log_abs(value):
            return math.log(abs(value.numerator)) - math.log(value.denominator)
        result["descriptive_order"] = (log_abs(coarse) - log_abs(fine)) / math.log(2)
    result["classification"] = classification
    return result


def norm(values):
    return math.hypot(*values)


def case_diagnostic(record, fixture, assessment):
    physical = {name: list(map(float, oracle.vector(values)))
                for name, values in record["physical_loads"].items()}
    algebraic = {name: list(map(float, oracle.vector(values)))
                 for name, values in record["algebraic_loads"].items()}
    force_ref = float(Fraction(fixture["mu"]) * Fraction(fixture["U_ref"]) * Fraction(fixture["L_ref"]))
    moment_ref = force_ref * float(Fraction(fixture["L_ref"]))
    total_force = [sum(pair) for pair in zip(physical["pressure_force"], physical["shear_force"])]
    total_moment = [sum(pair) for pair in zip(physical["pressure_moment"], physical["shear_moment"])]
    gaps = {}
    for quantity, total, scale in (("force", total_force, force_ref), ("moment", total_moment, moment_ref)):
        body = algebraic["body_" + quantity]
        components = [p - a for p, a in zip(total, body)]
        body_norm = norm(body)
        gaps[quantity] = {
            "components": components, "norm": norm(components),
            "components_over_reference": [v / scale for v in components],
            "norm_over_reference": norm(components) / scale,
            "norm_relative_to_algebraic": None if body_norm == 0 else norm(components) / body_norm,
            "components_relative_to_algebraic": [None if a == 0 else v / abs(a) for v, a in zip(components, body)],
        }
    result = {"counts": fixture["counts"], "spacing_m": list(map(float, fixture["spacing"])),
              "F_ref_N": force_ref, "M_ref_N_m": moment_ref, "physical_loads": physical,
              "algebraic_loads": algebraic, "physical_total_force_N": total_force,
              "physical_total_moment_N_m": total_moment, "physical_minus_algebraic": gaps,
              "actual_solve_report": record["solve_report"], "independent_assessment": assessment}
    if fixture["boundary"] == "polynomial-mms-dirichlet":
        result["mms_volume_weighted_l2"] = {"velocity": assessment["coarse_mms_velocity_l2"],
                                            "pressure": assessment["coarse_mms_pressure_l2"]}
    return result


def directional_series(cases):
    families = {}
    for kind in ("straight", "mms"):
        for axis in ("x", "y"):
            ids = [f"{kind}-base", f"{kind}-{axis}-medium", f"{kind}-{axis}-fine"]
            levels = [cases[case] for case in ids]
            series = {}
            for group in ("physical_loads", "algebraic_loads"):
                for quantity in levels[0][group]:
                    for component in range(3):
                        series[f"{group}.{quantity}.{component}"] = observed_order(
                            [level[group][quantity][component] for level in levels])
            for quantity in ("physical_total_force_N", "physical_total_moment_N_m"):
                for component in range(3):
                    series[f"{quantity}.{component}"] = observed_order([level[quantity][component] for level in levels])
            for quantity in ("force", "moment"):
                series[f"physical_minus_algebraic.{quantity}.norm_over_reference"] = observed_order(
                    [level["physical_minus_algebraic"][quantity]["norm_over_reference"] for level in levels])
            if kind == "mms":
                for quantity in ("velocity", "pressure"):
                    series["mms_volume_weighted_l2." + quantity] = observed_order(
                        [level["mms_volume_weighted_l2"][quantity] for level in levels])
            families[f"{kind}-{axis}"] = {"cases": ids, "refinement_ratio": 2,
                                            "scope": "conditional-coordinate-diagnostic", "series": series}
    return families


def capture(args):
    data, fixture_sha = load_family(args.fixtures_sha256)
    bindings = source_bindings(args.repo, args.source)
    request = argparse.Namespace(source=args.source, output=args.output, case=list(CASES),
                                 native_contract_sha256=fixture_sha)
    with selected_fixtures():
        report = run.execute(request, args.repo, data, fixture_sha)
    binding = {"schema": "rheon-submarine-directional-capture-v1", "study_profile": PROFILE,
               "status": "captured-awaiting-externally-pinned-cold-recheck", "source": args.source,
               "fixtures_sha256": fixture_sha, "source_files": bindings,
               "origin_sha256_candidate": report["origin_sha256"],
               "assurance": "candidate hash is not an externally supplied provenance pin", "N3": "pending"}
    run.write_json(Path(args.output) / "directional-capture.json", binding)
    return binding


def cold_recheck(args):
    data, fixture_sha = load_family(args.fixtures_sha256)
    bindings = source_bindings(args.repo, args.source)
    origin_sha = pin(args.origin_sha256, "origin-pin")
    capture_dir = Path(args.capture).resolve()
    oracle.require(not capture_dir.is_relative_to(HERE.parents[1]), "output-policy",
                   "native capture must be outside checkout")
    origin = capture_dir / "origin.json"
    oracle.require(oracle.digest(origin) == origin_sha, "origin-integrity",
                   "origin differs from the externally supplied pin")
    manifest = oracle.read_json(origin)
    oracle.require(isinstance(manifest.get("outputs"), dict) and set(manifest["outputs"]) == set(CASES),
                   "directional-completeness", "origin must contain all ten cases exactly")
    diagnostics = {}
    provenance = {}
    with selected_fixtures():
        for case in CASES:
            record_path = capture_dir / "cases" / case / "record.json"
            provenance[case] = run.verify_origin(origin, origin_sha, args.source, record_path,
                                                  fixture_sha, case, args.repo)
            provenance[case]["assurance"] = "externally supplied integrity pin; no signed/authenticated remote attestation"
            record = oracle.read_json(record_path)
            oracle.require(record.get("fixture_id") == case, "fixture-id", "wrong directional record association")
            assessment = oracle.validate_record(record, data["cases"][case], fixture_sha)
            oracle.require(isinstance(assessment.get("constitutive_controls"), list) and
                           len(assessment["constitutive_controls"]) == 34,
                           "control-fixture", "directional capture requires all frozen constitutive controls")
            diagnostics[case] = case_diagnostic(record, data["cases"][case], assessment)
    summary = {"schema": "rheon-submarine-directional-study-v1", "study_profile": PROFILE,
               "status": "cold-rechecked-directional-diagnostic", "source": args.source,
               "source_tree": manifest["source_tree"], "source_files": bindings,
               "fixtures_sha256": fixture_sha, "origin_sha256": origin_sha,
               "operator_profile": oracle.PROFILE, "record_schema": oracle.SCHEMA,
               "provenance": provenance, "cases": diagnostics, "families": directional_series(diagnostics),
               "order_resolution_heuristic": "4096*binary64-epsilon*sum(abs(three-level-values));no-unit-floor",
               "limits": "X/Y-only fixed-domain descriptive response; Z/coupled/domain uncertainty unresolved",
               "N3": "pending", "assurance": "integrity under external pins; no signed remote attestation"}
    output = run.output_directory(args.output, args.repo)
    run.write_json(output / "directional-study.json", summary)
    return summary


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("capture", "recheck"):
        command = commands.add_parser(name)
        command.add_argument("--source", required=True)
        command.add_argument("--fixtures-sha256", required=True)
        command.add_argument("--repo", type=Path, default=HERE.parents[1])
        command.add_argument("--output", required=True)
        if name == "recheck":
            command.add_argument("--capture", required=True)
            command.add_argument("--origin-sha256", required=True)
    args = parser.parse_args(argv)
    try:
        result = capture(args) if args.command == "capture" else cold_recheck(args)
        print(json.dumps(result, sort_keys=True, allow_nan=False))
        return 0
    except oracle.Failure as exc:
        print(json.dumps({"status": "failed", **exc.data}, sort_keys=True), file=sys.stderr)
        return 2
    except (OSError, KeyError, ValueError, TypeError, OverflowError) as exc:
        print(json.dumps({"status": "failed", "failure_kind": "unavailable", "claim_id": "directional-execution",
                          "detail": str(exc)}, sort_keys=True), file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        print(json.dumps({"status": "cancelled", "failure_kind": "unavailable"}), file=sys.stderr)
        return 130


if __name__ == "__main__":
    sys.exit(main())
