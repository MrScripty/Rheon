"""Focused compatibility, physical integration, full constraints and work tests."""
from fractions import Fraction as Q
from functools import lru_cache
import unittest
import numpy as np
import model as m
import exact as ex
import certificate as cert
from cell import initial,solve_cell,quadrature
from independent import verify as independent
from verify_cell import endpoint_convection


@lru_cache(maxsize=1)
def cell():return solve_cell(.05)


class Contracts(unittest.TestCase):
    def test_exact_tangent_compatibility_on_both_reviewed_fields(self):
        for kind in ['initial','pressure_state']:
            s=ex.instantaneous(kind)
            self.assertEqual([a+d for a,d in zip(m.base.mv(m.INITIAL['D'],s['acceleration']),s['moving_term'])],[Q(0)]*24)
            self.assertEqual(s['horizontal_momentum_rate'],0)
            self.assertEqual(s['energy_rate']+s['advection_loss']+s['strain_power'],0)
            self.assertEqual(s['pressure_work'],0)

    def test_exact_nonzero_moving_term_cannot_be_omitted(self):
        s=ex.instantaneous('pressure_state')
        self.assertGreater(max(map(abs,s['moving_term'])),Q(2,1000))
        self.assertTrue(any(m.base.mv(m.INITIAL['D'],s['acceleration'])))

    def test_projected_rows_cannot_hide_an_incompatible_full_component(self):
        w=ex.incompatible_component()
        weighted=[[q*a for q,a in zip(row,m.INITIAL['areas'])]for row in m.old.transpose(m.INITIAL['Q'])]
        self.assertEqual(m.base.mv(weighted,w),[Q(0)]*16)
        self.assertTrue(any(w))
        with self.assertRaisesRegex(ValueError,'full strong'):
            m.require(all(x==0 for x in w),'full strong compatibility')

    def test_reviewed_six_coordinate_lift_has_full_strong_rows_and_cap_trace(self):
        cap=(Q(1,7),Q(2,9),Q(1,13));interior=(Q(2,11),Q(3,17),Q(-1,19))
        z=ex.apply_chart(cap,interior)
        self.assertEqual(m.base.mv(m.INITIAL['D'],z),[Q(0)]*24)
        self.assertEqual([z[j]for j in m.SELECT],list(interior))
        self.assertEqual(m.base.dot(m.INITIAL['R'][3*4+1],z),-cap[2])

    def test_float_acceleration_pressure_matches_independent_exact_solve(self):
        for kind in ['initial','pressure_state']:
            e=ex.instantaneous(kind);z=np.array(e['z'],float)
            state=np.r_[0.,1.,.5,1.25,z[m.FREE]]
            _,v=m.evaluate(state)
            self.assertLess(np.max(np.abs(v['acceleration']-np.array(e['acceleration'],float))),1e-11)
            self.assertLess(np.max(np.abs(v['pressure']-np.array(e['pressure'],float))),1e-11)
            self.assertGreater(np.max(np.abs(v['spatial']['Q']@v['pressure'])),.01)

    def test_symbolic_row_relations_cover_all_physical_strong_rows(self):
        r=cert.row_identities()
        self.assertEqual(len(r['relations']),8)
        self.assertTrue(r['all_176_row_identity_entries_exact_zero'])
        self.assertTrue(r['full_row_reconstruction_exact'])
        self.assertTrue(r['fourth_cap_velocity_from_integrated_divergence_exact'])

    def test_interval_arithmetic_is_outward_and_excludes_zero_division(self):
        a=cert.Interval(Q(1,3),Q(2,3));b=cert.Interval(Q(-1,7),Q(2,11))
        p=a*b
        self.assertLessEqual(p.lo,Q(-2,21));self.assertGreaterEqual(p.hi,Q(4,33))
        self.assertLessEqual(a.lo,Q(1,3));self.assertGreaterEqual(a.hi,Q(2,3))
        with self.assertRaisesRegex(ValueError,'excludes zero'):a/b

    def test_neumann_rank_gate_rejects_singular_matrix_without_confidence_tolerance(self):
        center=[[Q(1),Q(0)],[Q(0),Q(1)]]
        I=[[cert.Interval(1),cert.Interval(0)],[cert.Interval(0),cert.Interval(1)]]
        self.assertEqual(cert.neumann_bound(center,I),0)
        I[1][1]=cert.Interval(0)
        with self.assertRaisesRegex(ValueError,'Neumann'):cert.neumann_bound(center,I)

    def test_all_22_integrated_momentum_rows_pass_but_pointwise_gate_fails(self):
        c=cell()
        self.assertEqual(len(c['integrated_momentum_residual']),22)
        self.assertLess(c['integrated_momentum_norm'],1e-11)
        self.assertTrue(c['solver_success'])
        self.assertGreater(c['max_pointwise_momentum_norm'],.001)
        self.assertFalse(c['pointwise_momentum_gate_pass'])

    def test_same_quadratic_path_closes_real_cap_motion_and_finite_local_GCL(self):
        c=cell()
        self.assertLess(c['max_strong_divergence'],1e-11)
        self.assertLess(c['max_strong_acceleration'],1e-11)
        self.assertLess(c['max_material_mismatch'],1e-11)
        self.assertTrue(all(abs(g)<=limit for g,limit in zip(c['actual_local_GCL_defect'],c['local_GCL_allowance'])))
        self.assertLess(c['quadrature_16_32_difference'],1e-15)

    def test_endpoint_velocity_times_mass_transfer_is_not_actual_momentum_flux(self):
        c=cell()
        self.assertGreater(np.max(np.abs(endpoint_convection(c)-c['integrated_convection'])),1e-8)

    def test_fixed_weighted_work_gate_rejects_despite_complete_energy_ledger(self):
        c=cell()
        self.assertFalse(c['work_gate_pass']);self.assertFalse(c['candidate_accepted']);self.assertFalse(c['new_public_step_enabled'])
        self.assertGreater(abs(c['actual_momentum_residual_work']),c['fixed_work_allowance']*100000)
        self.assertLess(abs(c['energy_ledger_error']),c['fixed_work_allowance'])
        self.assertLess(c['energy_new'],c['energy_old'])

    def test_independent_rational_operators_reproduce_actual_work_failure(self):
        r=independent(cell(),8)
        self.assertFalse(r['work_gate_pass'])
        self.assertLess(abs(float(r['independent_residual_work'])-cell()['actual_momentum_residual_work']),1e-14)
        self.assertLess(float(r['full_integrated_momentum_residual_max']),1e-11)


if __name__=='__main__':unittest.main()
