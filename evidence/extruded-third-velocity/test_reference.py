import unittest
import numpy as np
import reference
import third_exact
class ThirdEquations(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.rows=[third_exact.derive(k)for k in ['initial','pressure_state']]
 def test_exact_momentum_work_and_constant_rate_both_fixtures(self):
  for d in self.rows:
   for r in d['rows']:
    self.assertEqual(r['momentum_rate'],0);self.assertEqual(r['work_identity'],0)
   self.assertEqual(d['rows'][1]['instantaneous_derivative'],[0]*12)
 def test_independent_numerical_both_shear_blocks_match_exact_assembly(self):
  s=reference.m.spatial(np.array([0.,1.,.5,1.25]));mass,kx,ky,_,_=reference.blocks(s)
  d=self.rows[0]
  for actual,name in [(mass,'mass_matrix'),(kx,'shear_x_matrix'),(ky,'shear_y_matrix')]:np.testing.assert_allclose(actual,np.array(d[name],float),rtol=0,atol=1e-14)
  self.assertGreater(np.linalg.eigvalsh(mass).min(),0)
 def test_wrong_double_shear_and_omitted_shear_do_not_satisfy_work(self):
  for d in self.rows:
   r=d['rows'][0];self.assertGreater(r['shear_x_power'],0);self.assertGreater(r['shear_y_power'],0)
   self.assertNotEqual(r['energy_rate']+r['donor_loss']+2*(r['shear_x_power']+r['shear_y_power']),0)
   self.assertNotEqual(r['energy_rate']+r['donor_loss']+r['shear_x_power'],0)
if __name__=='__main__':unittest.main()
