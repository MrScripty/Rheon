"""Independent <=64-cell sealed pressure fixture; no learned model or training."""
from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path
import sys
import time

BEHAVIOR = "solve"
root = Path(sys.argv[-1]).parent
raw_manifest = Path(sys.argv[-1]).read_bytes()
manifest = json.loads(raw_manifest)
(root / "child-started").write_text("analytic fixture started", encoding="utf-8")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def matrix(counts, spacing, rho):
    n = math.prod(counts)
    require(1 <= n <= 64, "bounded independent dense solve")
    a = [[Fraction(0) for _ in range(n)] for _ in range(n)]
    strides = [1, counts[0], counts[0] * counts[1]]
    for i in range(n):
        for axis in range(3):
            if (i // strides[axis]) % counts[axis] + 1 == counts[axis]:
                continue
            j = i + strides[axis]
            area = spacing[(axis + 1) % 3] * spacing[(axis + 2) % 3]
            weight = Fraction(area / (rho * spacing[axis]))
            a[i][i] += weight
            a[j][j] += weight
            a[i][j] -= weight
            a[j][i] -= weight
    return a


def solve(a, rhs):
    # Remove the cell-zero gauge row/column. Exact rational elimination avoids
    # fixture roundoff masquerading as a host full-fraction transport defect.
    n = len(rhs)
    rows = [a[i][1:] + [Fraction(rhs[i])] for i in range(1, n)]
    for column in range(n - 1):
        pivot = next(i for i in range(column, n - 1) if rows[i][column])
        rows[column], rows[pivot] = rows[pivot], rows[column]
        scale = rows[column][column]
        rows[column] = [v / scale for v in rows[column]]
        for i in range(n - 1):
            if i != column:
                scale = rows[i][column]
                rows[i] = [v - scale * w for v, w in zip(rows[i], rows[column])]
    return [0.0] + [float(row[-1]) for row in rows]


print(json.dumps(dict(kind="adapter_ready", id=manifest["id"], version=manifest["version"],
    manifest_sha256=hashlib.sha256(raw_manifest).hexdigest(), device="cpu", optimizer_updates=0)), flush=True)
while raw := sys.stdin.buffer.readline(131073):
    require(len(raw) <= 131072 and raw.endswith(b"\n"), "bounded complete input")
    request = json.loads(raw)
    context = request["context"]
    geometry = context["grid"]
    counts, spacing, origin = geometry["counts_xyz"], geometry["spacing_xyz_m"], geometry["origin_xyz_m"]
    n = math.prod(counts)
    require(context["mode"] == "accepted_candidate" and request["id"] == context["generation"], "accepted context/correlation")
    require(request["schema"] == "rheon-pressure-context-request-v1" and request["stage"] == "sealed-mac-pressure-context-v1", "physical protocol")
    require(context["interval_end_s"] == context["interval_start_s"] + context["dt_s"], "actual endpoint")
    if context["prior"] is not None:
        prior = context["prior"]
        require(prior["status"] == dict(kind="accepted") and prior["generation"] + 1 == context["generation"], "accepted prior")
        require(prior["interval_end_s"] == context["interval_start_s"], "prior endpoint")
    fields = {}
    units = dict(rhs="m3/s2", initial_residual="m3/s2", previous_pressure="Pa", x="m", y="m", z="m", density="kg/m3", dt="s")
    require(set(request["inputs"]) == set(manifest["inputs"]), "arbitrary configured ports")
    for name, semantic in manifest["inputs"].items():
        field = request["inputs"][name]
        scalar = semantic in ("density", "dt")
        require(field["semantic"] == semantic and field["units"] == units[semantic], "physical meaning")
        require(field["scalar"] == "f64" and field["layout"] == "x_fast", "scalar/layout")
        require(field["association"] == ("global" if scalar else "cell") and field["dimensions"] == ([1, 1, 1] if scalar else counts), "association/shape")
        values = field["values"]
        require(len(values) == (1 if scalar else n) and all(math.isfinite(v) for v in values), "finite shape")
        fields[semantic] = values
    rho = fields["density"][0]
    require(fields["dt"][0] == context["dt_s"] and rho == manifest["support"]["density_kg_m3"], "actual physical scalars")
    rhs = fields["rhs"]
    require(any(value != 0.0 for value in rhs), "fixture must observe nonzero authoritative RHS")
    previous = fields.get("previous_pressure", [0.0] * n)
    require(previous[0] == 0.0, "canonical initial gauge")
    a = matrix(counts, spacing, rho)
    if context["prior"] is None or "previous_pressure" in fields:
        residual = [float(Fraction(rhs[i]) - sum(a[i][j] * Fraction(previous[j]) for j in range(n))) for i in range(n)]
        require(all(abs(x-y) <= 1e-12 * max(1.0, abs(x)) for x, y in zip(residual, fields["initial_residual"])), "authoritative full initial residual")
    # A full-pressure package may omit the previous-pressure port. Its later
    # residual reference is unavailable here; the full solve uses actual RHS.
    for axis, semantic in enumerate(("x", "y", "z")):
        if semantic in fields:
            stride = math.prod(counts[:axis])
            expected = [origin[axis] + ((i // stride) % counts[axis] + 0.5) * spacing[axis] for i in range(n)]
            require(fields[semantic] == expected, "x-fast physical centres")
    with (root / "requests.jsonl").open("ab") as log:
        log.write(raw)
    marker = root / "first-request-observed"
    first = not marker.exists()
    marker.write_text("observed consumed request", encoding="utf-8")
    if BEHAVIOR == "exit_once" and first:
        raise SystemExit(7)
    if BEHAVIOR == "stall_once" and first:
        time.sleep(60)
    if BEHAVIOR == "invalid_once" and first:
        print('{"kind":"candidate","bogus":1}', flush=True)
        continue
    result = solve(a, rhs)
    if BEHAVIOR == "bad_once" and first:
        result[1] += 1.0
    role = manifest["output"]["role"]
    if role == "pressure_correction":
        require(context["prior"] is not None, "correction requires accepted prior")
        result = [value - previous[i] for i, value in enumerate(result)]
    print(json.dumps(dict(kind="candidate", id=request["id"], role=role, pressure=result,
        device="cpu", optimizer_updates=0), allow_nan=False), flush=True)
