"""Fresh local opt-in bounded repeated Coulomb sphere impacts qualification. Never requests hosted CI."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
BASE = "717dd9c7d93a2c425e0e5e6160a12521367a2ea5"


LIB_MODULES = b"mod sphere_event_loop;\nmod sphere_friction_interval;\n"
LIB_EXPORT = b"pub use sphere_friction_interval::*;\n"
# Exact independently reviewable adapters at this new reconstruction checkpoint.
# No historical operator/pressure/shear formula is mutable through this gate.
ADAPTER_HASHES = {
    "src/sphere_interval.rs": "1cf44bc0870aa018f35a24512bf8ac07ab49e92460015efc53dc0b270fb4b6dc",
    "src/sphere_friction.rs": "6f22545eec80930e0735f7d24e9ebb59ec9aef101ff572e282398d5478df3475",
}

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def qualify(output, lean_bin, dependencies):
    output, lean_bin, dependencies = map(lambda p: Path(p).resolve(), (output, lean_bin, dependencies))
    if output.exists() or output.is_relative_to(ROOT):
        raise ValueError("fresh output outside repository required")
    if git("status", "--porcelain", "--untracked-files=all"):
        raise ValueError("clean committed source required")
    files = git("ls-files").splitlines()
    sources = {p: sha(ROOT / p) for p in files}
    baseline_files = git("ls-tree", "-r", "--name-only", BASE).splitlines()
    permitted = {"src/lib.rs", "src/sphere_interval.rs", "src/sphere_friction.rs", "proofs/Rheon.lean", "proofs/AxiomAudit.lean", "proofs/source-inventory.json", ".gitignore"}
    protected = [p for p in baseline_files if p not in permitted]
    for p in protected:
        old = subprocess.check_output(["git", "show", f"{BASE}:{p}"], cwd=ROOT)
        if hashlib.sha256(old).hexdigest() != sources.get(p):
            raise ValueError("protected preserved717 file changed: " + p)
    lib_old = subprocess.check_output(["git", "show", f"{BASE}:src/lib.rs"], cwd=ROOT)
    if (ROOT / "src/lib.rs").read_bytes() != lib_old.replace(b"mod sphere_interval;\n", LIB_MODULES + b"mod sphere_interval;\n").replace(b"pub use sphere_interval::*;\n", LIB_EXPORT + b"pub use sphere_interval::*;\n"):
        raise ValueError("unexpected export adaptation")
    for name, digest in ADAPTER_HASHES.items():
        if sources[name] != digest:
            raise ValueError("unreviewed shared-loop/selected-response adapter: " + name)
    old_friction = subprocess.check_output(["git", "show", f"{BASE}:src/sphere_friction.rs"], cwd=ROOT).decode()
    new_friction = (ROOT / "src/sphere_friction.rs").read_text()
    if old_friction.split("\nfn contact_stage", 1)[1] != new_friction.split("\nfn contact_stage", 1)[1]:
        raise ValueError("isolated Coulomb arithmetic changed")
    old_interval = subprocess.check_output(["git", "show", f"{BASE}:src/sphere_interval.rs"], cwd=ROOT).decode()
    new_interval = (ROOT / "src/sphere_interval.rs").read_text()
    if old_interval.split("pub const", 1)[1].split("impl SphericalRigidMotion", 1)[0] != new_interval.split("pub const", 1)[1].split("impl SphericalRigidMotion", 1)[0]:
        raise ValueError("existing frictionless public types changed")
    old_tail = old_interval.split("\nfn finite", 1)[1]
    new_tail = new_interval.replace("pub(crate) fn kinetic", "fn kinetic").split("\npub(crate) fn finite", 1)[1]
    if old_tail != new_tail:
        raise ValueError("existing frictionless accounting formula changed")
    manifest = json.loads((ROOT / "proofs/lake-manifest.json").read_text())
    dependency_pins = {}
    for package in manifest["packages"]:
        p = dependencies / package["name"]
        actual = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=p, text=True).strip()
        if actual != package["rev"]:
            raise ValueError("Lean dependency pin differs: " + package["name"])
        dependency_pins[package["name"]] = actual
    output.mkdir(parents=True)
    receipt = {"source_head": git("rev-parse", "HEAD"), "source_tree": git("rev-parse", "HEAD^{tree}"),
               "source_clean": True, "source_sha256": sources, "protected_sources": protected, "narrow_adaptations": {"src/sphere_interval.rs": "exact frozen shared-loop adapter; old public types/accounting unchanged", "src/sphere_friction.rs": "exact frozen selected-hit/departure adapter; old response formulas unchanged", "src/lib.rs": "exact three-line private engine and typed API insertion"},
               "lean_dependency_pins": dependency_pins, "commands": [], "qualified": False,
               "scope": "bounded repeated isolated Coulomb events; exact-real isotropic/radial finite-ledger premises; no persistent contact/force or fluid coupling", "hosted_ci_requested": False, "held_campaigns": 0}

    def run(command, name, cwd=ROOT, env=None, expected_error=None):
        r = subprocess.run(list(map(str, command)), cwd=cwd, env=env, capture_output=True,
                           text=True, timeout=180)
        log = r.stdout + r.stderr
        (output / name).write_text(log)
        receipt["commands"].append({"command": list(map(str, command)), "cwd": str(cwd),
                                    "returncode": r.returncode, "log": name,
                                    "log_sha256": sha(output / name)})
        if expected_error is None:
            if r.returncode:
                raise RuntimeError("command failed; inspect " + name)
        else:
            errors = [line.split(": error: ", 1)[1] for line in log.splitlines()
                      if ": error: " in line]
            if r.returncode == 0 or errors != [expected_error]:
                raise RuntimeError("negative probe must produce only its exact intended axiom error: " + name)
        return r.stdout

    try:
        receipt["toolchain"] = {"rustc": run(["rustc", "-Vv"], "rustc.log"),
                                "cargo": run(["cargo", "--version"], "cargo.log"),
                                "lean": run([lean_bin, "--version"], "lean-version.log")}
        run(["cargo", "test", "--locked", "--no-default-features", "--test", "sphere_friction_interval_contract", "--test", "sphere_friction_contract", "--test", "sphere_interval_contract", "--test", "sphere_contact_contract", "--test", "rigid_motion_contract", "--test", "rigid_impulse_contract", "--test", "mesh_traction_contract",
             "--test", "collision_contract", "--test", "translation_contract"], "native-debug.log")
        run(["cargo", "test", "--locked", "--no-default-features", "--lib", "sphere_"], "native-exact-sign.log")
        run(["cargo", "test", "--release", "--locked", "--no-default-features", "--test",
             "sphere_friction_interval_contract", "--test", "sphere_friction_contract", "--test", "sphere_interval_contract", "--test", "sphere_contact_contract", "--test", "rigid_motion_contract", "--test", "rigid_impulse_contract", "--test", "mesh_traction_contract"], "native-release.log")
        run(["cargo", "clippy", "--locked", "--no-default-features", "--example", "sphere_friction_interval", "--example", "sphere_friction", "--example", "sphere_contact", "--example", "rigid_motion", "--example", "sphere_interval",
             "--test", "sphere_friction_interval_contract", "--test", "sphere_friction_contract", "--test", "sphere_interval_contract", "--test", "sphere_contact_contract", "--test", "rigid_motion_contract", "--", "-D", "warnings"], "clippy.log")
        run(["rustfmt", "--edition", "2024", "--check", "src/lib.rs", "src/rigid_motion.rs",
             "src/sphere_event_loop.rs", "src/sphere_friction_interval.rs", "src/sphere_interval.rs", "examples/sphere_friction_interval.rs", "tests/sphere_friction_interval_contract.rs", "src/sphere_friction.rs", "examples/sphere_friction.rs", "tests/sphere_friction_contract.rs", "src/sphere_contact.rs", "src/sphere_departure.rs", "src/sphere_interval.rs", "examples/sphere_interval.rs", "tests/sphere_interval_contract.rs", "examples/sphere_contact.rs", "tests/sphere_contact_contract.rs"], "format.log")
        run([sys.executable, "proofs/scripts/check_sources.py"], "proof-source-gate.log")
        builds = {}
        for example in ["sphere_friction_interval", "sphere_friction", "sphere_contact", "rigid_motion", "sphere_interval"]:
            for profile in ["debug", "release"]:
                build = ["cargo", "build", "--locked", "--no-default-features", "--example", example]
                if profile == "release": build.append("--release")
                run(build, "build-" + example + "-" + profile + ".log")
                metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=ROOT))
                executable = Path(metadata["target_directory"]) / profile / "examples" / example
                copies = output / "binaries"; copies.mkdir(exist_ok=True)
                shutil.copyfile(executable, copies / (example + "-" + profile))
                builds[example + "-" + profile] = {"path": str(executable), "sha256": sha(executable)}
                for optimized in ([False, True] if profile == "debug" else [False]):
                    name = "oracle-" + example + "-" + profile + ("-optimized" if optimized else "")
                    command = [sys.executable] + (["-O"] if optimized else []) + ["tools/check_" + example + ".py", "--executable", str(executable), "--output", str(output / name)]
                    run(command, name + ".log")
        receipt["binaries"] = builds
        render_env = dict(os.environ, MPLCONFIGDIR=str(output / "mpl-cache"), XDG_CACHE_HOME=str(output / "cache"))
        run([sys.executable, "tools/render_sphere_contact.py", str(output / "oracle-sphere_contact-debug"), str(output / "rendered")], "render.log", env=render_env)

        run([sys.executable, "tools/render_sphere_interval.py", str(output / "oracle-sphere_interval-debug"), str(output / "rendered-sequence")], "render-sequence.log", env=render_env)

        run([sys.executable, "tools/render_sphere_friction.py", str(output / "oracle-sphere_friction-debug"), str(output / "rendered-friction")], "render-friction.log", env=render_env)

        run([sys.executable, "tools/render_sphere_friction_interval.py", str(output / "oracle-sphere_friction_interval-debug"), str(output / "rendered-friction-interval")], "render-friction-interval.log", env=render_env)

        # Compile ALL Rheon proof modules from current source into external output.
        # Only pinned third-party dependency caches are reused, not old Rheon proofs.
        proof_output = output / "lean-build"
        (proof_output / "Rheon").mkdir(parents=True)
        paths = [str(proof_output)] + [str(p / ".lake/build/lib/lean") for p in sorted(dependencies.iterdir())
                                      if (p / ".lake/build/lib/lean").is_dir()]
        env = dict(os.environ, LEAN_PATH=":".join(paths))
        modules = {"Rheon." + p.stem: p for p in (ROOT / "proofs/Rheon").glob("*.lean")}
        completed = set()
        while len(completed) < len(modules):
            ready = [name for name, p in modules.items() if name not in completed and
                     all(dep not in modules or dep in completed for dep in
                         re.findall(r"^import (Rheon\.[A-Za-z]+)$", p.read_text(), re.M))]
            if not ready:
                raise ValueError("proof module dependency cycle")
            for name in sorted(ready):
                p = modules[name]
                run([lean_bin, "-o", proof_output / "Rheon" / (p.stem + ".olean"),
                     str(p.relative_to(ROOT / "proofs"))], "lean-" + p.stem + ".log", ROOT / "proofs", env)
                completed.add(name)
        run([lean_bin, "-o", proof_output / "Rheon.olean", "Rheon.lean"], "lean-root.log", ROOT / "proofs", env)
        run([lean_bin, "AxiomAudit.lean"], "axiom-audit.log", ROOT / "proofs", env)
        audit = (ROOT / "proofs/AxiomAudit.lean").read_text()
        for label, declaration in [("axiom", "axiom injected : False"),
                                   ("sorry", "theorem injected : False := by sorry")]:
            probe = proof_output / ("negative-" + label + ".lean")
            probe.write_text(audit.replace("open Lean Elab Command", "namespace Rheon.SphereFrictionInterval\n" +
                             declaration + "\nend Rheon.SphereFrictionInterval\nopen Lean Elab Command"))
            run([lean_bin, probe], "negative-" + label + ".log", ROOT / "proofs", env,
                expected_error="Disallowed axiom " +
                ("Rheon.SphereFrictionInterval.injected" if label == "axiom" else "sorryAx") +
                " in Rheon.SphereFrictionInterval.injected")
        if sources != {p: sha(ROOT / p) for p in files} or git("rev-parse", "HEAD") != receipt["source_head"]:
            raise ValueError("source changed during qualification")
        if git("status", "--porcelain", "--untracked-files=all"):
            raise ValueError("source dirtied during qualification")
        for binary in builds.values():
            if sha(Path(binary["path"])) != binary["sha256"]:
                raise ValueError("native executable changed during qualification")
        receipt["qualified"] = True
    except Exception as error:
        receipt["failure"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        receipt["evidence_sha256"] = {str(p.relative_to(output)): sha(p) for p in sorted(output.rglob("*"))
                                     if p.is_file()}
        (output / "qualification.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return {"qualified": True, "source_head": receipt["source_head"], "output": str(output)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--lean-bin", type=Path, required=True)
    parser.add_argument("--lean-dependencies", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(qualify(args.output, args.lean_bin, args.lean_dependencies), indent=2))
