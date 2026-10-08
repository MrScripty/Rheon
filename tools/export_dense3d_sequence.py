"""Build/run exactly one bounded fixture; publish validated run.json last.

No external Python packages. Never retries physics or resumes a partial directory.
The Rust example emits no completion marker; a direct invocation is partial until
this orchestrator independently validates and publishes its completion manifest.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

import import_dense3d_sequence as contract

ROOT = Path(__file__).resolve().parents[1]
BASE_COMMIT = "9cd4587a54befa61bdfddc8e35014bd3c34f02fb"


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(65536), b""):
            result.update(block)
    return result.hexdigest()


def run(command, **kwargs):
    return subprocess.run(command, cwd=ROOT, check=True, text=True, capture_output=True,
                          timeout=120, **kwargs).stdout.strip()


def sources(producer_controls=False):
    paths = list((ROOT / "src").rglob("*.rs"))
    paths += [ROOT / p for p in ["examples/dense3d_sequence.rs", "tools/export_dense3d_sequence.py",
              "tools/import_dense3d_sequence.py", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"]]
    if producer_controls:
        paths.append(ROOT / "tools/import_dense3d_controls.py")
    return {str(p.relative_to(ROOT)): digest(p) for p in sorted(paths)}


def publish_manifest(directory, manifest):
    """No-overwrite atomic publication. A failure never retries the simulation."""
    pending = None
    published = False
    try:
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", prefix=".run-", suffix=".tmp",
                                         dir=directory, delete=False) as stream:
            pending = Path(stream.name)
            json.dump(manifest, stream, indent=2, allow_nan=False)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        # Both names are on the same filesystem. link fails if run.json exists.
        os.link(pending, directory / "run.json")
        published = True
        pending.unlink()
        pending = None
        descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    except BaseException:
        if published:
            (directory / "run.json").unlink(missing_ok=True)
        raise
    finally:
        if pending is not None:
            pending.unlink(missing_ok=True)


def write_controls(directory, manifest, author, source_note):
    """Finish actual owner-captured records; never manufacture interval controls.

    The intermediate array is written by the native sequence owner, not derived
    from old frame fields or the fixed global configuration. A failure leaves
    the directory incomplete and is never retried automatically.
    """
    import import_dense3d_controls as controls_contract

    interval_path = directory / "intervals.json"
    intervals = contract.parse(contract.bounded_read(interval_path, 65536))
    binding_keys = ("schema", "version", "geometry", "field_types", "units",
                    "pressure_semantics", "config", "limitations", "provenance")
    controls = {
        "schema": "rheon.dense3d.interval-controls", "version": 2,
        "run_binding": {key: manifest[key] for key in binding_keys},
        "frames_sha256": manifest["frames_sha256"],
        "axis_order": ["x", "y", "z"], "side_order": ["low", "high"],
        "speed_sign": "positive outward normal",
        "units": {"time": "s", "outward_speed": "m/s", "inlet_fraction": "dimensionless",
                  "source_rate": "m^3/s", "body_acceleration": "m/s^2"},
        "provenance": {"origin": "producer_emitted", "author": author, "source_note": source_note},
        "intervals": intervals,
    }
    data = (json.dumps(controls, separators=(",", ":"), ensure_ascii=False,
                       allow_nan=False) + "\n").encode("utf-8")
    if not 1 <= len(data) <= 65536:
        raise ValueError("controls file exceeds byte cap")
    # No-overwrite private file; it becomes complete only with run.json last.
    with (directory / "controls.json").open("xb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())
    manifest.update(controls_file="controls.json", controls_bytes=len(data),
                    controls_sha256=hashlib.sha256(data).hexdigest())
    summary = controls_contract.verify(directory, manifest=manifest)
    interval_path.unlink()
    # Make frames, controls and removal of private staging durable first.
    descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    return summary


def export(directory, producer_controls=False):
    if type(producer_controls) is not bool:
        raise ValueError("producer controls opt-in must be boolean")
    directory = Path(directory).absolute()
    if directory.exists():
        raise ValueError("fresh output directory required; no overwrite or resume")
    source_commit = run(["git", "rev-parse", "HEAD"])
    source_dirty = bool(run(["git", "status", "--porcelain", "--untracked-files=all"]))
    source_hashes = sources(True) if producer_controls else sources()
    toolchain = {"rustc": run(["rustc", "--version"]), "cargo": run(["cargo", "--version"])}
    build = ["cargo", "build", "--locked", "--no-default-features", "--example", "dense3d_sequence"]
    run(build)
    metadata = json.loads(run(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"]))
    executable = Path(metadata["target_directory"]) / "debug" / "examples" / "dense3d_sequence"
    executable_hash = digest(executable)
    command = [str(executable), str(directory)]
    if producer_controls:
        command.append("--producer-controls-v2")
    # Exactly one actual pilot. Any subprocess failure leaves frames partial.
    report = contract.parse(run(command))
    current_sources = sources(True) if producer_controls else sources()
    if source_hashes != current_sources or executable_hash != digest(executable):
        raise ValueError("source/executable changed during export; no completion")
    if source_commit != run(["git", "rev-parse", "HEAD"]):
        raise ValueError("source commit changed during export; no completion")
    frames = directory / "frames.jsonl"
    manifest = {
        "schema": "rheon.dense3d.accepted-sequence", "version": 2 if producer_controls else 1, "complete": True,
        "frame_count": 9, "frames_file": "frames.jsonl", "frames_sha256": digest(frames),
        "frames_bytes": frames.stat().st_size,
        "geometry": contract.GEOMETRY, "field_types": contract.FIELD_TYPES,
        "units": contract.UNITS, "config": contract.CONFIG,
        "pressure_semantics": contract.PRESSURE_SEMANTICS, "limitations": contract.LIMITATIONS,
        "provenance": {"base_commit": BASE_COMMIT, "source_commit": source_commit,
            "source_dirty": source_dirty, "source_sha256": source_hashes,
            "executable_sha256": executable_hash, "toolchain": toolchain, "command": command,
            "build_command": build},
    }
    if (report.get("frame_count") != 9 or report.get("time_s") != 0.5
        or report.get("carrier_version") != "8" or report.get("liquid_version") != "8"
        or report.get("frames_bytes") != frames.stat().st_size):
        raise ValueError("Rust export report disagrees with accepted sequence")
    if producer_controls:
        if type(report.get("controls_interval_count")) is not int or report["controls_interval_count"] != 8:
            raise ValueError("Rust export report disagrees with accepted control count")
        summary = write_controls(directory, manifest, "Rheon native accepted sequence owner",
            "Captured from actual accepted fixed-case calls; source, binary and accepted-state evidence require independent verification.")
        if source_hashes != sources(True) or executable_hash != digest(executable):
            raise ValueError("source/executable changed during controls publication; no completion")
        if source_commit != run(["git", "rev-parse", "HEAD"]):
            raise ValueError("source commit changed during controls publication; no completion")
        manifest_bytes = (json.dumps(manifest, indent=2, allow_nan=False) + "\n").encode("utf-8")
        if len(manifest_bytes) > contract.MANIFEST_LIMIT:
            raise ValueError("completion manifest exceeds byte cap")
    else:
        summary = contract.verify(directory, manifest=manifest)
    publish_manifest(directory, manifest)
    return {"directory": str(directory), "source_commit": source_commit,
            "source_dirty": source_dirty, "frames_sha256": manifest["frames_sha256"], **summary}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--producer-controls-v2", action="store_true",
                        help="explicit proposed-v2 producer controls; current pinned v1 readers refuse it")
    args = parser.parse_args()
    print(json.dumps(export(args.directory, producer_controls=args.producer_controls_v2), indent=2, allow_nan=False))
