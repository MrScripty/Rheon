"""Synthetic/exact arithmetic only: no native process, solve or campaign."""
import math, sys, unittest
from fractions import Fraction as F
from check_flat_wall_ritz import (Model, binary64_roundoff_radius,
    scatter_with_roundoff, linear_roundoff_certificate, coarse_moment_audit,
    traction)

class RoundoffContract(unittest.TestCase):
    def test_zero_subnormal_and_binade_neighbors(self):
        tiny=F(1,2**1074)
        for v in (0.,-0.,math.ulp(0.),-math.ulp(0.)):
            self.assertEqual(binary64_roundoff_radius(v),tiny/2)
        self.assertEqual(binary64_roundoff_radius(1.),F(1,2**53))
        self.assertEqual(binary64_roundoff_radius(math.nextafter(1.,0.)),F(1,2**54))
        self.assertEqual(binary64_roundoff_radius(-1.),F(1,2**53))

    def test_nonfinite_and_missing_neighbor_refuse(self):
        for v in (math.inf,-math.inf,math.nan,sys.float_info.max,-sys.float_info.max):
            with self.assertRaises(ValueError):binary64_roundoff_radius(v)

    def test_cancellation_scatter_has_exact_reference_one(self):
        cols=[{'terms':[(0,7,1.)]} for _ in range(3)]
        rounded,exact,radii=scatter_with_roundoff(cols,[1e16,1.,-1e16])
        self.assertEqual(exact[(0,7)],1)
        self.assertEqual(rounded[(0,7)],0.)
        self.assertGreaterEqual(radii[(0,7)],1)
        defect,bound=linear_roundoff_certificate(exact,rounded,radii,{(0,7):F(3)})
        self.assertEqual(defect,-3)
        self.assertGreaterEqual(bound,3)

    def test_underflow_rounding_cell_encloses_exact_product(self):
        cols=[{'terms':[(0,7,.5)]}]
        rounded,exact,radii=scatter_with_roundoff(cols,[math.ulp(0.)])
        self.assertEqual(exact[(0,7)],F(1,2**1075))
        self.assertEqual(rounded[(0,7)],0.)
        self.assertGreaterEqual(radii[(0,7)],exact[(0,7)])

    def test_bound_boundary_accepts_and_just_outside_rejects(self):
        exact={(0,1):F(0)};radii={(0,1):F(1,2**53)};coeff={(0,1):F(-7,3)}
        at={(0,1):radii[(0,1)]}
        defect,bound=linear_roundoff_certificate(exact,at,radii,coeff)
        self.assertEqual(abs(defect),bound)
        for sign in (-1,1):
            outside={(0,1):sign*radii[(0,1)]+sign*F(1,2**1074)}
            with self.assertRaisesRegex(ValueError,'exceeds scatter roundoff bound'):
                linear_roundoff_certificate(exact,outside,radii,coeff)

    def test_zero_functional_has_zero_bound(self):
        self.assertEqual(linear_roundoff_certificate({},{},{},{}),(0,0))
        with self.assertRaisesRegex(ValueError,'missing/negative scatter radius'):
            linear_roundoff_certificate({},{},{},{(0,1):F(1)})

    def test_exact_columns_and_nondyadic_q_coarse_identities(self):
        for n in (6,12):
            model=Model(n);cols=model.columns();q=[(i+1)/70 for i in range(len(cols))]
            rounded,exact,radii=scatter_with_roundoff(cols,q)
            for scheme,factor in [('p1',F(1,2)-F(3,n))]+([('normal_p2',-F(1,2))] if n==6 else []):
                audit=coarse_moment_audit(model,cols,exact,rounded,radii,scheme,factor)
                self.assertEqual(audit['exact_basis_columns_checked'],len(cols))
                self.assertEqual(audit['exact_Cq_identity_defect'],'0')
                self.assertLessEqual(abs(F(audit['rounded_stored_field_identity_defect'])),
                                     F(audit['propagated_scatter_roundoff_bound']))

    def test_perturbed_surface_moment_outside_derived_bound_refuses(self):
        model=Model(12);cols=model.columns();q=[(i+1)/70 for i in range(len(cols))]
        rounded,exact,radii=scatter_with_roundoff(cols,q);factor=F(1,4)
        audit=coarse_moment_audit(model,cols,exact,rounded,radii,'p1',factor)
        bound=F(audit['propagated_scatter_roundoff_bound'])
        # Select a nonzero exact geometric functional coefficient; choose a
        # perturbation from the derived bound itself, without a dimensional floor.
        for key in radii:
            f,t=traction(model,{key:F(1)},'p1');c=t[2]-factor*f[0]
            if c:break
        else:self.fail('missing functional coefficient')
        perturbed={k:F(v) for k,v in rounded.items()}
        perturbed[key]+=3*bound/c
        with self.assertRaisesRegex(ValueError,'exceeds scatter roundoff bound'):
            coarse_moment_audit(model,cols,exact,perturbed,radii,'p1',factor)

    def test_broken_exact_basis_identity_is_not_hidden_by_roundoff(self):
        model=Model(12);cols=model.columns();q=[(i+1)/70 for i in range(len(cols))]
        rounded,exact,radii=scatter_with_roundoff(cols,q)
        bad=[{'terms':[list(term) for term in col['terms']]} for col in cols]
        bad[-1]['terms'][1][2]+=.5
        with self.assertRaisesRegex(ValueError,'exact basis coarse identity mismatch'):
            coarse_moment_audit(model,bad,exact,rounded,radii,'p1',F(1,4))

if __name__=='__main__':unittest.main()
