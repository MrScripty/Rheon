"""Bind the proposal/examples and preserved frozen base without self-hashing logs."""
from pathlib import Path
import hashlib
import importlib.util
import json
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BASE = "f0b81b4a5f76cb706e645fad4f38faf028c5727e"
TREE = "59927a7fb02d949859b78f7e78865d1717f19870"
DOCUMENT = "docs/research-book/implementation/forcing-increment-state-contract.md"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def preserved():
    tree = subprocess.check_output(["git", "rev-parse", f"{BASE}^{{tree}}"], cwd=ROOT).decode().strip()
    require(tree == TREE, "frozen base tree")
    entries = subprocess.check_output(["git", "ls-tree", "-rz", BASE], cwd=ROOT).split(b"\0")
    digest = hashlib.sha256()
    count = 0
    for entry in entries:
        if not entry:
            continue
        metadata, raw_path = entry.split(b"\t", 1)
        mode, kind, object_id = metadata.split()
        require(kind == b"blob" and mode in (b"100644", b"100755"), "supported frozen file")
        path = raw_path.decode()
        content = (ROOT/path).read_bytes()
        actual = hashlib.sha1(b"blob "+str(len(content)).encode()+b"\0"+content).hexdigest().encode()
        require(actual == object_id, f"preserved frozen file: {path}")
        digest.update(raw_path+b"\0"+hashlib.sha256(content).digest())
        count += 1
    require(count == 6940, "complete frozen base inventory")
    return dict(files=count, path_content_sha256=digest.hexdigest())


def examples():
    normal = (HERE/"results-normal.json").read_bytes()
    require(normal == (HERE/"results-optimized.json").read_bytes(), "ordinary/optimized example parity")
    spec = importlib.util.spec_from_file_location("increment_contract_examples", HERE/"examples.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    result = module.run()
    require(result == json.loads(normal), "executed exact examples match record")
    require(len(result["cases"]) == 8 and len(result["rejected_corruptions"]) == 11, "example/control count")
    return result


def validate(receipt):
    require(receipt["base"] == BASE and receipt["base_tree"] == TREE, "receipt base identity")
    require(receipt["production_changes"] == receipt["owner_advances"] == receipt["acceptance_promotions"] == 0,
            "research only, no publication or promotion")
    require(receipt["original_native_refusals"] == 5 and receipt["original_temporal_band"] == "FAIL_ORIGINAL_TEMPORAL_BAND",
            "native and temporal failures retained")
    require(receipt["compensation_lifetime"] == "ephemeral workspace proposal only", "no retained hidden state")
    require(receipt["behavior_neutral"] is False and receipt["native_compensation_implemented"] is False,
            "no behavior neutrality/implementation claim")
    require(receipt["frozen_inventory"] == preserved(), "complete base binding")
    expected = {DOCUMENT} | {str(p.relative_to(ROOT)) for p in HERE.iterdir()
                            if p.is_file() and p.name in ("README.md", "examples.py", "verify.py", "results-normal.json", "results-optimized.json",
                                                        "examples-normal-stderr.log", "examples-optimized-stderr.log")}
    require(set(receipt["artifacts"]) == expected and len(expected) == 8, "closed proposal artifact set")
    for name, value in receipt["artifacts"].items():
        require(sha(ROOT/name) == value, f"proposal artifact binding: {name}")
    result = examples()
    return dict(status="PASS_RESEARCH_CONTRACT_ONLY", frozen_files=receipt["frozen_inventory"]["files"],
                artifacts=len(expected), analytical_examples=len(result["cases"]),
                rejected_analytical_corruptions=len(result["rejected_corruptions"]),
                production_changes=0, owner_advances=0,
                native_compensation_implemented=False, behavior_neutral=False)


if __name__ == "__main__":
    if "--freeze" in sys.argv:
        receipt = dict(base=BASE, base_tree=TREE, frozen_inventory=preserved(),
                       production_changes=0, owner_advances=0, acceptance_promotions=0,
                       original_native_refusals=5, original_temporal_band="FAIL_ORIGINAL_TEMPORAL_BAND",
                       compensation_lifetime="ephemeral workspace proposal only", behavior_neutral=False,
                       native_compensation_implemented=False)
        paths = [ROOT/DOCUMENT] + [HERE/name for name in ("README.md", "examples.py", "verify.py", "results-normal.json",
                    "results-optimized.json", "examples-normal-stderr.log", "examples-optimized-stderr.log")]
        receipt["artifacts"] = {str(p.relative_to(ROOT)): sha(p) for p in paths}
        examples()
        (HERE/"receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True)+"\n")
        print(json.dumps(dict(status="FROZEN_RESEARCH_CONTRACT_ONLY", receipt_sha256=sha(HERE/"receipt.json")), sort_keys=True))
    else:
        print(json.dumps(validate(json.loads((HERE/"receipt.json").read_text())), indent=2, sort_keys=True))
