#!/usr/bin/env python3
"""Build and admit the bounded native/WASM cycle and 3D flow packet outside Git.

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
import re
import shutil
import subprocess
import struct
import tempfile

FIXTURE = "force-cycle-2x2x1-v1"
FLOW_FIXTURE = "force-plume-24-v1"
FLOW_DT = .02
FLOW_STEPS = 80
FLOW_ENCODING = {
    "format": "little-endian-f32", "field_order": ["x", "y", "z", "tracer"],
    "field_lengths": [14400, 14400, 14400, 13824], "bytes_per_state": 228096,
}
FLOW_PARAMETERS = {
    "density": 1.2, "source_lower": [.30, .08, .38], "source_upper": [.50, .28, .58],
    "tracer_rate": 2., "vertical_acceleration": 0., "force_density": [.6, 1.2, -.3],
    "force_lower": [.25, .10, .30], "force_upper": [.60, .45, .65],
    "memory_limit": 64 * 1024 * 1024, "relative_residual": 1e-9,
    "absolute_residual": 1e-12, "predicted_divergence_limit": 1e-7,
    "pressure_max_iterations": 2000, "actual_divergence_limit": 1e-5, "max_courant": 1.,
}
WASM_BINDGEN = "0.2.105"
RUST_VERSION = "1.92.0"
PUBLIC_ARTIFACTS = (
    "rheon_solver.js", "rheon_solver_bg.wasm", "rheon_solver.d.ts",
    "rheon_solver_bg.wasm.d.ts", "native-reference.json",
    "native-flow-reference.json", "native-flow-fields.bin",
)
NATIVE_EXPORTER = "native/rheon-solver-reference"
NATIVE_FLOW_EXPORTER = "native/rheon-flow-reference"


def passed_contract_tests(output: str) -> int:
    """Count the tests Cargo actually completed, including every test binary."""
    results = re.findall(
        r"^test result: ok\. (\d+) passed; (\d+) failed; \d+ ignored; \d+ measured; \d+ filtered out;",
        output, flags=re.MULTILINE,
    )
    if not results or any(int(failed) for _, failed in results):
        raise ValueError("Cargo did not report a successful contract test run")
    count = sum(int(passed) for passed, _ in results)
    if not count:
        raise ValueError("Cargo reported no executed contract tests")
    return count


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


def validate_flow_reference(reference: dict, fields: bytes) -> None:
    """Admit the bounded full-tensor transcript, not a substituted render.

    This checks the native transcript's schema, ordered state ownership, field
    encoding and rollback/reset invariants. Scientific field diagnostics and
    native-versus-WASM comparisons belong to the independent browser qualifier.
    """
    if not isinstance(reference, dict) or type(reference.get("schema")) is not int or reference["schema"] != 1 or reference.get("fixture") != FLOW_FIXTURE:
        raise ValueError("unsupported native flow reference")
    if reference.get("encoding") != FLOW_ENCODING:
        raise ValueError("native flow field encoding mismatch")
    runs = reference.get("runs")
    if not isinstance(runs, list) or any(not isinstance(r, dict) for r in runs) or [r.get("method") for r in runs] != ["jacobi-pcg-v1", "sgs-pcg-v1"]:
        raise ValueError("native flow reference must cover both pressure implementations in order")
    probes = ("cancelled_rest", "cancelled_nonzero", "limit_rejection", "reset", "after_reset")
    expected_states = 2 * (FLOW_STEPS + 1 + len(probes))
    if len(fields) != expected_states * FLOW_ENCODING["bytes_per_state"]:
        raise ValueError("native flow binary length does not match its bounded state count")
    if type(reference.get("binary_bytes")) is not int or reference["binary_bytes"] != len(fields):
        raise ValueError("native flow declared binary length mismatch")
    all_frames = []
    for run in runs:
        frames = run.get("frames")
        if not isinstance(frames, list) or len(frames) != FLOW_STEPS + 1:
            raise ValueError("native flow frames must include baseline and all 80 accepted steps")
        if any(not isinstance(run.get(name), dict) for name in probes):
            raise ValueError("native flow transaction/reset probes are missing")
        all_frames.extend((frames[0], run["cancelled_rest"], *frames[1:31],
                           run["cancelled_nonzero"], *frames[31:],
                           run["limit_rejection"], run["reset"], run["after_reset"]))
    for frame in all_frames:
        if not isinstance(frame, dict) or set(frame) != {"kind", "status", "code", "detail", "byte_offset", "byte_length", "state", "report"}:
            raise ValueError("native flow frame must be an object")
        if type(frame.get("byte_offset")) is not int or frame["byte_offset"] < 0:
            raise ValueError("native flow frame byte offset is invalid")
        if type(frame.get("byte_length")) is not int or frame["byte_length"] != FLOW_ENCODING["bytes_per_state"]:
            raise ValueError("native flow frame byte length is invalid")
    for index, frame in enumerate(all_frames):
        if frame["byte_offset"] != index * FLOW_ENCODING["bytes_per_state"]:
            raise ValueError("native flow fields overlap, contain gaps or are out of execution order")
    # Decode the declared format, even on a host whose native byte order differs.
    # iter_unpack avoids allocating another full copy of the 39 MB field packet.
    if any(not math.isfinite(value) for (value,) in struct.iter_unpack("<f", fields)):
        raise ValueError("native flow fields contain nonfinite values")

    def field_bytes(frame):
        start = frame["byte_offset"]
        return fields[start:start + frame["byte_length"]]

    def finite(value):
        return type(value) in (int, float) and math.isfinite(value)

    def state(frame, method, steps, generation, time, dt):
        snapshot = frame.get("state")
        if not isinstance(snapshot, dict):
            raise ValueError("native flow state metadata is missing")
        expected = {
            "schema": 1, "fixture": FLOW_FIXTURE, "method": method,
            "grid": {"counts": [24] * 3, "spacing": [1 / 24] * 3, "origin": [0.] * 3},
            "parameters": FLOW_PARAMETERS,
            "source": {"lower": FLOW_PARAMETERS["source_lower"], "upper": FLOW_PARAMETERS["source_upper"],
                       "tracer_rate": 2., "vertical_acceleration": 0.},
            "generation": generation,
            "generation_u64": str(generation), "accepted_steps": steps,
            "max_steps": FLOW_STEPS, "source_steps": 30,
            "next_source_active": steps < 30, "requested_dt": FLOW_DT,
            "last_accepted_dt": dt, "time": time,
            "time_bits": f'{struct.unpack("<Q", struct.pack("<d", time))[0]:016x}',
        }
        if any(snapshot.get(key) != value for key, value in expected.items()):
            raise ValueError("native flow state ordering, time or fixed parameters mismatch")
        if any(type(snapshot.get(key)) is not int for key in ("schema", "generation", "accepted_steps", "max_steps", "source_steps")) or type(snapshot.get("next_source_active")) is not bool:
            raise ValueError("native flow state counters/flags have invalid types")
        if set(snapshot) != set(expected) | {"tracer_integral_at_source_cutoff", "managed_bytes", "metrics"}:
            raise ValueError("native flow state metadata schema mismatch")
        if type(snapshot.get("managed_bytes")) is not int or not 0 < snapshot["managed_bytes"] <= FLOW_PARAMETERS["memory_limit"]:
            raise ValueError("native flow managed allocation exceeds its declared bound")
        cutoff = snapshot.get("tracer_integral_at_source_cutoff")
        if (steps < 30 and cutoff is not None) or (steps >= 30 and (not finite(cutoff) or cutoff < 0)):
            raise ValueError("native flow source cutoff state is inconsistent")
        metrics = snapshot.get("metrics")
        if not isinstance(metrics, dict) or set(metrics) != {"max_u_v_w", "component_kinetic_energy", "actual_divergence_max", "courant", "tracer_min", "tracer_max", "tracer_integral", "tracer_centroid", "tracer_axis_stddev", "tracer_outside_source_fraction", "tracer_integral_change_since_cutoff", "tracer_adjacent_slice_rms"}:
            raise ValueError("native flow state measurements are missing")
        for key in ("max_u_v_w", "component_kinetic_energy", "tracer_adjacent_slice_rms"):
            values = metrics.get(key)
            if not isinstance(values, list) or len(values) != 3 or any(not finite(v) or v < 0 for v in values):
                raise ValueError("native flow vector measurements are invalid")
        for key in ("actual_divergence_max", "courant", "tracer_min", "tracer_max", "tracer_integral"):
            if not finite(metrics.get(key)) or metrics[key] < 0:
                raise ValueError("native flow scalar measurements are invalid")
        if metrics["actual_divergence_max"] > 1e-5 or metrics["courant"] > 1 or metrics["tracer_min"] > metrics["tracer_max"]:
            raise ValueError("native flow measurements violate fixed acceptance bounds")
        for key in ("tracer_centroid", "tracer_axis_stddev"):
            values = metrics.get(key)
            if values is not None and (not isinstance(values, list) or len(values) != 3 or any(not finite(v) or not 0 <= v <= 1 for v in values)):
                raise ValueError("native flow tracer geometry measurements are invalid")
        fraction = metrics.get("tracer_outside_source_fraction")
        change = metrics.get("tracer_integral_change_since_cutoff")
        if fraction is not None and (not finite(fraction) or not 0 <= fraction <= 1):
            raise ValueError("native flow tracer fraction is invalid")
        if (steps < 30 and change is not None) or (steps >= 30 and not finite(change)):
            raise ValueError("native flow tracer cutoff measurement is invalid")

    def accepted(frame, method, steps, generation, old_time, kind):
        if (frame.get("kind"), frame.get("status"), frame.get("code"), frame.get("detail")) != (kind, "accepted", "accepted", None):
            raise ValueError("native flow accepted frame status mismatch")
        report = frame.get("report")
        if not isinstance(report, dict) or not finite(report.get("dt")) or not 0 < report["dt"] <= FLOW_DT:
            raise ValueError("native flow accepted timestep is invalid")
        time = old_time + report["dt"]
        if report.get("time") != time or report.get("generation") != generation:
            raise ValueError("native flow report time/generation mismatch")
        for key in ("actual_divergence_max", "courant", "tracer_integral", "kinetic_energy"):
            if not finite(report.get(key)) or report[key] < 0:
                raise ValueError("native flow accepted report is invalid")
        if report["actual_divergence_max"] > 1e-5 or report["courant"] > 1:
            raise ValueError("native flow report violates fixed acceptance bounds")
        pressure = report.get("pressure")
        if not isinstance(pressure, dict) or type(pressure.get("iterations")) is not int or not 0 <= pressure["iterations"] <= 2000:
            raise ValueError("native flow pressure report is invalid")
        for key in ("true_residual_l2", "true_residual_max", "predicted_divergence_max"):
            if not finite(pressure.get(key)) or pressure[key] < 0:
                raise ValueError("native flow pressure residual is invalid")
        if pressure["predicted_divergence_max"] > 1e-7:
            raise ValueError("native flow predicted divergence exceeds the fixed bound")
        forces = report.get("forces")
        if not isinstance(forces, dict) or any(not finite(forces.get(key)) for key in ("kinetic_energy_before", "kinetic_energy_after", "applied_work")):
            raise ValueError("native flow force report is invalid")
        state(frame, method, steps, generation, time, report["dt"])
        return time

    for run in runs:
        method, frames = run["method"], run["frames"]
        baseline = frames[0]
        for frame, kind, generation in ((baseline, "baseline", 0), (run["reset"], "reset", FLOW_STEPS + 1)):
            if (frame.get("kind"), frame.get("status"), frame.get("code"), frame.get("detail"), frame.get("report")) != (kind, "snapshot", "snapshot", None, None):
                raise ValueError("native flow rest/reset frame status mismatch")
            state(frame, method, 0, generation, 0., None)
            if any(field_bytes(frame)):
                raise ValueError("native flow rest/reset fields are not bitwise zero")
        time = 0.
        for step, frame in enumerate(frames[1:], 1):
            time = accepted(frame, method, step, step, time, "accepted")
        for name, code, previous in (("cancelled_rest", "cancelled_before_commit", baseline), ("cancelled_nonzero", "cancelled_before_commit", frames[30]), ("limit_rejection", "step_limit", frames[-1])):
            frame = run[name]
            if (frame.get("kind"), frame.get("status"), frame.get("code"), frame.get("report")) != (name, "rejected", code, None) or not isinstance(frame.get("detail"), str) or not frame["detail"]:
                raise ValueError("native flow rejection status is invalid")
            if frame.get("state") != previous["state"] or field_bytes(frame) != field_bytes(previous):
                raise ValueError("native flow rejection does not preserve all accepted state bytes")
        retried = run["after_reset"]
        accepted(retried, method, 1, FLOW_STEPS + 2, 0., "after_reset")
        retry_state = {k: v for k, v in retried["state"].items() if k not in ("generation", "generation_u64")}
        first_state = {k: v for k, v in frames[1]["state"].items() if k not in ("generation", "generation_u64")}
        if retry_state != first_state or field_bytes(retried) != field_bytes(frames[1]):
            raise ValueError("native flow retry differs from the first accepted state")


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
    validate_flow_reference(_read_json(packet / "native-flow-reference.json"),
                            (packet / "native-flow-fields.bin").read_bytes())
    if receipt.get("flow_fixture") != FLOW_FIXTURE:
        raise ValueError("solver packet flow fixture mismatch")
    if type(receipt.get("contract_tests")) is not int or receipt["contract_tests"] <= 0:
        raise ValueError("solver packet has no completed contract tests")
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
    flow_native = receipt.get("native_flow_exporter")
    if flow_native != {"path": NATIVE_FLOW_EXPORTER, "sha256": digest(packet / NATIVE_FLOW_EXPORTER)}:
        raise ValueError("solver packet native flow exporter hash mismatch")
    reference = _read_json(packet / "native-reference.json")
    actual = json.loads(run([str(packet / NATIVE_EXPORTER)], repo))
    if actual != reference:
        raise ValueError("fresh native exporter differs from packet reference")
    with tempfile.TemporaryDirectory(prefix="rheon-flow-admission-") as temporary:
        fresh = Path(temporary) / "reference"
        run([str(packet / NATIVE_FLOW_EXPORTER), str(fresh)], repo)
        for name in ("native-flow-reference.json", "native-flow-fields.bin"):
            if (fresh / name).read_bytes() != (packet / name).read_bytes():
                raise ValueError(f"fresh native flow exporter differs from packet {name}")
    if receipt["source_sha256"] != source_hashes(repo):
        raise ValueError("solver sources changed during native admission")
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
    # Full flow contracts intentionally skip the expensive trajectory in debug;
    # release executes all three flow contracts along with the six cycle tests.
    test_output = run([cargo[0], "test", *cargo[1:], "--release"], repo, env=env)
    print(test_output, end="")
    test_count = passed_contract_tests(test_output)
    subprocess.run([cargo[0], "build", *cargo[1:], "--release", "--target", "wasm32-unknown-unknown", "--lib"], cwd=repo, env=env, check=True)
    subprocess.run([cargo[0], "build", *cargo[1:], "--release", "--bin", "rheon-solver-reference", "--bin", "flow-reference"], cwd=repo, env=env, check=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="rheon-solver-packet-", dir=output.parent) as tmp:
        stage = Path(tmp)
        subprocess.run([
            "wasm-bindgen", str(target_dir / "wasm32-unknown-unknown/release/rheon_solver.wasm"),
            "--target", "web", "--out-dir", str(stage), "--out-name", "rheon_solver",
        ], cwd=repo, env=env, check=True)
        (stage / "native").mkdir()
        shutil.copy2(target_dir / "release/rheon-solver-reference", stage / NATIVE_EXPORTER)
        shutil.copy2(target_dir / "release/flow-reference", stage / NATIVE_FLOW_EXPORTER)
        native_text = run([str(stage / NATIVE_EXPORTER)], repo)
        (stage / "native-reference.json").write_text(native_text)
        flow_output = stage / "flow-output"
        run([str(stage / NATIVE_FLOW_EXPORTER), str(flow_output)], repo)
        for name in ("native-flow-reference.json", "native-flow-fields.bin"):
            shutil.move(flow_output / name, stage / name)
        flow_output.rmdir()
        if source_hashes(repo) != initial_sources:
            raise ValueError("solver sources changed during build; rebuild from stable inputs")
        receipt = {
            "schema": 1,
            "fixture": FIXTURE,
            "flow_fixture": FLOW_FIXTURE,
            "wasm_bindgen_version": WASM_BINDGEN,
            "rustc": rust_version,
            "cargo": run(["cargo", "--version"], repo).strip(),
            "source_git_sha": run(["git", "rev-parse", "HEAD"], repo).strip(),
            "source_git_dirty": bool(run(["git", "status", "--porcelain"], repo).strip()),
            "source_sha256": initial_sources,
            "artifact_sha256": {name: digest(stage / name) for name in PUBLIC_ARTIFACTS},
            "native_exporter": {"path": NATIVE_EXPORTER, "sha256": digest(stage / NATIVE_EXPORTER)},
            "native_flow_exporter": {"path": NATIVE_FLOW_EXPORTER, "sha256": digest(stage / NATIVE_FLOW_EXPORTER)},
            "target": "wasm32-unknown-unknown",
            "profile": "release",
            "contract_tests": test_count,
            "limitations": [
                "The force cycle permits one accepted timestep per reset; the 24-cubed passive-tracer flow permits 80 fixed-input steps.",
                "Cancellation is a predetermined BeforeCommit callback, not asynchronous worker interruption.",
                "The native reference and WASM share source; independent four-edge equations are checked separately.",
                "The flow has a stationary fully filled box and passive tracer; it has no liquid free surface or impermeable internal obstacle.",
                "This build receipt does not claim browser qualification or Lean verification of the compiled timestep.",
            ],
        }
        (stage / "build-receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
        # These references were produced by the just-built host executables.
        # Admission at publish reruns both hosts; avoid another costly flow run
        # here while still checking every source/artifact and reference record.
        _validate_public(repo, stage)
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
