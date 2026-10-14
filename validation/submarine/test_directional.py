"""Focused diagnostic/provenance controls; no synthetic record claims a native solve."""
import argparse
import copy
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import directional as d
import oracle
import run


class Directional(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.baseline = oracle.read_json(d.HERE / "fixtures.json")
        cls.family = oracle.read_json(d.FIXTURES)
        cls.fixture_sha = oracle.digest(d.FIXTURES)
        cls.workspace = tempfile.TemporaryDirectory(prefix="rheon-directional-tests-")
        cls.repo = Path(cls.workspace.name) / "repo"
        destination = cls.repo / "validation/submarine"
        destination.mkdir(parents=True)
        for name in d.SOURCE_FILES:
            (destination / name).write_bytes((d.HERE / name).read_bytes())
        for arguments in (("init",), ("config", "user.name", "Directional test"),
                          ("config", "user.email", "directional-test@example.invalid"),
                          ("add", "."), ("commit", "-m", "test-only pinned verifier source")):
            subprocess.run(["git", "-C", str(cls.repo), *arguments], check=True, capture_output=True)
        cls.source = run.git(cls.repo, "rev-parse", "HEAD")

    @classmethod
    def tearDownClass(cls):
        cls.workspace.cleanup()

    def fails(self, claim, callback):
        with self.assertRaises(oracle.Failure) as caught:
            callback()
        self.assertEqual(caught.exception.data["claim_id"], claim)

    def test_exact_family_and_pin(self):
        self.assertEqual(d.load_family(self.fixture_sha)[0], self.family)
        self.assertEqual(set(self.family["cases"]), set(d.CASES))
        self.fails("directional-fixture-pin", lambda: d.load_family("0" * 64))

    def test_wrong_domain_body_control_forcing_spacing_cap_or_profile_rejected(self):
        mutations = [
            lambda f: f["cases"]["straight-x-fine"]["origin"].__setitem__(0, "-2"),
            lambda f: f["cases"]["mms-y-medium"]["body_upper"].__setitem__(1, "0.25"),
            lambda f: f["cases"]["straight-base"].__setitem__("mu", "2"),
            lambda f: f["cases"]["mms-base"].__setitem__("forcing", "zero"),
            lambda f: f["cases"]["straight-x-medium"]["spacing"].__setitem__(1, "0.125"),
            lambda f: f["limits"].__setitem__("max_dofs", 8192),
            lambda f: f["discretization"].__setitem__("pivot_min", "0"),
            lambda f: f["cases"].pop("mms-y-fine"),
            lambda f: f["cases"].__setitem__("unfrozen-case", f["cases"]["straight-base"]),
        ]
        for mutate in mutations:
            with self.subTest(mutation=mutate):
                bad = copy.deepcopy(self.family)
                mutate(bad)
                self.fails("directional-family", lambda: d.freeze_family(bad, self.baseline))

    def test_only_resolved_monotone_positive_series_get_order(self):
        resolved = d.observed_order([4.0, 2.0, 1.0])
        self.assertEqual(resolved["classification"], "resolved-monotone-positive-order")
        self.assertAlmostEqual(resolved["descriptive_order"], 1.0)
        for values, expected in [([1.0, 2.0, 1.0], "oscillatory"),
                                 ([1.0, 2.0, 4.0], "nonpositive-order"),
                                 ([1.0, 2.0, 3.0], "nonpositive-order"),
                                 ([1.0, 1.0, 1.0], "constant"),
                                 ([1.0, 1.0 + 1e-14, 1.0 + 2e-14], "noise-limited"),
                                 ([1.0, 2.0, 2.0], "noise-limited")]:
            with self.subTest(values=values):
                result = d.observed_order(values)
                self.assertEqual(result["classification"], expected)
                self.assertIsNone(result["descriptive_order"])
                self.assertNotIn("extrapolation", result)
                self.assertNotIn("GCI", result)

    def test_series_resolution_has_no_unit_floor_and_rejects_nonfinite(self):
        result = d.observed_order([4e-20, 2e-20, 1e-20])
        self.assertEqual(result["classification"], "resolved-monotone-positive-order")
        for values in ([1, 2], [1, 2, float("nan")], [1, True, 2], [1, 2, "3"]):
            self.fails("directional-series", lambda: d.observed_order(values))

    def test_reference_gap_and_zero_algebraic_moment(self):
        encode = lambda values: list(map(oracle.bits, values))
        record = {"physical_loads": {"pressure_force": encode([2, 0, 0]), "shear_force": encode([-0.5, 0, 0]),
                    "pressure_moment": encode([0, 0, 0]), "shear_moment": encode([0, 0, 0])},
                  "algebraic_loads": {"body_force": encode([1, 0, 0]), "outer_force": encode([-1, 0, 0]),
                    "body_moment": encode([0, 0, 0]), "outer_moment": encode([0, 0, 0])}, "solve_report": {}}
        result = d.case_diagnostic(record, self.family["cases"]["straight-base"], {})
        self.assertEqual(result["physical_total_force_N"], [1.5, 0, 0])
        gap = result["physical_minus_algebraic"]
        self.assertEqual(gap["force"]["norm_relative_to_algebraic"], 0.5)
        self.assertEqual(gap["force"]["norm_over_reference"], 50.0)
        self.assertIsNone(gap["moment"]["norm_relative_to_algebraic"])

    def test_capture_reuses_runner_and_restores_global(self):
        args = argparse.Namespace(fixtures_sha256=self.fixture_sha, repo=self.repo, source=self.source,
                                  output=str(Path(self.workspace.name) / "unused-capture"))
        previous = run.FIXTURES
        def execute(request, repo, fixtures, sha):
            self.assertEqual(run.FIXTURES, d.FIXTURES)
            self.assertEqual(request.case, list(d.CASES))
            self.assertEqual(fixtures, self.family)
            self.assertEqual(request.native_contract_sha256, sha)
            return {"origin_sha256": "1" * 64}
        with patch.object(run, "execute", side_effect=execute) as called, patch.object(run, "write_json"):
            result = d.capture(args)
            called.assert_called_once()
        self.assertEqual(run.FIXTURES, previous)
        self.assertEqual(result["status"], "captured-awaiting-externally-pinned-cold-recheck")
        with self.assertRaises(oracle.Failure), d.selected_fixtures():
            raise oracle.Failure("invalid", "test", "test-only failure")
        self.assertEqual(run.FIXTURES, previous)

    def origin_fixture(self, directory):
        """Hash-bound test envelope only; this fake executable is never executed."""
        out = Path(directory)
        archive = out / "source.tar"
        archive.write_bytes(subprocess.run(["git", "-C", str(self.repo), "archive", "--format=tar", self.source],
                                          check=True, capture_output=True).stdout)
        executable = out / "fake-executable"
        executable.write_bytes(b"test envelope only; never executed")
        fixture_copy = out / "fixtures.json"
        fixture_copy.write_bytes(d.FIXTURES.read_bytes())
        stream = out / "test-stream"
        stream.write_bytes(b"")
        manifest = {"schema": "rheon-submarine-i1-origin-v1", "terminal": "completed", "source_commit": self.source,
                    "source_tree": run.git(self.repo, "rev-parse", self.source + "^{tree}"), "dirty": False,
                    "source_archive_path": str(archive), "source_archive_sha256": oracle.digest(archive),
                    "fixtures_sha256": self.fixture_sha, "record_schema": oracle.SCHEMA, "operator_profile": oracle.PROFILE,
                    "executable": {"path": str(executable), "sha256": oracle.digest(executable)},
                    "validator": {"files": {p.name: oracle.digest(p) for p in
                                               [d.HERE / "run.py", d.HERE / "oracle.py", d.FIXTURES]}},
                    "commands": [], "outputs": {}}
        for case in d.CASES:
            record_path = out / "cases" / case / "record.json"
            record_path.parent.mkdir(parents=True)
            record_path.write_text(json.dumps({"fixture_id": case}))
            manifest["outputs"][case] = {"path": str(record_path), "sha256": oracle.digest(record_path)}
            log = {"path": str(stream), "sha256": oracle.digest(stream)}
            manifest["commands"].append({"argv": [str(executable), "--fixtures", str(fixture_copy), "--case", case,
                                                   "--output", str(record_path.parent)],
                                          "returncode": 0, "stdout": log, "stderr": log})
        origin = out / "origin.json"
        origin.write_text(json.dumps(manifest))
        return origin, manifest

    def recheck_args(self, directory, origin):
        return argparse.Namespace(fixtures_sha256=self.fixture_sha, repo=self.repo, source=self.source,
                                  capture=directory, origin_sha256=oracle.digest(origin),
                                  output=str(Path(directory) / "unused-summary"))

    def test_stale_origin_missing_case_and_wrong_external_pin_refused_before_numerics(self):
        for mutation, claim in [
            (lambda m: m.__setitem__("terminal", "active"), "origin-terminal"),
            (lambda m: m["outputs"].pop("mms-y-fine"), "directional-completeness"),
        ]:
            with self.subTest(claim=claim), tempfile.TemporaryDirectory() as directory:
                origin, manifest = self.origin_fixture(directory)
                mutation(manifest)
                origin.write_text(json.dumps(manifest))
                args = self.recheck_args(directory, origin)
                with patch.object(oracle, "validate_record") as numerical:
                    self.fails(claim, lambda: d.cold_recheck(args))
                    numerical.assert_not_called()
                self.assertFalse(Path(args.output).exists())
        with tempfile.TemporaryDirectory() as directory:
            origin, _ = self.origin_fixture(directory)
            args = self.recheck_args(directory, origin)
            args.origin_sha256 = "0" * 64
            self.fails("origin-integrity", lambda: d.cold_recheck(args))

    def test_actual_origin_verifier_detects_record_and_fixture_tamper(self):
        with tempfile.TemporaryDirectory() as directory:
            origin, manifest = self.origin_fixture(directory)
            case = d.CASES[0]
            record = Path(manifest["outputs"][case]["path"])
            with d.selected_fixtures():
                result = run.verify_origin(origin, oracle.digest(origin), self.source, record, self.fixture_sha, case, self.repo)
                self.assertEqual(result["status"], "externally-pinned-origin-integrity-checked")
                record.write_text('{"fixture_id":"tampered"}')
                self.fails("origin-output", lambda: run.verify_origin(
                    origin, oracle.digest(origin), self.source, record, self.fixture_sha, case, self.repo))
            record.write_text(json.dumps({"fixture_id": case}))
            (Path(directory) / "fixtures.json").write_text("{}")
            with d.selected_fixtures():
                self.fails("origin-command", lambda: run.verify_origin(
                    origin, oracle.digest(origin), self.source, record, self.fixture_sha, case, self.repo))

    def test_wrapper_bytes_must_match_exact_source(self):
        self.assertEqual(set(d.source_bindings(self.repo, self.source)),
                         {"validation/submarine/" + name for name in d.SOURCE_FILES})
        changed = self.repo / "validation/submarine/directional.py"
        original = changed.read_bytes()
        changed.write_bytes(original + b"\n# source mutation\n")
        subprocess.run(["git", "-C", str(self.repo), "add", "."], check=True, capture_output=True)
        subprocess.run(["git", "-C", str(self.repo), "commit", "-m", "test source mutation"], check=True, capture_output=True)
        altered = run.git(self.repo, "rev-parse", "HEAD")
        self.fails("directional-source", lambda: d.source_bindings(self.repo, altered))


if __name__ == "__main__":
    unittest.main()
