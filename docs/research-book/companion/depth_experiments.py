#!/usr/bin/env python3
"""Focused extension fixtures. Each tests a restricted mechanism, not a full solver."""
from pathlib import Path
import itertools,json
import numpy as np
from scipy import sparse
from scipy.sparse.linalg import spsolve
R=Path(__file__).resolve().parents[1]

def sampling():
    h=np.array([.2,.3,.4]);origin=np.array([-.1,.2,-.3]);x=np.array([.67,.84,1.12])
    coefficient=np.array([.7,-1.1,.4]);field=lambda p:1.2+coefficient@p
    def evaluate(stored_offset,sample_offset):
        s=(x-origin)/h-sample_offset;a=np.floor(s).astype(int);t=s-a
        total=0.
        for corner in itertools.product([0,1],repeat=3):
            c=np.array(corner);weight=np.prod(np.where(c,t,1-t))
            position=origin+h*(a+c+stored_offset)
            total+=weight*field(position)
        return total
    offsets={'cell':[.5,.5,.5],'x':[0,.5,.5],'y':[.5,0,.5],'z':[.5,.5,0]}
    errors={k:abs(evaluate(np.array(o),np.array(o))-field(x)) for k,o in offsets.items()}
    wrong=evaluate(np.array(offsets['x']),np.array(offsets['cell']))-field(x)
    assert max(errors.values())<1e-14 and abs(wrong+.07)<1e-14
    return {'affine_errors':errors,'wrong_x_offset_bias':wrong}
def rotation():
    def v(x):return np.array([-x[1],x[0]])
    rows=[]
    for steps in [10,20,40,80]:
        dt=1/steps;e=np.array([1.,0.]);m=e.copy()
        for _ in range(steps):
            e=e-dt*v(e)
            m=m-dt*v(m-.5*dt*v(m))
        exact=np.array([np.cos(1),-np.sin(1)])
        rows.append({'steps':steps,'euler_error':float(np.linalg.norm(e-exact)),
                     'midpoint_error':float(np.linalg.norm(m-exact))})
    return rows
def multigrid():
    n=31;A=sparse.diags([-np.ones(n-1),2*np.ones(n),-np.ones(n-1)],[-1,0,1]).toarray()
    P=np.zeros((31,15))
    for j in range(15):
        P[2*j+1,j]=1;P[2*j,j]=.5;P[2*j+2,j]=.5
    H=P.T@A@P
    assert np.max(abs(H-H.T))<1e-14 and np.linalg.eigvalsh(H).min()>0
    x=np.arange(1,n+1)/(n+1)
    exact=np.sin(np.pi*x)+.2*np.sin(13*np.pi*x);b=A@exact
    def cycle(u):
        for _ in range(3):u=u+(2/3)*(b-A@u)/np.diag(A)
        residual=b-A@u
        u=u+P@np.linalg.solve(H,P.T@residual)
        for _ in range(3):u=u+(2/3)*(b-A@u)/np.diag(A)
        return u
    u=np.zeros(n);history=[float(np.linalg.norm(b))]
    for _ in range(8):u=cycle(u);history.append(float(np.linalg.norm(b-A@u)))
    assert history[-1]<history[0]*1e-6
    z=np.arange(15)/15;e=P@z;correction=P@np.linalg.solve(H,P.T@(A@e))
    assert np.linalg.norm(e-correction)<1e-12
    return {'fine_unknowns':n,'coarse_unknowns':15,'coarse_min_eigenvalue':float(np.linalg.eigvalsh(H).min()),
            'residual_history':history,'coarse_space_correction_error':float(np.linalg.norm(e-correction))}
def plic_axis():
    fraction=.3;courant=.5
    overlap=lambda a,b,c,d:max(0.,min(b,d)-max(a,c))
    left=overlap(0,fraction,1-courant,1)
    right=overlap(1-fraction,1,1-courant,1)
    assert left==0 and abs(right-.3)<1e-15
    return {'fraction':fraction,'courant':courant,'left_filled_outflux':left,
            'right_filled_outflux':right,'uniform_fraction_flux':fraction*courant}
def curvature():
    radius=.5;p=np.array([radius,0.,0.]);rows=[]
    phi=lambda x:np.linalg.norm(x)-radius
    for h in [.1,.05,.025,.0125]:
        grad=np.zeros(3);H=np.zeros((3,3))
        for i in range(3):
            ei=np.eye(3)[i]*h
            grad[i]=(phi(p+ei)-phi(p-ei))/(2*h)
            H[i,i]=(phi(p+ei)-2*phi(p)+phi(p-ei))/h**2
            for j in range(i):
                ej=np.eye(3)[j]*h
                H[i,j]=H[j,i]=(phi(p+ei+ej)-phi(p+ei-ej)-phi(p-ei+ej)+phi(p-ei-ej))/(4*h*h)
        norm=np.linalg.norm(grad);k=(norm**2*np.trace(H)-grad@H@grad)/norm**3
        rows.append({'h':h,'curvature':float(k),'exact':2/radius,'error':float(abs(k-2/radius))})
    assert rows[-1]['error']<rows[0]['error']/50
    return rows
def viscosity():
    E=np.array([[-1,1,0,0],[0,-1,1,0],[0,0,-1,1]],float)
    mu=np.array([.1,10.,.5]);K=E.T@np.diag(mu)@E
    u=np.array([1.,-2.,3.,0.]);out=np.linalg.solve(np.eye(4)+.2*K,u)
    assert np.linalg.norm(K-K.T)==0
    assert np.linalg.eigvalsh(K).min()>-1e-12 and out@out<=u@u
    return {'eigenvalues':np.linalg.eigvalsh(K).tolist(),'energy_before':float(u@u/2),
            'energy_after':float(out@out/2),'constant_null_error':float(np.linalg.norm(K@np.ones(4)))}
def quadratic(r):
    a=abs(r)
    if a<.5:return .75-a*a
    if a<1.5:return .5*(1.5-a)**2
    return 0.
def apic():
    xp=np.array([.2,.3,.4]);a=np.array([.7,-.1,.5])
    C=np.array([[.2,-.4,.1],[.3,.2,-.6],[.1,.5,-.2]])
    nodes=np.array(list(itertools.product(range(-1,3),repeat=3)),float)
    weights=np.array([np.prod([quadratic(v) for v in x-xp]) for x in nodes])
    keep=weights>0;nodes=nodes[keep];weights=weights[keep];offset=nodes-xp
    D=np.einsum('i,ij,ik->jk',weights,offset,offset)
    velocity=np.array([a+C@x for x in nodes]);vp=weights@velocity
    H=np.einsum('i,ij,ik->jk',weights,velocity,offset);recovered=H@np.linalg.inv(D)
    first=weights@offset
    assert abs(weights.sum()-1)<1e-14 and np.linalg.norm(first)<1e-14
    assert np.linalg.norm(D-.25*np.eye(3))<1e-14
    assert np.linalg.norm(vp-(a+C@xp))<1e-14 and np.linalg.norm(recovered-C)<1e-14
    # Compare angular momentum of the represented affine particle with its grid deposit.
    grid_momentum=weights[:,None]*((a+C@xp)[None,:]+offset@C.T)
    Lgrid=np.cross(nodes,grid_momentum).sum(axis=0)
    Lparticle=np.cross(xp,a+C@xp)+np.einsum('i,ij->j',weights,np.cross(offset,offset@C.T))
    assert np.linalg.norm(Lgrid-Lparticle)<1e-14
    # Drop the largest support sample to expose moment loss, then normalize only.
    cut=np.delete(weights,np.argmax(weights));off=np.delete(offset,np.argmax(weights),axis=0)
    cut=cut/cut.sum();defect=np.linalg.norm(cut@off)
    assert defect>1e-3
    return {'support_nodes':len(weights),'weight_sum':float(weights.sum()),'first_moment_norm':float(np.linalg.norm(first)),
      'moment_matrix':D.tolist(),'affine_error':float(np.linalg.norm(recovered-C)),
      'angular_transfer_error':float(np.linalg.norm(Lgrid-Lparticle)),
      'truncated_renormalized_first_moment_defect':float(defect)}
def sph_gradient():
    h=1.;mass=.2;rho0=1.
    def W(r):
        s=r@r
        return 315/(64*np.pi*h**9)*(h*h-s)**3 if s<h*h else 0.
    def grad(r):
        s=r@r
        return -945/(32*np.pi*h**9)*(h*h-s)**2*r if s<h*h else np.zeros(3)
    x=np.array([[.1,.2,.3],[.45,.3,.2],[-.2,.35,.1]])
    def constraint(y):return sum(mass*W(y[0]-q) for q in y)/rho0-1
    analytical=np.zeros_like(x)
    for j in [1,2]:
        g=mass/rho0*grad(x[0]-x[j]);analytical[0]+=g;analytical[j]-=g
    numerical=np.zeros_like(x);eps=1e-6
    for i in range(3):
        for d in range(3):
            plus=x.copy();minus=x.copy();plus[i,d]+=eps;minus[i,d]-=eps
            numerical[i,d]=(constraint(plus)-constraint(minus))/(2*eps)
    error=np.max(abs(analytical-numerical))
    assert error<1e-8 and np.linalg.norm(analytical.sum(axis=0))<1e-14
    return {'constraint':float(constraint(x)),'gradient_linf_error':float(error),
            'translation_gradient_sum':analytical.sum(axis=0).tolist()}
def affine_boundary_adjoint():
    B=np.array([[-1.,0.],[1.,-1.],[0.,1.]])
    volume=np.array([1.,2.,.5]);area=np.array([.3,.8]);length=np.array([.4,.7])
    u=np.array([2.,-1.]);p=np.array([.4,-.7,1.2]);Q=np.array([.2,-.1,.05])
    D0=-(B@(area*u))/volume
    full=D0+Q/volume
    G=(B.T@p)/length
    lhs0=p@(volume*D0);rhs0=-G@(area*length*u)
    lhs=p@(volume*full);rhs=rhs0+p@Q
    assert abs(lhs0-rhs0)<1e-14 and abs(lhs-rhs)<1e-14
    defect=lhs-rhs0
    assert abs(defect)>.1 and abs(defect-p@Q)<1e-14
    return dict(homogeneous_identity_error=float(lhs0-rhs0),
      full_identity_error=float(lhs-rhs),omitted_boundary_term_defect=float(defect),
      pressure_boundary_work=float(p@Q))

def pressure_gradient_geometry():
    gradient=np.array([2.,3.,-.4]);normal=np.array([1.,0.,0.])
    origin=np.array([0.,0.,0.]);orthogonal=np.array([1.,0.,0.]);skew=np.array([1.,.4,0.])
    pressure=lambda x:.7+gradient@x
    exact=gradient@normal
    good=(pressure(orthogonal)-pressure(origin))/1.
    bad=(pressure(skew)-pressure(origin))/1.
    assert abs(good-exact)<1e-14 and abs(bad-exact-1.2)<1e-14
    return dict(normal_gradient=float(exact),orthogonal_two_point=float(good),
      skew_two_point=float(bad),skew_error=float(bad-exact))
def interface_normal_sign():
    # Unit cells centered at -0.5, 0.5, 1.5; liquid occupies x <= 0.3.
    centers=np.array([-.5,.5,1.5]);d=.3
    fractions=np.clip(d-(centers-.5),0.,1.)
    grad_f=(fractions[2]-fractions[0])/2.
    phi=centers-d;grad_phi=(phi[2]-phi[0])/2.
    fraction_normal=-np.sign(grad_f);levelset_normal=np.sign(grad_phi)
    assert fraction_normal==levelset_normal==1
    assert np.allclose(fractions,[1.,.3,0.])
    return dict(fractions=fractions.tolist(),fraction_gradient=float(grad_f),
      levelset_gradient=float(grad_phi),outward_normal=float(fraction_normal),
      wrong_positive_fraction_gradient_normal=float(np.sign(grad_f)))
def full_gauge_residual():
    B=np.array([[-1.,0.,0.],[1.,-1.,0.],[0.,1.,-1.],[0.,0.,1.]])
    u=np.array([.003,.002,.001]);b=B@u;pressure=np.zeros(4)
    residual=b-(B@B.T)@pressure;tol=.0015
    retained=float(np.max(abs(residual[1:])));full=float(np.max(abs(residual)))
    assert abs(b.sum())<1e-15 and retained<tol and full>tol
    return dict(compatible_sum=float(b.sum()),retained_max=retained,
      full_max=full,threshold=tol,reduced_would_pass=True,full_passes=False)

if __name__=='__main__':
    results={'pressure_gradient_geometry':pressure_gradient_geometry(),'interface_normal_sign':interface_normal_sign(),'full_gauge_residual':full_gauge_residual(),'affine_boundary_adjoint':affine_boundary_adjoint(),'sampling':sampling(),'rotation':rotation(),'multigrid':multigrid(),
      'plic_axis':plic_axis(),'curvature':curvature(),'viscosity':viscosity(),
      'apic':apic(),'sph_gradient':sph_gradient()}
    (R/'companion/depth-results.json').write_text(json.dumps(results,indent=2)+'\n')
    print(json.dumps(results,indent=2))
    print('PASS twelve restricted depth fixtures; no full liquid or particle solver claim')
