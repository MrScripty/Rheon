"""Meaningful exact pressure and actual-path gates, including python -O."""
from fractions import Fraction as Q
import unittest
import reference as r
import numerical as numerical


class Contracts(unittest.TestCase):
    def test_smallest_native_supported_geometry_and_full_pressure_image(self):
        a = r.assemble()
        self.assertEqual((len(a['points']), len(a['micro']), len(a['mass'])), (19, 24, 16))
        self.assertEqual(len(r.base.independent_columns(a['D'])), 16)
        self.assertEqual(len(r.base.independent_columns(r.transpose(a['B']))), 16)
        self.assertEqual(sum(a['mass']), Q(27, 8))

    def test_pressure_is_solved_not_assigned_and_zero_pressure_fails(self):
        a, solved = r.assemble(), r.force_solve()
        no_pressure = [x + r.DT * y - z for x, y, z in zip(r.base.mv(a['M'], solved['z']), r.base.mv(a['K'], solved['z']), r.base.mv(a['M'], solved['old']))]
        self.assertTrue(any(no_pressure))
        self.assertGreater(max(map(abs, r.base.mv(a['Q'], solved['p']))), Q(1, 100))
        force = r.base.mv(r.transpose(a['B']), solved['p'])
        self.assertEqual([x + r.DT * f for x, f in zip(no_pressure, force)], [Q(0)] * 22)

    def test_full_symmetric_strain_energy_and_pressure_adjoint(self):
        a, v = r.assemble(), r.force_solve()
        self.assertEqual(v['energy_new'] - v['energy_old'] + v['increment'] + r.DT * v['strain'], 0)
        test = [Q((i * 7) % 13 - 6, 11) for i in range(22)]
        self.assertEqual(r.base.dot(test, r.base.mv(r.transpose(a['B']), v['p'])), -sum(area * p * d for area, p, d in zip(a['areas'], r.base.mv(a['Q'], v['p']), r.base.mv(a['D'], test))))
        self.assertEqual(v['pressure_work'], 0)
        self.assertGreater(v['strain'], 0)

    def test_both_xy_components_are_required_and_x_momentum_is_conserved(self):
        a, v = r.assemble(), r.force_solve()
        self.assertTrue(any(v['z'][12:]))
        omit_y = v['z'][:12] + [Q(0)] * 10
        self.assertTrue(any(r.base.mv(a['D'], omit_y)))
        constant_x = [Q(1)] * 12 + [Q(0)] * 10
        self.assertEqual(r.base.dot(constant_x, r.base.mv(a['M'], v['z'])), r.base.dot(constant_x, r.base.mv(a['M'], v['old'])))

    def test_cap_is_material_and_bottom_fixed_through_rebuilt_path(self):
        v = r.force_solve()
        for t in (Q(0), Q(1, 80), Q(1, 40), Q(3, 80), r.DT):
            p = r.path_sample(t)
            for i in range(3):
                self.assertEqual(tuple(x.d for x in p['assembly']['points'][i]), (0, 0))
                self.assertEqual(tuple(x.d for x in p['assembly']['points'][i + 3]), v['cap_velocity'][i])
            self.assertTrue(all(x > 0 for x in p['assembly']['mass'] + p['assembly']['areas']))

    def test_exact_divergence_derivative_is_the_path_counterexample(self):
        initial = r.path_sample(Q(0))
        self.assertEqual(initial['divergence'], [Q(0)] * 24)
        self.assertLess(initial['divergence_rate'][6], Q(-2, 1000))
        self.assertGreater(max(map(abs, r.path_sample(r.DT)['divergence'])), Q(1, 10000))

    def test_global_mass_cannot_hide_actual_local_continuity_failure(self):
        end = r.path_sample(r.DT)
        self.assertEqual(sum(end['assembly']['mass']), Q(27, 8))
        self.assertEqual(sum(end['continuity_defect']), 0)
        self.assertEqual(end['continuity_defect'], end['divergence_mass'])
        self.assertGreater(max(map(abs, end['continuity_defect'])), Q(1, 100000))

    def test_force_only_pressure_cannot_be_reused_on_changed_geometry(self):
        end = r.path_sample(r.DT)
        v = r.force_solve()
        self.assertTrue(any(r.base.mv(end['assembly']['B'], v['z'])))
        self.assertNotEqual(r.base.dot(v['z'], r.base.mv(r.transpose(end['assembly']['B']), v['p'])), 0)

    def test_independent_float_geometry_and_flux_match_exact_snapshots(self):
        for t in [Q(0), r.DT / 2, r.DT]:
            p = r.path_sample(t)
            mass, rate, div, flux = numerical.actual(float(t))
            for actual, exact in [(mass, p['assembly']['mass']), (rate, p['assembly']['rate']), (div, p['divergence'])]:
                self.assertLess(max(abs(x-float(y)) for x,y in zip(actual,exact)), 2e-14)
            self.assertLess(max(abs(flux[ij]-float(f)) for ij,f in p['flux'].items()), 2e-14)

    def test_actual_integrated_flux_fails_local_GCL_despite_refined_quadrature(self):
        result = numerical.study()
        self.assertTrue(all(row['face_quadrature_16_32_difference'] < 1e-15 for row in result['rows']))
        self.assertGreater(result['rows'][0]['integrated_local_GCL_defect_max'], 4e-7)
        self.assertTrue(all(3.9 < ratio < 4.1 for ratio in result['defect_refinement_ratios']))
        self.assertEqual(result['accepted_advancing_steps'], 0)

    def test_bad_periodic_or_unbounded_candidate_path_is_refused(self):
        with self.assertRaisesRegex(ValueError, 'bounded'):
            r.mesh(r.DT + Q(1, 1000))
        with self.assertRaisesRegex(ValueError, 'periodic'):
            r.mesh(Q(0), ((Q(0), Q(0)), (Q(0), Q(0)), (Q(1), Q(0))))


if __name__ == '__main__':
    unittest.main()
