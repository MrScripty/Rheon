"""Check frozen source identity, recorded native passes and evidence hashes."""
import hashlib
import json
from pathlib import Path
import re
import subprocess

root = Path(__file__).resolve().parents[2]
packet = Path(__file__).resolve().parent
receipt = json.loads((packet / "receipt.json").read_text())

def git(*args):
    return subprocess.check_output(["git", *args], cwd=root)

source = receipt["qualified_source_commit"]
assert git("rev-parse", source + "^{tree}").decode().strip() == receipt["qualified_source_tree"]
assert git("show", "-s", "--format=%P", source).decode().split() == receipt["ordered_source_parents"]
for path, digest in receipt["source_sha256"].items():
    assert hashlib.sha256(git("show", source + ":" + path)).hexdigest() == digest, path
files = {str(p.relative_to(root)) for p in packet.rglob("*")
         if p.is_file() and p.name != "receipt.json" and "__pycache__" not in p.parts}
assert files == set(receipt["file_sha256"])
for path, digest in receipt["file_sha256"].items():
    assert hashlib.sha256((root / path).read_bytes()).hexdigest() == digest, path
for name, count in receipt["tests"].items():
    text = (packet / (name + ".log")).read_text()
    assert sum(map(int, re.findall(r"test result: ok\. (\d+) passed;", text))) == count
    assert "test result: FAILED" not in text
assert all(row["exit"] == 0 for row in json.loads((packet / "commands-green.json").read_text()))
assert all(row["exit"] == 0 for row in json.loads((packet / "supporting-commands.json").read_text()))
for base in json.loads((packet / "preservation.json").read_text()):
    assert git("rev-parse", base["base"] + "^{tree}").decode().strip() == base["tree"]
    for item in base["preserved"]:
        data = (root / item["path"]).read_bytes()
        assert hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest() == item["git_blob"]
        assert hashlib.sha256(data).hexdigest() == item["sha256"]
for path in ["src/simulation.rs", "src/pressure.rs", "src/operator.rs"]:
    assert git("show", source + ":" + path) == git("show", receipt["ordered_source_parents"][0] + ":" + path)
print("PASS exact composition source/tree/parents, native result counts, frozen blobs and packet hashes")
