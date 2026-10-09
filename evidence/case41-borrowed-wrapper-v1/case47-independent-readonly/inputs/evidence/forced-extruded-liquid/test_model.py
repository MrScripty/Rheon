import unittest
import numpy as np
import reference as r
import experiments as e


class ForceEquations(unittest.TestCase):
    def test_exact_signed_work_and_wrong_endpoint_load(self):
        self.assertEqual(len(e.exact_constant_load()),2)

    def test_semidiscrete_force_changes_pressure_and_all_components(self):
        for kind in ['initial','pressure_state']:
            q,eta=r.t.initial(kind);s=r.m.spatial(r.m.q_to_x(q));J,_=r.m.cap_chart(s);z=J@eta
            state=np.r_[r.m.q_to_x(q),z[r.m.FREE],r.third.XI]
            zero,v0=r.instantaneous(state,np.zeros(3));forced,v=r.instantaneous(state,r.A)
            self.assertGreater(np.max(np.abs(v['pressure']-v0['pressure'])),1e-3)
            self.assertGreater(np.linalg.norm(forced-zero),1e-3)
            body,_=r.source(v['spatial'],r.A)
            self.assertGreater(np.linalg.norm(body),1e-3)
            self.assertLess(np.max(np.abs((forced-zero)[10:]-r.A[2])),1e-13)

    def test_constant_third_acceleration_and_both_shears(self):
        for kind in ['initial','pressure_state']:
            q,eta=r.t.initial(kind);_,v,_=r.solve(q,eta,.025,r.A)
            w=e.third_work(v,.025,np.full(12,.25),r.A)
            self.assertLess(np.max(np.abs(np.array(w['coefficients'])-(.25+.025*r.A[2]))),128*r.t.EPS*.25)
            w=e.third_work(v,.025,r.third.XI,r.A)
            self.assertGreater(w['shear_x'],0);self.assertGreater(w['shear_y'],0)

    def test_forgetting_force_work_fails_existing_budget(self):
        q,eta=r.t.initial('pressure_state');_,v,_=r.solve(q,eta,.05,r.A)
        w=e.third_work(v,.05,r.third.XI,r.A)
        self.assertGreater(abs(w['ledger_error']+w['force_work']),w['work_allowance']*1000)
        self.assertGreater(abs(v['energy_ledger_error']+v['force_work']),v['fixed_work_allowance']*1000)


if __name__=='__main__':unittest.main()
