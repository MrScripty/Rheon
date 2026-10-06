"""Governing-equation contracts, including intentionally inconsistent variants."""
import unittest
from fractions import Fraction as Q
import taylor as t
b=t.b
class Equations(unittest.TestCase):
 def test_independent_tangent_and_full_differentiated_equations(self):
  for kind in ['initial','pressure_state']:
   d=t.derive(kind);self.assertTrue(all(d['exact_identities'].values()))
 def test_endpoint_pressure_is_not_literal_average(self):
  for kind in ['initial','pressure_state']:
   d=t.derive(kind);self.assertGreater(t.normmax(d['average_pressure_coefficient_bias']),Q(1,100))
 def test_first_pressure_difference_cannot_be_presented_as_evolved_pressure(self):
  for kind in ['initial','pressure_state']:
   d=t.derive(kind);self.assertGreater(t.normmax(d['endpoint_pressure_coefficient_bias']),Q(1,100))
 def test_interval_impulse_has_a_truncation_coefficient_not_a_wrong_limit(self):
  for kind in ['initial','pressure_state']:
   d=t.derive(kind);self.assertGreater(t.normmax(d['pressure_impulse_defect_coefficient']),Q(1,1000))
   self.assertEqual(d['tangent_unknowns'][6:],d['pressure'])
 def test_pressure_state_exposes_nonvanishing_frozen_divergence_variant(self):
  initial=t.derive('initial');state=t.derive('pressure_state');self.assertEqual(t.normmax(initial['frozen_divergence_counterexample']['nonvanishing_strong_rate_defect']),0);self.assertGreater(t.normmax(state['frozen_divergence_counterexample']['nonvanishing_strong_rate_defect']),Q(1,1000))
 def test_pressure_state_exposes_nonvanishing_frozen_mass_variant(self):
  initial=t.derive('initial');state=t.derive('pressure_state');self.assertEqual(t.normmax(initial['dropped_mass_derivative_counterexample']['nonvanishing_momentum_rate_defect']),0);self.assertGreater(t.normmax(state['dropped_mass_derivative_counterexample']['nonvanishing_momentum_rate_defect']),Q(1,10000))
 def test_initial_zero_flux_does_not_exempt_force_truncation(self):
  d=t.derive('initial');self.assertEqual(d['endpoint_donor_coefficient'],[Q(0)]*22);self.assertGreater(t.normmax(d['actual_path_momentum_defect_coefficient']),Q(1,100))
if __name__=='__main__':unittest.main()
