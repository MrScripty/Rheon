"""Independent exact spatial assembly, dense BE and semidiscrete exp reference.

Uses repaired rational geometry *values*, not its bottom-fixed motion diagnostic.
Here every node, including bottom tangential coordinates, moves at (a,0).
"""
import importlib.util
from functools import lru_cache
from pathlib import Path
import sys
import numpy as np
from scipy.linalg import eigh
ROOT=Path(__file__).resolve().parents[2]
PATH=ROOT/'evidence/fitted-height-periodic-repair/reference.py'
spec=importlib.util.spec_from_file_location('repaired_spatial_reference',PATH)
ref=importlib.util.module_from_spec(spec);sys.modules[spec.name]=ref;spec.loader.exec_module(ref)

def require(ok,message):
    if not ok: raise ValueError(message)

@lru_cache(maxsize=1)
def assembly():
    points,micro,ids,constraints,_=ref.mesh()
    # Replace the predecessor's moving-top/fixed-bottom Dual derivatives by
    # the material translation before testing the actual finite trajectory.
    points=[(ref.Dual(p[0].v,ref.Q(1,4)),ref.Dual(p[1].v,ref.Q(0))) for p in points]
    raw,rate,k,_,areas=ref.operators(points,micro,mu=ref.Q(1,20))
    require(all(x==0 for x in rate),'material mass rate zero')
    require(not ref.dual_flux(points,micro,ids),'actual material dual flux zero')
    mass=ref.merge(raw,ids);n=len(mass);r=ref.scalar_embedding(n,constraints);nr=len(r[0])
    km=ref.zeros(n,n)
    for i in range(len(points)):
        for j in range(len(points)):km[ids[i]][ids[j]]+=k[3*i+2][3*j+2]
    mr=[[sum(mass[i]*r[i][a]*r[i][b] for i in range(n)) for b in range(nr)] for a in range(nr)]
    kr=[[sum(r[i][a]*km[i][j]*r[j][b] for i in range(n) for j in range(n)) for b in range(nr)]for a in range(nr)]
    require(all(sum(row)==0 for row in kr),'exact constant nullspace')
    require(sum(mass)==ref.Q(57,16),'exact liquid mass')
    # A real finite rigid translation, not a linearized endpoint mass fit.
    translated=[(ref.Dual(p[0].v+ref.Q(1,8)),p[1]) for p in points]
    m2,rate2,k2,_,a2=ref.operators(translated,micro,mu=ref.Q(1,20))
    require(m2==raw and k2==k and a2==areas and all(x==0 for x in rate2),'finite rigid geometry invariance')
    free=[i for i in range(n) if i not in constraints]
    pos={ids[i]:(float(p[0].v),float(p[1].v)) for i,p in enumerate(points)}
    z0=np.array([pos[i][1]**2 for i in free]);R=np.array(r,float)
    return {'points':np.array([[float(p[0].v),float(p[1].v)]for p in points]),'triangles':micro,'ids':ids,'mass':np.array(mass,float),'R':R,'M':np.array(mr,float),'K':np.array(kr,float),'z0':z0,'free':free}

def exact_semidiscrete(a,t):
    eig,v=eigh(a['K'],a['M']);c=v.T@a['M']@a['z0']
    return v@(np.exp(-t*eig)*c)

if __name__=='__main__':
    import json
    a=assembly();rows=[]
    for dt in [.05,.025,.0125]:
        z=a['z0'].copy();maximum=0.;p0=float(np.sum(a['mass']*(a['R']@z)))
        for _ in range(round(.5/dt)):
            old=z.copy();z=np.linalg.solve(a['M']+dt*a['K'],a['M']@old)
            maximum=max(maximum,abs(.5*(z@a['M']@z-old@a['M']@old)+.5*((z-old)@a['M']@(z-old))+dt*z@a['K']@z))
        d=z-exact_semidiscrete(a,.5)
        rows.append({'dt':dt,'temporal_L2_error':float(np.sqrt(d@a['M']@d)),'momentum_drift':abs(float(np.sum(a['mass']*(a['R']@z)))-p0),'max_BE_work_error':maximum})
    require(rows[0]['temporal_L2_error']/rows[1]['temporal_L2_error']>1.8 and rows[1]['temporal_L2_error']/rows[2]['temporal_L2_error']>1.8,'first-order temporal study')
    print(json.dumps({'reference':'corrected periodic rational spatial assembly + independently integrated material trajectory','rows':rows,'scope':'semidiscrete temporal accuracy; no continuum spatial or general pressure/remap qualification'},indent=2))
