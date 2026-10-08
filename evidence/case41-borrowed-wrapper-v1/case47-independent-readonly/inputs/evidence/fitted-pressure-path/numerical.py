"""Physical face integration of the rejected path, independently of Rust.

Float quadrature refinement is measured, not a whole-interval error theorem.
The exact reference supplies the construction and algebraic counterexample.
"""
import json
from fractions import Fraction as Q
import numpy as np
import reference as r


def actual(time):
    cap_velocity = tuple(tuple(float(x) for x in v) for v in r.force_solve()['cap_velocity'])
    points, micro, ids, _, _ = r.mesh(Q(str(time)), cap_velocity)
    pos = np.array([[x.v for x in p] for p in points], float)
    motion = np.array([[x.d for x in p] for p in points], float)
    a = r.assemble()
    raw = np.array(r.base.mv(a['R'], r.force_solve()['z']), float).reshape(-1, 3)
    mass = np.zeros(16)
    rate = np.zeros(16)
    div = np.zeros(24)
    flux = {}
    for t, tri in enumerate(micro):
        p = pos[list(tri)]
        w = motion[list(tri)]
        ab, ac = p[1]-p[0], p[2]-p[0]
        dab, dac = w[1]-w[0], w[2]-w[0]
        area2 = np.linalg.det(np.array([ab, ac]))
        area_rate2 = dab[0]*ac[1]-dab[1]*ac[0]+ab[0]*dac[1]-ab[1]*dac[0]
        center = p.mean(axis=0)
        for i in range(3):
            j, k = (i+1)%3, (i+2)%3
            mass[ids[tri[i]]] += area2/2  # rho=3 / 3
            rate[ids[tri[i]]] += area_rate2/2
            gx, gy = (p[j,1]-p[k,1])/area2, (p[k,0]-p[j,0])/area2
            div[t] += gx*raw[tri[i],0]+gy*raw[tri[i],1]
            segment = center-(p[i]+p[j])/2
            relative = sum(weight*(raw[tri[node],:2]-w[node]) for node,weight in zip((i,j,k),(5/12,5/12,1/6)))
            f = 3*(relative[0]*segment[1]-relative[1]*segment[0])
            u,v = ids[tri[i]],ids[tri[j]]
            if u != v:
                pair = min(u,v),max(u,v)
                flux[pair] = flux.get(pair,0.)+(f if u<v else -f)
    return mass,rate,div,flux


def integrate(order, dt=0.05):
    x, w = np.polynomial.legendre.leggauss(order)
    positive, negative = {}, {}
    for node, weight in zip(x,w):
        flux = actual(dt*(1+node)/2)[3]
        for pair,f in flux.items():
            positive[pair] = positive.get(pair,0.)+dt*weight*max(f,0.)/2
            negative[pair] = negative.get(pair,0.)+dt*weight*max(-f,0.)/2
    return positive,negative


def study():
    rows=[]
    for dt in [0.05,0.025,0.0125]:
        p16,n16=integrate(16,dt);p32,n32=integrate(32,dt)
        g=actual(dt)[0]-actual(0.)[0]
        for (i,j),p in p32.items():
            transfer=p-n32[i,j]
            g[i]+=transfer;g[j]-=transfer
        refinement=max(max(abs(p16[ij]-p32[ij]),abs(n16[ij]-n32[ij])) for ij in p32)
        r.require(refinement<1e-15,'measured face quadrature refinement')
        r.require(np.max(np.abs(g))>1e-9,'integrated local GCL obstruction')
        r.require(abs(g.sum())<1e-15,'global mass passes rejected path')
        rows.append(dict(path_interval=dt,integrated_local_GCL_defect_max=float(np.max(np.abs(g))),
                         integrated_local_GCL_defect=g.tolist(),face_quadrature_16_32_difference=refinement,
                         global_mass_defect=float(g.sum()),physical_faces=len(p32),
                         positive_negative_face_integrals=[[i,j,p32[i,j],n32[i,j]] for i,j in sorted(p32)]))
    return dict(scope='physical integration of a rejected constant-coefficient pressure-solved cap path',
                rows=rows,defect_refinement_ratios=[rows[i]['integrated_local_GCL_defect_max']/rows[i+1]['integrated_local_GCL_defect_max'] for i in range(2)],
                accepted_advancing_steps=0,quad_bound_theorem=False)


if __name__=='__main__':
    print(json.dumps(study(),indent=2))
