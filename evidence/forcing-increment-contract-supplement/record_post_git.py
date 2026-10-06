"""Freeze actual post-Git checks without rewriting the supplement receipt."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BASE = "46a3d2986bc013ee1787f6af1d82b2d8c6ae50da"
EVIDENCE = "528885e7bc2070b398558b2ee4fbf27e77e10978"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def sha(data):
    return hashlib.sha256(data).hexdigest()


changes = git("diff", "--name-status", BASE, EVIDENCE).decode().splitlines()
require(len(changes) == 17, "closed additive 17-file supplement")
blobs = {}
for line in changes:
    status, path = line.split("\t")
    require(status == "A" and (path == "docs/research-book/implementation/forcing-increment-state-contract-supplement.md"
            or path.startswith("evidence/forcing-increment-contract-supplement/")), "additive reviewed paths only")
    data = git("show", f"{EVIDENCE}:{path}")
    require(data == (ROOT/path).read_bytes(), f"committed supplement blob: {path}")
    blobs[path] = sha(data)

commands = []
for label, extra in (("normal", []), ("optimized", ["-O"])):
    argv = [sys.executable, *extra, "evidence/forcing-increment-contract-supplement/verify.py"]
    start = time.monotonic()
    result = subprocess.run(argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    seconds = time.monotonic()-start
    stdout, stderr = f"post-git-{label}.log", f"post-git-{label}-stderr.log"
    (HERE/stdout).write_bytes(result.stdout)
    (HERE/stderr).write_bytes(result.stderr)
    commands.append(dict(argv=argv, exit=result.returncode, seconds=seconds, stdout=stdout, stderr=stderr,
                         stdout_sha256=sha(result.stdout), stderr_sha256=sha(result.stderr)))
    require(result.returncode == 0, f"actual post-Git {label} supplement verification")
require(commands[0]["stdout_sha256"] == commands[1]["stdout_sha256"], "post-Git verifier mode parity")
record = dict(status="PASS_ADDITIVE_RESEARCH_SPECIFICATION_ONLY", base=BASE, evidence=EVIDENCE,
              evidence_tree=git("rev-parse", f"{EVIDENCE}^{{tree}}").decode().strip(), committed_supplement_blobs=blobs,
              commands=commands, recorder_sha256=sha(Path(__file__).read_bytes()),
              receipt_sha256=sha((HERE/"receipt.json").read_bytes()), preserved_prior_files=6959,
              production_changes=0, owner_advances=0, new_evaluator_executed=False)
(HERE/"post-git-verification.json").write_text(json.dumps(record, indent=2, sort_keys=True)+"\n")
print(json.dumps(dict(status=record["status"], evidence=EVIDENCE, evidence_tree=record["evidence_tree"],
                      committed_supplement_blobs=len(blobs), receipt_sha256=record["receipt_sha256"]), sort_keys=True))
