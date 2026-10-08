"""Authored static publication fixtures; no solver or sequence pilot is launched."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import export_dense3d_sequence as exporter
from test_dense3d_sequence_import import synthetic_frames, synthetic_manifest


def authored_intervals(frames):
    """Test input, never evidence that native controls were captured or applied."""
    return [{
        "start_frame": k - 1, "end_frame": k,
        "start_time_s": frames[k - 1]["time_s"], "end_time_s": frames[k]["time_s"],
        "dt_s": frames[k]["dt_s"],
        "carrier_before": copy.deepcopy(frames[k - 1]["carrier_stamp"]),
        "carrier_after": copy.deepcopy(frames[k]["carrier_stamp"]),
        "liquid_before": copy.deepcopy(frames[k - 1]["liquid_stamp"]),
        "liquid_after": copy.deepcopy(frames[k]["liquid_stamp"]),
        "boundary_stamp": {"id": "47", "version": "0"},
        "inlet_stamp": {"id": "53", "version": "0"},
        "outward_speed_m_s": [[-0.25, 0.25], [0, 0], [0, 0]],
        "inlet_fraction": [[0, 0], [0, 0], [0, 0]],
        "source_mode": "none", "source_rate_m3_s": 0,
        "body_acceleration_m_s2": [0, 0, 0],
    } for k in range(1, 9)]


class ControlsPublication(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.frames = synthetic_frames()
        data = b"".join(json.dumps(frame, separators=(",", ":"), allow_nan=False).encode()
                        + b"\n" for frame in self.frames)
        (self.directory / "frames.jsonl").write_bytes(data)
        self.manifest = synthetic_manifest(data)
        self.manifest["version"] = 2
        self.manifest["provenance"]["source_sha256"]["tools/import_dense3d_controls.py"] = "4" * 64
        self.intervals = authored_intervals(self.frames)
        (self.directory / "intervals.json").write_text(json.dumps(self.intervals) + "\n")

    def write(self):
        return exporter.write_controls(self.directory, self.manifest,
            "Authored publication test", "Synthetic fixture; no simulation was run.")

    def test_controls_durable_and_verified_before_completion(self):
        result = self.write()
        self.assertFalse((self.directory / "run.json").exists())
        self.assertFalse((self.directory / "intervals.json").exists())
        data = (self.directory / "controls.json").read_bytes()
        self.assertEqual(self.manifest["controls_bytes"], len(data))
        self.assertEqual(self.manifest["controls_sha256"], hashlib.sha256(data).hexdigest())
        controls = json.loads(data)
        self.assertEqual(controls["intervals"], self.intervals)
        self.assertEqual(controls["run_binding"]["version"], 2)
        self.assertEqual(controls["run_binding"]["provenance"], self.manifest["provenance"])
        self.assertEqual(result["frames"], 9)
        exporter.publish_manifest(self.directory, self.manifest)
        import import_dense3d_controls as consumer
        self.assertEqual(consumer.verify(self.directory), result)
        import import_dense3d_sequence as old_reader
        with self.assertRaises(ValueError):
            old_reader.verify(self.directory)

    def test_missing_captured_records_cannot_be_fabricated_from_frames(self):
        (self.directory / "intervals.json").unlink()
        with self.assertRaises(FileNotFoundError):
            self.write()
        self.assertFalse((self.directory / "controls.json").exists())
        self.assertFalse((self.directory / "run.json").exists())

    def test_invalid_captured_association_cannot_complete(self):
        self.intervals[2]["carrier_before"]["version"] = "7"
        (self.directory / "intervals.json").write_text(json.dumps(self.intervals))
        with self.assertRaises(ValueError):
            self.write()
        self.assertFalse((self.directory / "run.json").exists())

    def test_controls_and_directory_sync_failure_leave_no_completion(self):
        for effect in [OSError("controls sync failure"), [None, OSError("directory sync failure")]]:
            with self.subTest(effect=repr(effect)), tempfile.TemporaryDirectory() as temporary:
                original = self.directory
                self.directory = Path(temporary)
                try:
                    for name in ("frames.jsonl", "intervals.json"):
                        (self.directory / name).write_bytes((original / name).read_bytes())
                    with mock.patch.object(exporter.os, "fsync", side_effect=effect):
                        with self.assertRaises(OSError):
                            self.write()
                    self.assertFalse((self.directory / "run.json").exists())
                finally:
                    self.directory = original

    def test_no_overwrite_of_existing_controls(self):
        (self.directory / "controls.json").write_bytes(b"retained")
        with self.assertRaises(FileExistsError):
            self.write()
        self.assertEqual((self.directory / "controls.json").read_bytes(), b"retained")
        self.assertFalse((self.directory / "run.json").exists())

    def test_invalid_provenance_refuses_completion(self):
        for author, note in [("", "note"), ("author", "\x00note"), ("a" * 1001, "note")]:
            with self.subTest(author=author[:20]), tempfile.TemporaryDirectory() as temporary:
                directory = Path(temporary)
                for name in ("frames.jsonl", "intervals.json"):
                    (directory / name).write_bytes((self.directory / name).read_bytes())
                with self.assertRaises(ValueError):
                    exporter.write_controls(directory, copy.deepcopy(self.manifest), author, note)
                self.assertFalse((directory / "run.json").exists())


if __name__ == "__main__":
    unittest.main()
