"""Verify the additive packet, preserved base and discriminating bindings."""
from pathlib import Path
import copy
import hashlib
import importlib.util
import json
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BASE = "46a3d2986bc013ee1787f6af1d82b2d8c6ae50da"
TREE = "5aa0c3fc85aef138a9ce39fe0c2dddd0c17cdfc6"
DOC = "docs/research-book/implementation/forcing-increment-state-contract-supplement.md"
NAMES = ("README.md", "capture.py", "examples.py", "verify.py", "comparison-policy.json", "selected-capture.json",
         "capture-stderr.log", "results-normal.json", "results-optimized.json", "examples-normal-stderr.log", "examples-optimized-stderr.log")


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def preserved():
    require(subprocess.check_output(["git", "rev-parse", f"{BASE}^{{tree}}"], cwd=ROOT).decode().strip() == TREE, "prior frozen tree")
    entries = subprocess.check_output(["git", "ls-tree", "-rz", BASE], cwd=ROOT).split(b"\0")
    digest, count = hashlib.sha256(), 0
    for entry in entries:
        if not entry:
            continue
        metadata, name = entry.split(b"\t", 1)
        mode, kind, object_id = metadata.split()
        require(mode in (b"100644", b"100755") and kind == b"blob", "supported frozen files")
        raw = (ROOT/name.decode()).read_bytes()
        require(hashlib.sha1(b"blob "+str(len(raw)).encode()+b"\0"+raw).hexdigest().encode() == object_id,
                f"prior frozen file preserved: {name.decode()}")
        digest.update(name+b"\0"+hashlib.sha256(raw).digest())
        count += 1
    require(count == 6959, "complete prior inventory")
    return dict(files=count, path_content_sha256=digest.hexdigest())


def module(name):
    spec = importlib.util.spec_from_file_location(f"supplement_{name}", HERE/f"{name}.py")
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


def policy_checks(p):
    def validate(row):
        require(row["status"] == "RESEARCH_SPECIFICATION_ONLY_NO_E1_EXECUTION" and row["baseline_source"] == "f0b81b4a5f76cb706e645fad4f38faf028c5727e", "frozen baseline policy")
        require(row["selector"] == dict(kind="pressure_state", field="nonconstant", load="forward", h=.00078125,
                accepted_version=1, accepted_time=.00078125, stamp_id=131, native_norm=1.1895444217920825e-13), "pinned comparison selector")
        e = row["E1"]
        require(e["geometry_policy"] == "native qB/cap/geometry/mass/D/B assembly, no latent geometry", "native geometry retained")
        require(e["chart_scalar_precision_bits"] == 106 and e["iterative_refinement_steps"] == 0, "fixed solve precision and budget")
        require(e["additional_fixed_workspace_bytes"] == 65536 and e["scalar_slot_bytes"] == 32
                and e["simultaneous_scalar_slots"] == dict(five_15x15_matrices=1125, twelve_22_vectors=264, one_24x22_D=528, coordinates=9, scalar_temporaries=8), "fixed simultaneous reservation")
        require(sum(e["simultaneous_scalar_slots"].values())*32 == e["scalar_storage_bytes"] == 61888
                and e["scalar_storage_bytes"]+e["fixed_descriptor_alignment_bytes"] == 65536, "reservation arithmetic")
        require(e["implemented"] is e["executed"] is e["behavior_neutral"] is e["retained_state_or_snapshots"] is False, "no E1 execution/hidden state/neutrality claim")
        require(row["production_changes"] == row["owner_advances"] == row["acceptance_promotions"] == 0, "research only")
        require(e["max_corrections"] == 7 and e["max_equation_calls"] == 200, "native controller budgets retained")
    validate(p)
    mutations = [
        ("alternate geometry silently substituted", lambda r: r["E1"].update(geometry_policy="latent q")),
        ("unfrozen scalar precision", lambda r: r["E1"].update(chart_scalar_precision_bits=128)),
        ("extra refinement budget", lambda r: r["E1"].update(iterative_refinement_steps=1)),
        ("unaccounted simultaneous workspace", lambda r: r["E1"].update(additional_fixed_workspace_bytes=61888)),
        ("unimplemented comparison claimed executed", lambda r: r["E1"].update(executed=True)),
        ("failed trajectory promoted", lambda r: r.update(acceptance_promotions=1)),
    ]
    rejected = []
    for name, mutation in mutations:
        altered = copy.deepcopy(p)
        mutation(altered)
        try:
            validate(altered)
        except ValueError:
            rejected.append(name)
        else:
            raise ValueError(f"missed policy corruption: {name}")
    return rejected


def numerical_checks():
    normal = (HERE/"results-normal.json").read_bytes()
    require(normal == (HERE/"results-optimized.json").read_bytes(), "normal/optimized example parity")
    example = module("examples").run()
    require(example == json.loads(normal) and len(example["cases"]) == 2 and len(example["rejected_corruptions"]) == 8,
            "actual additional examples and controls")
    capture = module("capture")
    expected = capture.select()
    packet = json.loads((HERE/"selected-capture.json").read_text())
    rejected_capture = capture.corruption_checks(packet, expected)
    rejected_policy = policy_checks(json.loads((HERE/"comparison-policy.json").read_text()))
    return dict(analytical_examples=2, analytical_corruptions=8, capture_corruptions=len(rejected_capture),
                policy_corruptions=len(rejected_policy), rejected_capture_corruptions=rejected_capture,
                rejected_policy_corruptions=rejected_policy)


def verify(receipt):
    require(receipt["base"] == BASE and receipt["base_tree"] == TREE, "receipt prior identity")
    require(receipt["frozen_inventory"] == preserved(), "prior full inventory binding")
    paths = {DOC} | {str((HERE/name).relative_to(ROOT)) for name in NAMES}
    require(set(receipt["artifacts"]) == paths and len(paths) == 12, "closed additive packet binding")
    for name, value in receipt["artifacts"].items():
        require(sha(ROOT/name) == value, f"supplement artifact binding: {name}")
    checks = numerical_checks()
    require(checks == receipt["checks"], "actual recorded controls")
    require(receipt["production_changes"] == receipt["owner_advances"] == receipt["acceptance_promotions"] == 0
            and receipt["new_evaluator_executed"] is False, "no native execution or promotion")
    return dict(status="PASS_ADDITIVE_RESEARCH_SPECIFICATION_ONLY", preserved_files=6959, bound_artifacts=12,
                **checks, production_changes=0, owner_advances=0, new_evaluator_executed=False)


if __name__ == "__main__":
    if "--freeze" in sys.argv:
        receipt = dict(base=BASE, base_tree=TREE, frozen_inventory=preserved(), checks=numerical_checks(),
                       production_changes=0, owner_advances=0, acceptance_promotions=0, new_evaluator_executed=False)
        paths = [ROOT/DOC]+[HERE/name for name in NAMES]
        receipt["artifacts"] = {str(p.relative_to(ROOT)): sha(p) for p in paths}
        (HERE/"receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True)+"\n")
        print(json.dumps(dict(status="FROZEN_ADDITIVE_RESEARCH_SPECIFICATION", receipt_sha256=sha(HERE/"receipt.json")), sort_keys=True))
    else:
        print(json.dumps(verify(json.loads((HERE/"receipt.json").read_text())), indent=2, sort_keys=True))
