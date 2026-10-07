"""Publication failures: no completion for partial export; no overwrite/rerun."""
import json
from pathlib import Path
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
                if command[:2] == ["cargo", "build"]:
                    return ""
                if command[:2] == ["cargo", "metadata"]:
                    return json.dumps({"target_directory": temporary})
                directory.mkdir()
                (directory / "frames.jsonl").write_text('{"frame":0}\n{')
                raise OSError("physics child failed")
            with mock.patch.object(exporter, "run", side_effect=fake_run), \
                 mock.patch.object(exporter, "sources", return_value={"file": "b" * 64}), \
                 mock.patch.object(exporter, "digest", return_value="c" * 64), \
                 mock.patch.object(exporter, "publish_manifest") as publish:
                with self.assertRaises(OSError):
                    exporter.export(directory)
                publish.assert_not_called()
            self.assertEqual(len([c for c in invocations if c[0].endswith("dense3d_sequence")]), 1)
            self.assertFalse((directory / "run.json").exists())


if __name__ == "__main__":
    unittest.main()
