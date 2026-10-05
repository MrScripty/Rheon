"""Bind executed coupled evidence to exact source and untouched frozen blobs."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
from verify_numerics import require, verify

root = Path(__file__).resolve().parents[2]
packet = Path(__file__).resolve().parent
receipt = json.loads((packet / "receipt.json").read_text())


def git(*args):
    return subprocess.check_output(["git", *args], cwd=root)


source = receipt["qualified_source_commit"]
require(git("rev-parse", source + "^{tree}").decode().strip() == receipt["qualified_source_tree"], "source tree")
require(git("show", "-s", "--format=%P", source).decode().split() == receipt["ordered_source_parents"], "source parents")
for path, digest in receipt["source_sha256"].items():
    require(hashlib.sha256(git("show", source + ":" + path)).hexdigest() == digest, "source digest: " + path)
files = {str(p.relative_to(root)) for p in packet.rglob("*")
         if p.is_file() and p.name != "receipt.json" and "__pycache__" not in p.parts}
require(files == set(receipt["file_sha256"]), "complete evidence inventory")
for path, digest in receipt["file_sha256"].items():
    require(hashlib.sha256((root / path).read_bytes()).hexdigest() == digest, "evidence digest: " + path)
for name, count in receipt["tests"].items():
    text = (packet / (name + ".log")).read_text()
    require(sum(map(int, re.findall(r"test result: ok\. (\d+) passed;", text))) == count, "native test count")
    require("test result: FAILED" not in text, "native test failure")
for filename in ["commands-final.json", "release-commands.json", "supporting-commands.json"]:
    require(all(row["exit"] == 0 for row in json.loads((packet / filename).read_text())), "failed command")
require(verify(packet / "demo") == json.loads((packet / "numerical-summary.json").read_text()), "independent numerical summary")
require(all(row["rejected"] for row in json.loads((packet / "negative-numerics.json").read_text())), "negative fixtures")
replay = json.loads((packet / "replay-receipt.json").read_text())
require(replay["byte_identical_files"] == 65 == len(replay["file_sha256"]), "replay file count")
for path, digest in replay["file_sha256"].items():
    require(hashlib.sha256((packet / "demo" / path).read_bytes()).hexdigest() == digest, "replay digest")
preservation = json.loads((packet / "preservation.json").read_text())
require(preservation["base_commit"] == receipt["ordered_source_parents"][0], "preservation base")
require(git("rev-parse", preservation["base_commit"] + "^{tree}").decode().strip() == preservation["base_tree"], "base tree")
require(preservation["count"] == len(preservation["files"]) == 1348, "frozen inventory count")
for item in preservation["files"]:
    data = (root / item["path"]).read_bytes()
    require(hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest() == item["git_blob"], "frozen blob")
    require(hashlib.sha256(data).hexdigest() == item["sha256"], "frozen digest")
for path in ["src/pressure.rs", "src/operator.rs", "src/advection.rs", "src/sampling.rs", "tests/liquid_volume_contract.rs"]:
    require(git("show", source + ":" + path) == git("show", preservation["base_commit"] + ":" + path), "original implementation/fixtures: " + path)
require("Ran 20 tests" in (packet / "python-harness.log").read_text()
        and (packet / "python-harness.log").read_text().rstrip().endswith("OK"), "existing harness qualification")
print("PASS exact coupled source/tree/parent, native results, independent numerics, replay hashes and 1348 frozen blobs")
