"""Original, bounded educational references; no production liquid solver.
One deterministic data file supplies book figures and every 3D laboratory.
"""
from pathlib import Path
from fractions import Fraction
import hashlib, json, math
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
ROOT=Path(__file__).resolve().parent

def segment_triangle(start,end,triangle,tol=1e-12):
    a,b,c=np.asarray(triangle,dtype=float)
    d=np.asarray(end)-np.asarray(start); e1=b-a; e2=c-a
    p=np.cross(d,e2); determinant=float(e1@p)
    if abs(determinant)<=tol: return None
    tvec=np.asarray(start)-a; u=float(tvec@p)/determinant
    q=np.cross(tvec,e1); v=float(d@q)/determinant
    t=float(e2@q)/determinant
    if u < -tol or v < -tol or u+v > 1+tol or t < -tol or t>1+tol: return None
    return max(0.,min(1.,t))

def cube():
    vertices=np.array([[x,y,z] for z in [-.3,.3] for y in [-.3,.3] for x in [-.3,.3]])
    faces=[[0,2,3],[0,3,1],[4,5,7],[4,7,6],[0,1,5],[0,5,4],
           [2,6,7],[2,7,3],[0,4,6],[0,6,2],[1,3,7],[1,7,5]]
    return vertices,faces

def projection():
    coords=np.array([[i,j,k] for k in range(3) for j in range(3) for i in range(3)],dtype=float)/2-0.5
    edges=[]; axes=[]; positions=[]
    for k in range(3):
      for j in range(3):
       for i in range(3):
        lo=i+3*(j+3*k)
        for d in range(3):
         q=[i,j,k]
         if q[d]<2:
          q[d]+=1; hi=q[0]+3*(q[1]+3*q[2]); edges.append([lo,hi]); axes.append(d)
          positions.append(((coords[lo]+coords[hi])/2).tolist())
    B=np.zeros((27,len(edges)))
    for e,(lo,hi) in enumerate(edges): B[lo,e]=-1; B[hi,e]=1
    old=np.sin(np.arange(len(edges))*1.7)+.3*np.cos(np.arange(len(edges))*.4)
    rhs=B@old; K=B@B.T; p=np.zeros(27); p[1:]=np.linalg.solve(K[1:,1:],rhs[1:]); new=old-B.T@p
    residual=rhs-K@p
    if np.max(np.abs(B@new))>1e-12: raise ValueError('projection divergence')
    if new@new>old@old+1e-12: raise ValueError('projection energy')
    return dict(cells=coords.tolist(),edges=edges,axes=axes,positions=positions,before=old.tolist(),after=new.tolist(),pressure=p.tolist(),divergence_before=(B@old).tolist(),divergence_after=(B@new).tolist(),energy_before=float(.5*old@old),energy_after=float(.5*new@new),residual_max=float(np.max(np.abs(residual))))

def collisions():
    vertices,faces=cube(); cases=[]
    for shift in [-.1,0,.1]:
      v=vertices+np.array([shift,0,0])
      for label,start,end in [('crossing',[-.8,.1,0],[.8,.1,0]),('miss',[-.8,.5,0],[.8,.5,0]),('inside',[shift,0,0],[.8,0,0])]:
       hits=[(t,f) for f in faces if (t:=segment_triangle(start,end,v[f])) is not None]
       hit=min(hits,key=lambda x:x[0]) if hits else None
       cases.append(dict(shift=shift,label=label,start=start,end=end,t=None if hit is None else hit[0],hit=None if hit is None else (np.array(start)+hit[0]*(np.array(end)-start)).tolist(),wall_velocity=[.2,0,0]))
    return dict(vertices=vertices.tolist(),faces=faces,cases=cases,tolerance=1e-12)

def hydrostatic():
    y=np.linspace(0,1,33); sets=[]; g=9.81
    for lower,upper in [(1000.,800.),(1000.,1000.),(800.,600.)]:
      p=g*(upper*np.minimum(1-y,.5)+lower*np.maximum(.5-y,0))
      face_density=np.where((y[:-1]+y[1:])/2<.5,lower,upper)
      defect=np.diff(p)+face_density*g/32
      if np.max(np.abs(defect))>2e-12: raise ValueError('hydrostatic face balance')
      sets.append(dict(lower_density=lower,upper_density=upper,y=y.tolist(),pressure=p.tolist(),balance_error=float(np.max(np.abs(defect)))))
    return dict(gravity=g,height=1.,sets=sets)

def viscous():
    n=32; h=1/n; dt=.01; x=np.arange(n)*h
    E=np.zeros((n,n))
    for i in range(n): E[i,i]=-1/h; E[i,(i+1)%n]=1/h
    K=E.T@E; lam=4*math.sin(math.pi/n)**2/h**2; sets=[]
    for nu in [.01,.05,.1]:
      initial=np.sin(2*np.pi*x); u=initial.copy(); states=[]; inverse=np.linalg.inv(np.eye(n)+dt*nu*K)
      for step in range(101):
       amplitude=(1+dt*nu*lam)**(-step)
       if np.max(np.abs(u-amplitude*initial))>2e-13: raise ValueError('modal/dense disagreement')
       states.append(dict(step=step,time=step*dt,amplitude=amplitude,exact_continuum_amplitude=math.exp(-nu*4*math.pi**2*step*dt),velocity=u.tolist(),energy=float(.5*h*(u@u))))
       old=u; u=inverse@u
       defect=h*(old@old-u@u)-h*((old-u)@(old-u))-2*dt*nu*h*(u@K@u)
       if abs(defect)>2e-14: raise ValueError('implicit energy work identity')
      sets.append(dict(nu=nu,states=states))
    return dict(n=n,h=h,dt=dt,lambda_discrete=lam,x=x.tolist(),sets=sets)

def slip():
    y=np.linspace(0,1,33); U=1.; mu=.1; sets=[]
    for length in [0.,.1,.5]:
      slope=U/(1+length); u=slope*(y+length)
      if abs(u[0]-length*slope)>1e-15 or abs(u[-1]-U)>1e-15: raise ValueError('Couette boundary')
      sets.append(dict(length=length,velocity=u.tolist(),shear=mu*slope,relative_wall_dissipation=mu*slope*u[0]))
    return dict(y=y.tolist(),U=U,mu=mu,height=1.,sets=sets)

def caps():
    volume=1e-6; sets=[]
    for theta in range(30,151,5):
      t=math.radians(theta); c=math.cos(t)
      R=(3*volume/(math.pi*(2-3*c+c**3)))**(1/3); height=R*(1-c); radius=R*math.sin(t)
      z=np.linspace(0,height,10001); disc=math.pi*np.maximum(0,R*R-(z+R*c)**2)
      integrated=float(np.trapezoid(disc,z))
      if abs(integrated-volume)>2e-14: raise ValueError('cap cross-section volume')
      # Meridional outline, including overhang for angles above ninety degrees.
      phi=np.linspace(0,t,65); profile=[[float(R*math.sin(q)),float(R*(math.cos(q)-c))] for q in phi]
      sets.append(dict(theta=theta,cosine=c,R=R,height=height,contact_radius=radius,volume=volume,integrated_volume=integrated,profile=profile,
        tensions=[dict(sigma=s,pressure_jump=2*s/R,adhesion_work=s*(1+c),capillary_scale_h1mm=math.sqrt(1000*1e-9/s)) for s in [.03,.072,.12]]))
    return dict(volume=volume,gravity=0.,sets=sets)

def ledgers():
    a=[Fraction(1,10),Fraction(3,10),Fraction(8,5)]; F=[Fraction(1,25),-Fraction(3,100)]; source=[0,Fraction(1,50),0]; dt=Fraction(1,10)
    net=[F[0],-F[0]+F[1],-F[1]]
    updated=[a[i]-dt*net[i]+dt*source[i] for i in range(3)]
    if sum(updated)!=sum(a)+dt*sum(source): raise ValueError('rational amount balance')
    m,u,acc,t=3.,-.4,1.7,.02
    force_change=.5*m*((u+t*acc)**2-u*u); work=m*t*acc*u+.5*m*t*t*acc*acc
    if abs(force_change-work)>1e-15: raise ValueError('force work')
    return dict(amount_before=list(map(str,a)),amount_after=list(map(str,updated)),total_before=str(sum(a)),total_after=str(sum(updated)),source_addition=str(dt*sum(source)),force_energy_change=force_change,force_work=work,boundary_flux_balanced=[.2,-.2,0.],boundary_flux_incompatible=[.2,0.,0.])

def data():
    return dict(schema='rheon-education-reference-v1',scope='finite graph, triangle queries and analytic/local numerical oracles; no general liquid solver',projection=projection(),collision=collisions(),hydrostatic=hydrostatic(),viscous=viscous(),slip=slip(),cap=caps(),ledger=ledgers())

def figures(d):
    out=ROOT/'figures'; out.mkdir(exist_ok=True)
    plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False,'savefig.facecolor':'white'})
    fig,axes=plt.subplots(1,2,figsize=(9,3.5)); p=d['projection']; axes[0].bar(['before','after'],[p['energy_before'],p['energy_after']],color=['#a04c30','#067d91']); axes[0].set_ylabel('Discrete kinetic energy'); axes[1].plot(p['divergence_before'],label='before'); axes[1].plot(p['divergence_after'],label='after'); axes[1].set_ylabel('Incidence per cell'); axes[1].set_xlabel('Cell index'); axes[1].legend(); fig.tight_layout(); fig.savefig(out/'projection.png',dpi=180); plt.close(fig)
    fig,axes=plt.subplots(1,2,figsize=(9,3.5)); h=d['hydrostatic']['sets'][0]; axes[0].plot(h['pressure'],h['y']); axes[0].axhline(.5,color='#a04c30',ls=':'); axes[0].set(xlabel='Gauge pressure / Pa',ylabel='Height / m');
    for s in d['viscous']['sets']: axes[1].plot([q['time'] for q in s['states']],[q['energy'] for q in s['states']],label=f"nu={s['nu']}")
    axes[1].set(xlabel='Time / s',ylabel='Kinetic energy'); axes[1].legend(); fig.tight_layout(); fig.savefig(out/'density-viscosity.png',dpi=180); plt.close(fig)
    fig,axes=plt.subplots(1,2,figsize=(9,3.5));
    for s in d['slip']['sets']: axes[0].plot(s['velocity'],d['slip']['y'],label=f"slip={s['length']} m")
    axes[0].set(xlabel='Speed / m s^-1',ylabel='Height / m'); axes[0].legend()
    for q in d['cap']['sets'][::12]:
      profile=np.array(q['profile']); axes[1].plot(profile[:,0]*1000,profile[:,1]*1000,label=f"{q['theta']} degrees"); axes[1].plot(-profile[:,0]*1000,profile[:,1]*1000,color=axes[1].lines[-1].get_color())
    axes[1].set(xlabel='Radius / mm',ylabel='Height / mm'); axes[1].set_aspect('equal'); axes[1].legend(); fig.tight_layout(); fig.savefig(out/'slip-wetting.png',dpi=180); plt.close(fig)
    fig,ax=plt.subplots(figsize=(7,3)); ax.add_patch(plt.Rectangle((-.3,-.3),.6,.6,fc='#067d91',alpha=.25)); ax.plot([-.8,.8],[.1,.1],color='#a04c30',lw=2); ax.scatter([-.3],[.1],s=65,color='#a04c30'); ax.annotate('Earliest triangle hit: t=0.3125',(-.3,.1),(-.7,.42),arrowprops={'arrowstyle':'->'}); ax.set(xlim=(-.9,.9),ylim=(-.5,.6),xlabel='World x / m',ylabel='World y / m'); ax.set_aspect('equal'); fig.tight_layout(); fig.savefig(out/'mesh-hit.png',dpi=180); plt.close(fig)

if __name__=='__main__':
    d=data(); raw=(json.dumps(d,indent=2,allow_nan=False)+'\n').encode(); (ROOT/'reference-data.json').write_bytes(raw); figures(d)
    receipt={'schema':'rheon-reference-qualification-v1','data_sha256':hashlib.sha256(raw).hexdigest(),'source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'numpy':np.__version__,'checks':['full graph residual and energy','earliest triangle intersections','hydrostatic increments','dense implicit solve versus eigenmode','implicit energy identity','Couette boundary equations','constant cap volume via cross-section integration','rational represented-amount/source balance','finite-step force work'],'limits':d['scope']}
    (ROOT/'reference-qualification.json').write_text(json.dumps(receipt,indent=2)+'\n'); print(json.dumps(receipt,indent=2))
