#!/usr/bin/env python3
"""Independent native-bit oracle for bounded M3 fixtures.

Reads SQLite directly with Python's standard library. Derives divergence by
coordinate/face lookups and reductions with math.fsum, independently of Rust's
streaming hypot and index arithmetic. This checks analytical test fixtures and
analysis-v1 selection identity, not archive acceptance or physical accuracy.
"""
import argparse
import hashlib
import math
import sqlite3
import struct
from pathlib import Path


def u64(value):
    return struct.pack("<Q", value)


def text(value):
    data = value.encode("utf-8")
    return u64(len(data)) + data


def optional(raw):
    return b"\0" if raw is None else b"\1" + raw


def load(path, ordinal):
    with sqlite3.connect(Path(path).resolve().as_uri() + "?mode=ro", uri=True) as db:
        run = db.execute("SELECT run_id,revision,solver,configuration,geometry,provenance FROM run WHERE id=1").fetchone()
        step = db.execute("SELECT generation,time,dt,stage,controls FROM steps WHERE ordinal=?", (ordinal,)).fetchone()
        if run is None or step is None:
            raise ValueError("missing run/publication")
        fields = db.execute("SELECT name,units,association,stage,scalar_type,evaluation_time,dimensions,components,data FROM fields WHERE ordinal=? ORDER BY field_index", (ordinal,)).fetchall()
    run_id, revision, solver, config, geometry, provenance = run
    counts = struct.unpack("<3Q", geometry[:24])
    spacing = struct.unpack("<3d", geometry[24:48])
    run_bytes = b"rheon-native-run-v1\0" + run_id + b"".join(map(text, (revision, solver, config, provenance))) + geometry
    generation, time, dt, stage, controls = step
    selected_bytes = b"rheon-native-selection-v1\0" + hashlib.sha256(run_bytes).digest() + u64(ordinal) + generation + time + optional(dt) + bytes([stage]) + text(controls) + u64(len(fields))
    decoded = {}
    for name, units, association, field_stage, scalar, evaluation, dimensions, components, raw in fields:
        selected_bytes += b"".join(map(text, (name, units, association, field_stage, scalar))) + optional(evaluation) + dimensions + u64(components) + u64(len(raw)) + raw
        if scalar in ("f32", "f64"):
            kind, width = ("f", 4) if scalar == "f32" else ("d", 8)
            decoded[name] = [v[0] for v in struct.iter_unpack("<" + kind, raw)]
            assert len(raw) % width == 0
            assert all(math.isfinite(v) for v in decoded[name])
    return counts, spacing, decoded, hashlib.sha256(selected_bytes).hexdigest()


def values(source, field):
    if field != "divergence":
        return source[2][field]
    return [value for value, _ in divergence_samples(source)]


def divergence_samples(source):
    counts, spacing, fields, _ = source
    nx, ny, nz = counts
    axes = ("velocity_x", "velocity_y", "velocity_z")
    lookup = []
    for axis, name in enumerate(axes):
        dims = [nx, ny, nz]
        dims[axis] += 1
        coordinates = ((i, j, k) for k in range(dims[2]) for j in range(dims[1]) for i in range(dims[0]))
        lookup.append(dict(zip(coordinates, fields[name], strict=True)))
    result = []
    for k in range(nz):
        for j in range(ny):
            for i in range(nx):
                coordinate = (i, j, k)
                derivatives = []
                for axis in range(3):
                    upper = list(coordinate)
                    upper[axis] += 1
                    derivatives.append((lookup[axis][tuple(upper)] - lookup[axis][coordinate]) / spacing[axis])
                assert all(math.isfinite(value) for value in derivatives)
                # Rust uses two sequential adds; Python fsum rounds the exact sum.
                # A conservative bound on their difference is 4*epsilon*sum|d|.
                # The additive term covers subnormal final-add rounding.
                bound = 4 * math.ulp(1.0) * math.fsum(abs(value) for value in derivatives) + 4 * math.ulp(0.0)
                value = math.fsum(derivatives)
                assert math.isfinite(value) and math.isfinite(bound)
                result.append((value, bound))
    return result


def roundoff_bounds(source, field):
    bounds = [bound for _, bound in divergence_samples(source)] if field == "divergence" else [0.0]
    return math.hypot(*bounds), max(bounds)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference")
    parser.add_argument("candidate")
    parser.add_argument("--reference-ordinal", type=int, default=0)
    parser.add_argument("--candidate-ordinal", type=int, default=0)
    parser.add_argument("--field", choices=("tracer", "divergence", "velocity_x", "velocity_y", "velocity_z", "interval_pressure"), required=True)
    parser.add_argument("--expect", nargs=4, metavar=("L2", "RMSE", "LINF", "RELATIVE_L2"), required=True)
    parser.add_argument("--expect-selection", required=True)
    parser.add_argument("--expect-candidate-selection", required=True)
    parser.add_argument("--remove-mean-offset", action="store_true")
    args = parser.parse_args()
    a = load(args.reference, args.reference_ordinal)
    b = load(args.candidate, args.candidate_ordinal)
    assert a[3] == args.expect_selection, "selection digest differs from independent byte oracle"
    assert b[3] == args.expect_candidate_selection, "candidate selection digest differs"
    x, y = values(a, args.field), values(b, args.field)
    assert len(x) == len(y) and len(x) > 0
    if args.remove_mean_offset:
        assert args.field == "interval_pressure"
        offset = math.fsum(candidate-reference for reference, candidate in zip(x, y, strict=True)) / len(x)
        y = [value-offset for value in y]
    squares = [(candidate-reference)**2 for reference, candidate in zip(x, y, strict=True)]
    l2 = math.sqrt(math.fsum(squares))
    norm = math.sqrt(math.fsum(v*v for v in x))
    results = [l2, math.sqrt(math.fsum(squares)/len(x)), max(abs(candidate-reference) for reference, candidate in zip(x, y, strict=True)), None if norm == 0 else l2/norm]
    expected = [None if v == "undefined" else float(v) for v in args.expect]
    reference_bound, reference_max = roundoff_bounds(a, args.field)
    candidate_bound, candidate_max = roundoff_bounds(b, args.field)
    error_bound = reference_bound + candidate_bound
    absolute_bounds = [error_bound, error_bound / math.sqrt(len(x)), reference_max + candidate_max]
    assert all(math.isfinite(value) for value in results[:3] + absolute_bounds + [norm, reference_bound])
    for name, actual, target, bound in zip(("L2", "RMSE", "Linf"), results[:3], expected[:3], absolute_bounds, strict=True):
        assert target is not None and math.isfinite(target)
        tolerance = max(1e-14, abs(target) * 2e-14)
        assert abs(actual-target) <= bound+tolerance, f"{name}: {actual} != {target}, arithmetic bound {bound}"
    actual, target = results[3], expected[3]
    if args.field == "divergence" and norm <= reference_bound:
        print("divergence relative-L2 oracle unavailable: reference norm interval reaches zero; absolute metrics and exact identities checked")
    elif actual is None or target is None:
        assert actual is target, "relative L2: unavailable mismatch"
    else:
        assert math.isfinite(target)
        # Propagate the independently bounded absolute errors through E/N.
        lower = max(l2-error_bound, 0.0) / (norm+reference_bound)
        upper = (l2+error_bound) / (norm-reference_bound)
        assert math.isfinite(lower) and math.isfinite(upper)
        tolerance = max(1e-14, abs(target) * 2e-14)
        assert lower-tolerance <= target <= upper+tolerance, f"relative L2: {target} outside arithmetic interval [{lower}, {upper}]"
    print(f"independent SQLite/native-bit oracle passed: {args.field}, {len(x)} samples")


if __name__ == "__main__":
    main()
