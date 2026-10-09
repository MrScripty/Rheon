#!/usr/bin/env python3
"""Independent exact rational cubature versus the bounded native mesh-load bridge.

No build, fluid solve, time advancement, or geometry campaign is performed.
Degree-two cubature integrates the interpolated fields directly; it does not
use the production consistent-nodal-force formula. Receipts stay outside Git.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from fractions import Fraction as Q
from pathlib import Path
import subprocess
import sys

TOLERANCE = 1e-12
ZERO = (Q(0), Q(0), Q(0))


def vector(values):
    return tuple(Q(str(x)) for x in values)


def add(a, b):
    return tuple(x + y for x, y in zip(a, b))


def sub(a, b):
    return tuple(x - y for x, y in zip(a, b))


def mul(a, scalar):
    return tuple(x * scalar for x in a)


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0])


def rational_sqrt(value):
    numerator, denominator = math.isqrt(value.numerator), math.isqrt(value.denominator)
    if numerator * numerator != value.numerator or denominator * denominator != value.denominator:
        raise ValueError("oracle fixtures require rational area and normal")
    return Q(numerator, denominator)


def case(name, mode, vertices, triangles, loads, reference=ZERO,
         velocity=(1, -2, 3), omega=(4, 5, -6)):
    return dict(name=name, mode=mode, vertices=[vector(p) for p in vertices],
                triangles=[tuple(t) for t in triangles],
                loads=([[Q(str(p)) for p in row] for row in loads]
                       if mode == "pressure" else
                       [[vector(t) for t in row] for row in loads]),
                reference=vector(reference), velocity=vector(velocity), omega=vector(omega))


def fixtures():
    triangle = [(0, 0, 0), (2, 0, 0), (0, 1, 0)]
    varying = case("varying-traction-centroid-counterexample", "traction", triangle,
                   [(0, 1, 2)], [[ZERO, (0, 0, 3), ZERO]],
                   velocity=(1, 2, 3), omega=(4, 5, 6))
    oblique = case("oblique-affine-pressure", "pressure",
                   [(0, 0, 0), (2, 0, 0), (0, "3/5", "4/5")],
                   [(0, 1, 2)], [[1, 2, 4]])
    cases = [varying, oblique]
    offset = vector((3, -2, 5))
    translated = {**oblique, "name": "translated-geometry-and-reference",
                  "vertices": [add(p, offset) for p in oblique["vertices"]],
                  "reference": offset}
    cases.append(translated)
    # A changed reference changes the translation component of the SAME field.
    rebased = {**oblique, "name": "rebased-reference-same-rigid-field",
               "reference": offset,
               "velocity": add(oblique["velocity"], cross(oblique["omega"], offset))}
    cases.append(rebased)
    cases.append({**oblique, "name": "pressure-reversed-winding",
                  "triangles": [(0, 2, 1)], "loads": [[1, 4, 2]]})
    cases.append({**varying, "name": "traction-reversed-winding-remapped-corners",
                  "triangles": [(0, 2, 1)], "loads": [[ZERO, ZERO, vector((0, 0, 3))]]})
    corners = varying["loads"][0]
    vertices = varying["vertices"] + [mul(add(varying["vertices"][i], varying["vertices"][j]), Q(1, 2))
                                        for i, j in [(0, 1), (1, 2), (2, 0)]]
    values = corners + [mul(add(corners[i], corners[j]), Q(1, 2))
                        for i, j in [(0, 1), (1, 2), (2, 0)]]
    subdivisions = [(0, 3, 5), (3, 1, 4), (5, 4, 2), (3, 4, 5)]
    cases.append({**varying, "name": "affine-traction-four-triangle-subdivision",
                  "vertices": vertices, "triangles": subdivisions,
                  "loads": [[values[i] for i in indices] for indices in subdivisions]})
    cases.append({**varying, "name": "duplicate-facet-counts-twice",
                  "triangles": [(0, 1, 2), (0, 1, 2)], "loads": [corners, corners]})
    cases.append(case("anisotropic-all-three-traction-components", "traction",
                      [(0, 0, 0), (4, 0, 0), (0, "1/2", 0)], [(0, 1, 2)],
                      [[(1, -2, 3), (-4, 5, -6), (7, -8, 9)]], reference=("1/4", "-3/2", 2)))
    cases.append(case("oblique-all-three-traction-components", "traction", oblique["vertices"],
                      [(0, 1, 2)], [[(1, 2, 3), (4, -5, 6), (-7, 8, -9)]]))
    cases.append(case("zero-loads-preserve-triangles", "traction", triangle,
                      [(0, 1, 2)], [[ZERO, ZERO, ZERO]]))
    cases.append({**oblique, "name": "zero-twist-preserves-force-and-torque",
                  "velocity": ZERO, "omega": ZERO})
    couple_vertices = triangle + [add(vector(p), vector((3, 0, 0))) for p in triangle]
    cases.append(case("disconnected-pure-couple", "traction", couple_vertices,
                      [(0, 1, 2), (3, 4, 5)],
                      [[(0, 0, 1)] * 3, [(0, 0, -1)] * 3]))
    # Explicit outward-oriented box, not a general watertightness assertion.
    box = [(0, 0, 0), (2, 0, 0), (2, 1, 0), (0, 1, 0),
           (0, 0, 1), (2, 0, 1), (2, 1, 1), (0, 1, 1)]
    indices = [(0, 2, 1), (0, 3, 2), (4, 5, 6), (4, 6, 7),
               (0, 1, 5), (0, 5, 4), (3, 7, 6), (3, 6, 2),
               (0, 4, 7), (0, 7, 3), (1, 2, 6), (1, 6, 5)]
    pressures = [Q(7 + x + 2*y + 3*z) for x, y, z in box]
    affine_box = case("closed-box-affine-pressure", "pressure", box, indices,
                      [[pressures[i] for i in row] for row in indices])
    cases.append(affine_box)
    cases.append({**affine_box, "name": "closed-box-affine-pressure-offset",
                  "loads": [[p + 11 for p in row] for row in affine_box["loads"]]})
    cases.append({**affine_box, "name": "closed-box-affine-pressure-center-reference",
                  "reference": vector((1, "1/2", "1/2"))})
    cases.append({**affine_box, "name": "closed-box-constant-pressure-cancels",
                  "loads": [[Q(13)] * 3 for _ in indices]})
    return cases


def interpolate(values, barycentric):
    return tuple(sum(barycentric[i] * values[i][d] for i in range(3)) for d in range(3))


def oracle(fixture):
    force, torque, power = ZERO, ZERO, Q(0)
    triangle_loads = []
    for ordinal, indices in enumerate(fixture["triangles"]):
        points = [fixture["vertices"][i] for i in indices]
        oriented_double_area = cross(sub(points[1], points[0]), sub(points[2], points[0]))
        double_area = rational_sqrt(dot(oriented_double_area, oriented_double_area))
        area = double_area / 2
        normal = mul(oriented_double_area, 1 / double_area)
        loads = fixture["loads"][ordinal]
        traction = ([mul(normal, -p) for p in loads]
                    if fixture["mode"] == "pressure" else loads)
        local_force, local_torque = ZERO, ZERO
        nodal = [ZERO, ZERO, ZERO]
        # Exact degree-two cubature: all products here have degree <= 2.
        for special in range(3):
            barycentric = [Q(2, 3) if i == special else Q(1, 6) for i in range(3)]
            position = interpolate(points, barycentric)
            traction_at_point = interpolate(traction, barycentric)
            weighted_force = mul(traction_at_point, area / 3)
            arm = sub(position, fixture["reference"])
            velocity = add(fixture["velocity"], cross(fixture["omega"], arm))
            local_force = add(local_force, weighted_force)
            local_torque = add(local_torque, cross(arm, weighted_force))
            power += dot(weighted_force, velocity)
            for i in range(3):
                nodal[i] = add(nodal[i], mul(weighted_force, barycentric[i]))
        force, torque = add(force, local_force), add(torque, local_torque)
        triangle_loads.append(dict(triangle=ordinal, area_m2=area, nodal_force=nodal,
                                   force=local_force, torque=local_torque))
    if power != dot(force, fixture["velocity"]) + dot(torque, fixture["omega"]):
        raise AssertionError("exact cubature violates rigid virtual-work identity")
    return dict(force=force, torque=torque, rigid_power=power, nodal_power=power,
                power_defect=Q(0), triangles=len(fixture["triangles"]), triangle_loads=triangle_loads)


def exact_json(value):
    if isinstance(value, Q):
        return str(value)
    if isinstance(value, (list, tuple)):
        return [exact_json(v) for v in value]
    if isinstance(value, dict):
        return {k: exact_json(v) for k, v in value.items()}
    return value


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_snapshot(root):
    listed = subprocess.run(["git", "ls-files", "-z"], cwd=root, check=True, capture_output=True).stdout
    paths = {p.decode() for p in listed.split(b"\0") if p}
    paths.update(["src/mesh_traction.rs", "examples/mesh_traction.rs", "tools/check_mesh_traction.py"])
    return {p: sha(root / p) for p in sorted(paths) if (root / p).is_file()}


def input_text(fixture):
    tokens = [fixture["mode"], str(len(fixture["vertices"])), str(len(fixture["triangles"]))]
    tokens.extend(repr(float(x)) for p in fixture["vertices"] for x in p)
    tokens.extend(str(i) for row in fixture["triangles"] for i in row)
    tokens.extend(repr(float(x)) for key in ["reference", "velocity", "omega"] for x in fixture[key])
    if fixture["mode"] == "pressure":
        tokens.extend(repr(float(x)) for row in fixture["loads"] for x in row)
    else:
        tokens.extend(repr(float(x)) for row in fixture["loads"] for t in row for x in t)
    text = " ".join(tokens) + "\n"
    if len(text.encode()) > 65536 or len(fixture["vertices"]) > 192 or len(fixture["triangles"]) > 64:
        raise ValueError("fixture exceeds bridge bounds")
    return text


def strict_object(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise ValueError(f"duplicate JSON key: {key}")
        obj[key] = value
    return obj


def compare(actual, expected, path="output"):
    if isinstance(expected, Q):
        if type(actual) not in (int, float) or not math.isfinite(actual):
            raise AssertionError(f"{path}: expected finite numeric value")
        # Absolute tolerance is fixed; no fixture-specific or relative widening.
        error = abs(Q(actual) - expected)
        if error > Q(str(TOLERANCE)):
            raise AssertionError(f"{path}: {actual!r} differs from exact {expected} by {error}")
        return float(error)
    if isinstance(expected, int):
        if type(actual) is not int or actual != expected:
            raise AssertionError(f"{path}: expected integer {expected}")
        return 0.0
    if isinstance(expected, (list, tuple)):
        if not isinstance(actual, list) or len(actual) != len(expected):
            raise AssertionError(f"{path}: list length mismatch")
        return max((compare(a, e, f"{path}[{i}]") for i, (a, e) in enumerate(zip(actual, expected))), default=0)
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise AssertionError(f"{path}: object required")
        return max((compare(actual[key], value, f"{path}.{key}") for key, value in expected.items()), default=0)
    raise TypeError(type(expected))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--executable", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    executable, output = args.executable.resolve(strict=True), args.output.resolve()
    if output == root or root in output.parents:
        raise ValueError("evidence directory must be outside the source repository")
    output.mkdir(parents=True, exist_ok=False)
    snapshot = source_snapshot(root)
    binary_sha = sha(executable)
    git_head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, check=True, capture_output=True, text=True).stdout.strip()
    git_status = subprocess.run(["git", "status", "--porcelain=v1"], cwd=root, check=True, capture_output=True, text=True).stdout
    cases = fixtures()
    anchors = {c["name"]: oracle(c) for c in cases}
    anchor_checks = [(cases[0]["name"], "force", vector((0, 0, 1))),
                     (cases[0]["name"], "torque", vector(("1/4", -1, 0))),
                     (cases[1]["name"], "force", vector((0, "28/15", "-7/5"))),
                     (cases[1]["name"], "torque", vector(("-11/12", "9/10", "6/5"))),
                     (cases[1]["name"], "rigid_power", Q(-143, 10)),
                     ("closed-box-affine-pressure", "force", vector((-2, -4, -6))),
                     ("closed-box-affine-pressure", "torque", vector((-1, 5, -3))),
                     ("closed-box-affine-pressure-center-reference", "torque", ZERO)]
    for name, field, value in anchor_checks:
        if anchors[name][field] != value:
            raise AssertionError(f"rational anchor mismatch: {name}.{field}")
    records = []
    receipt = dict(schema="rheon.mesh-traction.rational-comparison", version=1,
                   command=sys.argv, cwd=str(root), source_head=git_head, source_dirty=git_status,
                   source_sha256=snapshot, executable=str(executable), executable_sha256=binary_sha,
                   absolute_tolerance=TOLERANCE, oracle="exact Fraction degree-two triangle cubature",
                   coordinate_semantics="rational fixture ideals serialized once to nearest binary64; no IEEE proof",
                   physics_steps=0, held_campaigns=0, fixtures=records, qualified=False)
    try:
        for fixture in cases:
            expected = anchors[fixture["name"]]
            text = input_text(fixture)
            prefix = output / fixture["name"]
            prefix.with_suffix(".input.txt").write_text(text)
            prefix.with_suffix(".fixture.json").write_text(json.dumps(exact_json(fixture), indent=2) + "\n")
            prefix.with_suffix(".expected.json").write_text(json.dumps(exact_json(expected), indent=2) + "\n")
            run = subprocess.run([str(executable)], input=text, capture_output=True, text=True, timeout=10, cwd=root)
            prefix.with_suffix(".stdout.json").write_text(run.stdout)
            prefix.with_suffix(".stderr.txt").write_text(run.stderr)
            if run.returncode:
                raise AssertionError(f"{fixture['name']}: native exit {run.returncode}: {run.stderr}")
            actual = json.loads(run.stdout, object_pairs_hook=strict_object,
                                parse_constant=lambda x: (_ for _ in ()).throw(ValueError(x)))
            error = compare(actual, expected)
            records.append(dict(name=fixture["name"], command=[str(executable)],
                                input_sha256=sha(prefix.with_suffix(".input.txt")),
                                stdout_sha256=sha(prefix.with_suffix(".stdout.json")),
                                expected_sha256=sha(prefix.with_suffix(".expected.json")),
                                returncode=run.returncode, maximum_absolute_error=error))
            if sha(executable) != binary_sha:
                raise AssertionError("native executable changed during qualification")
        if source_snapshot(root) != snapshot:
            raise AssertionError("source changed during qualification")
        if subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, check=True, capture_output=True, text=True).stdout.strip() != git_head:
            raise AssertionError("source commit changed during qualification")
        receipt["qualified"] = True
    except Exception as error:
        receipt["failure"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        receipt["evidence_sha256"] = {p.name: sha(p) for p in sorted(output.iterdir()) if p.is_file()}
        (output / "qualification.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(dict(qualified=True, fixtures=len(records), executable_sha256=binary_sha,
                          output=str(output), absolute_tolerance=TOLERANCE)))


if __name__ == "__main__":
    main()
