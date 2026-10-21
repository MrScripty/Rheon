"""Bootstrap isolation contracts with an explicit fake distribution boundary.

These tests execute small fake installer/proxy subprocesses. They do not claim
to install or compile Rust; the real official-install smoke is separate.
"""
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import setup_ci_rust as ci


FAKE = r'''#!/usr/bin/python3
import json, os
from pathlib import Path
import sys
FAIL_STAGE = "none"
name = Path(sys.argv[0]).name
cargo = Path(os.environ["CARGO_HOME"])
rustup = Path(os.environ["RUSTUP_HOME"])
host = "x86_64-unknown-linux-gnu"
if name == "rustup-init":
    assert list(cargo.iterdir()) == [] and list(rustup.iterdir()) == []
    (cargo.parent / "bootstrap-observed.json").write_text(json.dumps({
        "argv": sys.argv[1:], "cargo": str(cargo), "rustup": str(rustup),
        "path": os.environ["PATH"],
        "selectors": {k:v for k,v in os.environ.items() if k.startswith(("RUST", "CARGO"))},
    }))
    if FAIL_STAGE == "install":
        sys.exit(17)
    (cargo / "bin").mkdir()
    for program in ["rustup", "rustc", "cargo", "rustfmt", "cargo-clippy"]:
        output = cargo / "bin" / program
        output.write_bytes(Path(__file__).read_bytes())
        output.chmod(0o700)
elif name == "rustup":
    if sys.argv[1:3] == ["toolchain", "install"]:
        (cargo.parent / "install-argv.json").write_text(json.dumps(sys.argv[1:]))
    elif sys.argv[1:3] == ["component", "list"]:
        for component in ["rustc", "cargo", "rust-std", "rustfmt", "clippy"]:
            if FAIL_STAGE != "component" or component != "clippy":
                print(component + "-" + host)
    else:
        sys.exit(19)
elif name == "rustc":
    print("rustc fixture\nrelease: " + ("1.91.0" if FAIL_STAGE == "version" else "1.92.0"))
elif name == "cargo":
    print("cargo 1.92.0 (explicit fake distribution fixture)")
else:
    print(name + " explicit fake distribution fixture")
'''


class SetupCiRustContract(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="rheon-ci-bootstrap-test-")
        self.root = Path(self.directory.name)
        self.repository = self.root / "repository"
        self.repository.mkdir()
        (self.repository / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "1.92.0"\ncomponents = ["rustfmt", "clippy"]\nprofile = "minimal"\n'
        )
        self.runner = self.root / "runner"
        self.runner.mkdir()
        self.env_file = self.root / "github-env"
        self.path_file = self.root / "github-path"
        self.env_file.write_text("previous=entry\n")
        self.path_file.write_text("/previous/path\n")

    def tearDown(self):
        self.directory.cleanup()

    def setup(self, stage="none", corrupt=False):
        payload = FAKE.replace('FAIL_STAGE = "none"', f'FAIL_STAGE = {stage!r}').encode()
        digest = hashlib.sha256(payload).hexdigest()
        with (
            patch.object(ci, "RUSTUP_INIT_SHA256", "0" * 64 if corrupt else digest),
            patch.object(ci.urllib.request, "urlopen", return_value=io.BytesIO(payload)) as request,
            patch.object(ci.platform, "system", return_value="Linux"),
            patch.object(ci.platform, "machine", return_value="x86_64"),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            result = ci.setup(self.repository, self.runner, self.env_file, self.path_file)
            request.assert_called_once_with(ci.RUSTUP_INIT_URL, timeout=60)
            return result

    def assert_unpublished(self):
        self.assertEqual(self.env_file.read_text(), "previous=entry\n")
        self.assertEqual(self.path_file.read_text(), "/previous/path\n")

    def test_fresh_homes_and_explicit_tools_ignore_polluted_runner_installation(self):
        polluted = self.root / "polluted"
        polluted.mkdir()
        (polluted / "cargo-clippy").write_text("runner partial-install sentinel")
        (polluted / "rustup").write_text("#!/bin/sh\nexit 77\n")
        (polluted / "rustup").chmod(0o700)
        with patch.dict(os.environ, {
            "CARGO_HOME": str(polluted), "RUSTUP_HOME": str(polluted),
            "RUSTUP_TOOLCHAIN": "unexpected", "RUSTUP_DIST_SERVER": "https://invalid.example",
            "RUSTFLAGS": "poisoned", "CARGO_ENCODED_RUSTFLAGS": "poisoned",
            "RUSTUP_INIT_SKIP_PATH_CHECK": "poisoned", "PATH": str(polluted),
        }):
            published = self.setup()
        cargo = Path(published["CARGO_HOME"])
        rustup = Path(published["RUSTUP_HOME"])
        self.assertEqual(cargo.parent, rustup.parent)
        self.assertEqual(cargo.parent.parent, self.runner)
        self.assertNotEqual(cargo, polluted)
        observed = json.loads((cargo.parent / "bootstrap-observed.json").read_text())
        self.assertEqual(observed["path"], "/usr/bin:/bin")
        self.assertEqual(observed["selectors"], published)
        self.assertEqual(observed["argv"], [
            "-y", "--no-modify-path", "--default-host", ci.HOST,
            "--default-toolchain", "none", "--profile", "minimal",
        ])
        install = json.loads((cargo.parent / "install-argv.json").read_text())
        self.assertEqual(install, [
            "toolchain", "install", "1.92.0-" + ci.HOST, "--profile", "minimal",
            "--component", "rustfmt", "--component", "clippy", "--no-self-update",
        ])
        self.assertEqual((polluted / "cargo-clippy").read_text(), "runner partial-install sentinel")
        self.assertEqual(published["RUSTUP_DIST_SERVER"], "https://static.rust-lang.org")
        self.assertEqual(published["RUSTUP_UPDATE_ROOT"], "https://static.rust-lang.org/rustup")
        for key, value in published.items():
            self.assertIn(f"{key}={value}\n", self.env_file.read_text())
        self.assertEqual(self.path_file.read_text(), f"/previous/path\n{cargo / 'bin'}\n")

    def test_each_setup_exclusively_creates_new_homes_without_reusing_previous_partial_state(self):
        first = self.setup()
        old = Path(first["RUSTUP_HOME"]) / "partial"
        old.write_text("do not repair or delete")
        second = self.setup()
        self.assertNotEqual(first["RUSTUP_HOME"], second["RUSTUP_HOME"])
        self.assertEqual(old.read_text(), "do not repair or delete")
        self.assertFalse((Path(second["RUSTUP_HOME"]) / "partial").exists())

    def test_checksum_mismatch_never_executes_or_exports(self):
        with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
            self.setup(corrupt=True)
        self.assert_unpublished()
        self.assertEqual(list(self.runner.glob("*/bootstrap-observed.json")), [])

    def test_download_size_is_bounded_before_writing_executable(self):
        target = self.root / "bootstrap"
        with (
            patch.object(ci, "MAX_BOOTSTRAP_BYTES", 4),
            patch.object(ci.urllib.request, "urlopen", return_value=io.BytesIO(b"12345")),
            self.assertRaisesRegex(ValueError, "download limit"),
        ):
            ci.download_bootstrap(target)
        self.assertFalse(target.exists())

    def test_failed_install_or_missing_components_or_wrong_version_never_export(self):
        for stage in ["install", "component", "version"]:
            with self.subTest(stage=stage), self.assertRaises((ValueError, ci.subprocess.CalledProcessError)):
                self.setup(stage)
            self.assert_unpublished()

    def test_unpinned_or_changed_component_contract_refuses_before_download(self):
        for fragment in [
            'channel = "stable"\ncomponents = ["rustfmt", "clippy"]\nprofile = "minimal"',
            'channel = "1.92.0"\ncomponents = ["rustfmt"]\nprofile = "minimal"',
        ]:
            (self.repository / "rust-toolchain.toml").write_text("[toolchain]\n" + fragment)
            with self.assertRaisesRegex(ValueError, "pinned minimal"), patch.object(ci, "download_bootstrap") as download:
                ci.setup(self.repository, self.runner, self.env_file, self.path_file)
            download.assert_not_called()
            self.assert_unpublished()
        self.assertEqual(list(self.runner.iterdir()), [])

    def test_repaired_workflows_use_shared_setup_and_retain_required_qualification_commands(self):
        workflows = Path(__file__).resolve().parent.parent / ".github/workflows"
        rust = (workflows / "rust-rheon.yml").read_text()
        book = (workflows / "research-book-pages.yml").read_text()
        self.assertEqual(rust.count("run: python3 tools/setup_ci_rust.py"), 2)
        self.assertEqual(book.count("run: python3 tools/setup_ci_rust.py"), 1)
        executable = rust.split("\n  executable-contracts:\n", 1)[1].split("\n  core-and-executable:\n", 1)[0]
        self.assertLess(executable.index("name: Check isolated Rust bootstrap contracts"), executable.index("run: python3 tools/setup_ci_rust.py"))
        self.assertIn("python3 -m unittest discover -s tools -p test_setup_ci_rust.py", executable)
        self.assertIn("python3 -O -m unittest discover -s tools -p test_setup_ci_rust.py", executable)
        lessons, build = book.split("\n  build:\n", 1)
        build, _ = build.split("\n  deploy:\n", 1)
        self.assertNotIn("run: python3 tools/setup_ci_rust.py", lessons)
        self.assertLess(build.index("run: python3 tools/setup_ci_rust.py"), build.index("cargo +1.92.0 build"))
        for workflow in [rust, book]:
            self.assertNotIn("rustup toolchain install", workflow)
            self.assertEqual(workflow.count("'tools/setup_ci_rust.py'"), 2)
            self.assertEqual(workflow.count("'tools/test_setup_ci_rust.py'"), 2)
        self.assertIn("cargo clippy --locked ${{ matrix.flags }} --all-targets -- -D warnings", rust)
        self.assertIn("cargo test --locked ${{ matrix.flags }}", rust)
        self.assertIn("cargo fmt --all --check", rust)
        self.assertIn("rustup target add --toolchain 1.92.0 wasm32-unknown-unknown", book)
        self.assertIn("python3 tools/qualify_aligned_strain.py", book)
        self.assertIn("python3 docs/education/check_native_admission.py", book)


if __name__ == "__main__":
    unittest.main()
