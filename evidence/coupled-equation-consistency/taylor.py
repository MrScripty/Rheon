"""Independent exact rational Taylor assembly of the native/host spatial model.

Reuses only frozen topology/embedding and initial field definitions. Reconstructs
physical PS geometry, gradients, mass, full strain, pressure pairing and flux.
No NumPy force arrays or finite differences enter the coefficients.
"""
from dataclasses import dataclass
from fractions import Fraction as Q
from functools import lru_cache
import json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'fitted-discrete-refinement'))
import temporal
m=temporal.m;b=m.base

def require(ok,message):
 if not ok:raise ValueError(message)
@dataclass(frozen=True)
class Jet:
 c:tuple
 def __init__(self,a=0,b=0,c=0):object.__setattr__(self,'c',(Q(a),Q(b),Q(c)))
 def __add__(self,other):
  other=other if isinstance(other,Jet)else Jet(other)
  return Jet(*(a+b for a,b in zip(self.c,other.c)))
 __radd__=__add__
 def __neg__(self):return Jet(*(-x for x in self.c))
 def __sub__(self,o):return self+-asjet(o)
 def __rsub__(self,o):return asjet(o)+-self
 def __mul__(self,o):
  o=asjet(o);return Jet(*(sum(self.c[j]*o.c[i-j]for j in range(i+1))for i in range(3)))
 __rmul__=__mul__
 def __truediv__(self,o):
  o=asjet(o);require(o.c[0]!=0,'nonzero exact series divisor');r=[]
  for i in range(3):r.append((self.c[i]-sum(o.c[j]*r[i-j]for j in range(1,i+1)))/o.c[0])
  return Jet(*r)
 def derivative(self):return Jet(self.c[1],2*self.c[2])
 def __rtruediv__(self,o):return asjet(o)/self

def asjet(x):return x if isinstance(x,Jet)else Jet(x)
def add(a,b):return tuple(x+y for x,y in zip(a,b))
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def scale(a,s):return tuple(x*s for x in a)
def cross(a,b):return a[0]*b[1]-a[1]*b[0]
def zeros(n,k):return [[Jet()for _ in range(k)]for _ in range(n)]
def mv(a,x):return [sum((v*w for v,w in zip(row,x)),Jet())for row in a]
def at(a,k):return [[v.c[k]for v in row]for row in a]
def coeff(x,k):return [v.c[k]for v in x]
def normmax(x):return max(map(abs,x))

def geometry(q,eta,alpha):
 capq=[Jet(q[i],eta[i],alpha[i]/2)for i in range(3)]
 cap=[(capq[0],capq[2]),(capq[1],Q(9,4)-capq[2]),(capq[0]+1,capq[2])]
 points=[(Jet(Q(i,2)),Jet())for i in range(3)]+cap
 macro=[(0,1,4),(0,4,3),(1,2,5),(1,5,4)];centers=[]
 for tri in macro:centers.append(len(points));points.append(scale(add(add(points[tri[0]],points[tri[1]]),points[tri[2]]),Q(1,3)))
 adjacent={}
 for t,tri in enumerate(macro):
  for k in range(3):adjacent.setdefault(tuple(sorted((tri[k],tri[(k+1)%3]))),[]).append(t)
 left,right=(0,3),(2,5);adjacent[left].append(adjacent[right][0]);adjacent[right].append(adjacent[left][0]);edges={}
 for edge,owners in sorted(adjacent.items()):
  a,c=(points[i]for i in edge)
  if len(owners)==2:
   p,r=(points[centers[i]]for i in owners)
   if edge==left:r=(r[0]-1,r[1])
   if edge==right:r=(r[0]+1,r[1])
   fraction=cross(sub(p,a),sub(r,p))/cross(sub(c,a),sub(r,p));require(0<fraction.c[0]<1,'exact PS split interior');pos=add(a,scale(sub(c,a),fraction))
  else:pos=scale(add(a,c),Q(1,2))
  edges[edge]=len(points);points.append(pos)
 micro=[]
 for t,tri in enumerate(macro):
  for k in range(3):
   a,c=tri[k],tri[(k+1)%3];e=edges[tuple(sorted((a,c)))];micro.extend([(a,e,centers[t]),(e,c,centers[t])])
 require(micro==[tuple(x)for x in m.TRI.tolist()],'same exact micro topology')
 return points,micro

def operators(q,eta,alpha):
 points,micro=geometry(q,eta,alpha);R=m.INITIAL['R'];ids=m.INITIAL['ids'];D=zeros(24,22);K=zeros(22,22);B=zeros(16,22);mass=[Jet()for _ in range(16)];areas=[];gradients=[]
 for t,tri in enumerate(micro):
  p=[points[i]for i in tri];area2=cross(sub(p[1],p[0]),sub(p[2],p[0]));require(area2.c[0]>0,'positive exact microarea');area=area2/2;areas.append(area);grad=[]
  for i in range(3):
   j,k=(i+1)%3,(i+2)%3;grad.append(((p[j][1]-p[k][1])/area2,(p[k][0]-p[j][0])/area2));mass[ids[tri[i]]]+=area
  gradients.append(grad)
  exx=[sum((grad[i][0]*R[3*tri[i]][j]for i in range(3)),Jet())for j in range(22)]
  eyy=[sum((grad[i][1]*R[3*tri[i]+1][j]for i in range(3)),Jet())for j in range(22)]
  exy=[sum((grad[i][1]*R[3*tri[i]][j]+grad[i][0]*R[3*tri[i]+1][j]for i in range(3)),Jet())for j in range(22)]
  for j in range(22):
   D[t][j]=exx[j]+eyy[j]
   for k in range(22):K[j][k]+=Q(1,20)*area*(2*exx[j]*exx[k]+2*eyy[j]*eyy[k]+exy[j]*exy[k])
 for t in range(24):
  for p,col in enumerate(m.PIVOT):
   for j in range(22):B[p][j]-=D[t][col]*areas[t]*D[t][j]
 Rn=b.zeros(48,22)
 for i,node in enumerate(ids):
  for d in range(3):Rn[3*node+d]=R[3*i+d]
 M=zeros(22,22)
 for i in range(16):
  for j in range(22):
   for k in range(22):M[j][k]+=mass[i]*sum(Rn[3*i+d][j]*Rn[3*i+d][k]for d in range(3))
 return dict(points=points,micro=micro,Rn=Rn,D=D,K=K,B=B,M=M,mass=mass,areas=areas,gradients=gradients)

def lift(s,eta,alpha):
 known=[Jet(eta[i],alpha[i])for i in range(6)]+[-Jet(eta[2],alpha[2])]
 z=[Jet()for _ in range(22)]
 for k,j in enumerate(temporal.KNOWN):z[j]=known[k]
 A0=[[s['D'][i][j].c[0]for j in temporal.UNKNOWN]for i in temporal.LIFT_ROWS]
 for order in range(3):
  rhs=[]
  for i in temporal.LIFT_ROWS:
   knownpart=sum(s['D'][i][j].c[k]*z[j].c[order-k]for j in temporal.KNOWN for k in range(order+1))
   otherpart=sum(s['D'][i][j].c[k]*z[j].c[order-k]for j in temporal.UNKNOWN for k in range(1,order+1));rhs.append(-knownpart-otherpart)
  solution=b.solve(A0,rhs)
  for j,v in zip(temporal.UNKNOWN,solution):c=list(z[j].c);c[order]=v;z[j]=Jet(*c)
 require(all(v.c==(Q(0),)*3 for v in mv(s['D'],z)),'all full strong coefficients zero')
 return z

def flux(s,z):
 raw=mv(m.INITIAL['R'],z);u=[None]*16
 for i,node in enumerate(m.INITIAL['ids']):u[node]=raw[3*i:3*i+3]
 f={}
 for tri in s['micro']:
  p=[s['points'][i]for i in tri];w=[tuple(v.derivative()for v in pos)for pos in p];center=scale(add(add(p[0],p[1]),p[2]),Q(1,3))
  for i in range(3):
   j,k=(i+1)%3,(i+2)%3;segment=sub(center,scale(add(p[i],p[j]),Q(1,2)))
   relative=[sum((weight*(raw[3*tri[l]+d]-w[l][d])for l,weight in zip((i,j,k),(Q(5,12),Q(5,12),Q(1,6)))),Jet())for d in range(2)]
   value=3*cross(relative,segment);a,c=m.INITIAL['ids'][tri[i]],m.INITIAL['ids'][tri[j]];pair=min(a,c),max(a,c)
   f[pair]=f.get(pair,Jet())+(value if a<c else -value)
 convection=zeros(16,3)
 for(i,j),v in f.items():
  # Only coefficients 0/1 are used: PS motion's omitted third derivative does
  # not enter them. At f0=0 use the exact right-sided first flux sign.
  sign=next((x for x in v.c[:2]if x),Q(0));donor=u[i]if sign>=0 else u[j]
  for d in range(3):convection[i][d]+=v*donor[d];convection[j][d]-=v*donor[d]
 return f,u,mv(m.old.transpose(s['Rn']),[x for row in convection for x in row])

@lru_cache(maxsize=2)
def derive(kind):
 exact_z=m.old.force_solve()['old'if kind=='initial'else'z'];raw=b.mv(m.INITIAL['R'],exact_z);eta=[raw[9],raw[12],raw[10],exact_z[0],exact_z[1],exact_z[4]];q=[Q(0),Q(1,2),Q(1)]
 s=operators(q,eta,[Q(0)]*6)
 for name in ['D','K','B','M']:require(at(s[name],0)==m.INITIAL[name],'independent exact '+name+' assembly matches frozen model')
 f,u,c=flux(s,[Jet(x)for x in exact_z]);M0=at(s['M'],0);B0=at(s['B'],0);D0=at(s['D'],0);D1=at(s['D'],1);rows=m.ROWS.tolist();Bt=m.old.transpose(B0)
 g=[a+b+c for a,b,c in zip(b.mv(at(s['M'],1),exact_z),coeff(c,0),b.mv(at(s['K'],0),exact_z))]
 full=[row+bt for row,bt in zip(M0,Bt)]+[D0[i]+[Q(0)]*16 for i in rows]
 solution=b.solve(full,[-x for x in g]+[-b.dot(D1[i],exact_z)for i in rows])
 acc,p=solution[:22],solution[22:];from exact import instantaneous
 old=instantaneous(kind);require(acc==old['acceleration']and p==old['pressure'],'independent 38-row tangent equals frozen tangent exactly')
 alpha=[acc[j]for j in temporal.KNOWN[:6]];s=operators(q,eta,alpha);z=lift(s,eta,alpha);require(coeff(z,0)==exact_z and coeff(z,1)==acc,'exact initial lift and physical acceleration')
 f,u,c=flux(s,z);require({ij:v.c[0]for ij,v in f.items()}==old['physical_flux'],'independent physical initial face fluxes')
 require([a+b for a,b in zip(coeff(s['mass'],1),b.flux_divergence({ij:v.c[0]for ij,v in f.items()},16))]==[Q(0)]*16,'independent exact initial GCL')
 momentum=mv(s['M'],z);cdot=coeff(c,1);F=mv(s['K'],z);Bp=mv(m.old.transpose(s['B']),[Jet(x)for x in p]);Fdot=[a+b for a,b in zip(coeff(F,1),coeff(Bp,1))]
 L=[Q(0)]*22
 for(i,j),v in f.items():
  donor=i if v.c[0]>=0 else j
  for k in range(22):L[k]+=sum((s['Rn'][3*i+d][k]-s['Rn'][3*j+d][k])*v.c[0]*u[donor][d].c[1]/2 for d in range(3))
 from rate import leading
 require(L==leading(kind),'independent exact endpoint donor coefficient')
 J=[]
 for j in range(6):
  e=[Q(int(i==j))for i in range(6)];J.append(coeff(lift(s,e,[Q(0)]*6),0))
 J=m.old.transpose(J);MJ=[[b.dot(row,col)for col in m.old.transpose(J)]for row in M0];A=[row+bp for row,bp in zip(MJ,Bt)];require(all(x==0 for row in [[b.dot(a,col)for col in m.old.transpose(Bt)]for a in m.old.transpose(J)]for x in row),'exact J transpose B transpose zero')
 rhsdot=[-2*x-y-z for x,y,z in zip(coeff(momentum,2),cdot,Fdot)];udot=b.solve(A,rhsdot)
 rhsdiscrete=[-x-y/2-l-f for x,y,l,f in zip(coeff(momentum,2),cdot,L,Fdot)];u1=b.solve(A,rhsdiscrete)
 force_defect=[l+f/2 for l,f in zip(L,Fdot)];bias=b.solve(A,[-x for x in force_defect]);require([x-y/2 for x,y in zip(u1,udot)]==bias,'exact discrete/average tangent bias identity')
 # Independent differentiated full 38-row tangent, rather than chart equations.
 D2=at(s['D'],2);M1=at(s['M'],1);M2=at(s['M'],2)
 rhs2=[-2*x-2*y-c-f for x,y,c,f in zip(b.mv(M1,acc),b.mv(M2,exact_z),cdot,Fdot)]+[-2*b.dot(D1[i],acc)-2*b.dot(D2[i],exact_z)for i in rows]
 sol2=b.solve(full,rhs2);expected_accdot=[2*x+y for x,y in zip(coeff(z,2),b.mv(J,udot[:6]))];require(sol2[:22]==expected_accdot and sol2[22:]==udot[6:],'independent full differentiated tangent equals chart derivative exactly')
 require([a+2*b+c for a,b,c in zip(b.mv(D0,sol2[:22]),b.mv(D1,acc),[2*v for v in b.mv(D2,exact_z)])]==[Q(0)]*24,'all differentiated full tangent rows exactly zero')
 impulse=[x+y/2 for x,y in zip(b.mv(Bt,bias[6:]),b.mv(m.old.transpose(at(s['B'],1)),p))]
 frozen=b.solve(full,[-x for x in g]+[Q(0)]*16);frozen_defect=b.mv(D1,exact_z);mass_defect=b.mv(M1,exact_z)
 return dict(kind=kind,q=q,eta=eta,z=exact_z,acceleration=acc,pressure=p,tangent_unknowns=alpha+p,tangent_unknown_derivative=udot,discrete_unknown_first_coefficient=u1,average_pressure_coefficient_bias=bias[6:],endpoint_pressure_coefficient_bias=[x-y for x,y in zip(u1[6:],udot[6:])],force_defect_coefficient=force_defect,actual_path_momentum_defect_coefficient=[-x for x in force_defect],endpoint_donor_coefficient=L,pressure_impulse_defect_coefficient=impulse,frozen_divergence_counterexample=dict(acceleration=frozen[:22],pressure=frozen[22:],nonvanishing_strong_rate_defect=frozen_defect),dropped_mass_derivative_counterexample=dict(nonvanishing_momentum_rate_defect=[-x for x in mass_defect]),exact_identities=dict(initial_full_tangent=True,initial_GCL=True,all_chart_constraint_coefficients=True,independent_full_tangent_derivative=True,discrete_bias=True))

def serial(x):
 if isinstance(x,Q):return str(x)
 if isinstance(x,dict):return {k:serial(v)for k,v in x.items()}
 if isinstance(x,(list,tuple)):return [serial(v)for v in x]
 return x
if __name__=='__main__':
 rows=[]
 for kind in ['initial','pressure_state']:
  d=derive(kind);print('derived exact coefficients',kind,file=sys.stderr,flush=True);rows.append(serial(d))
 print(json.dumps(dict(scope='exact one-sided Taylor coefficients of declared semidiscrete donor model; no IEEE/mesh-family/continuum theorem',rows=rows),indent=2))
