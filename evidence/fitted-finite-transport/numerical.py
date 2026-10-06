"""Dense research BE replay with actual finite flux and time-dependent ODE reference.

No native run is claimed. References retain trace reactions and changing mass;
velocity convexity is deliberately tested as a failing claim, not assumed.
"""
from functools import lru_cache
import json
import numpy as np
import sympy as sp
from scipy.integrate import solve_ivp
from reference import symbolic_path,spatial,integrated_flux,q,ref,require

@lru_cache(maxsize=1)
def functions():
    s=symbolic_path();pairs=list(s['flux']);pfn=sp.lambdify(q,s['points'],'numpy');mfn=sp.lambdify(q,s['mass'],'numpy');dmfn=sp.lambdify(q,[sp.diff(m,q)/4 for m in s['mass']],'numpy');ffn=sp.lambdify(q,list(s['flux'].values()),'numpy');R=s['R'];ids=np.array(s['ids']);tri=np.array(s['micro']);pid=ids[tri]
    # Real antiderivatives are evaluated at high precision before subtracting;
    # a float logarithm difference is not used as the independent flux reference.
    import mpmath as mp
    primitive=sp.lambdify(q,list(s['primitive'].values()),'mpmath')
    return s,pairs,pfn,mfn,dmfn,ffn,R,ids,tri,pid,primitive

def actual(offset):
    s,pairs,pfn,mfn,dmfn,ffn,R,ids,tri,pid,_=functions();p=np.array(pfn(offset),float);v=p[tri];ab=v[:,1]-v[:,0];ac=v[:,2]-v[:,0];a2=ab[:,0]*ac[:,1]-ab[:,1]*ac[:,0];require(np.all(a2>0),'positive numerical triangle geometry');g=np.empty((len(tri),3,2))
    for i in range(3):
        j,k=(i+1)%3,(i+2)%3;g[:,i,0]=(v[:,j,1]-v[:,k,1])/a2;g[:,i,1]=(v[:,k,0]-v[:,j,0])/a2
    kel=.05*.5*a2[:,None,None]*np.einsum('tik,tjk->tij',g,g);K=np.zeros((32,32))
    for i in range(3):
        for j in range(3):np.add.at(K,(pid[:,i],pid[:,j]),kel[:,i,j])
    return np.array(mfn(offset),float),np.array(dmfn(offset),float),K,{ij:float(f)/4 for ij,f in zip(pairs,ffn(offset))},R,p

@lru_cache(maxsize=128)
def finite(left,right):
    import mpmath as mp
    _,pairs,*_,primitive=functions()
    with mp.workdps(60):
        a=mp.mpf(left.numerator)/left.denominator;b=mp.mpf(right.numerator)/right.denominator;fa=primitive(a);fb=primitive(b)
        return {ij:float(y-x)for ij,x,y in zip(pairs,fa,fb)}

def donor(mass,flux):
    A=np.diag(mass)
    for (i,j),f in flux.items():
        if f>0:A[i,i]+=f;A[j,i]-=f
        else:A[j,j]-=f;A[i,j]+=f
    return A

def initial():
    m,_,_,_,R,p=actual(0.);ids=symbolic_path()['ids'];free=[int(np.argmax(R[:,j]))for j in range(R.shape[1])];z=np.array([next(p[i,1] for i,k in enumerate(ids) if k==node)**2 for node in free]);return z

@lru_cache(maxsize=1)
def ode_reference():
    z=initial()
    def rhs(t,z):
        m,dm,K,f,R,_=actual(.25*t);u=R@z;c=donor(np.zeros(32),f)@u
        return np.linalg.solve(R.T@(m[:,None]*R),-R.T@(dm*u+c+K@u))
    fine=solve_ivp(rhs,(0.,.5),z,method='DOP853',atol=2e-13,rtol=2e-13)
    tighter=solve_ivp(rhs,(0.,.5),z,method='DOP853',atol=2e-14,rtol=3e-14)
    require(fine.success and tighter.success,'actual time-dependent ALE ODE integration')
    require(np.max(np.abs(fine.y[:,-1]-tighter.y[:,-1]))<2e-11,'independent ODE tolerance refinement')
    return tighter.y[:,-1],fine.nfev,tighter.nfev

def study(dt):
    dt=ref.Q(dt);z=initial();R=symbolic_path()['R'];m0,_,_,_,_,_=actual(0.);initial_p=float(m0@(R@z));steps=[];offset=ref.Q(0)
    for n in range(int(ref.Q(1,2)/dt)):
        new_offset=offset+dt/4;oldm,_,_,_,_,_=actual(float(offset));m,_,K,_,_,p=actual(float(new_offset));F=finite(offset,new_offset);A=donor(m,F);old=R@z;operator=R.T@(A+float(dt)*K)@R;rhs=R.T@(oldm*old);z=np.linalg.solve(operator,rhs);v=R@z
        g=m-oldm
        for (i,j),f in F.items():g[i]+=f;g[j]-=f
        e0=float(oldm@(old*old)/2);e1=float(m@(v*v)/2);inc=float(oldm@((v-old)**2)/2);adv=sum(abs(f)*(v[i]-v[j])**2/2 for (i,j),f in F.items());strain=float(v@K@v);res=operator@z-rhs;work=e1-e0+inc+adv+float(dt)*strain;rw=float(z@res);gcl_work=float(g@(v*v)/2)
        require(abs(work-rw+gcl_work)<3e-15,'actual finite changing-mass donor/strain work')
        require(abs(float(m@v)-initial_p)<2e-12,'full trajectory conservative third momentum')
        require(np.max(np.abs(g))<2e-15,'actual integrated face GCL; no flux fitting')
        require(e1<e0 and adv>0 and strain>0,'genuine conservative transport and viscosity evolution')
        steps.append({'step':n+1,'time':float(dt*(n+1)),'offset':float(new_offset),'mass':float(sum(m)),'momentum_z':float(m@v),'energy_before_z':e0,'energy_after_z':e1,'increment_loss':inc,'advection_loss':float(adv),'strain_power':strain,'true_residual':float(np.linalg.norm(res)),'work_error':work-rw+gcl_work,'GCL_max':float(np.max(np.abs(g)))})
        offset=new_offset
    exact,_,_=ode_reference();d=z-exact;err=float(np.sqrt((R@d)@(m*(R@d))))
    return {'dt':float(dt),'temporal_L2_error':err,'max_momentum_drift':max(abs(s['momentum_z']-initial_p)for s in steps),'steps':steps,'velocity_z':list(v),'physical_nodes':p.tolist(),'mass':list(m)}

def limitations():
    s=symbolic_path();m0,_,_,_,R,_=actual(0.);m1,_,_,f,_,_=actual(.125);F=finite(ref.Q(0),ref.Q(1,8));A=donor(m1,F);Ar=R.T@A@R;mapfree=np.linalg.solve(Ar,R.T@(m0[:,None]*R));i,j=np.unravel_index(np.argmin(mapfree),mapfree.shape);old=R[:,j];new=R@mapfree[:,j]
    bad_g=m1-m0
    wrong_face_error=max(abs(F[ij]-.5*f[ij])for ij in F)
    for (i,j),f0 in f.items():bad_g[i]+=.5*f0;bad_g[j]-=.5*f0
    require(new.min() < -.01,'preserved failure: constrained velocity convex range')
    require(wrong_face_error>1e-4,'physical face integration rejects endpoint-only flux')
    require(np.max(np.abs(bad_g))<2e-15,'negative witness: incorrect physical face flux still passes mass marginals')
    return {'constrained_convex_range':'FAIL; no constrained positivity claim','old_velocity_range':[float(old.min()),float(old.max())],'new_velocity_range':[float(new.min()),float(new.max())],'negative_free_transfer_entries':int(np.sum(mapfree< -1e-14)),'minimum_free_transfer_coefficient':float(mapfree.min()),'symmetric_part_min_eigenvalue':float(np.linalg.eigvalsh((Ar+Ar.T)/2).min()),'endpoint_frozen_flux_GCL_max':float(np.max(np.abs(bad_g))),'endpoint_face_integral_error':wrong_face_error,'endpoint_flux_status':'FAIL physical face provenance despite passing GCL; not an accepted temporal integrator'}

if __name__=='__main__':
    result={'scope':'research actual finite ALE integration and constrained transport/viscosity; no native stepping','limitations':limitations(),'rows':[study(ref.Q(1,20)),study(ref.Q(1,40)),study(ref.Q(1,80))]};errors=[r['temporal_L2_error']for r in result['rows']];ratios=[errors[i]/errors[i+1]for i in range(2)];require(all(1.8<r<2.2 for r in ratios),'actual first-order nonautonomous temporal convergence');result['temporal_ratios']=ratios;result['ODE_function_evaluations']=ode_reference()[1:];print(json.dumps(result,indent=2))
