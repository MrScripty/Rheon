"""Verify source binding, complete packet hashes and preserved base Git blobs."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

from verify_numerics import verify


root = Path(__file__).resolve().parents[2]
packet = root / "evidence/liquid-volume"
receipt_path = packet / "receipt.json"
receipt = json.loads(receipt_path.read_text())


def git(*args):
    return subprocess.check_output(["git", *args], cwd=root, text=True).strip()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


source = receipt["qualified_source_commit"]
assert git("rev-parse", source + "^{tree}") == receipt["qualified_source_tree"]
assert git("show", "-s", "--format=%P", source).split() == receipt["ordered_source_parents"]
expected = {
    str(path.relative_to(root)) for path in packet.rglob("*")
    if path.is_file() and path != receipt_path and "__pycache__" not in path.parts
} | set(receipt["changed_source_paths"])
assert set(receipt["file_sha256"]) == expected
for path, digest in receipt["file_sha256"].items():
    assert sha(root / path) == digest, path
for path in receipt["changed_source_paths"]:
    committed = subprocess.check_output(["git", "show", source + ":" + path], cwd=root)
    assert committed == (root / path).read_bytes(), path
inventory = json.loads((packet / "preserved-base-inventory.json").read_text())
assert inventory["base_commit"] == receipt["base_commit"]
assert len(inventory["files"]) == inventory["preserved_count"] == 1482
for item in inventory["files"]:
    data = (root / item["path"]).read_bytes()
    blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
    assert blob == item["git_blob"], item["path"]
    assert hashlib.sha256(data).hexdigest() == item["sha256"], item["path"]
for feature, expected_count in receipt["acceptance"]["tests"].items():
    log = (packet / ("test-" + feature + ".log")).read_text()
    assert sum(map(int, re.findall(r"test result: ok\. (\d+) passed;", log))) == expected_count
    assert not re.search(r"test result: FAILED", log)
assert verify(packet) == json.loads((packet / "numerical-summary.json").read_text())
red = json.loads((packet / "red-signed-zero-reconstruction.json").read_text())
source_text = subprocess.check_output(
    ["git", "show", red["accepted_source_commit"] + ":src/liquid_volume.rs"],
    cwd=root, text=True,
)
assert source_text.count(red["accepted_block"]) == 1
pre_fix = source_text.replace(red["accepted_block"], red["pre_fix_block"])
assert hashlib.sha256(pre_fix.encode()).hexdigest() == red["reconstructed_pre_fix_source_sha256"]
assert red["reconstructed_pre_fix_source_sha256"] == json.loads(
    (packet / "red-signed-zero-receipt.json").read_text()
)["pre_fix_source_sha256"]
area = json.loads((packet / "red-face-area-reconstruction.json").read_text())
source_text = (root / "src/liquid_volume.rs").read_text()
assert source_text.count(area["accepted_block"]) == 1
pre_fix = source_text.replace(area["accepted_block"], area["pre_fix_block"])
assert hashlib.sha256(pre_fix.encode()).hexdigest() == area["reconstructed_pre_fix_source_sha256"]
assert area["reconstructed_pre_fix_source_sha256"] == json.loads(
    (packet / "red-face-area-receipt.json").read_text()
)["pre_fix_source_sha256"]
print("PASS complete packet hashes, qualified source/tree/parent, 1482 base blobs,")
print("feature test counts, independent numerics and exact pre-fix reconstruction")
