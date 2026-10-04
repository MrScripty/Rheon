"""Independent acceptance oracles for the educational references."""
import math, unittest
from fractions import Fraction
import numpy as np
import reference as ref
class References(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.data=ref.data()
    def test_closed_cube_earliest_path_not_endpoint(self):
        records=self.data['collision']['cases']
        for i,shift in enumerate([-.1,0,.1]):
            crossing,miss,inside=records[3*i:3*i+3]
            self.assertAlmostEqual(crossing['t'],(.5+shift)/1.6)
            self.assertIsNone(miss['t'])
            self.assertAlmostEqual(inside['hit'][0],shift+.3)
    def test_hand_layer_pressure(self):
        p=self.data['hydrostatic']['sets'][0]['pressure']
        self.assertAlmostEqual(p[0],8829.)
        self.assertAlmostEqual(p[16],3924.)
        self.assertEqual(p[-1],0.)
    def test_projection_full_gauge_row_and_energy(self):
        p=self.data['projection'];u=np.array(p['after']);flux=np.zeros(len(p['cells']))
        for speed,(lo,hi) in zip(u,p['edges']):flux[lo]-=speed;flux[hi]+=speed
        self.assertLess(max(abs(flux)),1e-12)
        self.assertEqual(p['pressure'][0],0.)
        self.assertLess(p['energy_after'],p['energy_before'])
    def test_viscosity_dense_mode_and_monotone_energy(self):
        for group in self.data['viscous']['sets']:
            states=group['states'];energies=[s['energy'] for s in states]
            self.assertTrue(all(b<=a for a,b in zip(energies,energies[1:])))
            self.assertAlmostEqual(states[0]['energy'],.25)
            self.assertAlmostEqual(states[-1]['energy'],.25*states[-1]['amplitude']**2,places=13)
    def test_slip_wall_equations(self):
        for p in self.data['slip']['sets']:
            self.assertAlmostEqual(p['velocity'][0],p['length']/(1+p['length']))
            self.assertAlmostEqual(p['shear'],.1/(1+p['length']))
            self.assertGreaterEqual(p['relative_wall_dissipation'],0.)
    def test_cap_hemisphere_and_adhesion(self):
        p=next(p for p in self.data['cap']['sets'] if p['theta']==90)
        self.assertAlmostEqual(2*math.pi*p['R']**3/3,1e-6,places=16)
        for p in self.data['cap']['sets']:
            self.assertLess(abs(p['integrated_volume']-1e-6),2e-14)
            for t in p['tensions']:
                self.assertAlmostEqual(t['pressure_jump']*p['R'],2*t['sigma'])
                self.assertTrue(0<=t['adhesion_work']<=2*t['sigma'])
    def test_rational_source_ledger(self):
        ledger=self.data['ledger'];actual=list(map(Fraction,ledger['amount_after']))
        self.assertEqual(actual,[Fraction(12,125),Fraction(309,1000),Fraction(1597,1000)])
        self.assertEqual(sum(actual),Fraction(1001,500))
    def test_force_work_signed(self):
        ledger=self.data['ledger'];self.assertLess(ledger['force_energy_change'],0)
        self.assertAlmostEqual(ledger['force_energy_change'],ledger['force_work'],places=15)
if __name__=='__main__':unittest.main()
