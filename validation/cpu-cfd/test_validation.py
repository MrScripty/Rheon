import csv
import io
import json
from pathlib import Path
import tempfile
import unittest
import subprocess
from unittest.mock import patch

import run
from oracle import pressure_wrench


def policy_fixture():
    """Synthetic policy inputs only; these do not qualify a numerical solver."""
    rows = []
    for method in run.METHODS:
        for n in (6, 12, 24):
            rows.append(dict(case="pressure_mesh", method=method, n=n, steps=0,
                             continuum_linf=1/n**2, dt_s=0, iterations=10))
    for n in (8, 16, 32):
        rows.append(dict(case="viscosity_mesh", method="symmetric-strain-explicit-v1", n=n,
                         steps=n*n, dt_s=0.1/(n*n), continuum_linf=1/n**2, time_linf=0.01/n**2))
    for steps in (16, 32, 64):
        rows.append(dict(case="viscosity_time", method="symmetric-strain-explicit-v1", n=12,
                         steps=steps, dt_s=0.1/steps, continuum_linf=0.01, time_linf=0.01/steps))
    return [{k: r.get(k, 0.25 if k == "energy_j" else 0) for k in run.COLUMNS} for r in rows]


def encode(rows):
    out = io.StringIO()
    writer = csv.DictWriter(out, fieldnames=run.COLUMNS)
    writer.writeheader()
    writer.writerows(rows)
    return out.getvalue()


class AcceptanceTests(unittest.TestCase):
    def test_policy_accepts_declared_orders(self):
        assessment = run.evaluate(encode(policy_fixture()))
        self.assertEqual(len(assessment["rows"]), 12)
        self.assertEqual(assessment["observed_orders"]["pressure_mesh/jacobi-pcg-v1"], [2, 2])

    def test_missing_duplicate_unknown_case_and_nonfinite_reject(self):
        good = policy_fixture()
        for rows, diagnostic in ((good[:-1], "missing suite cases"),
                                 (good+[good[0]], "unexpected or duplicate case"),
                                 ([dict(good[0], case="aircraft")]+good[1:], "unexpected or duplicate case"),
                                 ([dict(good[0], continuum_linf="nan")]+good[1:], "nonfinite continuum_linf")):
            with self.subTest(rows=rows), self.assertRaisesRegex(ValueError, diagnostic):
                run.evaluate(encode(rows))

    def test_physical_and_refinement_failures_reject(self):
        for change, diagnostic in (({"wall_normal_max_m_per_s": 0.01}, "wall/flux gate"),
                                   ({"net_flux_m3_per_s": 0.1}, "wall/flux gate"),
                                   ({"divergence_max_per_s": 1}, "divergence gate"),
                                   ({"energy_growth_j": 1}, "energy growth gate"),
                                   ({"residual_max": 1}, "pressure solve gate"),
                                   ({"continuum_linf": 0}, "zero error"),
                                   ({"continuum_linf": 0.0001}, "refinement order gate"),
                                   ({"iterations": 0}, "pressure solve gate")):
            rows = policy_fixture()
            rows[0].update(change)
            with self.subTest(change=change), self.assertRaisesRegex(ValueError, diagnostic):
                run.evaluate(encode(rows))

    def test_wrong_time_and_mesh_contamination_reject(self):
        for index, change, diagnostic in ((6, {"time_linf": 1}, "mesh study temporal contamination"),
                                         (9, {"dt_s": 0.2}, "time/energy gate"),
                                         (11, {"time_linf": 0.0007}, "refinement order gate")):
            rows = policy_fixture()
            rows[index].update(change)
            with self.subTest(change=change), self.assertRaisesRegex(ValueError, diagnostic):
                run.evaluate(encode(rows))

    def test_columns_and_short_rows_reject(self):
        text = encode(policy_fixture())
        for broken, diagnostic in ((text.replace("continuum_linf", "unknown"), "invalid CSV columns"),
                                   (text+"pressure_mesh\n", "incomplete CSV row")):
            with self.assertRaisesRegex(ValueError, diagnostic):
                run.evaluate(broken)

    def test_failed_process_retains_failure_without_completion_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            cargo = root/"failing-cargo"
            cargo.write_text("#!/bin/sh\nexit 7\n")
            cargo.chmod(0o700)
            output = root/"receipt"
            with patch.object(run, "source_identity", return_value={"test_only": True}), \
                    patch.object(run, "command", return_value="test-only tool metadata"):
                with self.assertRaisesRegex(ValueError, "exited 7"):
                    run.execute(output, str(cargo))
                self.assertFalse((output/"receipt.json").exists())
                failed = json.loads((output/"failed.json").read_text())
                self.assertEqual(failed["exit_code"], 7)
                self.assertEqual(failed["status"], "failed")
                with self.assertRaises(FileExistsError):
                    run.execute(output, str(cargo))
                self.assertEqual(json.loads((output/"failed.json").read_text()), failed)

    def test_changed_source_rejects_an_otherwise_accepted_measurement(self):
        def complete(args, **kwargs):
            kwargs["stdout"].write(encode(policy_fixture()))
            return subprocess.CompletedProcess(args, 0)
        with tempfile.TemporaryDirectory() as tmp, \
                patch.object(run.platform, "platform", return_value="test Linux"), \
                patch.object(run, "command", return_value="test metadata"), \
                patch.object(run.subprocess, "run", side_effect=complete), \
                patch.object(run, "source_identity", side_effect=[{"hash": "old"}, {"hash": "new"}]):
            output = Path(tmp)/"run"
            with self.assertRaisesRegex(ValueError, "source changed"):
                run.execute(output)
            self.assertFalse((output/"receipt.json").exists())
            self.assertIn("source changed", json.loads((output/"failed.json").read_text())["failure"])

    def test_timeout_and_metadata_failure_retain_failed_receipt(self):
        for failure in (subprocess.TimeoutExpired("cargo", 1), OSError("metadata unavailable")):
            with tempfile.TemporaryDirectory() as tmp, \
                    patch.object(run.platform, "platform", return_value="test Linux"), \
                    patch.object(run, "source_identity", return_value={"test": True}), \
                    patch.object(run, "command", return_value="test metadata"), \
                    patch.object(run.subprocess, "run", side_effect=failure):
                output = Path(tmp)/"run"
                with self.assertRaises((subprocess.TimeoutExpired, OSError)):
                    run.execute(output)
                self.assertFalse((output/"receipt.json").exists())
                self.assertEqual(json.loads((output/"failed.json").read_text())["status"], "failed")

    def test_initial_source_metadata_failure_retains_failed_receipt(self):
        with tempfile.TemporaryDirectory() as tmp, \
                patch.object(run, "source_identity", side_effect=OSError("git unavailable")):
            output = Path(tmp)/"run"
            with self.assertRaisesRegex(OSError, "git unavailable"):
                run.execute(output)
            self.assertFalse((output/"receipt.json").exists())
            self.assertEqual(json.loads((output/"failed.json").read_text())["failure"], "git unavailable")


class PressureLoadOracleTests(unittest.TestCase):
    def samples(self, gradient, offset):
        # Original rectangular body; face midpoint quadrature integrates affine
        # pressure force exactly. Two-by-two tangential quadrature also integrates
        # pressure torque's quadratic tangential terms (midpoint alone does not).
        center, widths = [1, 2, 3], [2, 3, 4]
        samples = []
        for d in range(3):
            a, b = (d+1)%3, (d+2)%3
            for sign in (-1, 1):
                for qa in (-1, 1):
                    for qb in (-1, 1):
                        position = center.copy()
                        position[d] += sign*widths[d]/2
                        position[a] += qa*widths[a]/(2*3**0.5)
                        position[b] += qb*widths[b]/(2*3**0.5)
                        normal = [0]*3
                        normal[d] = sign
                        p = offset+sum(g*x for g, x in zip(gradient, position))
                        samples.append((position, normal, widths[a]*widths[b]/4, p))
        return samples

    def assert_vector(self, actual, expected):
        for a, b in zip(actual, expected):
            self.assertAlmostEqual(a, b, places=8)

    def test_affine_pressure_force_moment_gauge_and_origin(self):
        # Divergence theorem F=-V grad(p), M=(center-origin) cross F.
        for offset in (0, 10000):
            samples = self.samples([1, 2, -3], offset)
            force, moment = pressure_wrench(samples, [0, 0, 0])
            self.assert_vector(force, [-24, -48, 72])
            self.assert_vector(moment, [288, -144, 0])
            self.assert_vector(pressure_wrench(samples, [1, 2, 3])[1], [0, 0, 0])

    def test_hydrostatic_buoyancy_sign(self):
        force, moment = pressure_wrench(self.samples([0, -1000*9.81, 0], 1e6), [1, 2, 3])
        self.assert_vector(force, [0, 1000*9.81*24, 0])
        self.assert_vector(moment, [0, 0, 0])

    def test_invalid_surface_rejects(self):
        for sample in (([0, 0, 0], [2, 0, 0], 1, 0), ([0, 0, 0], [1, 0, 0], -1, 0),
                       ([0, 0, 0], [1, 0, 0], 1, float("nan"))):
            with self.assertRaisesRegex(ValueError, "invalid pressure surface sample"):
                pressure_wrench([sample], [0, 0, 0])


if __name__ == "__main__":
    unittest.main()
