"""Verify all files, including nested/master receipts, against frozen Git."""
import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[2]
receipt = json.loads(Path(__file__).with_name("receipt.json").read_text())


def require(ok, message):
    if not ok:
        raise ValueError(message)


def git(*args):
    return subprocess.check_output(["git", *args], cwd=root)


for packet in receipt["packets"]:
    commit = packet["evidence_commit"]
    prefix = packet["prefix"]
    require(git("rev-parse", commit + "^{tree}").decode().strip() == packet["evidence_tree"], "evidence tree")
    tracked = set(git("ls-tree", "-r", "--name-only", commit, "--", prefix).decode().splitlines())
    inventory = {row["path"] for row in packet["files"]}
    require(tracked == inventory, "frozen packet path inventory")
    live = {str(p.relative_to(root)) for p in (root / prefix).rglob("*")
            if p.is_file() and "__pycache__" not in p.parts}
    require(live == inventory, "live packet path inventory")
    require(prefix + "/receipt.json" in inventory, "master receipt binding")
    require(prefix + "/legacy-replay/receipt.json" in inventory, "nested replay receipt binding")
    for row in packet["files"]:
        frozen = git("show", commit + ":" + row["path"])
        require(frozen == (root / row["path"]).read_bytes(), "frozen bytes: " + row["path"])
        require(hashlib.sha256(frozen).hexdigest() == row["sha256"], "file SHA-256")
        require(hashlib.sha1(b"blob " + str(len(frozen)).encode() + b"\0" + frozen).hexdigest() == row["git_blob"], "Git blob")
    master = json.loads(git("show", commit + ":" + prefix + "/receipt.json"))
    source = packet["source_commit"]
    require(master["qualified_source_commit"] == source, "qualified source")
    require(master["qualified_source_tree"] == packet["source_tree"], "qualified source tree")
    require(git("rev-parse", source + "^{tree}").decode().strip() == packet["source_tree"], "source Git tree")
    require(git("show", "-s", "--format=%P", source).decode().split() == master["ordered_source_parents"], "ordered source parents")
    print("PASS", prefix, len(inventory), "files including all master and nested receipts")
print("PASS complete frozen packet/source bindings; no historical bytes modified")
