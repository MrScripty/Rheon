"""Bind completed comparison evidence to Git without rerunning native E1."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BASE = "78b7365513ccf317e554d36f968da36b55f04065"
EVIDENCE = "4326926f826bcda9ff7b483605a86a11becb6c98"
TREE = "bb895e632bda8c61b06c7b5e490183d8e7482537"
RECEIPT = "e32b08711a94747b93c706ae04cf5d7eb11fa94b75aa81a86b1369c327ed75c0"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def sha(data):
    return hashlib.sha256(data).hexdigest()


require(git("rev-parse", f"{EVIDENCE}^{{tree}}").decode().strip() == TREE,
        "frozen completed evidence tree")
require(sha((HERE/"receipt.json").read_bytes()) == RECEIPT, "frozen comparison receipt")
changes = git("diff", "--name-status", BASE, EVIDENCE).decode().splitlines()
require(len(changes) == 66, "closed additive 66-file completed evidence")
blobs = {}
for line in changes:
    status, path = line.split("\t")
    require(status == "A" and path.startswith("evidence/forcing-increment-comparison/"),
            "additive comparison evidence only")
    data = git("show", f"{EVIDENCE}:{path}")
    require(data == (ROOT/path).read_bytes(), f"committed completed evidence blob: {path}")
    blobs[path] = sha(data)

commands = []
outputs = []
for label, extra in (("normal", []), ("optimized", ["-O"])):
    argv = [sys.executable, *extra, "evidence/forcing-increment-comparison/verify.py"]
    start = time.monotonic()
    result = subprocess.run(argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    seconds = time.monotonic()-start
    stdout, stderr = f"post-git-{label}.log", f"post-git-{label}-stderr.log"
    (HERE/stdout).write_bytes(result.stdout)
    (HERE/stderr).write_bytes(result.stderr)
    commands.append(dict(argv=argv, exit=result.returncode, seconds=seconds, stdout=stdout, stderr=stderr,
                         stdout_sha256=sha(result.stdout), stderr_sha256=sha(result.stderr)))
    require(result.returncode == 0, f"actual post-Git {label} comparison verification")
    outputs.append(json.loads(result.stdout))
require(commands[0]["stdout_sha256"] == commands[1]["stdout_sha256"], "post-Git verifier mode parity")
require(outputs[0]["preserved_files"] == 6996 and outputs[0]["artifacts"] == 75,
        "complete preserved and result inventories")
record = dict(status="PASS_BOUNDED_EXPERIMENT_BINDING_ONLY", base=BASE, evidence=EVIDENCE,
              evidence_tree=TREE, committed_completed_blobs=blobs, commands=commands,
              recorder_sha256=sha(Path(__file__).read_bytes()), receipt_sha256=RECEIPT,
              checks=outputs[0], production_changes=0, owner_advances=0, searches=0,
              numerical_execution_repeated=False, production_adopted=False)
(HERE/"post-git-verification.json").write_text(json.dumps(record, indent=2, sort_keys=True)+"\n")
print(json.dumps(dict(status=record["status"], evidence=EVIDENCE, evidence_tree=TREE,
                      committed_completed_blobs=len(blobs), receipt_sha256=RECEIPT), sort_keys=True))
