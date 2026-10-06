"""Independent exact rational third momentum, both shears and work identities."""
from fractions import Fraction as Q
import json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'coupled-equation-consistency'))
import taylor
import reference
b=taylor.b;m=taylor.m
def require(ok,s):
 if not ok:raise ValueError(s)
def derive(kind):
 d=taylor.derive(kind);s=taylor.operators(d['q'],d['eta'],d['tangent_unknowns'][:6]);z=taylor.lift(s,d['eta'],d['tangent_unknowns'][:6]);flux,_,_=taylor.flux(s,z)
 r=[s['Rn'][3*i][:12]for i in range(16)];one=[Q(1)]*12
 require(b.mv(r,one)==[Q(1)]*16,'exact constant third embedding')
 mass=taylor.coeff(s['mass'],0);mdot=taylor.coeff(s['mass'],1);M=b.zeros(12,12);Kx=b.zeros(12,12);Ky=b.zeros(12,12)
 for i in range(16):
  for j in range(12):
   for k in range(12):M[j][k]+=mass[i]*r[i][j]*r[i][k]
 for t,tri in enumerate(s['micro']):
  for axis,K in enumerate([Kx,Ky]):
   e=[sum(s['gradients'][t][i][axis].c[0]*r[m.IDS[tri[i]]][j]for i in range(3))for j in range(12)]
   for j in range(12):
    for k in range(12):K[j][k]+=Q(1,20)*s['areas'][t].c[0]*e[j]*e[k]
 require(b.mv(Kx,one)==[Q(0)]*12 and b.mv(Ky,one)==[Q(0)]*12,'both exact shear nullspaces')
 K=[[x+y for x,y in zip(a,c)]for a,c in zip(Kx,Ky)];rows=[]
 for field in ['nonconstant','constant']:
  xi=[Q(float(x))for x in reference.initial_xi(field)];w=b.mv(r,xi);c=[Q(0)]*16;mix=Q(0)
  for(i,j),f in flux.items():
   f=f.c[0];g=f*(w[i]if f>=0 else w[j]);c[i]+=g;c[j]-=g;mix+=abs(f)*(w[i]-w[j])**2/2
  force=[x+y for x,y in zip(b.mv(b.transpose(r),[a*ww+cc for a,ww,cc in zip(mdot,w,c)]),b.mv(K,xi))];dxi=b.solve(M,[-x for x in force]);dw=b.mv(r,dxi)
  momentum=sum(a*ww+mm*dd for a,ww,mm,dd in zip(mdot,w,mass,dw));tdot=sum(a*ww**2/2+mm*ww*dd for a,ww,mm,dd in zip(mdot,w,mass,dw));sx=b.dot(xi,b.mv(Kx,xi));sy=b.dot(xi,b.mv(Ky,xi))
  require(momentum==0 and tdot+mix+sx+sy==0,'exact conservative third momentum and work')
  if field=='constant':require(dxi==[Q(0)]*12 and mix==sx==sy==0,'exact constant third preservation')
  else:require(sx>0 and sy>0,'both nonconstant shears exercised')
  rows.append(dict(field=field,initial_coefficients=xi,instantaneous_derivative=dxi,momentum_rate=momentum,energy_rate=tdot,donor_loss=mix,shear_x_power=sx,shear_y_power=sy,work_identity=tdot+mix+sx+sy))
 return dict(kind=kind,rows=rows,scalar_embedding=r,mass_matrix=M,shear_x_matrix=Kx,shear_y_matrix=Ky,pressure_third_force=0)
if __name__=='__main__':print(json.dumps(taylor.serial(dict(scope='exact semidiscrete third block of reviewed periodic z-invariant model; conditional algebra, no advancing IEEE or continuum theorem',rows=[derive(k)for k in ['initial','pressure_state']])),indent=2))
