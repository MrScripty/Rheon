"""Packet admission rejects stale or edited data independently of assertions."""
from pathlib import Path
import copy
import json
import struct
import subprocess
import tempfile
import unittest

import build_solver as solver


def exact_reference():
    """Independent graph projection c*(u.c)/(c.c), not a Rust output fixture."""
    runs = []
    for method in ("jacobi-pcg-v1", "sgs-pcg-v1"):
        def state(generation, accepted):
            c = [1., -1., -1., 1.]
            factor = .5 / sum(v*v for v in c) if accepted else 0.
            values = [v*factor for v in c]
            fields = {"x": [0., values[0], 0., 0., values[1], 0.], "y": [0., 0., values[2], values[3], 0., 0.], "z": [0.]*8, "tracer": [0.]*4}
            # Native reset writes positive zero, independently of c's sign.
            if not accepted:
                fields = {k: [0.]*len(v) for k, v in fields.items()}
            return {"schema": 1, "fixture": solver.FIXTURE, "method": method,
                    "grid": {"counts": [2, 2, 1], "spacing": [1., 1., 1.], "origin": [0., 0., 0.]},
                    **fields, "bits": {k: [struct.unpack("<I", struct.pack("<f", x))[0] for x in v] for k, v in fields.items()},
                    "time": .25 if accepted else 0., "time_bits": "3fd0000000000000" if accepted else "0000000000000000",
                    "generation": generation, "generation_u64": str(generation), "accepted_since_reset": accepted}

        def accepted(generation):
            return {"status": "accepted", "code": "accepted", "detail": None, "state": state(generation, True),
                    "report": {"dt": .25, "time": .25, "generation": generation,
                               "pressure": {"iterations": 1, "true_residual_l2": 0., "true_residual_max": 0., "predicted_divergence_max": 0.},
                               "actual_divergence_max": 0., "courant": .25*(.125+.125), "tracer_integral": 0.,
                               "kinetic_energy": .5*4*.125**2,
                               "forces": {"kinetic_energy_before": 0., "kinetic_energy_after": .5*.5**2, "applied_work": .5*.5**2}}}

        def rejected(code, old):
            return {"status": "rejected", "code": code, "detail": "rejected fixture probe", "state": old, "report": None}

        baseline, first = state(0, False), accepted(1)
        runs.append({"method": method, "baseline": baseline,
                     "invalid_dt": rejected("invalid_time_step", baseline),
                     "cancelled_rest": rejected("cancelled_before_commit", baseline),
                     "accepted": first,
                     "cancelled_nonzero": rejected("cancelled_before_commit", first["state"]),
                     "second_accept": rejected("reset_required", first["state"]),
                     "reset": state(2, False), "after_reset": accepted(3)})
    return {"schema": 1, "fixture": solver.FIXTURE, "dt": .25, "runs": runs}


class SolverPacket(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo, self.packet, self.edition = [self.root/name for name in ("repo", "packet", "edition")]
        self.repo.mkdir(); self.packet.mkdir()
        for path in solver.source_paths(self.repo):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(path.name)
        source = self.repo/"src/example.rs"
        source.parent.mkdir(exist_ok=True); source.write_text("pub fn example() {}")
        self.reference = exact_reference()
        for name in solver.PUBLIC_ARTIFACTS:
            (self.packet/name).write_bytes(b"\0asm\x01\0\0\0" if name.endswith(".wasm") else name.encode())
        (self.packet/"native-reference.json").write_text(json.dumps(self.reference))
        native = self.packet/solver.NATIVE_EXPORTER
        native.parent.mkdir()
        # Admission plumbing test: a transparent executable stub, not claimed
        # evidence that the scientific solver or WASM ran.
        native.write_text("#!/usr/bin/env python3\nprint(" + repr(json.dumps(self.reference)) + ")\n")
        native.chmod(0o755)
        self.receipt = {"schema": 1, "fixture": solver.FIXTURE, "wasm_bindgen_version": solver.WASM_BINDGEN,
                        "rustc": "rustc 1.92.0 (fixture)", "target": "wasm32-unknown-unknown", "profile": "release",
                        "source_git_sha": "f"*40, "source_git_dirty": False,
                        "source_sha256": solver.source_hashes(self.repo),
                        "artifact_sha256": {n: solver.digest(self.packet/n) for n in solver.PUBLIC_ARTIFACTS},
                        "native_exporter": {"path": solver.NATIVE_EXPORTER, "sha256": solver.digest(native)}}
        self.write_receipt()

    def write_receipt(self):
        (self.packet/"build-receipt.json").write_text(json.dumps(self.receipt))

    def test_good_packet_publish_and_read_only_verification(self):
        solver.validate_packet(self.repo, self.packet)
        presentation = solver.publish(self.repo, self.edition, self.packet)
        before = {str(p): solver.digest(p) for p in self.edition.rglob("*") if p.is_file()}
        self.assertEqual(solver.verify_published(self.repo, self.edition), presentation)
        self.assertFalse((self.edition/"solver/native").exists())
        self.assertEqual(before, {str(p): solver.digest(p) for p in self.edition.rglob("*") if p.is_file()})

    def test_unavailable_has_no_invented_reference_or_arrays(self):
        self.assertIsNone(solver.validate_packet(self.repo, None))
        self.assertEqual(solver.publish(self.repo, self.edition, None), {"schema": 1, "included": False})
        solver.verify_published(self.repo, self.edition)
        self.assertEqual([p.name for p in self.edition.iterdir()], ["solver-presentation.json"])
        (self.edition/"solver").mkdir()
        with self.assertRaisesRegex(ValueError, "stale solver assets"):
            solver.verify_published(self.repo, self.edition)

    def test_changed_added_and_missing_solver_sources_reject(self):
        path = self.repo/"src/example.rs"
        original = path.read_text()
        for mutation in ("changed", None):
            with self.subTest(mutation=mutation):
                if mutation is None: path.unlink()
                else: path.write_text(mutation)
                with self.assertRaisesRegex(ValueError, "current source bytes"):
                    solver.validate_packet(self.repo, self.packet)
                path.write_text(original)
        extra = path.with_name("new.rs"); extra.write_text("pub fn new() {}")
        with self.assertRaisesRegex(ValueError, "current source bytes"):
            solver.validate_packet(self.repo, self.packet)

    def test_public_binary_and_native_exporter_tampering_reject(self):
        for name in ("rheon_solver.js", "rheon_solver_bg.wasm", "native-reference.json", solver.NATIVE_EXPORTER):
            with self.subTest(name=name):
                path = self.packet/name; original = path.read_bytes()
                path.write_bytes(original+b"changed")
                with self.assertRaisesRegex(ValueError, "hash mismatch"):
                    solver.validate_packet(self.repo, self.packet)
                path.write_bytes(original)

    def test_rehashed_false_physics_or_rollback_still_reject(self):
        mutations = [
            lambda r: r["runs"][0]["accepted"]["state"]["x"].__setitem__(1, .25),
            lambda r: r["runs"][0]["accepted"]["report"].__setitem__("kinetic_energy", .125),
            lambda r: r["runs"][0]["cancelled_nonzero"].__setitem__("state", r["runs"][0]["baseline"]),
            lambda r: r["runs"][0]["reset"].__setitem__("generation", 0),
            lambda r: r["runs"][1]["accepted"]["report"]["pressure"].__setitem__("true_residual_max", float("inf")),
        ]
        for mutate in mutations:
            reference = copy.deepcopy(self.reference); mutate(reference)
            with self.assertRaises(ValueError): solver.validate_reference(reference)

    def test_wrong_target_profile_and_compiler_reject(self):
        for key, bad in (("target", "native"), ("profile", "debug"), ("rustc", "rustc 1.91.0 (wrong)"), ("wasm_bindgen_version", "0.2.104")):
            with self.subTest(key=key):
                old = self.receipt[key]; self.receipt[key] = bad; self.write_receipt()
                with self.assertRaisesRegex(ValueError, "mismatch"):
                    solver.validate_packet(self.repo, self.packet)
                self.receipt[key] = old; self.write_receipt()

    def test_rehashed_native_output_and_presentation_changes_reject(self):
        native = self.packet/solver.NATIVE_EXPORTER
        native.write_text("#!/usr/bin/env python3\nprint('{}')\n")
        self.receipt["native_exporter"]["sha256"] = solver.digest(native); self.write_receipt()
        with self.assertRaisesRegex(ValueError, "fresh native exporter"):
            solver.validate_packet(self.repo, self.packet)
        self.edition.mkdir(); (self.edition/"solver").mkdir()
        import shutil
        for name in (*solver.PUBLIC_ARTIFACTS, "build-receipt.json"):
            shutil.copyfile(self.packet/name, self.edition/"solver"/name)
        (self.edition/"solver-presentation.json").write_text('{"schema":1,"included":true,"source_git_sha":"wrong"}')
        with self.assertRaisesRegex(ValueError, "presentation"):
            solver.verify_published(self.repo, self.edition)

    def test_checkout_ancestors_other_checkouts_and_nonempty_output_reject(self):
        for path in (self.repo, self.repo/"nested", self.root):
            with self.assertRaisesRegex(ValueError, "repository"):
                solver.external_directory(self.repo, path)
        other = self.root/"other"; other.mkdir()
        subprocess.run(["git", "init", "--quiet", str(other)], check=True)
        with self.assertRaisesRegex(ValueError, "every Git checkout"):
            solver.external_directory(self.repo, other/"output")
        with self.assertRaisesRegex(ValueError, "absent or empty"):
            solver.build(self.repo, self.packet)


if __name__ == "__main__":
    unittest.main()
