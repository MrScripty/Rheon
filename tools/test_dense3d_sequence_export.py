"""Publication failures: no completion for partial export; no overwrite/rerun."""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest import mock

import export_dense3d_sequence as exporter


class PublicationContract(unittest.TestCase):
    def test_atomic_complete_manifest_and_no_overwrite(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            exporter.publish_manifest(directory, {"complete": True})
            self.assertEqual(json.loads((directory / "run.json").read_text()), {"complete": True})
            original = (directory / "run.json").read_bytes()
            with self.assertRaises(FileExistsError):
                exporter.publish_manifest(directory, {"complete": False})
            self.assertEqual((directory / "run.json").read_bytes(), original)
            self.assertEqual(list(directory.iterdir()), [directory / "run.json"])

    def test_manifest_write_sync_link_and_directory_sync_failures(self):
        for fail_at in ["dump", "file_sync", "link", "directory_sync"]:
            with self.subTest(fail_at=fail_at), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                if fail_at == "dump":
                    patch = mock.patch.object(exporter.json, "dump", side_effect=OSError("write failure"))
                elif fail_at == "link":
                    patch = mock.patch.object(exporter.os, "link", side_effect=OSError("link failure"))
                else:
                    effect = OSError("sync failure") if fail_at == "file_sync" else [None, OSError("directory sync failure")]
                    patch = mock.patch.object(exporter.os, "fsync", side_effect=effect)
                with patch, self.assertRaises(OSError):
                    exporter.publish_manifest(directory, {"complete": True})
                self.assertFalse((directory / "run.json").exists())
                self.assertEqual(list(directory.iterdir()), [])

    def test_fresh_directory_preflight_before_build_or_physics(self):
        with tempfile.TemporaryDirectory() as temporary, mock.patch.object(exporter, "run") as run:
            with self.assertRaisesRegex(ValueError, "fresh"):
                exporter.export(Path(temporary))
            run.assert_not_called()

    def test_subprocess_failure_never_publishes_or_retries_physics(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "partial"
            invocations = []
            def fake_run(command):
                invocations.append(command)
                if command[:3] == ["git", "rev-parse", "HEAD"]:
                    return "a" * 40
                if command[:2] == ["git", "status"]:
                    return ""
                if command[0] in ["rustc", "cargo"] and command[1] == "--version":
                    return "test toolchain"
                directory.mkdir()
                (directory / "frames.jsonl").write_text('{"frame":0}\n{')
                raise OSError("physics child failed")
            with mock.patch.object(exporter, "run", side_effect=fake_run), \
                 mock.patch.object(exporter, "sources", return_value={"file": "b" * 64}), \
                 mock.patch.object(exporter, "digest", return_value="c" * 64), \
                 mock.patch.object(exporter, "build_executable", return_value=Path(temporary) / "dense3d_sequence"), \
                 mock.patch.object(exporter, "publish_manifest") as publish:
                with self.assertRaises(OSError):
                    exporter.export(directory)
                publish.assert_not_called()
            self.assertEqual(len([c for c in invocations if c[0].endswith("dense3d_sequence")]), 1)
            self.assertFalse((directory / "run.json").exists())


class BuildArtifactContract(unittest.TestCase):
    def fixture(self, executable="/configured-target/debug/examples/dense3d_sequence"):
        target = {"name": "dense3d_sequence", "kind": ["example"], "crate_types": ["bin"],
                  "src_path": str(exporter.ROOT / "examples" / "dense3d_sequence.rs")}
        package = {"name": "rheon", "id": "local-rheon", "manifest_path": str(exporter.ROOT / "Cargo.toml"),
                   "targets": [target]}
        metadata = {"packages": [package], "target_directory": "/stale-decoy"}
        artifact = {"reason": "compiler-artifact", "package_id": "local-rheon",
                    "manifest_path": package["manifest_path"], "target": target,
                    "profile": {"test": False, "opt_level": "1", "debug_assertions": False},
                    "features": [], "executable": executable, "fresh": True}
        return metadata, artifact

    def select(self, metadata, messages):
        output = "\n".join(json.dumps(message) for message in messages)
        with mock.patch.object(exporter, "run", side_effect=[json.dumps(metadata), output]) as run:
            result = exporter.build_executable()
            self.assertEqual(run.call_args_list[-1].args[0], exporter.BUILD_COMMAND)
            return result

    def test_reported_executable_with_configured_dev_profile_and_other_artifacts(self):
        metadata, artifact = self.fixture()
        dependency = copy.deepcopy(artifact)
        dependency["package_id"] = "dependency-rheon"
        other_example = copy.deepcopy(artifact)
        other_example["target"]["name"] = "other_example"
        result = self.select(metadata, [dependency, other_example, artifact,
                                       {"reason": "build-finished", "success": True}])
        self.assertEqual(result, Path(artifact["executable"]))
        self.assertNotIn("stale-decoy", str(result))

    def test_missing_ambiguous_wrong_identity_profile_features_or_path_fail_closed(self):
        mutations = {
            "package": lambda a: a.__setitem__("package_id", "foreign-rheon"),
            "manifest": lambda a: a.__setitem__("manifest_path", "/other/Cargo.toml"),
            "name": lambda a: a["target"].__setitem__("name", "other"),
            "kind": lambda a: a["target"].__setitem__("kind", ["bin"]),
            "crate_type": lambda a: a["target"].__setitem__("crate_types", ["lib"]),
            "source": lambda a: a["target"].__setitem__("src_path", "/other/example.rs"),
            "test_profile": lambda a: a["profile"].__setitem__("test", True),
            "missing_profile": lambda a: a.pop("profile"),
            "missing_profile_test": lambda a: a["profile"].pop("test"),
            "numeric_profile_test": lambda a: a["profile"].__setitem__("test", 0),
            "features": lambda a: a.__setitem__("features", ["png-export"]),
            "no_executable": lambda a: a.__setitem__("executable", None),
            "relative_executable": lambda a: a.__setitem__("executable", "debug/examples/dense3d_sequence"),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                metadata, artifact = self.fixture()
                artifact = copy.deepcopy(artifact)
                mutate(artifact)
                with self.assertRaises(ValueError):
                    self.select(metadata, [artifact, {"reason": "build-finished", "success": True}])
        metadata, artifact = self.fixture()
        for messages in [[], [artifact], [artifact, {"reason": "build-finished", "success": False}],
                         [artifact, artifact, {"reason": "build-finished", "success": True}]]:
            with self.subTest(messages=len(messages)), self.assertRaises(ValueError):
                self.select(metadata, messages)
        for change in ["duplicate_package", "missing_example"]:
            metadata, artifact = self.fixture()
            if change == "duplicate_package":
                metadata["packages"].append(copy.deepcopy(metadata["packages"][0]))
            else:
                metadata["packages"][0]["targets"] = []
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.select(metadata, [artifact, {"reason": "build-finished", "success": True}])

    @unittest.skipUnless(shutil.which("cargo") and shutil.which("rustc"), "Cargo/Rust toolchain unavailable")
    def test_real_cargo_default_env_target_and_external_config_ignore_stale_decoy(self):
        # Tiny executable fixture only; no simulation, exported frames or dataset.
        host = next(line.split(": ", 1)[1] for line in subprocess.check_output(
            ["rustc", "-vV"], text=True).splitlines() if line.startswith("host: "))
        for mode in ("default", "env_target", "external_config"):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as temporary:
                base = Path(temporary)
                project = base / "project"
                (project / "examples").mkdir(parents=True)
                (project / "Cargo.toml").write_text('[package]\nname="rheon"\nversion="0.1.0"\nedition="2024"\n')
                (project / "examples/dense3d_sequence.rs").write_text('fn main() { println!("current-artifact"); }\n')
                target_directory = base / "target"
                cargo_home = base / "external-cargo-home"
                cargo_home.mkdir()
                marker = base / "stale-decoy-ran"
                stale = target_directory / "debug/examples/dense3d_sequence"
                if mode != "default":
                    stale.parent.mkdir(parents=True)
                    stale.write_text('#!/bin/sh\necho stale-decoy\ntouch "' + str(marker) + '"\n')
                    stale.chmod(0o755)
                environment = os.environ.copy()
                environment.pop("CARGO_BUILD_TARGET", None)
                environment.pop("CARGO_TARGET_DIR", None)
                environment["CARGO_HOME"] = str(cargo_home)
                environment["CARGO_TARGET_DIR"] = str(target_directory)
                if mode == "env_target":
                    environment["CARGO_BUILD_TARGET"] = host
                if mode == "external_config":
                    (cargo_home / "config.toml").write_text('[build]\ntarget="' + host + '"\n')
                subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=project, env=environment,
                               check=True, capture_output=True, timeout=30)
                with mock.patch.object(exporter, "ROOT", project), mock.patch.dict(os.environ, environment, clear=True):
                    executable = exporter.build_executable()
                    result = exporter.run([str(executable)])
                expected = target_directory / host if mode != "default" else target_directory
                self.assertEqual(executable, expected / "debug/examples/dense3d_sequence")
                self.assertEqual(result, "current-artifact")
                self.assertFalse(marker.exists())
                if mode != "default":
                    self.assertIn("stale-decoy", stale.read_text())


if __name__ == "__main__":
    unittest.main()
