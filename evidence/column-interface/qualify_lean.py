"""Qualify in an isolated, provisioned copy of the original pinned Lean project.

Usage: qualify_lean.py ISOLATED_PROJECT FRESH_OUTPUT. Requires lake/lean on PATH.
The historical proofs project and its frozen inventory are never modified.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
project, output = map(lambda value: Path(value).resolve(), sys.argv[1:])
if project == ROOT / "proofs":
    raise ValueError("use an isolated project copy")
for name in ("lean-toolchain", "lake-manifest.json", "lakefile.toml"):
    if (project / name).read_bytes() != (ROOT / "proofs" / name).read_bytes():
        raise ValueError("dependency pin changed: " + name)
if subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=project / ".lake/packages/mathlib").decode().strip() != "c44e0c8ee63ca166450922a373c7409c5d26b00b":
    raise ValueError("mathlib checkout mismatch")
output.mkdir()
good = (HERE / "ColumnSurface.lean").read_text()
(project / "AuditColumnSurface.lean").write_bytes((HERE / "AuditColumnSurface.lean").read_bytes())
(project / ".lake/build/lib/lean").mkdir(parents=True, exist_ok=True)
commands = []


def run(name, args):
    with (output / (name + ".log")).open("w") as stream:
        code = subprocess.run(args, cwd=project, stdout=stream, stderr=subprocess.STDOUT).returncode
    commands.append({"name": name, "command": args, "exit": code})
    (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
    return code


compile_command = ["lake", "env", "lean", "-o", ".lake/build/lib/lean/ColumnSurface.olean", "ColumnSurface.lean"]
audit_command = ["lake", "env", "lean", "AuditColumnSurface.lean"]
variants = {"sorry": good.replace("  field_simp\n  all_goals ring", "  sorry", 1),
            "axiom": good.replace("end\nend Rheon.ColumnSurface", "axiom forbidden_review_assumption : False\nend\nend Rheon.ColumnSurface")}
results = []
try:
    (project / "ColumnSurface.lean").write_text(good)
    if run("build", compile_command) or run("audit", audit_command):
        raise ValueError("positive Lean qualification failed")
    for name, text in variants.items():
        (project / "ColumnSurface.lean").write_text(text)
        build = run(name + "-build", compile_command)
        audit = run(name + "-audit", audit_command)
        if build != 0 or audit == 0:
            raise ValueError("negative audit did not reject: " + name)
        results.append({"variant": name, "source_sha256": hashlib.sha256(text.encode()).hexdigest(),
                        "compile_exit": build, "audit_exit": audit})
finally:
    (project / "ColumnSurface.lean").write_text(good)
    if run("restored-build", compile_command) or run("restored-audit", audit_command):
        raise ValueError("restored qualification failed")
(output / "receipt.json").write_text(json.dumps({"source_sha256": hashlib.sha256(good.encode()).hexdigest(),
    "audit_sha256": hashlib.sha256((HERE / "AuditColumnSurface.lean").read_bytes()).hexdigest(),
    "lean_version": subprocess.check_output(["lake", "env", "lean", "--version"], cwd=project).decode().strip(),
    "mathlib_commit": "c44e0c8ee63ca166450922a373c7409c5d26b00b", "audited_declarations": 10,
    "negative_variants": results}, indent=2) + "\n")
print("PASS positive, two negative audits and restored exact source")
