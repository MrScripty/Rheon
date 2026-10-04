import unittest
import numpy as np
from force_balance import pressure_fixture, check_pressure_force


class PressureForce(unittest.TestCase):
    def test_fixed_pressure_gradient_matches_hand_force(self):
        gradient, pressure, expected = pressure_fixture()
        np.testing.assert_array_equal(expected, [6., -9., 2., -10., -12., 1.])
        self.assertEqual(check_pressure_force(gradient@pressure), 0.)

    def test_force_sign_scale_and_component_perturbations_reject(self):
        gradient, pressure, expected = pressure_fixture()
        perturbed = expected.copy()
        perturbed[2] += .125
        for candidate in [-(gradient@pressure), 2*(gradient@pressure), perturbed]:
            with self.subTest(candidate=candidate):
                with self.assertRaisesRegex(ValueError, 'independent pressure-force mismatch'):
                    check_pressure_force(candidate)

    def test_gradient_orientation_scale_and_component_perturbations_reject(self):
        gradient, pressure, _ = pressure_fixture()
        perturbed = gradient.copy()
        perturbed[2, 3] += .125
        for candidate in [-gradient, gradient*2, perturbed]:
            with self.subTest(candidate=candidate):
                with self.assertRaisesRegex(ValueError, 'independent pressure-force mismatch'):
                    check_pressure_force(candidate@pressure)

    def test_nonfinite_force_rejects(self):
        force = pressure_fixture()[2]
        force[0] = np.nan
        with self.assertRaisesRegex(ValueError, 'six finite components'):
            check_pressure_force(force)


if __name__ == '__main__':
    unittest.main()
