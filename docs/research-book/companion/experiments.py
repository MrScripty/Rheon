#!/usr/bin/env python3
"""Reference numerical experiments, not a production solver."""
from pathlib import Path
from fractions import Fraction as F
import json,sys,platform
import numpy as np
import scipy
from scipy import sparse
from scipy.sparse.linalg import cg,spsolve
ROOT=Path(__file__).resolve().parents[1]
def box_incidence(n):
    rows=[];cols=[];data=[];e=0
    cell=lambda i,j,k:i+n*(j+n*k)
    for k in range(n):
        for j in range(n):
            for i in range(n):
                for d in ((1,0,0),(0,1,0),(0,0,1)):
                    a=(i+d[0],j+d[1],k+d[2])
                    if max(a)>=n:continue
                    rows.extend([cell(i,j,k),cell(*a)])
                    cols.extend([e,e]);data.extend([-1.,1.]);e+=1
    return sparse.csr_matrix((data,(rows,cols)),shape=(n**3,e))
def sl(q,c):
    departure=np.arange(len(q))-c
    lo=np.floor(departure).astype(int);f=departure-lo
    return (1-f)*q[lo%len(q)]+f*q[(lo+1)%len(q)]
def corrected_transport(q,c):
    forward=sl(q,c);backward=sl(forward,-c)
    candidate=forward+0.5*(q-backward)
    lo=np.floor(np.arange(len(q))-c).astype(int)
    a=q[lo%len(q)];b=q[(lo+1)%len(q)]
    return np.where((candidate>=np.minimum(a,b))&(candidate<=np.maximum(a,b)),candidate,forward)
def projection_experiment():
    B=box_incidence(8);A=(B@B.T).tocsr()
    rng=np.random.default_rng(20261003);u=rng.normal(size=B.shape[1]);dt=.025;b=B@u/dt
    reduced=A[1:,1:];rhs=b[1:];histories={'cg':[],'jacobi':[]}
    def record(p):histories['cg'].append(float(np.linalg.norm(rhs-reduced@p)))
    psmall,info=cg(reduced,rhs,rtol=1e-11,atol=0,maxiter=2000,callback=record)
    assert info==0
    p=np.r_[0.,psmall];res=b-A@p;new=u-dt*(B.T@p)
    rdiff=np.max(abs(B@new-dt*res))
    q=np.ones(len(b));flux=rng.normal(size=B.shape[1]);qnext=q+dt*(B@flux)
    jac=np.zeros(len(rhs));diag=reduced.diagonal()
    for _ in range(300):
        jac+=(2/3)*(rhs-reduced@jac)/diag
        histories['jacobi'].append(float(np.linalg.norm(rhs-reduced@jac)))
    assert np.max(abs(np.asarray(B.sum(axis=0))))==0
    assert rdiff<1e-12 and np.linalg.norm(B@new)<1e-7
    assert abs(q.sum()-qnext.sum())<1e-12
    assert np.linalg.norm(A@np.ones(A.shape[0]))==0
    B32=B.astype(np.float32);p32=p.astype(np.float32);u32=u.astype(np.float32);dt32=np.float32(dt)
    residual32=B32@u32/dt32-(B32@B32.T)@p32
    new32=u32-dt32*(B32.T@p32)
    return dict(grid=[8,8,8],cells=B.shape[0],internal_faces=B.shape[1],
      cg_iterations=len(histories['cg']),initial_divergence_l2=float(np.linalg.norm(B@u)),
      final_divergence_l2=float(np.linalg.norm(B@new)),residual_identity_linf_error_f64=float(rdiff),
      residual_identity_linf_error_f32=float(np.max(abs(B32@new32-dt32*residual32))),
      energy_before=float(u@u/2),energy_after=float(new@new/2),
      mass_before=float(q.sum()),mass_after=float(qnext.sum()),histories=histories)
def transport_experiment():
    result=[]
    for n in [32,64,128,256]:
        x=np.arange(n)/n;q0=np.sin(2*np.pi*x);q=q0.copy();m=q0.copy()
        for _ in range(2*n):q=sl(q,.5);m=corrected_transport(m,.5)
        result.append(dict(n=n,steps=2*n,sl_l2=float(np.linalg.norm(q-q0)/np.sqrt(n)),
          limited_corrected_l2=float(np.linalg.norm(m-q0)/np.sqrt(n)),
          sl_amplitude=float((q.max()-q.min())/2),corrected_amplitude=float((m.max()-m.min())/2)))
    q=np.zeros(64);q[20:36]=1;mass=float(q.sum())
    for _ in range(128):q=sl(q,.5)
    assert q.min()>=0 and q.max()<=1 and abs(q.sum()-mass)<1e-12
    W=np.array([[.5,.5,0],[.5,.5,0],[.5,.5,0]])
    assert (W@np.array([1.,0.,0.])).sum()==1.5
    return dict(sine_period=result,box_after=q.tolist(),box_min=float(q.min()),
      box_max=float(q.max()),box_mass_error=float(q.sum()-mass),
      nonconservative_W=W.tolist(),mass_counterexample=[1,1.5])
def diffusion_experiment():
    n=64;x=np.arange(n)/n;q=np.sin(2*np.pi*x);nu=.01;dt=.001;steps=100
    L=sparse.diags([-np.ones(n-1),2*np.ones(n),-np.ones(n-1)],[-1,0,1],format='lil')
    L[0,-1]=-1;L[-1,0]=-1;L=L.tocsr()*n*n
    system=sparse.eye(n)+nu*dt*L;energy=[]
    for _ in range(steps):q=spsolve(system,q);energy.append(float(q@q))
    assert all(a>=b for a,b in zip(energy,energy[1:]))
    amp=float((q.max()-q.min())/2);analytic=float(np.exp(-nu*(2*np.pi)**2*dt*steps))
    return dict(n=n,nu=nu,dt=dt,steps=steps,amplitude=amp,
      continuum_amplitude=analytic,amplitude_error=abs(amp-analytic),energy=energy)
def memory():
    return [dict(n=n,cells=n**3,staggered_faces=3*n*n*(n+1),
      layout_bytes=4*(2*3*n*n*(n+1)+8*n**3)+n**3,
      apic_particle_bytes_8_per_cell=8*n**3*64) for n in [32,64,96,128,256]]
def exact_checks():
    q=[F(1,3),F(1,2),F(1,6)];f=[F(2,7),F(-1,5),F(1,9)];dt=F(1,10)
    updated=[q[i]+dt*(f[(i-1)%3]-f[i]) for i in range(3)]
    assert sum(updated)==sum(q)==1
    assert F(1)-F(3,2)*(F(1)-F(0))==F(-1,2)
    return dict(periodic_mass=str(sum(updated)),cfl_failure='-1/2',
      updated_rationals=[str(x) for x in updated])
if __name__=='__main__':
    data=dict(environment=dict(python=sys.version.split()[0],numpy=np.__version__,scipy=scipy.__version__,
      platform=platform.platform()),exact=exact_checks(),projection=projection_experiment(),
      transport=transport_experiment(),diffusion=diffusion_experiment(),memory=memory())
    (ROOT/'companion/results.json').write_text(json.dumps(data,indent=2)+'\n')
    print('PASS projection, transport, diffusion, rational and memory checks')
    print(json.dumps({k:v for k,v in data['projection'].items() if k!='histories'},indent=2))
