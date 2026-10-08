"""Meaningful exact work, physical provenance, equation and rollback regressions."""
from fractions import Fraction as Q
from functools import lru_cache
import copy
import json
import unittest
import numpy as np
import temporal as step
import trajectory
import importlib.util
from pathlib import Path
_spec=importlib.util.spec_from_file_location("discrete_work_replay",Path(__file__).with_name("verify_cell.py"))
verify_cell=importlib.util.module_from_spec(_spec);_spec.loader.exec_module(verify_cell)

@lru_cache(maxsize=1)
def cell():return step.solve(.05)


def exact_identity(reverse=False):
 mass0=[Q(2),Q(3),Q(4)]
 faces=[(0,1,Q(1,5),Q(2,5)),(1,2,Q(1,7),Q(1,11)),(0,2,Q(1,13),Q(1,17))]
 if reverse:faces=[(0,1,Q(1),Q(1))]
 u0=[[Q(1),Q(2),Q(3)],[Q(4),Q(5),Q(6)],[Q(-1),Q(-2),Q(-3)]]
 u1=[[Q(1,2),Q(1,3),Q(1,5)],[Q(2),Q(-1),Q(3)],[Q(-2),Q(1),Q(4)]]
 mass1=mass0[:];c=[[Q(0)]*3 for _ in mass0];mix=Q(0);wrong=Q(0)
 for i,j,p,n in faces:
  mass1[i]-=p-n;mass1[j]+=p-n
  for d in range(3):
   f=p*u1[i][d]-n*u1[j][d];c[i][d]+=f;c[j][d]-=f
   jump=(u1[i][d]-u1[j][d])**2
   mix+=(p+n)*jump/2;wrong+=abs(p-n)*jump/2
 energy=lambda mass,u:sum(mass[i]*sum(v*v for v in u[i])/2 for i in range(3))
 inertia=[[mass1[i]*u1[i][d]-mass0[i]*u0[i][d]+c[i][d]for d in range(3)]for i in range(3)]
 work=sum(u1[i][d]*inertia[i][d]for i in range(3)for d in range(3))
 be=sum(mass0[i]*sum((u1[i][d]-u0[i][d])**2 for d in range(3))/2 for i in range(3))
 return energy(mass1,u1)-energy(mass0,u0)+be+mix-work,mix,wrong

class Contracts(unittest.TestCase):
 def test_original_floor_is_reproduced_and_not_hidden(self):
  d=json.loads(Path(__file__).with_name('trials').joinpath('original-floor.json').read_text())
  self.assertEqual(d['outcome'],'REJECTED');self.assertEqual(d['step'],5)
  self.assertTrue(all(t['rate_norm']>1e-13 for t in d['trace'][2:]))
 def test_repair_solves_exact_frozen_failed_input_at_same_target(self):
  d=json.loads(Path(__file__).with_name('trials').joinpath('original-floor.json').read_text());trace=[]
  r=step.solve(.00625,state=(np.array(d['initial_q']),np.array(d['initial_eta'])),diagnostics=False,trace=trace)
  self.assertLessEqual(trace[-1]['rate_norm'],1e-13);self.assertLess(r['direct_momentum_rate_norm'],1e-11)
 def test_known_values_are_stored_exactly_and_all_full_rows_checked(self):
  c=cell();eta=np.array(c['end_eta']);z=np.array(c['end_reduced_velocity'])
  np.testing.assert_array_equal(z[step.KNOWN],step.SIGN*eta[step.COORD])
  self.assertLess(max(c['full_path_maxima'][:3]),1e-11)
 def test_small_matrix_has_fifteen_rank_rows_and_seven_exact_known_values(self):
  self.assertEqual(len(step.LIFT_ROWS),15);self.assertEqual(len(step.KNOWN),7)
  np.testing.assert_array_equal(np.sort(np.r_[step.KNOWN,step.UNKNOWN]),np.arange(22))
 def test_stable_and_original_inertia_have_same_physical_residual_gate(self):
  c=cell();self.assertLess(c['finite_momentum_rate_norm'],1e-11);self.assertLess(c['direct_momentum_rate_norm'],1e-11)
  self.assertLess(max(map(abs,c['inertia_assembly_difference'])),1e-14)
 def test_exact_pressure_state_quadratic_flux_coefficient_is_nonzero(self):
  from rate import leading
  self.assertTrue(any(leading('pressure_state')));self.assertEqual(leading('initial'),[Q(0)]*22)

 def test_exact_changing_mass_vector_work_identity(self):self.assertEqual(exact_identity()[0],0)
 def test_bidirectional_zero_net_mass_still_mixes(self):
  ledger,mix,wrong=exact_identity(True);self.assertEqual(ledger,0);self.assertGreater(mix,0);self.assertEqual(wrong,0)
 def test_full_22_discrete_momentum_rate_equations(self):
  c=cell();self.assertEqual(len(c['finite_momentum_residual']),22);self.assertLess(c['finite_momentum_rate_norm'],1e-11)
 def test_all_full_moving_constraints_and_material_cap(self):self.assertTrue(all(v<=1e-11 for v in cell()['full_path_maxima'][:3]))
 def test_actual_local_mass_not_only_total_mass(self):self.assertTrue(all(abs(g)<=a for g,a in zip(cell()['actual_local_GCL_defect'],cell()['local_GCL_allowance'])))
 def test_same_fixed_weighted_work_scale(self):
  c=cell();self.assertLess(abs(c['discrete_residual_work']),c['fixed_work_allowance']);self.assertLess(abs(c['energy_ledger_error']),c['fixed_work_allowance']);self.assertLess(c['fixed_work_allowance'],5e-14)
 def test_dissipations_are_positive_and_separately_named(self):
  c=cell();self.assertGreater(c['backward_Euler_loss'],1e-6);self.assertGreater(c['mixing_loss'],1e-5);self.assertGreater(c['viscous_loss'],.002)
 def test_pressure_is_solved_nonzero_with_zero_adjoint_work(self):
  c=cell();self.assertGreater(max(map(abs,c['end_pressure'])),.001);self.assertLess(abs(c['pressure_work']),c['fixed_work_allowance'])
 def test_different_physical_momentum_equations_remain_failed(self):
  c=cell();self.assertFalse(c['continuous_momentum_gate_pass']);self.assertFalse(c['actual_physical_integrated_momentum_gate_pass']);self.assertGreater(c['actual_integrated_physical_momentum_norm'],1e-7)
 def test_endpoint_donor_is_explicitly_different_momentum_flux(self):self.assertGreater(cell()['endpoint_vs_actual_momentum_max'],1e-8)
 def test_nonzero_horizontal_momentum_is_conserved(self):self.assertLess(abs(cell()['horizontal_momentum_change']),1e-11)
 def test_zero_pressure_fails_true_equations(self):
  c=cell();u=np.array(c['unknowns']);u[6:]=0;v=step.evaluate(np.array(c['initial_q']),np.array(c['initial_eta']),u,c['interval'],32)
  with self.assertRaisesRegex(ValueError,'momentum'):step.qualify(v,c['interval'])
 def test_wrong_pressure_adjoint_fails_work_and_momentum(self):
  c=cell();v=step.evaluate(np.array(c['initial_q']),np.array(c['initial_eta']),np.array(c['unknowns']),c['interval'],32)
  v['residual']=v['residual'].copy();v['residual'][0]+=1e-4
  with self.assertRaisesRegex(ValueError,'momentum'):step.qualify(v,c['interval'])
 def test_physical_face_circulation_rejected_even_when_marginals_balance(self):
  c=copy.deepcopy(cell());rows=c['positive_negative_face_integrals'];lookup={(i,j):k for k,(i,j,_,_)in enumerate(rows)};tri=list(map(int,step.m.IDS[step.m.TRI[0]]))
  for i,j in zip(tri,tri[1:]+tri[:1]):rows[lookup[min(i,j),max(i,j)]][2 if i<j else 3]+=1/1024
  with self.assertRaises(ValueError):verify_cell.verify(c)
 def test_nonzero_accepted_owner_cancellation_preserves_all_fields(self):
  o=trajectory.HostOwner();o.advance(.05);before=trajectory.fingerprint(o.accepted)
  with self.assertRaises(trajectory.Cancelled):o.advance(.025,cancel=lambda s:s=='before_solve')
  self.assertEqual(before,trajectory.fingerprint(o.accepted));self.assertEqual(o.accepted.stamp,1)
 def test_bounds_refuse_without_advancing(self):
  o=trajectory.HostOwner();before=trajectory.fingerprint(o.accepted)
  for h in [0,-1,float('nan'),.051]:
   with self.assertRaises(ValueError):o.advance(h)
   self.assertEqual(before,trajectory.fingerprint(o.accepted))
 def test_full_replay_and_no_public_step(self):
  c=cell();self.assertFalse(c['new_public_step_enabled']);self.assertFalse(verify_cell.verify(c)['new_public_step_enabled'])

if __name__=='__main__':unittest.main()
