"""Bounded two-cell CPU equation fixture; no learned model or training."""
import hashlib
import json
import math
from pathlib import Path
import sys
import time

BEHAVIOR = "solve"
manifest_path = Path(sys.argv[-1])
root = manifest_path.parent
raw_manifest = manifest_path.read_bytes()
manifest = json.loads(raw_manifest)


def require(condition, message):
    if not condition:
        raise ValueError(message)


print(json.dumps(dict(kind="adapter_ready", id=manifest["id"],
    version=manifest["version"], manifest_sha256=hashlib.sha256(raw_manifest).hexdigest(),
    device="cpu", optimizer_updates=0)), flush=True)
while raw := sys.stdin.buffer.readline(8193):
    require(len(raw) <= 8192 and raw.endswith(b"\n"), "bounded complete request")
    request = json.loads(raw)
    context = request["context"]
    require(request["kind"] == "predict" and request["schema"] == "rheon-pressure-context-request-v1", "request schema")
    require(request["stage"] == "sealed-mac-pressure-context-v1", "physical stage")
    require(context["mode"] == "accepted_candidate" and context["run_id"] == "771", "accepted Session context")
    require(context["grid"] == dict(counts_xyz=[2, 1, 1], spacing_xyz_m=[0.25]*3, origin_xyz_m=[0.0]*3), "two physical cells")
    require(context["dt_s"] == 0.0625 and context["interval_end_s"] == context["interval_start_s"] + 0.0625, "actual interval")
    require(context["generation"] == request["id"] >= 2, "native prefix before prediction")
    prior = context["prior"]
    require(prior["status"] == dict(kind="accepted") and prior["generation"] + 1 == request["id"], "accepted prior lineage")
    require(prior["interval_end_s"] == context["interval_start_s"], "prior endpoint")
    require(set(request["inputs"]) == set(manifest["inputs"]), "configured named ports")
    fields = {}
    units = dict(rhs="m3/s2", previous_pressure="Pa", initial_residual="m3/s2", x="m", density="kg/m3", dt="s")
    for name, semantic in manifest["inputs"].items():
        field = request["inputs"][name]
        scalar = semantic in ("density", "dt")
        require(field["semantic"] == semantic and field["units"] == units[semantic], "physical port meaning")
        require(field["association"] == ("global" if scalar else "cell"), "port association")
        require(field["dimensions"] == ([1, 1, 1] if scalar else [2, 1, 1]), "port dimensions")
        require(field["scalar"] == "f64" and field["layout"] == "x_fast", "port scalar/layout")
        values = field["values"]
        require(len(values) == (1 if scalar else 2) and all(math.isfinite(v) for v in values), "bounded finite values")
        fields[semantic] = values
    b = fields["rhs"]
    previous = fields["previous_pressure"]
    rho, dt = fields["density"][0], fields["dt"][0]
    require(rho == 2 and dt == 0.0625 and fields["x"] == [0.125, 0.375], "physical scalars/centres")
    require(b[1] != 0 and b[0] == -b[1] and previous[0] == 0, "nonzero full compatible RHS and prior gauge")
    # Independent two-cell sealed operator: k=h/rho, A p=(-k*p1,k*p1).
    k = 0.25 / rho
    require(fields["initial_residual"] == [b[0] + k*previous[1], b[1] - k*previous[1]], "actual b-A*accepted_prior")
    with (root / "requests.jsonl").open("ab") as log:
        log.write(raw)
    marker = root / "first-request-observed"
    first = not marker.exists()
    marker.write_text("observed real request", encoding="utf-8")
    if BEHAVIOR == "exit_once" and first:
        raise SystemExit(7)
    if BEHAVIOR == "stall_once" and first:
        time.sleep(60)
    full = [0.0, b[1] * rho / 0.25]
    if BEHAVIOR == "bad_once" and first:
        full[1] += 1.0
    role = manifest["output"]["role"]
    result = full if role == "full_pressure" else [full[i] - previous[i] for i in range(2)]
    print(json.dumps(dict(kind="candidate", id=request["id"], role=role,
        pressure=result, device="cpu", optimizer_updates=0), allow_nan=False), flush=True)
