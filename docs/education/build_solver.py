#!/usr/bin/env python3
"""Build and admit the bounded native/WASM force-cycle packet outside Git.

This does not modify the solver, install tools, deploy a site, or confer browser
qualification. The book's Chromium checks separately compare the worker result.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import struct
import tempfile

FIXTURE = "force-cycle-2x2x1-v1"
WASM_BINDGEN = "0.2.105"
RUST_VERSION = "1.92.0"
PUBLIC_ARTIFACTS = (
    "rheon_solver.js", "rheon_solver_bg.wasm", "rheon_solver.d.ts",
    "rheon_solver_bg.wasm.d.ts", "native-reference.json",
)
NATIVE_EXPORTER = "native/rheon-solver-reference"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_paths(repo: Path) -> list[Path]:
    paths = [repo / name for name in (
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
        "browser/solver/Cargo.toml", "browser/solver/Cargo.lock",
        "browser/solver/README.md", "docs/education/build_solver.py",
        "docs/education/test_build_solver.py",
        "tests/forces_contract.rs",
    )]
    paths.extend((repo / "src").rglob("*.rs"))
    paths.extend((repo / "browser/solver/src").rglob("*.rs"))
    paths.extend((repo / "browser/solver/tests").rglob("*.rs"))
    return sorted(set(paths))


def source_hashes(repo: Path) -> dict[str, str]:
    return {str(p.relative_to(repo)): digest(p) for p in source_paths(repo)}


def run(args: list[str], repo: Path, *, env=None) -> str:
    return subprocess.check_output(args, cwd=repo, env=env, text=True)


def external_directory(repo: Path, path: Path) -> Path:
    path = path.resolve()
    if path == repo or repo in path.parents or path in repo.parents:
        raise ValueError("solver artifacts must be outside the repository and its ancestors")
    probe = path
    while not probe.exists():
        probe = probe.parent
    if subprocess.run(["git", "-C", str(probe), "rev-parse", "--is-inside-work-tree"], capture_output=True).returncode == 0:
        raise ValueError("solver artifacts must be outside every Git checkout")
    return path


def _read_json(path: Path):
    return json.loads(path.read_text(), parse_constant=lambda s: (_ for _ in ()).throw(ValueError(s)))


def validate_reference(reference: dict) -> None:
    """Independently derive the fixed cycle and audit every exported transition."""
    if reference.get("schema") != 1 or reference.get("fixture") != FIXTURE or reference.get("dt") != .25:
        raise ValueError("unsupported native reference")
    runs = reference.get("runs", [])
    if [r.get("method") for r in runs] != ["jacobi-pcg-v1", "sgs-pcg-v1"]:
        raise ValueError("native reference must cover both pressure implementations")
    # c*(u.c)/(c.c), c=(1,-1,-1,1), u=(.5,0,0,0).
    fields = {"x": [0., .125, 0., 0., -.125, 0.], "y": [0., 0., -.125, .125, 0., 0.], "z": [0.]*8, "tracer": [0.]*4}
    grid = {"counts": [2, 2, 1], "spacing": [1., 1., 1.], "origin": [0., 0., 0.]}
    for run in runs:
        for name, generation, advanced in (("baseline", 0, False), ("accepted", 1, True), ("reset", 2, False), ("after_reset", 3, True)):
            state = run[name] if name in ("baseline", "reset") else run[name]["state"]
            expected = {
                "schema": 1, "fixture": FIXTURE, "method": run["method"], "grid": grid,
                "time": .25 if advanced else 0., "generation": generation,
                "generation_u64": str(generation), "accepted_since_reset": advanced,
                "time_bits": "3fd0000000000000" if advanced else "0000000000000000",
            }
            for key, value in expected.items():
                if state.get(key) != value:
                    raise ValueError(f"native reference {name} {key} differs from fixed fixture")
            for key, values in fields.items():
                expected_values = values if advanced else [0.] * len(values)
                expected_bits = [struct.unpack("<I", struct.pack("<f", v))[0] for v in expected_values]
                if state.get(key) != expected_values or state.get("bits", {}).get(key) != expected_bits:
                    raise ValueError(f"native reference {name} {key} differs from exact cycle")
            if advanced:
                outcome = run[name]
                if outcome.get("status") != "accepted" or outcome.get("code") != "accepted" or outcome.get("detail") is not None:
                    raise ValueError("native reference acceptance status is inconsistent")
                report = outcome["report"]
                report_expected = {"dt": .25, "time": .25, "generation": generation, "kinetic_energy": 1/32, "courant": 1/16, "actual_divergence_max": 0., "tracer_integral": 0., "forces": {"kinetic_energy_before": 0., "kinetic_energy_after": 1/8, "applied_work": 1/8}}
                if any(report.get(k) != v for k, v in report_expected.items()):
                    raise ValueError("native reference force work/energy/Courant differs from exact cycle")
                pressure = report["pressure"]
                if not isinstance(pressure.get("iterations"), int) or not 0 < pressure["iterations"] <= 1000:
                    raise ValueError("invalid native pressure iteration count")
                for key in ("true_residual_l2", "true_residual_max", "predicted_divergence_max"):
                    value = pressure.get(key)
                    if not isinstance(value, (float, int)) or not math.isfinite(value) or not 0 <= value <= 1e-9:
                        raise ValueError("invalid native pressure residual")
        for name, code, state in (("invalid_dt", "invalid_time_step", run["baseline"]), ("cancelled_rest", "cancelled_before_commit", run["baseline"]), ("cancelled_nonzero", "cancelled_before_commit", run["accepted"]["state"]), ("second_accept", "reset_required", run["accepted"]["state"])):
            outcome = run[name]
            if outcome.get("status") != "rejected" or outcome.get("code") != code or outcome.get("report") is not None or not outcome.get("detail") or outcome.get("state") != state:
                raise ValueError(f"native reference {name} does not preserve accepted state")


def _validate_public(repo: Path, packet: Path) -> dict:
    receipt = _read_json(packet / "build-receipt.json")
    if receipt.get("schema") != 1 or receipt.get("fixture") != FIXTURE:
        raise ValueError("unsupported solver packet")
    if receipt.get("wasm_bindgen_version") != WASM_BINDGEN:
        raise ValueError("solver packet wasm-bindgen version mismatch")
    if receipt.get("target") != "wasm32-unknown-unknown" or receipt.get("profile") != "release":
        raise ValueError("solver packet target/profile mismatch")
    if not str(receipt.get("rustc", "")).startswith(f"rustc {RUST_VERSION} "):
        raise ValueError("solver packet compiler version mismatch")
    if receipt.get("source_sha256") != source_hashes(repo):
        raise ValueError("solver packet does not match current source bytes")
    artifacts = {name: digest(packet / name) for name in PUBLIC_ARTIFACTS}
    if receipt.get("artifact_sha256") != artifacts:
        raise ValueError("solver packet public artifact hash mismatch")
    if (packet / "rheon_solver_bg.wasm").read_bytes()[:8] != b"\0asm\x01\0\0\0":
        raise ValueError("solver artifact is not WebAssembly version 1")
    reference = _read_json(packet / "native-reference.json")
    validate_reference(reference)
    return receipt


def validate_packet(repo: Path, packet: Path | None) -> dict | None:
    """Reject stale source/artifact receipts and independently rerun the host.

    A packet is trusted local build output. The executable is permitted only at
    its fixed path after source and artifact hash validation; no receipt-supplied
    command or arbitrary executable path is accepted.
    """
    if packet is None:
        return None
    repo, packet = Path(repo).resolve(), Path(packet).resolve()
    receipt = _validate_public(repo, packet)
    native = receipt.get("native_exporter")
    if native != {"path": NATIVE_EXPORTER, "sha256": digest(packet / NATIVE_EXPORTER)}:
        raise ValueError("solver packet native exporter hash mismatch")
    reference = _read_json(packet / "native-reference.json")
    actual = json.loads(run([str(packet / NATIVE_EXPORTER)], repo))
    if actual != reference:
        raise ValueError("fresh native exporter differs from packet reference")
    return receipt


def _presentation(receipt: dict, path: Path) -> dict:
    return {"schema": 1, "included": True, "fixture": FIXTURE,
            "source_git_sha": receipt["source_git_sha"],
            "build_receipt_sha256": digest(path)}


def verify_published(repo: Path, edition: Path) -> dict:
    """Read-only public source/artifact/oracle validation, without a host binary."""
    repo, edition = Path(repo).resolve(), Path(edition).resolve()
    presentation = _read_json(edition / "solver-presentation.json")
    if presentation == {"schema": 1, "included": False}:
        if (edition / "solver").exists():
            raise ValueError("unavailable solver presentation has stale solver assets")
        return presentation
    packet = edition / "solver"
    receipt = _validate_public(repo, packet)
    if presentation != _presentation(receipt, packet / "build-receipt.json"):
        raise ValueError("solver presentation does not match its build receipt")
    return presentation


def publish(repo: Path, out: Path, packet: Path | None) -> dict:
    """Copy admitted public assets into an existing external book output."""
    repo = Path(repo).resolve()
    out = external_directory(repo, Path(out))
    out.mkdir(parents=True, exist_ok=True)
    if packet is None:
        if (out / "solver").exists():
            raise ValueError("refuse unavailable presentation over stale solver assets")
        presentation = {"schema": 1, "included": False}
        (out / "solver-presentation.json").write_text(json.dumps(presentation) + "\n")
        return presentation
    packet = Path(packet).resolve()
    receipt = validate_packet(repo, packet)
    destination = out / "solver"
    destination.mkdir(parents=True, exist_ok=True)
    for name in (*PUBLIC_ARTIFACTS, "build-receipt.json"):
        shutil.copyfile(packet / name, destination / name)
    presentation = _presentation(receipt, packet / "build-receipt.json")
    (out / "solver-presentation.json").write_text(json.dumps(presentation, indent=2, sort_keys=True) + "\n")
    verify_published(repo, out)
    return presentation


def build(repo: Path, output: Path, target_dir: Path | None = None) -> dict:
    repo = Path(repo).resolve()
    output = external_directory(repo, Path(output))
    if output.exists() and any(output.iterdir()):
        raise ValueError("solver packet destination must be absent or empty")
    wasm_version = run(["wasm-bindgen", "--version"], repo).strip()
    if wasm_version != f"wasm-bindgen {WASM_BINDGEN}":
        raise ValueError(f"requires wasm-bindgen-cli {WASM_BINDGEN}, found {wasm_version}")
    rust_version = run(["rustc", "--version"], repo).strip()
    if not rust_version.startswith(f"rustc {RUST_VERSION} "):
        raise ValueError(f"requires rustc {RUST_VERSION}, found {rust_version}")
    initial_sources = source_hashes(repo)
    env = os.environ.copy()
    if target_dir is None:
        target_dir = Path(env.get("CARGO_TARGET_DIR", str(output.parent / "rheon-solver-target")))
    target_dir = external_directory(repo, target_dir)
    env["CARGO_TARGET_DIR"] = str(target_dir)
    manifest = str(repo / "browser/solver/Cargo.toml")
    cargo = ["cargo", "--locked", "--manifest-path", manifest]
    # Commands keep --locked as a subcommand option for older Cargo versions.
    subprocess.run([cargo[0], "test", *cargo[1:]], cwd=repo, env=env, check=True)
    subprocess.run([cargo[0], "build", *cargo[1:], "--release", "--target", "wasm32-unknown-unknown", "--lib"], cwd=repo, env=env, check=True)
    subprocess.run([cargo[0], "build", *cargo[1:], "--release", "--bin", "rheon-solver-reference"], cwd=repo, env=env, check=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="rheon-solver-packet-", dir=output.parent) as tmp:
        stage = Path(tmp)
        subprocess.run([
            "wasm-bindgen", str(target_dir / "wasm32-unknown-unknown/release/rheon_solver.wasm"),
            "--target", "web", "--out-dir", str(stage), "--out-name", "rheon_solver",
        ], cwd=repo, env=env, check=True)
        (stage / "native").mkdir()
        shutil.copy2(target_dir / "release/rheon-solver-reference", stage / NATIVE_EXPORTER)
        native_text = run([str(stage / NATIVE_EXPORTER)], repo)
        (stage / "native-reference.json").write_text(native_text)
        if source_hashes(repo) != initial_sources:
            raise ValueError("solver sources changed during build; rebuild from stable inputs")
        receipt = {
            "schema": 1,
            "fixture": FIXTURE,
            "wasm_bindgen_version": WASM_BINDGEN,
            "rustc": rust_version,
            "cargo": run(["cargo", "--version"], repo).strip(),
            "source_git_sha": run(["git", "rev-parse", "HEAD"], repo).strip(),
            "source_git_dirty": bool(run(["git", "status", "--porcelain"], repo).strip()),
            "source_sha256": initial_sources,
            "artifact_sha256": {name: digest(stage / name) for name in PUBLIC_ARTIFACTS},
            "native_exporter": {"path": NATIVE_EXPORTER, "sha256": digest(stage / NATIVE_EXPORTER)},
            "target": "wasm32-unknown-unknown",
            "profile": "release",
            "contract_tests": 6,
            "limitations": [
                "One accepted fixed-input timestep per native reset; no validated long-time trajectory.",
                "Cancellation is a predetermined BeforeCommit callback, not asynchronous worker interruption.",
                "The native reference and WASM share source; independent four-edge equations are checked separately.",
                "This build receipt does not claim browser qualification or Lean verification of the compiled timestep.",
            ],
        }
        (stage / "build-receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
        validate_packet(repo, stage)
        output.mkdir(parents=True, exist_ok=True)
        for path in stage.iterdir():
            destination = output / path.name
            if path.is_dir():
                shutil.copytree(path, destination)
            else:
                shutil.copy2(path, destination)
    return receipt


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", "--out", dest="out", type=Path, required=True)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--target-dir", type=Path)
    args = parser.parse_args()
    receipt = build(args.repo, args.out, args.target_dir)
    print(json.dumps({"packet": str(args.out.resolve()), "fixture": FIXTURE, "artifact_sha256": receipt["artifact_sha256"]}, indent=2))


if __name__ == "__main__":
    main()
