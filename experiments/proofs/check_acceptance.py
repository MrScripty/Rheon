"""Compile and kernel-audit the isolated experimental acceptance contracts.

Run with the pinned Lean 4.19.0 bin directory on PATH. No original proof
inventory or import root is edited. Temporary audit/probe sources are removed.
"""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
PROOFS = ROOT / "proofs"
SOURCE = HERE / "AlignedStepAcceptance.lean"
AUDIT = HERE / "AlignedStepAcceptanceAudit.lean"
NAMESPACE = "RheonExperiment.AlignedStepAcceptance."
MATHLIB = "c44e0c8ee63ca166450922a373c7409c5d26b00b"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def command(args, cwd=PROOFS):
    result = subprocess.run(args, cwd=cwd, text=True, capture_output=True)
    require(result.returncode == 0, "command failed: " + " ".join(map(str, args))
            + "\n" + result.stdout + result.stderr)
    return result.stdout + result.stderr


def kernel_check(text):
    # Temporary files stay outside the reviewed original proof source tree.
    with tempfile.TemporaryDirectory(prefix="aligned-acceptance-") as temporary:
        path = Path(temporary) / "AcceptanceAudit.lean"
        path.write_text(text)
        return subprocess.run(["lake", "env", "lean", str(path)], cwd=PROOFS,
                              text=True, capture_output=True)


def main():
    print(command(["python3", str(PROOFS / "scripts/check_sources.py")]).strip())
    version = command(["lake", "env", "lean", "--version"]).strip()
    require("version 4.19.0" in version, "experimental audit requires pinned Lean 4.19.0")
    print(version)
    actual_mathlib = command(["git", "rev-parse", "HEAD"],
                             PROOFS / ".lake/packages/mathlib").strip()
    require(actual_mathlib == MATHLIB, "checked-out mathlib differs from the pin")
    print(command(["lake", "build", "Rheon"]).strip())
    print(command(["lake", "env", "lean", str(SOURCE)]).strip())

    source = SOURCE.read_text()
    audit = AUDIT.read_text()
    # The explicit membership list must include every declared source contract.
    declared = {NAMESPACE + name for name in
                re.findall(r"^(?:def|theorem)\s+(\w+)", source, flags=re.MULTILINE)}
    expected = set(re.findall(r"`(" + re.escape(NAMESPACE) + r"\w+)", audit))
    require(declared == expected, "experimental audit declaration membership differs")
    combined = "import Lean.Util.CollectAxioms\n" + source + "\n" + audit
    positive = kernel_check(combined)
    require(positive.returncode == 0, positive.stdout + positive.stderr)
    output = positive.stdout + positive.stderr
    audited = set(re.findall(r"AUDITED (" + re.escape(NAMESPACE) + r"\S+):", output))
    require(expected <= audited, "kernel audit skipped expected experimental declarations")
    require("Experimental axiom audit passed" in output, "kernel audit did not complete")
    print(output.strip())

    # These probes must fail in the actual kernel-assumption audit, independently
    # of Python lexical checks and with Python optimization enabled as well.
    probes = [
        (source + "\nnamespace RheonExperiment.AlignedStepAcceptance\n"
         "axiom injected : False\nend RheonExperiment.AlignedStepAcceptance\n" + audit,
         "Disallowed axiom", "custom axiom"),
        (source + "\nnamespace RheonExperiment.AlignedStepAcceptance\n"
         "theorem injected : False := by sorry\n"
         "end RheonExperiment.AlignedStepAcceptance\n" + audit,
         "Disallowed axiom", "admitted proof"),
        (source + "\n" + audit.replace("let expected : Array Name := #[",
         "let expected : Array Name := #[`Nat.add_zero,"),
         "Expected declaration not audited: Nat.add_zero", "skipped membership"),
    ]
    for probe, diagnostic, label in probes:
        result = kernel_check("import Lean.Util.CollectAxioms\n" + probe)
        require(result.returncode != 0 and diagnostic in result.stdout + result.stderr,
                "negative probe failed: " + label + "\n" + result.stdout + result.stderr)
        print("PASS experimental audit rejects " + label)

    record = {
        "lean_version": version,
        "mathlib_head": actual_mathlib,
        "namespace": NAMESPACE.rstrip("."),
        "source_declarations": len(declared),
        "kernel_audited_declarations": len(audited),
        "allowed_axioms": ["propext", "Classical.choice", "Quot.sound"],
        "sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                   for path in [SOURCE, AUDIT, Path(__file__).resolve()]},
        "original_inventory_sha256": hashlib.sha256(
            (PROOFS / "source-inventory.json").read_bytes()).hexdigest(),
        "python_optimization": sys.flags.optimize,
        "negative_kernel_probes_passed": len(probes),
        "scope": "conditional exact-real contracts; enclosure containment is a premise",
    }
    print(json.dumps(record, sort_keys=True))


if __name__ == "__main__":
    main()
