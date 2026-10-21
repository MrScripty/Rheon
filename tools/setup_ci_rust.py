#!/usr/bin/env python3
"""Install the repository's pinned Rust into fresh GitHub-job-owned homes.

Do not repair, copy, or invoke the runner image's Rust installation. A failed
setup publishes no usable tool environment; its disposable directory remains
under RUNNER_TEMP for diagnosis and runner cleanup. This is Ubuntu x86-64 CI
setup, not a hermetic build or a reproducibility claim.
"""
import hashlib
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tempfile
import tomllib
import urllib.request


HOST = "x86_64-unknown-linux-gnu"
RUSTUP_VERSION = "1.28.2"
RUSTUP_INIT_URL = (
    f"https://static.rust-lang.org/rustup/archive/{RUSTUP_VERSION}/{HOST}/rustup-init"
)
# Official adjacent .sha256, verified 2026-10-16; pin bootstrap bytes as well as
# the repository's rust-toolchain.toml version/components.
RUSTUP_INIT_SHA256 = "20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c"
MAX_BOOTSTRAP_BYTES = 32 * 1024 * 1024
DIST_SERVER = "https://static.rust-lang.org"
UPDATE_ROOT = "https://static.rust-lang.org/rustup"


def download_bootstrap(destination):
    with urllib.request.urlopen(RUSTUP_INIT_URL, timeout=60) as response:
        data = response.read(MAX_BOOTSTRAP_BYTES + 1)
    if len(data) > MAX_BOOTSTRAP_BYTES:
        raise ValueError("rustup bootstrap exceeds download limit")
    if hashlib.sha256(data).hexdigest() != RUSTUP_INIT_SHA256:
        raise ValueError("rustup bootstrap SHA-256 mismatch")
    destination.write_bytes(data)
    destination.chmod(0o700)


def run(command, environment, capture=False):
    result = subprocess.run(
        [str(arg) for arg in command], env=environment, check=True,
        capture_output=capture, text=True, timeout=900,
    )
    if capture:
        print(result.stdout, end="", flush=True)
        return result.stdout
    return ""


def setup(repository, runner_temp, github_env, github_path):
    if platform.system() != "Linux" or platform.machine() != "x86_64":
        raise ValueError("CI Rust setup supports Linux x86-64 only")
    configuration = tomllib.loads((repository / "rust-toolchain.toml").read_text())
    toolchain = configuration["toolchain"]
    channel = toolchain["channel"]
    if (
        not isinstance(channel, str)
        or re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", channel) is None
        or toolchain.get("profile") != "minimal"
        or toolchain.get("components") != ["rustfmt", "clippy"]
        or toolchain.keys() != {"channel", "profile", "components"}
    ):
        raise ValueError("expected a pinned minimal Rust toolchain with rustfmt and clippy")
    if not runner_temp.is_absolute() or not runner_temp.is_dir():
        raise ValueError("RUNNER_TEMP must name an existing absolute directory")
    # mkdtemp exclusively creates this job's root. Never reuse a predictable
    # preexisting home or import runner toolchains/cache into it.
    root = Path(tempfile.mkdtemp(prefix="rheon-ci-rust-", dir=runner_temp))
    cargo_home = root / "cargo"
    rustup_home = root / "rustup"
    cargo_home.mkdir()
    rustup_home.mkdir()
    selected = f"{channel}-{HOST}"
    environment = {
        key: value for key, value in os.environ.items()
        if not key.startswith(("CARGO", "RUST"))
    }
    environment.update({
        "CARGO_HOME": str(cargo_home),
        "RUSTUP_HOME": str(rustup_home),
        "RUSTUP_TOOLCHAIN": selected,
        "RUSTUP_DIST_SERVER": DIST_SERVER,
        "RUSTUP_UPDATE_ROOT": UPDATE_ROOT,
        # Download/install must not discover preinstalled runner Rust proxies.
        "PATH": "/usr/bin:/bin",
    })
    bootstrap = root / "rustup-init"
    print(f"Install official rustup {RUSTUP_VERSION} into {root}", flush=True)
    download_bootstrap(bootstrap)
    run([
        bootstrap, "-y", "--no-modify-path", "--default-host", HOST,
        "--default-toolchain", "none", "--profile", "minimal",
    ], environment)
    rustup = cargo_home / "bin/rustup"
    run([
        rustup, "toolchain", "install", selected, "--profile", "minimal",
        "--component", "rustfmt", "--component", "clippy", "--no-self-update",
    ], environment)
    installed = set(run([
        rustup, "component", "list", "--installed", "--toolchain", selected,
    ], environment, capture=True).splitlines())
    required = {f"{component}-{HOST}" for component in ("rustc", "cargo", "rust-std", "rustfmt", "clippy")}
    if not required <= installed:
        raise ValueError("installed Rust toolchain lacks required components")
    # Use the newly installed proxies explicitly, including their +toolchain
    # support needed by existing book build commands.
    binaries = cargo_home / "bin"
    rustc = run([binaries / "rustc", "--version", "--verbose"], environment, capture=True)
    cargo = run([binaries / "cargo", "--version"], environment, capture=True)
    if f"release: {channel}" not in rustc.splitlines() or not cargo.startswith(f"cargo {channel} "):
        raise ValueError("installed Rust/Cargo version differs from repository pin")
    run([binaries / "rustfmt", "--version"], environment, capture=True)
    run([binaries / "cargo-clippy", "--version"], environment, capture=True)
    published = {key: environment[key] for key in (
        "CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "RUSTUP_DIST_SERVER", "RUSTUP_UPDATE_ROOT",
    )}
    # Export only after download, installation, versions, and components pass.
    with github_env.open("a", encoding="utf-8") as output:
        for key, value in published.items():
            if "\n" in value or "\r" in value:
                raise ValueError("invalid CI tool path")
            output.write(f"{key}={value}\n")
    with github_path.open("a", encoding="utf-8") as output:
        output.write(f"{binaries}\n")
    return published


def main():
    try:
        setup(
            Path(__file__).resolve().parent.parent,
            Path(os.environ["RUNNER_TEMP"]),
            Path(os.environ["GITHUB_ENV"]),
            Path(os.environ["GITHUB_PATH"]),
        )
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"CI Rust setup failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
