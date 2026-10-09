"""Synthetic import fixtures only: never launches physics or comparison cases."""
import copy
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import import_dense3d_sequence as consumer


def synthetic_frames():
    frames = []
    for step in range(9):
        fields = {}
        for name, shape in consumer.GEOMETRY["field_shapes"].items():
            fields[name] = [0.25 if step and name == "velocity_x" else 0.0] * math.prod(shape)
        profile = [0.5 * sum(math.comb(step, k) * (1 / 4)**k * (3 / 4)**(step-k)
                             for k in range(step+1) if 4 <= i-k < 8) for i in range(16)]
        fields["fraction"] = profile * 32
        if step == 1:
            fields["pressure"] = [-250.0 * (i % 16) for i in range(512)]
        diag = None
        if step:
            diag = {"pressure_iterations": 1 if step == 1 else 0,
                    "pressure_residual_m3_s2": 0.0,
                    "carrier_divergence_s_inv": 0.0, "liquid_divergence_s_inv": 0.0,
                    "volume_before_m3": 0.125, "volume_after_m3": 0.125, "mass_after_kg": 100.0,
                    "inward_m3": 0.0, "outward_m3": 0.0, "source_m3": 0.0, "balance_m3": 0.0,
                    "rounding_budget_m3": 64 * sys.float_info.epsilon * 0.25}
        frames.append({"frame": step, "time_s": step * 0.0625, "dt_s": 0.0625 if step else 0.0,
                       "carrier_stamp": {"id": "43", "version": str(step)},
                       "liquid_stamp": {"id": "41", "version": str(step)},
                       "fields": fields, "diagnostics": diag})
    return frames


def synthetic_manifest(data):
    paths = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "examples/dense3d_sequence.rs",
             "tools/import_dense3d_sequence.py", "tools/export_dense3d_sequence.py", "src/lib.rs",
             "src/dense3d_sequence.rs", "src/liquid_step.rs", "src/simulation.rs",
             "src/geometry.rs", "src/pressure.rs"]
    return {"schema": consumer.SCHEMA, "version": 1, "complete": True, "frame_count": 9,
            "frames_file": "frames.jsonl", "frames_sha256": hashlib.sha256(data).hexdigest(),
            "frames_bytes": len(data), "geometry": copy.deepcopy(consumer.GEOMETRY),
            "field_types": copy.deepcopy(consumer.FIELD_TYPES), "units": copy.deepcopy(consumer.UNITS),
            "pressure_semantics": consumer.PRESSURE_SEMANTICS, "config": copy.deepcopy(consumer.CONFIG),
            "limitations": copy.deepcopy(consumer.LIMITATIONS),
            "provenance": {"base_commit": consumer.BASE_COMMIT, "source_commit": "1" * 40,
                           "source_dirty": False, "source_sha256": {path: "2" * 64 for path in paths},
                           "executable_sha256": "3" * 64, "toolchain": {"rustc": "rustc synthetic", "cargo": "cargo synthetic"},
                           "build_command": ["cargo", "build", "--locked", "--no-default-features", "--example", "dense3d_sequence"],
                           "command": ["target/debug/examples/dense3d_sequence", "fresh-synthetic-directory"]}}


class ImportContract(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.frames = synthetic_frames()
        self.save()

    def save(self, data=None):
        if data is None:
            data = b"".join(json.dumps(frame, separators=(",", ":"), allow_nan=False).encode() + b"\n" for frame in self.frames)
        self.manifest = synthetic_manifest(data)
        (self.directory / "frames.jsonl").write_bytes(data)
        self.save_manifest()

    def save_manifest(self):
        (self.directory / "run.json").write_text(json.dumps(self.manifest), encoding="utf8")

    def reject(self, pattern=None):
        with self.assertRaisesRegex(ValueError, pattern or ".*"):
            consumer.verify(self.directory)

    def test_exact_synthetic_contract_and_prepublication_override(self):
        result = consumer.verify(self.directory)
        self.assertEqual(result["frames"], 9)
        self.assertEqual(result["volume_m3"], 0.125)
        self.assertEqual(result["mass_kg"], 100.0)
        self.assertEqual(result["centroid_x_m"], 0.5)
        self.assertEqual(result["maxima"]["binomial_fraction_error"], 0.0)
        (self.directory / "run.json").unlink()
        with self.assertRaises(FileNotFoundError):
            consumer.verify(self.directory)
        self.assertEqual(consumer.verify(self.directory, manifest=self.manifest), result)
        self.assertFalse((self.directory / "run.json").exists())

    def test_json_artifact_build_command_is_backward_compatible(self):
        self.manifest["provenance"]["build_command"].append("--message-format=json-render-diagnostics")
        self.save_manifest()
        self.assertEqual(consumer.verify(self.directory)["frames"], 9)
        self.manifest["provenance"]["build_command"].append("--release")
        self.save_manifest()
        self.reject("core-only build command")

    def test_frame_semantic_corruptions_rehash_before_validation(self):
        mutations = {
            "shape": lambda f: f[2]["fields"]["velocity_x"].pop(),
            "native_f32_overflow": lambda f: f[2]["fields"]["tracer"].__setitem__(0, 1e40),
            "native_f32_underflow": lambda f: f[2]["fields"]["tracer"].__setitem__(0, 1e-100),
            "field_boolean": lambda f: f[2]["fields"]["pressure"].__setitem__(0, True),
            "field_string": lambda f: f[2]["fields"]["pressure"].__setitem__(0, "0.0"),
            "stamp_gap": lambda f: f[3]["liquid_stamp"].__setitem__("version", "4"),
            "stamp_noncanonical": lambda f: f[3]["carrier_stamp"].__setitem__("version", "03"),
            "stamp_overflow": lambda f: f[3]["carrier_stamp"].__setitem__("version", "18446744073709551616"),
            "stamp_number": lambda f: f[3]["carrier_stamp"].__setitem__("id", 43),
            "wrong_id": lambda f: f[3]["liquid_stamp"].__setitem__("id", "42"),
            "clock": lambda f: f[4].__setitem__("time_s", 0.2),
            "dt": lambda f: f[4].__setitem__("dt_s", 0.125),
            "frame_boolean": lambda f: f[1].__setitem__("frame", True),
            "constructor_velocity": lambda f: f[0]["fields"]["velocity_x"].__setitem__(0, 0.25),
            "constructor_pressure": lambda f: f[0]["fields"]["pressure"].__setitem__(0, 1),
            "constructor_diagnostics": lambda f: f[0].__setitem__("diagnostics", {}),
            "donor": lambda f: f[4]["fields"]["fraction"].__setitem__(4, 0.0),
            "divergence": lambda f: f[4]["fields"]["velocity_x"].__setitem__(4, 0.24),
            "pressure_oracle": lambda f: f[1]["fields"]["pressure"].__setitem__(4, -999),
            "divergence_report": lambda f: f[4]["diagnostics"].__setitem__("carrier_divergence_s_inv", 1e-6),
            "balance_budget": lambda f: f[4]["diagnostics"].__setitem__("rounding_budget_m3", 1.0),
            "ledger_volume": lambda f: f[4]["diagnostics"].__setitem__("volume_after_m3", 0.1),
            "mass_units": lambda f: f[4]["diagnostics"].__setitem__("mass_after_kg", 125.0),
            "residual_units": lambda f: f[4]["diagnostics"].__setitem__("pressure_residual_m3_s2", 1.0),
            "iterations_boolean": lambda f: f[1]["diagnostics"].__setitem__("pressure_iterations", True),
            "negative_outflow": lambda f: f[4]["diagnostics"].__setitem__("outward_m3", -1.0),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                self.frames = synthetic_frames()
                mutate(self.frames)
                self.save()
                self.reject()

    def test_nonfinite_every_diagnostic_and_field(self):
        for name in consumer.DIAGNOSTICS - {"pressure_iterations"}:
            with self.subTest(diagnostic=name):
                self.frames = synthetic_frames()
                self.frames[2]["diagnostics"][name] = math.nan
                data = b"".join(json.dumps(frame).encode() + b"\n" for frame in self.frames)
                self.save(data)
                self.reject("nonfinite")
        for name in consumer.FIELD_TYPES:
            with self.subTest(field=name):
                self.frames = synthetic_frames()
                self.frames[2]["fields"][name][0] = math.inf
                data = b"".join(json.dumps(frame).encode() + b"\n" for frame in self.frames)
                self.save(data)
                self.reject("nonfinite")

    def test_f32_decimal_roundtrip_does_not_require_f64_equality(self):
        # Ten significant digits of a valid f32 need not equal its widened f64.
        self.frames[2]["fields"]["tracer"][0] = 0.1000000015
        self.save()
        self.assertEqual(consumer.verify(self.directory)["frames"], 9)

    def test_manifest_contract_corruptions(self):
        mutations = {
            "wrong_axis": lambda m: m["geometry"].__setitem__("axis_order", ["y", "x", "z"]),
            "wrong_offset": lambda m: m["geometry"]["face_offsets"].__setitem__("x", [0.5, 0, 0.5]),
            "wrong_shape": lambda m: m["geometry"]["field_shapes"].__setitem__("velocity_y", [9, 16, 4]),
            "wrong_unit": lambda m: m["units"].__setitem__("pressure_residual", "Pa"),
            "wrong_type": lambda m: m["field_types"].__setitem__("fraction", "f32"),
            "config": lambda m: m["config"].__setitem__("represented_density_kg_m3", 1000),
            "boolean_config": lambda m: m["config"].__setitem__("max_courant", True),
            "pressure_semantics": lambda m: m.__setitem__("pressure_semantics", "independently evolved endpoint"),
            "limitations": lambda m: m.__setitem__("limitations", []),
            "hash": lambda m: m.__setitem__("frames_sha256", "0" * 64),
            "hash_format": lambda m: m.__setitem__("frames_sha256", "A" * 64),
            "byte_count": lambda m: m.__setitem__("frames_bytes", 1),
            "complete": lambda m: m.__setitem__("complete", False),
            "complete_number": lambda m: m.__setitem__("complete", 1),
            "version_boolean": lambda m: m.__setitem__("version", True),
            "commit": lambda m: m["provenance"].__setitem__("source_commit", "main"),
            "dirty": lambda m: m["provenance"].__setitem__("source_dirty", "false"),
            "source_coverage": lambda m: m["provenance"]["source_sha256"].pop("Cargo.lock"),
            "source_path": lambda m: m["provenance"]["source_sha256"].__setitem__("../secret", "1" * 64),
            "executable_hash": lambda m: m["provenance"].__setitem__("executable_sha256", "short"),
            "toolchain": lambda m: m["provenance"]["toolchain"].__setitem__("cargo", ""),
            "build_command": lambda m: m["provenance"]["build_command"].remove("--no-default-features"),
            "command": lambda m: m["provenance"].__setitem__("command", []),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                self.save()
                mutate(self.manifest)
                self.save_manifest()
                self.reject()

    def test_strict_json_malformed_truncated_duplicate_nonfinite_overflow(self):
        original = (self.directory / "frames.jsonl").read_bytes()
        changes = {
            "malformed": original.replace(b'"frame":0', b'"frame":]', 1),
            "duplicate": original.replace(b'"frame":0', b'"frame":0,"frame":0', 1),
            "nan": original.replace(b'"time_s":0.0', b'"time_s":NaN', 1),
            "infinity": original.replace(b'"time_s":0.0', b'"time_s":Infinity', 1),
            "overflow": original.replace(b'"time_s":0.0', b'"time_s":1e9999', 1),
            "underflow": original.replace(b'"time_s":0.0', b'"time_s":1e-9999', 1),
            "integer_overflow": original.replace(b'"frame":0', b'"frame":18446744073709551616', 1),
            "truncated": original[:-10],
            "missing_newline": original[:-1],
            "extra_frame": original + original.splitlines()[0] + b"\n",
            "missing_frame": b"\n".join(original.split(b"\n")[1:]),
            "blank_line": original + b"\n",
            "invalid_utf8": original.replace(b'"time_s"', b'"time_\xff"', 1),
        }
        for name, data in changes.items():
            with self.subTest(name=name):
                self.save(data)
                self.reject()
        self.save(original)
        path = self.directory / "run.json"
        text = path.read_text()
        path.write_text(text.replace('"version": 1', '"version": 1, "version": 1', 1))
        self.reject("duplicate")

    def test_bounded_file_and_line_controls(self):
        (self.directory / "run.json").write_bytes(b" " * (consumer.MANIFEST_LIMIT + 1))
        self.reject("byte cap")
        self.save()
        data = (self.directory / "frames.jsonl").read_bytes()
        long_line = data.replace(b"\n", b" " * consumer.LINE_LIMIT + b"\n", 1)
        self.save(long_line)
        self.reject("line byte cap")
        self.save()
        (self.directory / "frames.jsonl").write_bytes(b" " * (consumer.FRAMES_LIMIT + 1))
        self.reject("byte cap")

    def test_requirements_active_in_optimized_python(self):
        self.frames[3]["carrier_stamp"]["version"] = "5"
        self.save()
        program = "import sys;sys.path.insert(0,sys.argv[1]);import import_dense3d_sequence as m;m.verify(sys.argv[2])"
        result = subprocess.run([sys.executable, "-O", "-c", program,
                                 str(Path(__file__).resolve().parent), str(self.directory)],
                                capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("stamp continuity", result.stderr)


if __name__ == "__main__":
    unittest.main()
