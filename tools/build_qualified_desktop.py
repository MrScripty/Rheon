#!/usr/bin/env python3
"""Build both desktop binaries from clean, pinned, tracked-only source material.

This is a local custody record, not a hermetic build, signature, reproducibility
claim, model qualification, or proof of every file read by committed build code.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath, PureWindowsPath
import shutil
import stat
import subprocess
import tomllib

VERSION = 1
TOOLCHAIN = "1.92.0"
BINARIES = ("rheon-desktop", "rheon-ai-compare")
MAX_FILES = 20_000
MAX_FILE_BYTES = 64 * 1024 * 1024
MAX_SOURCE_BYTES = 1024 * 1024 * 1024
MAX_MANIFEST_BYTES = 16 * 1024 * 1024
MAX_MESSAGES_BYTES = 64 * 1024 * 1024
MATERIAL_NAME = "build-material.json"
SIDECAR_NAME = "build-sidecar.json"
DIGEST_PLACEHOLDER = "sha256(exact build-material.json bytes)"
OUTPUT_PLACEHOLDER = "<fresh-output>"


class BuildError(ValueError):
    """The selected controlled build contract could not be established."""


def require(condition, diagnostic):
    if not condition:
        raise BuildError(diagnostic)


def encoded(value):
    return (json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + "\n").encode()


def file_sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(65536), b""):
            digest.update(block)
    return digest.hexdigest()


def write_new(path, value):
    raw = encoded(value)
    require(len(raw) <= MAX_MANIFEST_BYTES, "manifest byte limit")
    with path.open("xb") as stream:
        stream.write(raw)


def capture(command, *, env, cwd=None, limit=65536):
    result = subprocess.run(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, check=True, timeout=60)
    require(len(result.stdout) <= limit and len(result.stderr) <= limit,
            "command metadata byte limit")
    return result.stdout


def git_environment():
    # Neither inherited GIT_DIR nor global fsmonitor/config is build authority.
    return {"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C",
            "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_NO_REPLACE_OBJECTS": "1", "GIT_OPTIONAL_LOCKS": "0"}


def git(repo, *args, limit=65536):
    return capture(["git", "-c", "core.fsmonitor=false", "-C", str(repo), *args],
                   env=git_environment(), limit=limit)


def inside_checkout(path):
    # Empty sandbox .git markers are not real checkouts.
    for parent in (path, *path.parents):
        if not (parent / ".git").exists():
            continue
        try:
            top = git(parent, "rev-parse", "--show-toplevel").decode().strip()
        except subprocess.CalledProcessError:
            continue
        if Path(top).resolve() == parent:
            return True
    return False


def safe_relative(name):
    require(isinstance(name, str) and 0 < len(name.encode()) <= 4096
            and all(c.isprintable() for c in name), "unsafe tracked path")
    parts = name.split("/")
    require(not PurePosixPath(name).is_absolute() and not PureWindowsPath(name).drive
            and "\\" not in name and all(p not in ("", ".", "..") for p in parts)
            and all(p.lower() != ".git" for p in parts), "unsafe tracked path")
    return Path(*parts)


def clean_head(source, commit):
    require(len(commit) == 40 and all(c in "0123456789abcdef" for c in commit),
            "full SHA-1 source commit required")
    require(git(source, "rev-parse", "--show-toplevel").decode().strip() == str(source),
            "source must be checkout root")
    require(git(source, "rev-parse", "HEAD").decode().strip() == commit,
            "source HEAD differs from requested commit")
    require(not git(source, "status", "--porcelain=v1", "-z", "--untracked-files=all",
                    "--ignore-submodules=none", limit=MAX_MANIFEST_BYTES),
            "source must have clean committed HEAD")


def export_tracked(source, commit, destination):
    # No tar extraction or export-ignore/export-subst interpretation: each file
    # comes from its exact committed blob, with only regular Git modes admitted.
    entries = git(source, "ls-tree", "-r", "-z", "--long", commit,
                  limit=MAX_MANIFEST_BYTES).split(b"\0")
    require(entries[-1] == b"", "invalid Git tree framing")
    require(0 < len(entries) - 1 <= MAX_FILES, "tracked file count limit")
    inventory, total = {}, 0
    destination.mkdir()
    for entry in entries[:-1]:
        metadata, raw_name = entry.split(b"\t", 1)
        mode, kind, blob, size = metadata.split()
        require(mode in (b"100644", b"100755") and kind == b"blob",
                "only tracked regular files are supported; no symlinks/submodules")
        name = raw_name.decode("utf-8", errors="strict")
        relative = safe_relative(name)
        length = int(size)
        total += length
        require(length <= MAX_FILE_BYTES and total <= MAX_SOURCE_BYTES,
                "tracked source byte limit")
        raw = git(source, "cat-file", "blob", blob.decode("ascii"), limit=MAX_FILE_BYTES)
        require(len(raw) == length, "Git blob size changed")
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as stream:
            stream.write(raw)
        target.chmod(0o755 if mode == b"100755" else 0o644)
        inventory[name] = {"git_blob": blob.decode("ascii"), "git_mode": mode.decode("ascii"),
                           "bytes": length, "sha256": hashlib.sha256(raw).hexdigest()}
    return inventory


def no_cargo_config(source_export, cargo_home):
    # Cargo can discover a workspace above the exported package even without a
    # path key in the tracked manifest. Such an ancestor is outside this recipe.
    require(not any(os.path.lexists(p / "Cargo.toml") for p in source_export.parents),
            "external Cargo workspace ancestor is unsupported")
    paths = [p / ".cargo" / n for p in (source_export, *source_export.parents)
             for n in ("config", "config.toml")]
    paths += [cargo_home / n for n in ("config", "config.toml")]
    paths += [p for p in source_export.rglob("*")
              if p.parent.name == ".cargo" and p.name in ("config", "config.toml")]
    require(not any(os.path.lexists(p) for p in paths), "Cargo configuration is unsupported")


def validate_cargo_paths(source_export, inventory):
    def contained(base, raw):
        require(isinstance(raw, str) and not Path(raw).is_absolute()
                and not PureWindowsPath(raw).drive and "\\" not in raw,
                "external Cargo path input is unsupported")
        resolved = (base / raw).resolve()
        require(resolved == source_export or source_export in resolved.parents,
                "external Cargo path input is unsupported")
        require(resolved.exists(), "Cargo path input missing from tracked export")

    def walk(base, value):
        if isinstance(value, dict):
            for key, child in value.items():
                if key == "path":
                    contained(base, child)
                else:
                    walk(base, child)
        elif isinstance(value, list):
            for child in value:
                walk(base, child)

    for name in inventory:
        if PurePosixPath(name).name != "Cargo.toml":
            continue
        manifest = source_export / name
        data = tomllib.loads(manifest.read_text())
        walk(manifest.parent, data)
        package = data.get("package", {})
        workspace = data.get("workspace", {})
        require(isinstance(package, dict) and isinstance(workspace, dict),
                "invalid Cargo package/workspace declaration")
        for key in ("build", "workspace"):
            if isinstance(package.get(key), str):
                contained(manifest.parent, package[key])
        for key in ("members", "default-members", "exclude"):
            members = workspace.get(key, [])
            require(isinstance(members, list), "invalid Cargo workspace path list")
            for member in members:
                # Globs can select only within the export; Cargo owns expansion.
                require(isinstance(member, str), "invalid Cargo workspace path")
                prefix = member.split("*", 1)[0].split("?", 1)[0].split("[", 1)[0]
                contained(manifest.parent, prefix or ".")


def validate_export(source_export, inventory):
    actual = set()
    for path in source_export.rglob("*"):
        require(not path.is_symlink(), "export acquired a symlink")
        if path.is_dir():
            continue
        require(stat.S_ISREG(path.stat().st_mode), "export acquired nonregular material")
        name = path.relative_to(source_export).as_posix()
        actual.add(name)
        require(name in inventory and path.stat().st_size == inventory[name]["bytes"]
                and file_sha256(path) == inventory[name]["sha256"]
                and bool(path.stat().st_mode & 0o111) == (inventory[name]["git_mode"] == "100755"),
                "tracked export changed during build")
    require(actual == set(inventory), "tracked export changed during build")


def trusted_directory(path):
    require(Path(path).is_absolute(), "absolute toolchain directory required")
    path = Path(path).resolve(strict=True)
    require(path.is_dir() and not inside_checkout(path), "toolchain directory must be outside Git")
    return path


def binary_inventory(messages, source_export, target):
    result = {}
    require(messages.stat().st_size <= MAX_MESSAGES_BYTES, "Cargo message byte limit")
    with messages.open() as stream:
        for line in stream:
            message = json.loads(line)
            require(isinstance(message, dict), "invalid Cargo message object")
            if message.get("reason") != "compiler-artifact":
                continue
            artifact = message.get("target")
            require(isinstance(artifact, dict), "invalid Cargo artifact target")
            name = artifact.get("name")
            if name not in BINARIES or artifact.get("kind") != ["bin"]:
                continue
            require(message.get("manifest_path") == str(source_export / "Cargo.toml")
                    and name not in result, "ambiguous desktop Cargo artifact")
            require(isinstance(message.get("executable"), str), "missing desktop executable")
            path = Path(message["executable"])
            require(path.is_absolute() and target in path.parents and ".." not in path.parts
                    and not path.is_symlink(), "invalid desktop executable")
            require(not target.is_symlink() and path.is_file() and path.stat().st_size > 0,
                    "desktop executable outside fresh target")
            for ancestor in path.parents:
                if ancestor == target:
                    break
                require(not ancestor.is_symlink(), "symlink desktop executable ancestor")
            path = path.resolve(strict=True)
            require(target in path.parents, "desktop executable outside fresh target")
            result[name] = {"path": path.relative_to(target.parent).as_posix(),
                            "bytes": path.stat().st_size, "sha256": file_sha256(path)}
    require(set(result) == set(BINARIES), "both exact desktop executables required")
    return result


def build(source, commit, output, *, cargo_home, rustup_home, toolchain_path,
          offline=False, timeout_seconds=1800):
    """Build trusted committed code; output never reuses an existing directory."""
    source = Path(source).resolve(strict=True)
    clean_head(source, commit)
    require(not os.path.lexists(output), "output must be a new directory")
    output = Path(output).resolve()
    require(output.parent.is_dir() and not inside_checkout(output), "output must be outside Git")
    cargo_home, rustup_home = trusted_directory(cargo_home), trusted_directory(rustup_home)
    path_dirs = [trusted_directory(p) for p in toolchain_path.split(os.pathsep) if p]
    require(path_dirs and len(path_dirs) == len(toolchain_path.split(os.pathsep)),
            "explicit nonempty toolchain PATH required")
    selected_path = os.pathsep.join(map(str, path_dirs))
    require(isinstance(timeout_seconds, int) and 1 <= timeout_seconds <= 86400, "build timeout limit")
    cargo, rustc = (shutil.which(name, path=selected_path) for name in ("cargo", "rustc"))
    require(cargo is not None and rustc is not None, "declared PATH needs cargo and rustc")
    output.mkdir()
    try:
        source_export, target = output / "source", output / "build"
        inventory = export_tracked(source, commit, source_export)
        require("Cargo.lock" in inventory and "rust-toolchain.toml" in inventory
                and "Cargo.toml" in inventory, "tracked Cargo/lock/toolchain inputs required")
        toolchain = tomllib.loads((source_export / "rust-toolchain.toml").read_text())
        require(toolchain.get("toolchain", {}).get("channel") == TOOLCHAIN,
                "controlled desktop toolchain differs")
        no_cargo_config(source_export, cargo_home)
        validate_cargo_paths(source_export, inventory)
        for directory in (target, output / "home", output / "tmp"):
            directory.mkdir()
        env = {"PATH": selected_path, "CARGO_HOME": str(cargo_home), "RUSTUP_HOME": str(rustup_home),
               "RUSTUP_TOOLCHAIN": TOOLCHAIN, "HOME": OUTPUT_PLACEHOLDER + "/home",
               "TMPDIR": OUTPUT_PLACEHOLDER + "/tmp", "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
               "CARGO_TARGET_DIR": OUTPUT_PLACEHOLDER + "/build", "CARGO_INCREMENTAL": "0", "CARGO_BUILD_JOBS": "1",
               "RHEON_SOURCE_REVISION": commit, "RHEON_BUILD_MATERIAL_SHA256": DIGEST_PLACEHOLDER}
        probe_env = dict(env, HOME=str(output / "home"), TMPDIR=str(output / "tmp"),
                         CARGO_TARGET_DIR=str(target))
        tools = {}
        for name, executable in (("cargo", cargo), ("rustc", rustc)):
            path = Path(executable).resolve(strict=True)
            require(path.is_file(), "trusted tool must resolve to a regular file")
            tools[name] = {"executable": str(path), "sha256": file_sha256(path),
                           "version_verbose": capture([executable, "+" + TOOLCHAIN, "--version", "--verbose"], env=probe_env).decode()}
        command = [cargo, "+" + TOOLCHAIN, "build", "--locked", "--release", "--no-default-features",
                   "--features", "desktop", "--bin", BINARIES[0], "--bin", BINARIES[1], "--message-format=json"]
        if offline:
            command.append("--offline")
        recipe = {"command": command, "profile": "release", "features": ["desktop"],
                  "default_features": False, "binaries": list(BINARIES), "environment": env}
        wrapper_hash = file_sha256(Path(__file__))
        material = {"schema_version": VERSION, "kind": "rheon-qualified-desktop-material-v1",
                    "source_commit": commit, "source_tree": git(source, "rev-parse", "HEAD^{tree}").decode().strip(),
                    "tracked_source_material": inventory, "recipe": recipe, "trusted_toolchain": tools,
                    "builder_sha256": wrapper_hash,
                    "provenance_limit": "tracked source material and declared trusted tools only; external reads by committed code unsupported; not hermetic, authenticated, reproducible, model or hardware qualification"}
        material_path = output / MATERIAL_NAME
        write_new(material_path, material)
        material_hash = file_sha256(material_path)
        actual_env = dict(probe_env, RHEON_BUILD_MATERIAL_SHA256=material_hash)
        with (output / "cargo.jsonl").open("xb") as stdout, (output / "cargo.log").open("xb") as stderr:
            subprocess.run(command, cwd=source_export, env=actual_env, stdout=stdout,
                           stderr=stderr, check=True, timeout=timeout_seconds)
        clean_head(source, commit)
        validate_export(source_export, inventory)
        require(file_sha256(material_path) == material_hash
                and file_sha256(Path(__file__)) == wrapper_hash, "build material/controller changed")
        for tool in tools.values():
            require(file_sha256(Path(tool["executable"])) == tool["sha256"], "trusted tool changed during build")
        binaries = binary_inventory(output / "cargo.jsonl", source_export, target)
        sidecar = {"schema_version": VERSION, "kind": "rheon-qualified-desktop-build-v1", "status": "completed",
                   "source_commit": commit, "source_tree": material["source_tree"],
                   "material": {"path": MATERIAL_NAME, "sha256": material_hash},
                   "recipe_sha256": hashlib.sha256(encoded(recipe)).hexdigest(), "binaries": binaries,
                   "invocation": {"source_repository": str(source), "output": str(output),
                                  "source_export": str(source_export), "target": str(target),
                                  "environment": actual_env, "timeout_seconds": timeout_seconds},
                   "provenance_limit": material["provenance_limit"]}
        write_new(output / SIDECAR_NAME, sidecar)
        return sidecar
    except (OSError, ValueError, UnicodeError, subprocess.SubprocessError) as error:
        diagnostic = "".join(c for c in str(error) if c.isprintable())[:2000]
        write_new(output / "build-failed.json", {"schema_version": VERSION, "status": "failed",
                  "source_commit": commit, "diagnostic": diagnostic})
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-repo", type=Path, required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cargo-home", type=Path, required=True)
    parser.add_argument("--rustup-home", type=Path, required=True)
    parser.add_argument("--toolchain-path", required=True, help="explicit trusted PATH directory list")
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--timeout-seconds", type=int, default=1800)
    args = parser.parse_args()
    try:
        build(args.source_repo, args.source_commit, args.output, cargo_home=args.cargo_home,
              rustup_home=args.rustup_home, toolchain_path=args.toolchain_path,
              offline=args.offline, timeout_seconds=args.timeout_seconds)
        print(json.dumps({"status": "completed", "sidecar": str(args.output / SIDECAR_NAME),
                          "sidecar_sha256": file_sha256(args.output / SIDECAR_NAME)}))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        parser.exit(1, f"Controlled desktop build failed: {error}\n")


if __name__ == "__main__":
    main()
