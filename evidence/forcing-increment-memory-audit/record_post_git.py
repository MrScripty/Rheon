"""Bind static audit to committed evidence and the inspected binary bytes."""
from pathlib import Path
import ctypes
import gzip
import hashlib
import json
import re
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BASE = "acec66b0d50556ea6a482ec796fb0f5d36810aca"
EVIDENCE = "4f77c987061e7cfe9b33825a0f91fb0b679997a2"
TREE = "45c18656d47463e7ba5c87be6de104ed0c7378c0"
RECEIPT = "6b35e75619d8778222e4d55b6353b75fb4c78ebac9a17ff7083a4e314c4c65ac"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


require(git("rev-parse", EVIDENCE+"^{tree}").decode().strip() == TREE, "committed static-audit tree")
require(sha((HERE/"receipt.json").read_bytes()) == RECEIPT, "frozen static-audit receipt")
blobs = {}
changes = git("diff", "--name-status", BASE, EVIDENCE).decode().splitlines()
require(len(changes) == 23, "closed additive 23-file static audit")
for line in changes:
    status, path = line.split("\t")
    require(status == "A" and path.startswith("evidence/forcing-increment-memory-audit/"), "only additive static audit paths")
    data = git("show", f"{EVIDENCE}:{path}")
    require(data == (ROOT/path).read_bytes(), "actual committed audit blob: "+path)
    blobs[path] = sha(data)

binding = json.loads((HERE/"binary-binding.json").read_text())
disassembly_bindings = {}
for name, key in (("frozen", "frozen"), ("native-reference", "native_reference")):
    binary = binding[key+"_binary"]
    require(sha(Path(binary).read_bytes()) == binding[key+"_binary_sha256"], "unchanged inspected binary")
    raw = subprocess.check_output(["objdump", "-d", "-C", binary]).decode()
    selected = []
    for part in re.split(r"(?m)(?=^[a-f0-9]+ <)", raw):
        header = part.splitlines()[0] if part.splitlines() else ""
        if name == "frozen":
            keep = "research_increment_comparison::" in header and any(
                x in header for x in ("scalar::", "Work::point", "Work::partition", "research_pinned_comparison"))
        else:
            keep = "rheon::coupled_discrete::Work::" in header and any(x in header for x in ("point", "partition", "equation"))
        if keep:
            selected.append(part)
    actual = "".join(selected).encode()
    archive = "frozen-relevant-disassembly.txt" if name == "frozen" else "native-reference-disassembly.txt"
    require(actual == gzip.decompress((HERE/(archive+".gz")).read_bytes()), "archived instructions are actual bound binary disassembly")
    for label, argv in (("symbols", ["nm", "-S", "-nC", binary]),
                        ("relocations", ["readelf", "-rW", binary]),
                        ("headers", ["readelf", "-hW", binary])):
        require(subprocess.check_output(argv) == (HERE/(name+"-"+label+".txt")).read_bytes(), "actual ELF metadata binding")
    disassembly_bindings[name] = dict(binary_sha256=binding[key+"_binary_sha256"], selected_disassembly_sha256=sha(actual))

# Resolve the default libc function pointer without invoking memset or E1.
old = ROOT/"evidence/forcing-increment-comparison"
libc_binding = json.loads((old/"memset-binding.json").read_text())
libc = ctypes.CDLL(libc_binding["library"])
address = ctypes.cast(libc.memset, ctypes.c_void_p).value
resolved = None
for line in Path("/proc/self/maps").read_text().splitlines():
    fields = line.split()
    start, end = (int(v, 16) for v in fields[0].split("-"))
    if start <= address < end:
        require(fields[-1] == libc_binding["library"], "default memset belongs to the bound installed libc")
        resolved = address-start+int(fields[2], 16)
        break
require(resolved == libc_binding["resolved_file_address"], "same actual default IFUNC entry")

commands = []
for label, extra in (("normal", []), ("optimized", ["-O"])):
    argv = [sys.executable, *extra, "evidence/forcing-increment-memory-audit/verify.py"]
    start = time.monotonic()
    result = subprocess.run(argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    seconds = time.monotonic()-start
    stdout, stderr = f"post-git-{label}.log", f"post-git-{label}-stderr.log"
    (HERE/stdout).write_bytes(result.stdout)
    (HERE/stderr).write_bytes(result.stderr)
    commands.append(dict(argv=argv, exit=result.returncode, seconds=seconds, stdout=stdout, stderr=stderr,
                         stdout_sha256=sha(result.stdout), stderr_sha256=sha(result.stderr)))
    require(result.returncode == 0, "actual post-Git static-audit reader pass")
require(commands[0]["stdout_sha256"] == commands[1]["stdout_sha256"], "actual post-Git reader mode parity")
record = dict(status="PASS_ADDITIVE_STATIC_MEMORY_AUDIT_ONLY", base=BASE, evidence=EVIDENCE, evidence_tree=TREE,
              committed_audit_blobs=blobs, disassembly_bindings=disassembly_bindings,
              current_default_memset_file_address=resolved, commands=commands, receipt_sha256=RECEIPT,
              recorder_sha256=sha(Path(__file__).read_bytes()), preserved_prior_files=7068,
              numerical_execution_repeated=False, owner_advances=0, production_changes=0)
(HERE/"post-git-verification.json").write_text(json.dumps(record,indent=2,sort_keys=True)+"\n")
print(json.dumps(dict(status=record["status"], evidence=EVIDENCE, evidence_tree=TREE,
                      committed_audit_blobs=len(blobs), receipt_sha256=RECEIPT),sort_keys=True))
