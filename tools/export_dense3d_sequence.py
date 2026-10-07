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
BUILD_COMMAND = ["cargo", "build", "--locked", "--no-default-features", "--example",
                 "dense3d_sequence", "--message-format=json-render-diagnostics"]


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(65536), b""):
            result.update(block)
    return result.hexdigest()


def run(command, **kwargs):
    return subprocess.run(command, cwd=ROOT, check=True, text=True, capture_output=True,
                          timeout=120, **kwargs).stdout.strip()


def sources():
    paths = list((ROOT / "src").rglob("*.rs"))
    paths += [ROOT / p for p in ["examples/dense3d_sequence.rs", "tools/export_dense3d_sequence.py",
              "tools/import_dense3d_sequence.py", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"]]
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


def build_executable():
    """Select only this build's reported example, including configured targets.

    The controlled build command selects dev, while Cargo profile.test excludes
    harnesses without rejecting valid dev-profile compiler-setting overrides.
    """
    metadata = contract.parse(run(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"]))
    manifest_path = str(ROOT / "Cargo.toml")
    packages = [package for package in metadata["packages"]
                if package.get("manifest_path") == manifest_path and package.get("name") == "rheon"]
    contract.require(len(packages) == 1, "exact local Rheon package required")
    package = packages[0]
    identity = {"name": "dense3d_sequence", "kind": ["example"], "crate_types": ["bin"],
                "src_path": str(ROOT / "examples" / "dense3d_sequence.rs")}
    def matches(target):
        return type(target) is dict and all(target.get(key) == value for key, value in identity.items())
    contract.require(sum(matches(target) for target in package["targets"]) == 1,
                     "exact dense3D example target required")
    messages = [contract.parse(line) for line in run(BUILD_COMMAND).splitlines()]
    contract.require(messages and messages[-1].get("reason") == "build-finished"
                     and messages[-1].get("success") is True, "successful Cargo build required")
    artifacts = [message for message in messages if message.get("reason") == "compiler-artifact"
                 and message.get("package_id") == package["id"]
                 and message.get("manifest_path") == manifest_path and matches(message.get("target"))]
    contract.require(len(artifacts) == 1, "one unambiguous dense3D compiler artifact required")
    artifact = artifacts[0]
    contract.require(type(artifact.get("profile")) is dict and artifact["profile"].get("test") is False
                     and artifact.get("features") == [], "core-only non-test compiler artifact required")
    executable = artifact.get("executable")
    contract.require(type(executable) is str and executable and Path(executable).is_absolute(),
                     "absolute Cargo executable path required")
    return Path(executable)


def export(directory):
    directory = Path(directory).absolute()
    if directory.exists():
        raise ValueError("fresh output directory required; no overwrite or resume")
    source_commit = run(["git", "rev-parse", "HEAD"])
    source_dirty = bool(run(["git", "status", "--porcelain", "--untracked-files=all"]))
    source_hashes = sources()
    toolchain = {"rustc": run(["rustc", "--version"]), "cargo": run(["cargo", "--version"])}
    executable = build_executable()
    executable_hash = digest(executable)
    command = [str(executable), str(directory)]
    # Exactly one actual pilot. Any subprocess failure leaves frames partial.
    report = contract.parse(run(command))
    if source_hashes != sources() or executable_hash != digest(executable):
        raise ValueError("source/executable changed during export; no completion")
    if source_commit != run(["git", "rev-parse", "HEAD"]):
        raise ValueError("source commit changed during export; no completion")
    frames = directory / "frames.jsonl"
    manifest = {
        "schema": "rheon.dense3d.accepted-sequence", "version": 1, "complete": True,
        "frame_count": 9, "frames_file": "frames.jsonl", "frames_sha256": digest(frames),
        "frames_bytes": frames.stat().st_size,
        "geometry": contract.GEOMETRY, "field_types": contract.FIELD_TYPES,
        "units": contract.UNITS, "config": contract.CONFIG,
        "pressure_semantics": contract.PRESSURE_SEMANTICS, "limitations": contract.LIMITATIONS,
        "provenance": {"base_commit": BASE_COMMIT, "source_commit": source_commit,
            "source_dirty": source_dirty, "source_sha256": source_hashes,
            "executable_sha256": executable_hash, "toolchain": toolchain, "command": command,
            "build_command": BUILD_COMMAND},
    }
    summary = contract.verify(directory, manifest=manifest)
    if (report.get("frame_count") != 9 or report.get("time_s") != 0.5
        or report.get("carrier_version") != "8" or report.get("liquid_version") != "8"
        or report.get("frames_bytes") != frames.stat().st_size):
        raise ValueError("Rust export report disagrees with accepted sequence")
    publish_manifest(directory, manifest)
    return {"directory": str(directory), "source_commit": source_commit,
            "source_dirty": source_dirty, "frames_sha256": manifest["frames_sha256"], **summary}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    print(json.dumps(export(args.directory), indent=2, allow_nan=False))
