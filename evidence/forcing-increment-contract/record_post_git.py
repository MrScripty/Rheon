"""Record exact committed proposal blobs plus ordinary/optimized verification."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BASE = "f0b81b4a5f76cb706e645fad4f38faf028c5727e"
EVIDENCE = "fd73aece1dc2a8c0308f9f6f80935276f86a08b5"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def sha(data):
    return hashlib.sha256(data).hexdigest()


tree = git("rev-parse", f"{EVIDENCE}^{{tree}}").decode().strip()
changes = git("diff", "--name-status", BASE, EVIDENCE).decode().splitlines()
require(len(changes) == 13, "closed 13-file proposal commit")
blobs = {}
for line in changes:
    status, path = line.split("\t")
    require(status == "A" and (path == "docs/research-book/implementation/forcing-increment-state-contract.md"
                              or path.startswith("evidence/forcing-increment-contract/")), "additive research paths only")
    data = git("show", f"{EVIDENCE}:{path}")
    require(data == (ROOT/path).read_bytes(), f"committed proposal blob: {path}")
    blobs[path] = sha(data)

commands = []
for label, extra in (("normal", []), ("optimized", ["-O"])):
    argv = [sys.executable, *extra, "evidence/forcing-increment-contract/verify.py"]
    start = time.monotonic()
    result = subprocess.run(argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    seconds = time.monotonic()-start
    stdout, stderr = f"post-git-{label}.log", f"post-git-{label}-stderr.log"
    (HERE/stdout).write_bytes(result.stdout)
    (HERE/stderr).write_bytes(result.stderr)
    commands.append(dict(argv=argv, exit=result.returncode, seconds=seconds, stdout=stdout, stderr=stderr,
                         stdout_sha256=sha(result.stdout), stderr_sha256=sha(result.stderr)))
    require(result.returncode == 0, f"actual post-Git {label} verification")
require(commands[0]["stdout_sha256"] == commands[1]["stdout_sha256"], "post-Git verifier parity")
record = dict(status="PASS_RESEARCH_CONTRACT_ONLY", evidence=EVIDENCE, evidence_tree=tree,
              base=BASE, committed_proposal_blobs=blobs, commands=commands,
              receipt_sha256=sha((HERE/"receipt.json").read_bytes()),
              recorder_sha256=sha(Path(__file__).read_bytes()), production_changes=0, owner_advances=0,
              no_native_compensated_implementation=True, original_native_refusals_retained=5)
(HERE/"post-git-verification.json").write_text(json.dumps(record, indent=2, sort_keys=True)+"\n")
print(json.dumps(dict(status=record["status"], evidence=EVIDENCE, evidence_tree=tree,
                      committed_proposal_blobs=len(blobs), receipt_sha256=record["receipt_sha256"]), sort_keys=True))
