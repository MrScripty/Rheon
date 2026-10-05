"""Physical trajectory and algebraic controls independent of native evaluation."""
import unittest
import numpy as np
from reference import assembly,exact_semidiscrete,ref,require
class Contracts(unittest.TestCase):
    def test_exact_material_geometry_and_actual_zero_flux(self):
        # assembly includes exact finite translated geometry and actual dual
        # flux tests after replacing all mesh derivatives by material speed.
        a=assembly();self.assertEqual(len(a['mass']),32);self.assertEqual(a['R'].shape,(32,24))
    def test_constant_nullspace_and_positive_reduced_mass(self):
        a=assembly();self.assertLess(np.max(np.abs(a['K']@np.ones(24))),2e-15)
        self.assertGreater(np.min(np.linalg.eigvalsh(a['M'])),0.)
        self.assertGreater(np.min(np.linalg.eigvalsh(a['K'])),-2e-15)
    def test_full_work_and_full_momentum_with_nontrivial_viscosity(self):
        a=assembly();z=a['z0'];dt=.025;v=np.linalg.solve(a['M']+dt*a['K'],a['M']@z);d=v-z
        work=.5*(v@a['M']@v-z@a['M']@z)+.5*(d@a['M']@d)+dt*v@a['K']@v
        self.assertLess(abs(work),2e-15);self.assertLess(abs(np.sum(a['mass']*(a['R']@d))),2e-15)
        self.assertGreater(float(v@a['K']@v),0.);self.assertGreater(float(d@a['M']@d),0.)
    def test_real_temporal_error_halves(self):
        a=assembly();errors=[]
        for dt in [.05,.025,.0125]:
            z=a['z0'].copy()
            for _ in range(round(.5/dt)):z=np.linalg.solve(a['M']+dt*a['K'],a['M']@z)
            d=z-exact_semidiscrete(a,.5);errors.append(np.sqrt(d@a['M']@d))
        self.assertTrue(1.8<errors[0]/errors[1]<2.2);self.assertTrue(1.8<errors[1]/errors[2]<2.2)
    def test_old_bottom_fixed_motion_is_not_material_zero_flux(self):
        points,micro,ids,*_=ref.mesh();self.assertTrue(ref.dual_flux(points,micro,ids))
        # This negative witness forbids silently treating the predecessor's
        # bottom-fixed instantaneous motion as this material step's trajectory.
    def test_wrong_diagonal_mass_equation_changes_actual_solution(self):
        a=assembly();m=a['M'];k=a['K'];z=a['z0'];dt=.025
        true=np.linalg.solve(m+dt*k,m@z);diag=np.diag(np.diag(m));bad=np.linalg.solve(diag+dt*k,diag@z)
        self.assertGreater(np.sqrt((bad-true)@m@(bad-true)),1e-4)
    def test_no_diffusion_fails_native_time_target(self):
        a=assembly();d=a['z0']-exact_semidiscrete(a,.5)
        self.assertGreater(np.sqrt(d@a['M']@d),.02)
if __name__=='__main__':unittest.main()
