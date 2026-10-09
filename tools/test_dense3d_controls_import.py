"""Authored synthetic metadata tests only; no producer or physics is launched."""
import copy
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import import_dense3d_controls as controls
import import_dense3d_sequence as v1
from test_dense3d_sequence_import import synthetic_frames, synthetic_manifest


def synthetic_v2_metadata(frames, frame_bytes):
    """Visibly authored test structure, not a captured producer/native run."""
    manifest = synthetic_manifest(frame_bytes)
    manifest["version"] = 2
    manifest["provenance"]["source_sha256"]["tools/import_dense3d_controls.py"] = "4" * 64
    intervals = []
    for index in range(1, 9):
        before, after = frames[index - 1], frames[index]
        intervals.append({"start_frame": index - 1, "end_frame": index,
            "start_time_s": before["time_s"], "end_time_s": after["time_s"], "dt_s": after["dt_s"],
            "carrier_before": copy.deepcopy(before["carrier_stamp"]), "carrier_after": copy.deepcopy(after["carrier_stamp"]),
            "liquid_before": copy.deepcopy(before["liquid_stamp"]), "liquid_after": copy.deepcopy(after["liquid_stamp"]),
            "boundary_stamp": {"id": "47", "version": "0"}, "inlet_stamp": {"id": "53", "version": "0"},
            "outward_speed_m_s": [[-.25, .25], [0.0, 0.0], [0.0, 0.0]],
            "inlet_fraction": [[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
            "source_mode": "none", controls.SOURCE_RATE_KEY: 0.0, "body_acceleration_m_s2": [0.0, 0.0, 0.0]})
    sidecar = {"schema": controls.CONTROLS_SCHEMA, "version": 2,
        "run_binding": {key: copy.deepcopy(manifest[key]) for key in controls.BINDING_KEYS},
        "frames_sha256": manifest["frames_sha256"], "axis_order": ["x", "y", "z"],
        "side_order": ["low", "high"], "speed_sign": "positive outward normal",
        "units": copy.deepcopy(controls.CONTROL_UNITS),
        # This required origin value exercises the proposed packet validator;
        # the author/source_note identify the entire fixture as synthetic.
        "provenance": {"origin": "producer_emitted", "author": "Authored synthetic unit-test fixture",
            "source_note": "Synthetic test data only; no native producer, accepted capture, or pilot was run."},
        "intervals": intervals}
    return manifest, sidecar


class ControlsImport(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.frames = synthetic_frames()
        self.frame_bytes = b"".join(json.dumps(f, separators=(",", ":"), allow_nan=False).encode() + b"\n" for f in self.frames)
        self.manifest, self.controls = synthetic_v2_metadata(self.frames, self.frame_bytes)
        (self.directory / "frames.jsonl").write_bytes(self.frame_bytes)
        self.save()

    def save(self, raw=None):
        if raw is None:
            raw = json.dumps(self.controls, separators=(",", ":"), allow_nan=False).encode()
        (self.directory / "controls.json").write_bytes(raw)
        self.manifest.update(controls_file="controls.json", controls_bytes=len(raw),
                             controls_sha256=hashlib.sha256(raw).hexdigest())
        self.save_manifest()

    def save_manifest(self):
        (self.directory / "run.json").write_text(json.dumps(self.manifest), encoding="utf8")

    def reject(self, pattern=None):
        with self.assertRaisesRegex(ValueError, pattern or ".*"):
            controls.verify(self.directory)

    def reset(self):
        self.manifest, self.controls = synthetic_v2_metadata(self.frames, self.frame_bytes)
        (self.directory / "frames.jsonl").write_bytes(self.frame_bytes)

    def test_authored_synthetic_contract_and_no_mutation(self):
        originals = {name: (self.directory / name).read_bytes() for name in ("run.json", "frames.jsonl", "controls.json")}
        original_controls = copy.deepcopy(self.controls)
        result = controls.verify(self.directory)
        self.assertEqual(result["version"], 2)
        self.assertEqual(result["frames"], 9)
        self.assertEqual(result["accepted_steps"], 8)
        self.assertEqual(result["control_intervals"], 8)
        self.assertEqual(result["volume_m3"], 0.125)
        self.assertEqual(result["controls_provenance"]["origin"], "producer_emitted")
        self.assertEqual(self.controls, original_controls)
        for name, raw in originals.items():
            self.assertEqual((self.directory / name).read_bytes(), raw)

    def test_explicit_prepublication_override_writes_nothing(self):
        before = (self.directory / "frames.jsonl").read_bytes()
        expected = controls.verify(self.directory)
        (self.directory / "run.json").unlink()
        with self.assertRaises(FileNotFoundError):
            controls.verify(self.directory)
        self.assertEqual(controls.verify(self.directory, manifest=self.manifest), expected)
        self.assertFalse((self.directory / "run.json").exists())
        self.assertEqual((self.directory / "frames.jsonl").read_bytes(), before)
        self.assertEqual(set(p.name for p in self.directory.iterdir()), {"frames.jsonl", "controls.json"})

    def test_v1_and_authored_preview_are_never_silently_upgraded(self):
        old = synthetic_manifest(self.frame_bytes)
        self.assertEqual(v1.verify(self.directory, manifest=old)["version"], 1)
        with self.assertRaisesRegex(ValueError, "v2 version"):
            controls.verify(self.directory, manifest=old)
        with self.assertRaisesRegex(ValueError, "manifest keys"):
            v1.verify(self.directory)
        for origin in ("developer_authored", "authored_v1_preview", "consumer_authored", None):
            with self.subTest(origin=origin):
                self.reset(); self.controls["provenance"]["origin"] = origin; self.save()
                self.reject("provenance.origin")
        self.reset(); self.controls["version"] = 1; self.save()
        self.reject("controls version")

    def test_exact_keys_all_objects(self):
        mutations = {
            "manifest extra": lambda m,c: m.__setitem__("family", "invented"),
            "controls extra": lambda m,c: c.__setitem__("source_family", "invented"),
            "controls missing": lambda m,c: c.pop("speed_sign"),
            "binding extra": lambda m,c: c["run_binding"].__setitem__("family", "invented"),
            "binding missing": lambda m,c: c["run_binding"].pop("provenance"),
            "units extra": lambda m,c: c["units"].__setitem__("velocity", "m/s"),
            "provenance extra": lambda m,c: c["provenance"].__setitem__("source_commit", "1" * 40),
            "interval extra": lambda m,c: c["intervals"][0].__setitem__("requested_dt_s", .0625),
            "interval missing": lambda m,c: c["intervals"][0].pop(controls.SOURCE_RATE_KEY),
            "stamp extra": lambda m,c: c["intervals"][0]["carrier_before"].__setitem__("kind", "carrier"),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                self.reset(); mutate(self.manifest,self.controls); self.save(); self.reject()

    def test_run_binding_every_member_and_numeric_type_identity(self):
        for key in controls.BINDING_KEYS:
            with self.subTest(binding=key):
                self.reset(); self.controls["run_binding"][key] = None; self.save(); self.reject("run_binding")
        mutations = {
            "integer density to float": lambda c:c["run_binding"]["config"].__setitem__("carrier_density_kg_m3",1000.0),
            "float spacing to integer": lambda c:c["run_binding"]["geometry"]["spacing_m"].__setitem__(0,0),
            "integer count to float": lambda c:c["run_binding"]["geometry"]["counts"].__setitem__(0,16.0),
            "float dt to boolean": lambda c:c["run_binding"]["config"].__setitem__("requested_dt_s",True),
            "integer zero to float": lambda c:c["run_binding"]["geometry"]["origin_m"].__setitem__(0,0.0),
            "boolean dirty to integer": lambda c:c["run_binding"]["provenance"].__setitem__("source_dirty",0),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                self.reset(); mutate(self.controls); self.save(); self.reject("type run_binding")
        # Existing v1 numeric semantics remain intact if both manifest and copy
        # use the same permitted numeric type; the new gate is identity of copies.
        self.reset(); self.manifest["config"]["carrier_density_kg_m3"] = 1000.0
        self.controls["run_binding"]["config"]["carrier_density_kg_m3"] = 1000.0
        self.save(); self.assertEqual(controls.verify(self.directory)["control_intervals"],8)

    def test_interval_roster_association_clocks_and_stamps(self):
        mutations = {
            "missing interval": lambda c:c["intervals"].pop(),
            "extra interval": lambda c:c["intervals"].append(copy.deepcopy(c["intervals"][0])),
            "reverse": lambda c:c["intervals"].reverse(),
            "constructor entry": lambda c:c["intervals"][0].__setitem__("end_frame",0),
            "gap": lambda c:c["intervals"][3].__setitem__("start_frame",2),
            "frame boolean": lambda c:c["intervals"][0].__setitem__("start_frame",False),
            "frame float": lambda c:c["intervals"][0].__setitem__("end_frame",1.0),
            "wrong start clock": lambda c:c["intervals"][0].__setitem__("start_time_s",.0625),
            "wrong end clock": lambda c:c["intervals"][0].__setitem__("end_time_s",.125),
            "wrong dt": lambda c:c["intervals"][0].__setitem__("dt_s",.125),
            "time type drift": lambda c:c["intervals"][0].__setitem__("start_time_s",0),
            "time boolean": lambda c:c["intervals"][0].__setitem__("start_time_s",False),
            "stamp wrong id": lambda c:c["intervals"][0]["carrier_before"].__setitem__("id","41"),
            "stamp gap": lambda c:c["intervals"][0]["liquid_after"].__setitem__("version","2"),
            "stamp leading zero": lambda c:c["intervals"][0]["carrier_before"].__setitem__("version","00"),
            "stamp number": lambda c:c["intervals"][0]["liquid_before"].__setitem__("id",41),
            "stamp overflow": lambda c:c["intervals"][0]["carrier_after"].__setitem__("version","18446744073709551616"),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                self.reset(); mutate(self.controls); self.save(); self.reject()

    def test_fixed_control_values_sign_units_and_explicit_absence(self):
        mutations = {
            "schema": lambda c:c.__setitem__("schema",v1.SCHEMA),
            "axis": lambda c:c.__setitem__("axis_order",["y","x","z"]),
            "side": lambda c:c.__setitem__("side_order",["high","low"]),
            "sign": lambda c:c.__setitem__("speed_sign","positive Cartesian component"),
            "unit": lambda c:c["units"].__setitem__("source_rate","m^3"),
            "boundary id": lambda c:c["intervals"][0]["boundary_stamp"].__setitem__("id","48"),
            "boundary version": lambda c:c["intervals"][0]["boundary_stamp"].__setitem__("version","1"),
            "inlet id": lambda c:c["intervals"][0]["inlet_stamp"].__setitem__("id","52"),
            "speed sign": lambda c:c["intervals"][0]["outward_speed_m_s"][0].__setitem__(0,.25),
            "speed nonzero": lambda c:c["intervals"][0]["outward_speed_m_s"][1].__setitem__(0,1e-300),
            "speed shape": lambda c:c["intervals"][0]["outward_speed_m_s"].pop(),
            "inlet": lambda c:c["intervals"][0]["inlet_fraction"][0].__setitem__(0,.5),
            "source zero Some": lambda c:c["intervals"][0].__setitem__("source_mode","uniform"),
            "source null": lambda c:c["intervals"][0].__setitem__("source_mode",None),
            "source absent null": lambda c:c["intervals"][0].__setitem__(controls.SOURCE_RATE_KEY,None),
            "source nonzero": lambda c:c["intervals"][0].__setitem__(controls.SOURCE_RATE_KEY,1e-300),
            "body nonzero": lambda c:c["intervals"][0]["body_acceleration_m_s2"].__setitem__(1,1e-300),
        }
        for name, mutate in mutations.items():
            with self.subTest(name=name):
                self.reset(); mutate(self.controls); self.save(); self.reject()
        self.reset()
        for interval in self.controls["intervals"]:
            interval[controls.SOURCE_RATE_KEY] = 0
            interval["outward_speed_m_s"][1] = [0,0]
            interval["body_acceleration_m_s2"] = [0,0,0]
        self.save(); self.assertEqual(controls.verify(self.directory)["control_intervals"],8)

    def test_boolean_numbers_and_nonfinite_json_rejected(self):
        targets = [("source",lambda i,v:i.__setitem__(controls.SOURCE_RATE_KEY,v)),
                   ("speed",lambda i,v:i["outward_speed_m_s"][1].__setitem__(0,v)),
                   ("inlet",lambda i,v:i["inlet_fraction"][1].__setitem__(0,v)),
                   ("body",lambda i,v:i["body_acceleration_m_s2"].__setitem__(0,v)),
                   ("clock",lambda i,v:i.__setitem__("start_time_s",v))]
        for label, mutate in targets:
            for value in (False,True,math.nan,math.inf,-math.inf):
                with self.subTest(target=label,value=value):
                    self.reset(); mutate(self.controls["intervals"][0],value)
                    self.save(json.dumps(self.controls,separators=(",", ":")).encode());self.reject()

    def test_provenance_text_limits_and_control_characters(self):
        for key in ("author","source_note"):
            for value in ("",None,True,"a" * 1001,"line\nnext","tab\tx","del\x7f","c1\x85","\ud800"):
                with self.subTest(key=key,value=repr(value)):
                    self.reset();self.controls["provenance"][key]=value;self.save();self.reject("provenance")
        self.reset();self.controls["provenance"]["author"]="é" * 1000;self.save()
        self.assertEqual(controls.verify(self.directory)["control_intervals"],8)

    def test_raw_byte_hashes_counts_caps_and_source_coverage(self):
        self.manifest["controls_sha256"]="0" * 64;self.save_manifest();self.reject("SHA256 mismatch")
        self.reset();self.save();self.manifest["controls_bytes"]+=1;self.save_manifest();self.reject("byte count")
        self.reset();self.save();raw=(self.directory / "controls.json").read_bytes()
        (self.directory / "controls.json").write_bytes(raw+b" ");self.reject("byte count")
        for key,value in (("controls_file","../controls.json"),("controls_bytes",True),("controls_bytes",0),
                          ("controls_bytes",65537),("controls_sha256","A" * 64),("version",True),("version",2.0)):
            with self.subTest(key=key,value=value):
                self.reset();self.save();self.manifest[key]=value;self.save_manifest();self.reject()
        self.reset();self.save();(self.directory / "controls.json").write_bytes(b" " * (controls.CONTROLS_LIMIT+1));self.reject("byte cap")
        self.reset();self.save();(self.directory / "run.json").write_bytes(b" " * (controls.MANIFEST_LIMIT+1));self.reject("byte cap")
        self.reset();self.manifest["provenance"]["source_sha256"].pop("tools/import_dense3d_controls.py")
        self.save();self.reject("source hash coverage")
        self.reset();self.controls["frames_sha256"]="0" * 64;self.save();self.reject("frames_sha256")
        self.reset();self.save();(self.directory / "frames.jsonl").write_bytes(self.frame_bytes+b" ");self.reject("frames byte count")

    def test_malformed_duplicate_truncated_overflow_controls_rehashed(self):
        raw=(self.directory / "controls.json").read_bytes()
        variants = {"duplicate":raw.replace(b'"version":2',b'"version":2,"version":2',1),
                    "malformed":raw.replace(b'"version":2',b'"version":]',1),"truncated":raw[:-5],
                    "nonfinite":raw.replace(b'"source_rate_m3_s":0.0',b'"source_rate_m3_s":NaN',1),
                    "overflow":raw.replace(b'"source_rate_m3_s":0.0',b'"source_rate_m3_s":1e9999',1),
                    "underflow":raw.replace(b'"source_rate_m3_s":0.0',b'"source_rate_m3_s":1e-9999',1),
                    "integer overflow":raw.replace(b'"end_frame":1',b'"end_frame":18446744073709551616',1),
                    "invalid UTF8":raw.replace(b'"author"',b'"auth\xffor"',1)}
        for name,data in variants.items():
            with self.subTest(name=name):
                self.reset();self.save(data);self.reject()
        self.reset();self.save();raw=(self.directory / "run.json").read_bytes()
        (self.directory / "run.json").write_bytes(raw.replace(b'"version": 2',b'"version": 2, "version": 2',1));self.reject("duplicate")

    def test_original_numerical_semantics_still_gate_rehashed_frames(self):
        mutations = {"donor":lambda f:f[4]["fields"]["fraction"].__setitem__(4,0.),
                     "pressure":lambda f:f[1]["fields"]["pressure"].__setitem__(4,-999.),
                     "divergence":lambda f:f[4]["fields"]["velocity_x"].__setitem__(4,.24),
                     "clock":lambda f:f[4].__setitem__("time_s",.2),
                     "stamp":lambda f:f[4]["carrier_stamp"].__setitem__("version","5")}
        for name,mutate in mutations.items():
            with self.subTest(name=name):
                self.reset();bad=synthetic_frames();mutate(bad)
                data=b"".join(json.dumps(f,separators=(",", ":")).encode()+b"\n" for f in bad)
                self.manifest,self.controls=synthetic_v2_metadata(bad,data)
                (self.directory / "frames.jsonl").write_bytes(data);self.save();self.reject()

    def test_direct_controls_api_preserves_raw_adjacent_numeric_types(self):
        self.assertEqual(controls.validate_controls(self.controls,self.manifest,self.frames)["control_intervals"],8)
        frames=copy.deepcopy(self.frames);frames[0]["time_s"]=0
        with self.assertRaisesRegex(ValueError,"type adjacent start_time_s"):
            controls.validate_controls(self.controls,self.manifest,frames)

    def test_requirements_survive_optimized_python(self):
        self.controls["run_binding"]["config"]["carrier_density_kg_m3"]=1000.0;self.save()
        program="import sys;sys.path.insert(0,sys.argv[1]);import import_dense3d_controls as m;m.verify(sys.argv[2])"
        result=subprocess.run([sys.executable,"-O","-c",program,str(Path(__file__).resolve().parent),str(self.directory)],
                              capture_output=True,text=True)
        self.assertNotEqual(result.returncode,0)
        self.assertIn("type run_binding.config.carrier_density",result.stderr)


if __name__ == "__main__":
    unittest.main()
