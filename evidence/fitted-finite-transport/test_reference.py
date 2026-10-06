import unittest
import numpy as np
import sympy as sp
from reference import symbolic_path,integrated_flux,spatial,q,ref,require
from numerical import actual,finite,initial,donor,study,limitations,ode_reference
class Contracts(unittest.TestCase):
    def test_exact_whole_path_geometry_and_flux(self):
        s=symbolic_path();self.assertEqual(len(s['flux']),76);self.assertEqual(sum(F.has(sp.log)for F in s['primitive'].values()),56)
        # The symbolic constructor proves all 48 positive microareas and
        # 32 masses, no poles/reversals and actual GCL over the whole interval.
        self.assertTrue(all(sp.diff(m,q,2)==0 for m in s['mass']))
    def test_independent_rational_and_vectorized_geometry_agree(self):
        for offset in [ref.Q(0),ref.Q(1,16),ref.Q(1,8)]:
            mass,rate,k,R,p=spatial(offset);m2,r2,k2,_,R2,p2=actual(float(offset))
            self.assertTrue(np.allclose(mass,m2,rtol=0,atol=2e-15));self.assertTrue(np.allclose(rate,r2,rtol=0,atol=2e-15));self.assertTrue(np.allclose(k,k2,rtol=0,atol=2e-15));self.assertTrue(np.array_equal(R,R2));self.assertTrue(np.allclose([[float(x.v),float(y.v)]for x,y in p],p2,rtol=0,atol=2e-15))
    def test_finite_GCL_with_actual_face_primitives(self):
        s=symbolic_path();F=finite(ref.Q(0),ref.Q(1,8));m0=actual(0.)[0];m1=actual(.125)[0];g=m1-m0
        for (i,j),f in F.items():g[i]+=f;g[j]-=f
        self.assertLess(np.max(np.abs(g)),2e-15);self.assertEqual(sum(x!=0 for x in m1-m0),22)
        exact=integrated_flux(ref.Q(0),ref.Q(1,8));self.assertLess(max(abs(exact[ij]-F[ij])for ij in F),2e-17)
    def test_gauss16_actual_face_integrals(self):
        x,w=np.polynomial.legendre.leggauss(16)
        for left,right in [(ref.Q(0),ref.Q(1,8)),(ref.Q(0),ref.Q(1,160)),(ref.Q(19,160),ref.Q(1,8))]:
            F=finite(left,right);G={ij:0. for ij in F}
            for z,weight in zip(x,w):
                offset=float((left+right)/2)+float((right-left)/2)*z;f=actual(offset)[3]
                for ij in G:G[ij]+=float((right-left)*2)*weight*f[ij] # dq/a = 4dq
            self.assertLess(max(abs(F[ij]-G[ij])for ij in F),3e-17)
    def test_wrong_flux_passes_marginals_but_fails_physical_provenance(self):
        r=limitations();self.assertLess(r['endpoint_frozen_flux_GCL_max'],2e-15);self.assertGreater(r['endpoint_face_integral_error'],.001)
    def test_constrained_convex_bound_claim_is_rejected(self):
        r=limitations();self.assertLess(r['new_velocity_range'][0],-.01);self.assertEqual(r['negative_free_transfer_entries'],93)
        self.assertGreater(r['symmetric_part_min_eigenvalue'],0.)
    def test_actual_work_momentum_and_nonautonomous_temporal_convergence(self):
        rows=[study(ref.Q(1,20)),study(ref.Q(1,40)),study(ref.Q(1,80))];errors=[r['temporal_L2_error']for r in rows]
        for i in range(2):self.assertTrue(1.8<errors[i]/errors[i+1]<2.2)
        self.assertTrue(all(r['max_momentum_drift']<2e-12 for r in rows));self.assertTrue(all(s['advection_loss']>0 for r in rows for s in r['steps']))
    def test_nonzero_GCL_defect_work_sign_exact(self):
        # Deliberately inconsistent algebra fixture checks the defect sign;
        # it is rejected as a physical step, not used to fit accepted masses.
        Q=ref.Q;m0=[Q(2),Q(3)];m1=[Q(5,2),Q(7,2)];F=Q(1,4);old=[Q(1,3),Q(2,5)];v=[Q(4,7),Q(-1,8)]
        A=[[m1[0]+F,Q(0)],[-F,m1[1]]];res=[sum(A[i][j]*v[j]for j in range(2))-m0[i]*old[i]for i in range(2)];g=[m1[0]-m0[0]+F,m1[1]-m0[1]-F]
        lhs=sum((m1[i]*v[i]**2-m0[i]*old[i]**2+m0[i]*(v[i]-old[i])**2)/2 for i in range(2))+F*(v[0]-v[1])**2/2
        rw=sum(v[i]*res[i]for i in range(2));gw=sum(g[i]*v[i]**2/2 for i in range(2));self.assertEqual(lhs-rw+gw,0);self.assertNotEqual(lhs-rw-gw,0)
    def test_pressure_dimension_is_measured_only_at_specific_frames(self):
        ranks=[]
        for offset in [ref.Q(0),ref.Q(1,16),ref.Q(1,8)]:
            p,t,ids,c,bottom=ref.offset_mesh(offset);_,_,_,div,_=ref.operators(p,t);n=max(ids)+1;rx=ref.scalar_embedding(n,c);ry=ref.scalar_embedding(n,c,bottom);reduced=ref.zeros(len(t),len(rx[0])+len(ry[0]))
            for a,row in enumerate(div):
                for i in range(len(p)):
                    for j in range(len(rx[0])):reduced[a][j]+=row[3*i]*rx[ids[i]][j]
                    for j in range(len(ry[0])):reduced[a][len(rx[0])+j]+=row[3*i+1]*ry[ids[i]][j]
            ranks.append(len(ref.independent_columns(reduced)))
        self.assertEqual(ranks,[32,32,32]) # three exact snapshots, not uniform inf-sup
if __name__=='__main__':unittest.main()
