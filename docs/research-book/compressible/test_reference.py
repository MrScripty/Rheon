"""Focused unit + continuum checks; also run with python -O."""

import copy
import unittest

import numpy as np

from generate import build_case, generate, qualify, source_hashes, validate_qualification, validate_results
from oracles import sod_exact, sod_star
from reference import Config, acoustic_initial, conservative, integrate, primitive, sod_initial, stable_dt, step


class ReferenceTests(unittest.TestCase):
    def test_uniform_three_component_state_is_stationary(self):
        config = Config((4, 5, 6))
        state = conservative(np.full(config.shape, 1.7), np.array([0.2, -0.4, 0.3]), 0.9)
        final = step(state, stable_dt(state, config), config)
        np.testing.assert_array_equal(final, state)
        density, velocity, pressure = primitive(final)
        np.testing.assert_allclose(pressure, 0.9, atol=1e-15)
        np.testing.assert_allclose(velocity[0, 0, 0], [0.2, -0.4, 0.3])

    def test_periodic_conservation_and_three_component_wave_evolution(self):
        config = Config((8, 8, 8))
        initial = acoustic_initial(config)
        final = integrate(initial, config, [0, 0.08])[-1][1]
        np.testing.assert_allclose(final.sum(axis=(0, 1, 2)), initial.sum(axis=(0, 1, 2)), rtol=1e-13, atol=1e-11)
        rho, velocity, pressure = primitive(final)
        self.assertGreater(float(rho.min()), 0)
        self.assertGreater(float(pressure.min()), 0)
        for axis in range(3):
            self.assertGreater(float(np.ptp(velocity[..., axis])), 1e-3)
            self.assertGreater(float(np.max(np.abs(np.diff(rho, axis=axis)))), 1e-4)
        self.assertGreater(float(np.max(np.abs(final - initial))), 0.005)

    def test_invalid_density_pressure_nonfinite_rejected_without_mutation(self):
        config = Config((4, 4, 4))
        valid = sod_initial(config)
        for channel, value, message in ((0, 0.0, "nonpositive density"), (4, -1.0, "nonpositive or nonfinite pressure"),
                                        (2, np.nan, "expected finite")):
            with self.subTest(message=message):
                state = valid.copy()
                state[0, 0, 0, channel] = value
                before = state.copy()
                with self.assertRaisesRegex(ValueError, message):
                    step(state, 1e-3, config)
                np.testing.assert_array_equal(state, before)

    def test_summed_direction_cfl_is_enforced(self):
        config = Config((4, 5, 6))
        state = conservative(np.ones(config.shape), np.zeros(3), 1.0)
        self.assertAlmostEqual(stable_dt(state, config), config.cfl / (np.sqrt(1.4) * 15))
        with self.assertRaisesRegex(ValueError, "summed-direction CFL"):
            step(state, stable_dt(state, config) * 1.01, config)
        with self.assertRaisesRegex(ValueError, "cfl"):
            Config(cfl=0.31)
        with self.assertRaisesRegex(ValueError, "gamma"):
            primitive(state, gamma=1)

    def test_output_inside_checkout_rejected_before_generation(self):
        from pathlib import Path
        with self.assertRaisesRegex(ValueError, "outside every Git checkout"):
            generate(Path(__file__).resolve().parent / "forbidden-generated-fixture")

    def test_exact_sod_star_against_published_standard_values(self):
        # Standard gamma=1.4 Sod star-state constants, independent of FV solver.
        pressure, velocity = sod_star()
        self.assertAlmostEqual(pressure, 0.3031301780506468, places=12)
        self.assertAlmostEqual(velocity, 0.9274526200489499, places=12)
        rho, vel, p = sod_exact(np.array([0.1, 0.9]), 0.08)
        np.testing.assert_array_equal(rho, [1.0, 0.125])
        np.testing.assert_array_equal(vel, [0.0, 0.0])
        np.testing.assert_array_equal(p, [1.0, 0.1])

    def test_analytic_acoustic_and_sod_refinement(self):
        report = qualify()
        self.assertEqual(report["status"], "satisfied")
        self.assertLess(report["acoustic"]["finest_relative_phase_speed_error"], 0.08)

    def test_packet_round_trip_and_targeted_corruption(self):
        case = build_case("oblique_acoustic", acoustic_initial, "test", 0.01)
        report = qualify()
        report["max_relative_conservation_drift"] = max(frame["diagnostics"]["relative_conservation_drift"] for frame in case["frames"])
        packet = {"schema_version": 1, "provenance": {"source_sha256": source_hashes()}, "cases": [case], "qualification": report}
        validate_results(packet)
        corrupted = copy.deepcopy(packet)
        corrupted["cases"][0]["frames"][1]["pressure"][0] += 0.01
        with self.assertRaisesRegex(ValueError, "primitive mismatch: pressure"):
            validate_results(corrupted)
        corrupted = copy.deepcopy(packet)
        corrupted["cases"][0]["frames"][1]["diagnostics"]["mass"] += 0.01
        with self.assertRaisesRegex(ValueError, "diagnostic mismatch: mass"):
            validate_results(corrupted)
        corrupted = copy.deepcopy(report)
        corrupted["acoustic"]["phase_speeds"][-1] += 0.1
        with self.assertRaisesRegex(ValueError, "qualification mismatch: acoustic.phase_speeds"):
            validate_qualification(corrupted)
        corrupted = copy.deepcopy(report)
        corrupted["sod"]["density_l1_errors"][-1] += 0.1
        with self.assertRaisesRegex(ValueError, "qualification mismatch: sod.density_l1_errors"):
            validate_qualification(corrupted)


if __name__ == "__main__":
    unittest.main()
