"""Bind the original pressure-forward refused second step; never run Rheon."""
from pathlib import Path
from fractions import Fraction as Q
import copy
import hashlib
import json
import math
import struct

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
FROZEN_SOURCE = "f0b81b4a5f76cb706e645fad4f38faf028c5727e"
SELECTOR = ("pressure_state", "forward", 0.00078125)


def require(ok, message):
    if not ok:
        raise ValueError(message)


def bits(x):
    return struct.pack(">d", x)


def qstr(x):
    x = Q(x)
    return f"{x.numerator}/{x.denominator}"


def select():
    provenance = {}

    def source(name):
        raw = (ROOT/name).read_bytes()
        provenance[name] = dict(sha256=hashlib.sha256(raw).hexdigest(), selected_lines=[])
        return raw

    def records(name):
        raw = source(name)
        return [(index+1, line, json.loads(line)) for index, line in enumerate(raw.splitlines()) if line.startswith(b"{")]

    def selected_line(name, item):
        index, raw, row = item
        provenance[name]["selected_lines"].append(dict(line=index, raw_sha256=hashlib.sha256(raw).hexdigest()))
        return row

    inputs_name = "evidence/forcing-residual-arithmetic/inputs.json"
    inputs = json.loads(source(inputs_name))
    matches = [(i, r) for i, r in enumerate(inputs["rows"])
               if (r["kind"], r["load"], r["h"]) == SELECTOR and r["accepted_version"] == 1]
    require(len(matches) == 1 and matches[0][0] == 3, "unique frozen row index 3")
    _, input_row = matches[0]
    provenance[inputs_name]["selection"] = "rows[3]"
    pub_name = "evidence/solver-terminal-validation/trace-finer.jsonl"
    pubs = records(pub_name)
    chosen = {}
    for version in (0, 1):
        matches = [item for item in pubs if (item[2].get("kind"), item[2].get("load"), item[2].get("h")) == SELECTOR
                   and item[2].get("model") and item[2].get("field") == "nonconstant" and item[2].get("step") == version]
        require(len(matches) == 1, "unique actual publication")
        chosen[version] = selected_line(pub_name, matches[0])
    old = chosen[1]
    require(old["stamp"] == input_row["stamp"] == {"id": 131, "version": 1}, "actual accepted identity")
    require(bits(old["time"]) == bits(input_row["time"]) == bits(.00078125), "actual accepted time")
    for public_key, input_key in (("end_q", "q"), ("end_eta", "eta"), ("velocity", "old"), ("mass", "mass")):
        require(old[public_key] == input_row[input_key], f"accepted source field: {public_key}")
    require(old["forcing"]["acceleration"] == [.0625, -.125, .03125] and old["forcing"]["force_count"] == 1,
            "whole-domain forward force")
    require(old["pressure_coefficients"] == old["unknowns"][6:] != input_row["unknown"][6:],
            "accepted and refused candidate pressures remain distinct")

    trace_name = "evidence/solver-terminal-validation/trace-finer-stderr.log"
    trace = records(trace_name)
    active, retry = False, False
    checks, terminals, refusals, observations = [], [], [], []
    for item in trace:
        row = item[2]
        if row["event"] == "case":
            active = (row["kind"], row["load"], row["h"]) == SELECTOR
            retry = False
        elif active and row["event"] == "retry_begin":
            retry = True
        elif active and not retry:
            if row["event"] == "newton_check" and row["accepted_version"] == 1:
                checks.append(item)
            elif row["event"] == "terminal_validation":
                terminals.append(item)
            elif row["event"] == "refusal" and row["accepted_version"] == 1:
                refusals.append(item)
            elif row["event"] == "post_window_observation":
                observations.append(item)
    require(len(checks) == 7 and len(terminals) == len(refusals) == 1 and len(observations) == 2,
            "original second-step controller window, excluding retry")
    first_check = selected_line(trace_name, checks[0])
    terminal = selected_line(trace_name, terminals[0])
    refusal = selected_line(trace_name, refusals[0])
    observed = [selected_line(trace_name, item) for item in observations]
    require(first_check["q"] == input_row["q"] and first_check["eta"] == input_row["eta"], "trace start coordinates")
    require(first_check["accepted_time"] == old["time"], "trace start time")
    require(refusal["reason"] == "correction_budget_exhausted" and refusal["calls"] == terminal["calls"] == 50
            and terminal["corrections"] == 7 and terminal["converged"] is False, "baseline controller refusal")
    require(terminal["rate_norm"] == input_row["expected_norm"] == 1.1895444217920825e-13 > 1e-13,
            "pinned terminal norm and unchanged refusal")
    require([r["order"] for r in observed] == [16, 32], "both post-window orders")
    require(observed[0]["unknown"] == input_row["unknown"] and observed[0]["rate"] == input_row["expected_rate"],
            "fixed terminal candidate fields")
    require(all(r["did_not_participate_in_acceptance"] is True for r in observed), "observations not acceptance")

    native_name = "evidence/forcing-residual-arithmetic/native-probe-start-chart.log"
    native_items = [item for item in records(native_name) if item[2]["case"] == 3]
    require(len(native_items) == 1, "unique captured native equation")
    native = selected_line(native_name, native_items[0])
    require(native["h"] == input_row["h"] and native["rate"] == input_row["expected_rate"]
            and native["norm"] == input_row["expected_norm"] and native["published"] is False,
            "exact native capture binding")
    require(native["start_mass"] == old["mass"] and native["end_mass"] == observed[0]["candidate_mass"]
            and native["end_velocity"] == observed[0]["candidate_planar_velocity"], "captured masses and candidate planar velocity")
    e0 = [[sum(Q(a)*Q(b) for a, b in zip(native["r"][i][d], native["start_z"]))-Q(old["velocity"][i][d])
           for d in range(2)] for i in range(16)]
    e1 = [[sum(Q(a)*Q(b) for a, b in zip(native["r"][i][d], native["end_z"]))-Q(native["end_velocity"][i][d])
           for d in range(2)] for i in range(16)]
    projected = [sum(Q(native["r"][i][d][v])*Q(native["end_mass"][i])*(e1[i][d]-e0[i][d])
                     for i in range(16) for d in range(2))/Q(input_row["h"]) for v in range(22)]
    squared = sum(x*x for x in projected)
    norm = math.sqrt(float(squared))
    chart_gap = max(abs(a-b) for a, b in zip(native["start_z"], native["rebuilt_start_z"]))
    require(chart_gap == 0 and float(max(abs(x) for row in e0 for x in row)) == 5.551115123125783e-17
            and float(max(abs(x) for row in e1 for x in row)) == 5.551115123125783e-17,
            "zero chart gap does not imply zero exact nodal embedding defects")
    require(norm == 7.359808400080196e-17, "reviewed projected rate norm")
    native_squared = sum(Q(x)*Q(x) for x in native["rate"])
    require(native_squared > Q(1e-13)**2, "exact captured squared norm remains refused")
    for name in ("examples/forcing_temporal_probe.rs", "src/coupled_discrete.rs", "src/translated_viscous.rs", "src/fitted_height.rs",
                 "evidence/forcing-residual-arithmetic/native-probe-start-chart.rs",
                 "evidence/forcing-residual-arithmetic/native-probe-start-chart-stderr.log",
                 "evidence/forcing-residual-arithmetic/native_probe.rs", "evidence/forcing-residual-arithmetic/native_inputs.rs"):
        source(name)
    return dict(frozen_source=FROZEN_SOURCE, original_diagnostic_source=inputs["frozen_head"], input_row_index=3,
                selector=dict(kind="pressure_state", field="nonconstant", load="forward", h=.00078125,
                              accepted_version=1, stamp_id=131, accepted_time=.00078125, attempted_step=2),
                source_provenance=provenance, initial_publication=chosen[0], accepted_publication=old,
                input_row=input_row, first_second_step_check=first_check, terminal_validation=terminal,
                refusal=refusal, post_window_observations=observed, fixed_native_candidate=native,
                exact_embedding_diagnostic=dict(chart_reconstruction_gap=chart_gap, e0=[[qstr(x) for x in row] for row in e0],
                    e1=[[qstr(x) for x in row] for row in e1], projected_rate=list(map(qstr, projected)),
                    projected_rate_squared_norm=qstr(squared), projected_rate_norm=norm,
                    projected_rate_max_component=float(max(map(abs, projected))),
                    native_squared_rate_norm=qstr(native_squared), units="reduced momentum rate for projection; velocity for e0/e1",
                    scope="exact arithmetic on captured binary fields; no causal explanation or new chart/geometry solve"),
                owner_advances=0, acceptance_promotions=0, new_evaluator_executed=False)


def corruption_checks(packet, expected):
    def validate(p):
        require(p == expected, "complete selected source field binding")
    validate(packet)
    mutations = [
        ("wrong accepted version", lambda p: p["accepted_publication"]["stamp"].update(version=0)),
        ("wrong load", lambda p: p["selector"].update(load="reversed")),
        ("wrong interval", lambda p: p["selector"].update(h=.0015625)),
        ("wrong accepted time", lambda p: p["accepted_publication"].update(time=.0015625)),
        ("candidate pressure substituted for accepted pressure", lambda p: p["accepted_publication"].update(pressure_coefficients=p["input_row"]["unknown"][6:])),
        ("nonconstant third input erased", lambda p: p["input_row"]["old"][0].__setitem__(2, 0.0)),
        ("false candidate publication", lambda p: p["fixed_native_candidate"].update(published=True)),
        ("zero chart gap misreported as zero nodal defect", lambda p: p["exact_embedding_diagnostic"].update(projected_rate_norm=0.0)),
    ]
    rejected = []
    for name, mutation in mutations:
        altered = copy.deepcopy(packet)
        mutation(altered)
        try:
            validate(altered)
        except ValueError:
            rejected.append(name)
        else:
            raise ValueError(f"missed capture corruption: {name}")
    return rejected


if __name__ == "__main__":
    print(json.dumps(select(), indent=2, sort_keys=True))
