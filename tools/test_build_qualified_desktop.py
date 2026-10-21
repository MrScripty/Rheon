"""Controlled-export contract tests; fake tools produce no native-build evidence.

Run normally and with python3 -O. Git operations touch temporary fixtures only.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import build_qualified_desktop as b


FAKE_TOOL = r'''import json
import os
from pathlib import Path
import sys

if "--version" in sys.argv:
    print("FAKE controlled toolchain contract fixture; NOT a native compiler")
    raise SystemExit(0)
root = Path.cwd()
target = Path(os.environ["CARGO_TARGET_DIR"])
output = target.parent
(output / "fake-invocation.json").write_text(json.dumps(dict(args=sys.argv[1:], env=dict(os.environ), cwd=str(root))))
behavior = (root / "fixture-behavior").read_text().strip()
if behavior == "missing-include":
    (root / "src" / "ignored.rs").read_bytes()
if behavior == "exit":
    raise SystemExit(7)
if behavior == "change-source":
    (root / "src" / "main.rs").write_text("changed by fake build")
if behavior == "change-material":
    (output / "build-material.json").write_text("{}")
if behavior == "bad-json":
    print("not JSON")
    raise SystemExit(0)
if behavior == "bad-message":
    print("[]")
    raise SystemExit(0)
if behavior == "missing-executable":
    print(json.dumps(dict(reason="compiler-artifact", manifest_path=str(root / "Cargo.toml"),
        target=dict(name="rheon-desktop", kind=["bin"]))))
    raise SystemExit(0)
for name in ("rheon-desktop", "rheon-ai-compare"):
    if behavior == "missing-binary" and name == "rheon-ai-compare":
        continue
    executable = target / "release" / name
    executable.parent.mkdir(exist_ok=True)
    if behavior == "symlink-binary":
        outside = output / (name + "-outside")
        outside.write_bytes(b"FAKE binary fixture")
        executable.symlink_to(outside)
    else:
        executable.write_bytes(b"FAKE binary fixture: " + name.encode())
    print(json.dumps(dict(reason="compiler-artifact", manifest_path=str(root / "Cargo.toml"),
        target=dict(name=name, kind=["bin"]), executable=str(executable))))
'''


class Fixture:
    def __init__(self, base, behavior="success"):
        self.base = base
        self.source = base / "repo"
        self.source.mkdir()
        self.tools = base / "trusted-tools"
        self.tools.mkdir()
        self.cargo_home = base / "cargo-home"
        self.cargo_home.mkdir()
        self.rustup_home = base / "rustup-home"
        self.rustup_home.mkdir()
        for name in ("cargo", "rustc"):
            path = self.tools / name
            path.write_text(f"#!{sys.executable}\n" + FAKE_TOOL)
            path.chmod(0o755)
        self.put("Cargo.toml", '[package]\nname="rheon"\nversion="0.0.0"\nedition="2024"\n'
                 '[[bin]]\nname="rheon-desktop"\npath="src/main.rs"\n'
                 '[[bin]]\nname="rheon-ai-compare"\npath="src/compare.rs"\n')
        self.put("Cargo.lock", 'version = 4\n')
        self.put("rust-toolchain.toml", '[toolchain]\nchannel="1.92.0"\n')
        self.put("src/main.rs", 'fn main() {} // $Format:%H$\n')
        self.put("src/compare.rs", 'fn main() {}\n')
        self.put("fixture-behavior", behavior)
        self.put(".gitignore", 'src/ignored.rs\nignored-build.rs\n')
        self.git("init", "--quiet")
        self.commit()

    def put(self, name, value):
        path = self.source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(value)

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.source), *args],
                                       stderr=subprocess.PIPE, env=b.git_environment()).decode().strip()

    def commit(self):
        self.git("add", "--all")
        self.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                 "commit", "--quiet", "-m", "controlled build fixture")
        self.head = self.git("rev-parse", "HEAD")

    def build(self, name="output", **changes):
        args = dict(cargo_home=self.cargo_home, rustup_home=self.rustup_home,
                    toolchain_path=str(self.tools), offline=True, timeout_seconds=30)
        args.update(changes)
        return b.build(self.source, self.head, self.base / name, **args)


class ControlledBuildTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="rheon-build-contract-")
        self.base = Path(self.temp.name)

    def tearDown(self):
        self.temp.cleanup()

    def test_exact_tracked_bytes_recipe_versions_and_binary_sidecar_are_bound(self):
        f = Fixture(self.base)
        f.put(".gitattributes", 'src/main.rs export-subst\nsrc/archive-hidden.rs export-ignore\n')
        f.put("src/archive-hidden.rs", "tracked despite archive attribute\n")
        f.put("script", "#!/bin/sh\nexit 0\n")
        (f.source / "script").chmod(0o755)
        f.commit()
        f.put("src/ignored.rs", "UNTRACKED INPUT MUST NOT APPEAR\n")
        sidecar = f.build()
        output = self.base / "output"
        material_path = output / b.MATERIAL_NAME
        material = json.loads(material_path.read_text())
        self.assertEqual(sidecar["material"]["sha256"], b.file_sha256(material_path))
        self.assertEqual(sidecar["source_commit"], f.head)
        self.assertEqual(sidecar["source_tree"], f.git("rev-parse", "HEAD^{tree}"))
        tracked = material["tracked_source_material"]
        self.assertNotIn("src/ignored.rs", tracked)
        self.assertFalse((output / "source/src/ignored.rs").exists())
        self.assertTrue((output / "source/src/archive-hidden.rs").is_file())
        self.assertIn("$Format:%H$", (output / "source/src/main.rs").read_text())
        self.assertEqual(tracked["script"]["git_mode"], "100755")
        for name, record in tracked.items():
            original = (f.source / name).read_bytes()
            self.assertEqual((output / "source" / name).read_bytes(), original)
            self.assertEqual(record["sha256"], hashlib.sha256(original).hexdigest())
        for name in b.BINARIES:
            record = sidecar["binaries"][name]
            binary = output / record["path"]
            self.assertEqual(record["sha256"], b.file_sha256(binary))
            self.assertEqual(record["bytes"], binary.stat().st_size)
        self.assertEqual(sidecar["recipe_sha256"], hashlib.sha256(b.encoded(material["recipe"])).hexdigest())
        self.assertIn("NOT a native compiler", material["trusted_toolchain"]["cargo"]["version_verbose"])
        self.assertEqual(material["recipe"]["features"], ["desktop"])
        self.assertEqual(material["recipe"]["profile"], "release")
        self.assertIn("--locked", material["recipe"]["command"])
        self.assertIn("--no-default-features", material["recipe"]["command"])
        self.assertIn("--offline", material["recipe"]["command"])
        self.assertFalse((output / "source/.git").exists())
        self.assertFalse((output / "build-failed.json").exists())

    def test_arbitrary_inherited_build_inputs_are_cleared_and_exact_stamps_injected(self):
        f = Fixture(self.base)
        hostile = {"RUSTFLAGS": "secret injected code", "CARGO_ENCODED_RUSTFLAGS": "other",
                   "RUSTC_WRAPPER": "/untracked/wrapper", "CARGO_BUILD_TARGET": "wrong-target",
                   "CARGO_TARGET_DIR": str(self.base / "old-target"), "CARGO_PROFILE_RELEASE_OPT_LEVEL": "0",
                   "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER": "wrong-linker",
                   "RHEON_SOURCE_REVISION": "forged", "RHEON_BUILD_MATERIAL_SHA256": "forged",
                   "UNRELATED_PRIVATE_VALUE": "never record this", "LD_PRELOAD": "missing-library"}
        with patch.dict(os.environ, hostile):
            sidecar = f.build()
        output = self.base / "output"
        invocation = json.loads((output / "fake-invocation.json").read_text())
        env = invocation["env"]
        for key in hostile:
            if key not in ("RHEON_SOURCE_REVISION", "RHEON_BUILD_MATERIAL_SHA256", "CARGO_TARGET_DIR"):
                self.assertNotIn(key, env)
        self.assertEqual(env["RHEON_SOURCE_REVISION"], f.head)
        self.assertEqual(env["RHEON_BUILD_MATERIAL_SHA256"], sidecar["material"]["sha256"])
        self.assertEqual(env["CARGO_TARGET_DIR"], str(output / "build"))
        self.assertEqual(env["CARGO_INCREMENTAL"], "0")
        self.assertEqual(env["CARGO_BUILD_JOBS"], "1")
        self.assertEqual(invocation["cwd"], str(output / "source"))
        self.assertEqual(env["HOME"], str(output / "home"))

    def test_fresh_output_locations_do_not_change_material_identity(self):
        f = Fixture(self.base)
        first = f.build("first")
        second = f.build("second", timeout_seconds=31)
        self.assertEqual(first["material"], second["material"])
        self.assertEqual((self.base / "first" / b.MATERIAL_NAME).read_bytes(),
                         (self.base / "second" / b.MATERIAL_NAME).read_bytes())
        self.assertNotEqual(first["invocation"], second["invocation"])
        material = (self.base / "first" / b.MATERIAL_NAME).read_text()
        self.assertNotIn(str(self.base / "first"), material)
        self.assertNotIn(str(self.base / "second"), material)
        self.assertNotIn(str(f.source), material)

    def test_missing_ignored_relative_include_fails_without_completed_sidecar(self):
        f = Fixture(self.base, "missing-include")
        f.put("src/main.rs", 'include!("ignored.rs");\n')
        f.commit()
        f.put("src/ignored.rs", "ignored source cannot be laundered\n")
        with self.assertRaises(subprocess.CalledProcessError):
            f.build()
        self.assertFalse((self.base / "output" / b.SIDECAR_NAME).exists())
        self.assertTrue((self.base / "output" / b.MATERIAL_NAME).is_file())
        self.assertEqual(json.loads((self.base / "output/build-failed.json").read_text())["status"], "failed")

    def test_dirty_head_wrong_commit_untracked_files_existing_output_and_inside_git_refuse(self):
        f = Fixture(self.base)
        f.put("src/main.rs", "dirty\n")
        with self.assertRaisesRegex(b.BuildError, "clean"):
            f.build()
        f.git("checkout", "--", "src/main.rs")
        f.put("untracked.rs", "not ignored\n")
        with self.assertRaisesRegex(b.BuildError, "clean"):
            f.build()
        (f.source / "untracked.rs").unlink()
        actual = f.head
        f.head = "0" * 40
        with self.assertRaisesRegex(b.BuildError, "HEAD differs"):
            f.build()
        f.head = actual[:7]
        with self.assertRaisesRegex(b.BuildError, "full SHA-1"):
            f.build()
        f.head = actual
        (self.base / "output").mkdir()
        with self.assertRaisesRegex(b.BuildError, "new directory"):
            f.build()
        with self.assertRaisesRegex(b.BuildError, "outside Git"):
            b.build(f.source, f.head, f.source / "nested-output", cargo_home=f.cargo_home,
                    rustup_home=f.rustup_home, toolchain_path=str(f.tools))

    def test_symlink_submodule_unsafe_names_and_export_budget_are_fail_closed(self):
        f = Fixture(self.base)
        (f.source / "alias").symlink_to("src/main.rs")
        f.commit()
        with self.assertRaisesRegex(b.BuildError, "no symlinks/submodules"):
            f.build("symlink")
        (f.source / "alias").unlink()
        f.commit()
        (f.source / "nested").mkdir()
        f.git("update-index", "--add", "--cacheinfo", "160000", f.head, "nested")
        f.git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
              "commit", "--quiet", "-m", "gitlink")
        f.head = f.git("rev-parse", "HEAD")
        with self.assertRaisesRegex(b.BuildError, "no symlinks/submodules"):
            f.build("submodule")
        for name in ("../escape", "/absolute", "C:drive", "a\\b", "a//b", "a/./b", ".git/config", "bad\nname"):
            with self.subTest(name=name), self.assertRaises(b.BuildError):
                b.safe_relative(name)
        separate = self.base / "bounded"
        separate.mkdir()
        g = Fixture(separate)
        for constant, limit in (("MAX_FILES", 1), ("MAX_FILE_BYTES", 1), ("MAX_SOURCE_BYTES", 1)):
            with self.subTest(constant=constant), patch.object(b, constant, limit), self.assertRaises(b.BuildError):
                g.build(constant)

    def test_cargo_configs_and_external_or_missing_declared_paths_refuse_before_build(self):
        f = Fixture(self.base)
        external_workspace = self.base / "Cargo.toml"
        external_workspace.write_text('[workspace]\nmembers=["output/source"]\n')
        with self.assertRaisesRegex(b.BuildError, "external Cargo workspace ancestor"):
            f.build("ancestor-workspace")
        external_workspace.unlink()
        for suffix in ("config", "config.toml"):
            config = f.cargo_home / suffix
            config.write_text("[build]\nrustflags=['untracked']\n")
            with self.assertRaisesRegex(b.BuildError, "Cargo configuration"):
                f.build(suffix)
            config.unlink()
        original = (f.source / "Cargo.toml").read_text()
        for number, path in enumerate(("../outside", str(self.base / "outside"), "missing")):
            f.put("Cargo.toml", original + f'\n[dependencies]\nexternal={{path={json.dumps(path)}}}\n')
            f.commit()
            with self.assertRaises(b.BuildError):
                f.build(f"path-{number}")
        f.put("Cargo.toml", original)
        f.put(".cargo/config.toml", "[build]\nrustflags=['tracked override']\n")
        f.commit()
        with self.assertRaisesRegex(b.BuildError, "Cargo configuration"):
            f.build("tracked-config")

    def test_failed_mutated_missing_malformed_or_symlink_artifacts_never_complete(self):
        f = Fixture(self.base)
        for behavior in ("exit", "change-source", "change-material", "bad-json", "bad-message",
                         "missing-executable", "missing-binary", "symlink-binary"):
            f.put("fixture-behavior", behavior)
            f.commit()
            with self.subTest(behavior=behavior), self.assertRaises((b.BuildError, subprocess.CalledProcessError, json.JSONDecodeError)):
                f.build(behavior)
            output = self.base / behavior
            self.assertFalse((output / b.SIDECAR_NAME).exists())
            failure = json.loads((output / "build-failed.json").read_text())
            self.assertEqual(failure["status"], "failed")
            self.assertEqual(failure["source_commit"], f.head)


if __name__ == "__main__":
    unittest.main()
