"""Bind exact qualified source, all packet files and preserved historical bytes.

Only this root receipt is excluded from its own hash map. Nested receipts are
included. Pass --evidence-commit to bind the root receipt and complete inventory
to a frozen Git commit as well. No assert-dependent requirements.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
PREFIX = "evidence/column-interface"
MASTER = PREFIX + "/receipt.json"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def verify(receipt, evidence_commit=None):
    source = receipt["qualified_source_commit"]
    require(git("rev-parse", source + "^{tree}").decode().strip() == receipt["qualified_source_tree"], "source tree")
    require(git("show", "-s", "--format=%P", source).decode().split() == receipt["ordered_source_parents"], "source parents")
    for path, expected in receipt["source_sha256"].items():
        data = git("show", source + ":" + path)
        require(sha(data) == expected and data == (ROOT / path).read_bytes(), "qualified source bytes: " + path)
    live = {str(p.relative_to(ROOT)) for p in (ROOT / PREFIX).rglob("*")
            if p.is_file() and "__pycache__" not in p.parts and p != ROOT / MASTER}
    require(live == set(receipt["file_sha256"]), "complete packet inventory")
    for path, expected in receipt["file_sha256"].items():
        require(sha((ROOT / path).read_bytes()) == expected, "packet SHA-256: " + path)
    preservation = json.loads((ROOT / PREFIX / "preservation.json").read_text())
    base = preservation["base_commit"]
    require(git("rev-parse", base + "^{tree}").decode().strip() == preservation["base_tree"], "preservation base tree")
    tracked = set(git("ls-tree", "-r", "--name-only", base, "--", "evidence", "proofs", "docs/research-book", "docs/education").decode().splitlines())
    require(tracked == {row["path"] for row in preservation["files"]}, "complete historical inventory")
    for row in preservation["files"]:
        data = (ROOT / row["path"]).read_bytes()
        require(sha(data) == row["sha256"], "historical bytes: " + row["path"])
        blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
        require(blob == row["git_blob"], "historical Git blob")
    if evidence_commit:
        tracked = set(git("ls-tree", "-r", "--name-only", evidence_commit, "--", PREFIX).decode().splitlines())
        require(tracked == live | {MASTER}, "frozen evidence inventory")
        for path in tracked:
            require(git("show", evidence_commit + ":" + path) == (ROOT / path).read_bytes(), "frozen evidence bytes: " + path)
        require(json.loads(git("show", evidence_commit + ":" + MASTER)) == receipt, "frozen root receipt")
    return len(live)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-commit")
    parser.add_argument("--negative-self-test", action="store_true")
    args = parser.parse_args()
    receipt = json.loads((ROOT / MASTER).read_text())
    count = verify(receipt, args.evidence_commit)
    if args.negative_self_test:
        for variant in ("nested_inventory", "nested_hash", "source_tree"):
            changed = copy.deepcopy(receipt)
            nested = PREFIX + "/legacy-replay/receipt.json"
            if variant == "nested_inventory":
                del changed["file_sha256"][nested]
            elif variant == "nested_hash":
                changed["file_sha256"][nested] = "0" * 64
            else:
                changed["qualified_source_tree"] = "0" * 40
            try:
                verify(changed, args.evidence_commit)
            except ValueError:
                print("REJECTED", variant)
            else:
                raise ValueError("negative receipt accepted: " + variant)
    print("PASS", count, "packet files including nested receipts; qualified source and complete historical bytes")
