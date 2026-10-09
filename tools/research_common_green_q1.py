"""Bounded shared Q1 energy/Green candidate, not a production or accuracy repair.

Two unchanged native fixtures only. No new unknowns, pressure, time integration
or coupling. All polynomial integration and transpose actions are exact rational.
"""
import argparse
from bisect import bisect_right
from fractions import Fraction as Q
from functools import lru_cache
from itertools import product
import hashlib,json,struct,subprocess
from pathlib import Path
import check_viscous_boundary_wrench as native_checker

ROOT=Path(__file__).resolve().parents[1]
BASE='7fd70976e88d244c6e07a167b5f9d39f06cad425'
BINS={'9f1f77115fac7c8907ee592cd534e030fc07d907b66ab0ce0d2064e661b6d504':'debug',
      'cfd7d8590914f63253f3450fe1ce4240b5b3f4e9ab8e60b6fd2df58a6b39155e':'release'}
BITS=list(product((0,1),repeat=3));ZERO=(0,0,0)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
exact=lambda x:Q.from_float(struct.unpack('>d',bytes.fromhex(x))[0])
def clean(p):return {k:v for k,v in p.items() if v}
def plus(a,b):
 r=dict(a)
 for k,v in b.items():r[k]=r.get(k,Q(0))+v
 return clean(r)
def scale(a,s):return clean({k:v*s for k,v in a.items()})
def mul(a,b):
 r={}
 for k,v in a.items():
  for l,w in b.items():
   t=tuple(x+y for x,y in zip(k,l));r[t]=r.get(t,Q(0))+v*w
 return clean(r)
def diff(a,j):
 r={}
 for k,v in a.items():
  if k[j]:
   t=list(k);t[j]-=1;r[tuple(t)]=v*k[j]
 return r
def evaluate(a,point):
 return sum((v*Q(point[0])**k[0]*Q(point[1])**k[1]*Q(point[2])**k[2] for k,v in a.items()),Q(0))

def integ(a):
 return sum((v/Q(product_int(k)) for k,v in a.items()),Q(0))
def product_int(k):
 z=1
 for p in k:z*=p+1
 return z
def face(a,j,side):
 r={}
 for k,v in a.items():
  t=list(k);t[j]=0;r[tuple(t)]=r.get(tuple(t),Q(0))+v*Q(side)**k[j]
 return clean(r)
def cross(a,b):return [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
def padd(a,b):return [x+y for x,y in zip(a,b)]
def serial(x):
 if isinstance(x,Q):return str(x)
 if isinstance(x,dict):return {str(k):serial(v) for k,v in x.items()}
 if isinstance(x,(list,tuple)):return [serial(v) for v in x]
 return x

SHAPE=[]
for bit in BITS:
 p={ZERO:Q(1)}
 for j,b in enumerate(bit):
  t=list(ZERO);t[j]=1
  p=mul(p,{tuple(t):Q(1)} if b else {ZERO:Q(1),tuple(t):Q(-1)})
 SHAPE.append(p)

def interpolate_poly(values):
 p={}
 for a,v in zip(SHAPE,values):p=plus(p,scale(a,v))
 return p

@lru_cache(None)
def element_matrix(lengths,mu):
 volume=lengths[0]*lengths[1]*lengths[2]
 grad=[[scale(diff(p,j),1/lengths[j]) for j in range(3)] for p in SHAPE]
 K=[]
 for i,a in product(range(3),range(8)):
  row=[]
  for k,b in product(range(3),range(8)):
   p=mul(grad[a][k],grad[b][i])
   if i==k:
    for j in range(3):p=plus(p,mul(grad[a][j],grad[b][j]))
   row.append(mu*volume*integ(p))
  K.append(row)
 assert all(K[a][b]==K[b][a] for a,b in product(range(24),repeat=2))
 # A second assembly uses the explicit positive canonical strain quadrature.
 simpson=[(Q(0),Q(1,6)),(Q(1,2),Q(2,3)),(Q(1),Q(1,6))]
 for degree in range(4):assert sum(w*t**degree for t,w in simpson)==Q(1,degree+1)
 checked=[[Q(0)]*24 for _ in range(24)]
 for terms in product(simpson,repeat=3):
  point=[t[0] for t in terms];weight=mu*volume*terms[0][1]*terms[1][1]*terms[2][1]
  assert weight>=0
  rows=[]
  for i in range(3):
   rows.append((2*weight,[evaluate(grad[a][i],point) if c==i else Q(0) for c,a in product(range(3),range(8))]))
  for i,j in [(0,1),(0,2),(1,2)]:
   rows.append((weight,[evaluate(grad[a][j],point) if c==i else evaluate(grad[a][i],point) if c==j else Q(0) for c,a in product(range(3),range(8))]))
  for w,row in rows:
   for a,b in product(range(24),repeat=2):checked[a][b]+=w*row[a]*row[b]
 if checked!=K:raise ValueError('positive canonical Simpson Gram differs from exact element integration')
 return K

def bracket(knots,t):
 if t<=knots[0]:return [(knots[0],Q(1))]
 if t>=knots[-1]:return [(knots[-1],Q(1))]
 b=bisect_right(knots,t);a=b-1
 if knots[a]==t:return [(t,Q(1))]
 return [(knots[a],(knots[b]-t)/(knots[b]-knots[a])),(knots[b],(t-knots[a])/(knots[b]-knots[a]))]

def tensor_map(knots,point,nodes):
 total={}
 for terms in product(*(bracket(k,t) for k,t in zip(knots,point))):
  p=tuple(t[0] for t in terms);w=terms[0][1]*terms[1][1]*terms[2][1]
  total=plus(total,scale(nodes[p],w))
 return total

class Candidate:
 def __init__(self,raw):
  m=raw['meta'];self.mu=exact(m['mu']);self.h=list(map(exact,m['spacing']));self.o=list(map(exact,m['origin']))
  self.counts=m['counts'];self.F=len(raw['active']);self.c=[Q(3,2)]*3
  self.end=[self.o[j]+self.h[j]*self.counts[j] for j in range(3)]
  self.lo=[self.o[j]+self.h[j]*m['lo'][j] for j in range(3)]
  self.hi=[self.o[j]+self.h[j]*m['hi'][j] for j in range(3)]
  active={(a['axis'],tuple(a['coordinates'])):a for a in raw['active']}
  self.maps=[];self.knots=[]
  for i in range(3):
   axes=[[self.o[j]+self.h[j]*(k+(Q(0) if i==j else Q(1,2))) for k in range(self.counts[j]+(i==j))] for j in range(3)]
   base={}
   for coords in product(*(range(len(k)) for k in axes)):
    p=tuple(axes[j][coords[j]] for j in range(3));a=active.get((i,coords))
    if a is not None:
     assert p==tuple(map(exact,a['position']));v={a['id']:Q(1)}
    elif all(self.lo[j]<=p[j]<=self.hi[j] for j in range(3)):v=self.rigid(p,i,0)
    elif p[i] in (self.o[i],self.end[i]):v=self.rigid(p,i,6)
    else:raise ValueError('unexplained missing MAC component sample')
    base[p]=v
   augmented=[sorted(set(axes[j]+[self.o[j],self.end[j],self.lo[j],self.hi[j]])) for j in range(3)]
   nodes={}
   for p in product(*augmented):
    if all(self.lo[j]<=p[j]<=self.hi[j] for j in range(3)):v=self.rigid(p,i,0)
    else:
     clamp=tuple(max(axes[j][0],min(axes[j][-1],p[j])) for j in range(3))
     v=tensor_map(axes,clamp,base)
     v=plus(v,plus(self.rigid(p,i,6),scale(self.rigid(clamp,i,6),-1)))
    nodes[p]=v
   self.knots.append(augmented);self.maps.append(nodes)
  self.overlay=[sorted(set(t for k in self.knots for t in k[j])) for j in range(3)]
  self.cache={}
  for a in raw['active']:
   p=tuple(map(exact,a['position']));assert self.node(a['axis'],p)=={a['id']:Q(1)}

 def rigid(self,p,i,offset):
  r=[p[j]-self.c[j] for j in range(3)];v={self.F+offset+i:Q(1)}
  for j in range(3):
   e=[Q(0)]*3;e[j]=1;t=cross(e,r)[i]
   if t:v[self.F+offset+3+j]=t
  return v

 def node(self,i,p):
  key=(i,p)
  if key not in self.cache:self.cache[key]=tensor_map(self.knots[i],p,self.maps[i])
  return self.cache[key]

 def run(self,raw):
  data=next(f for f in raw['fields'] if f['name']=='polynomial-tilted-curl')
  U=list(map(exact,data['values']))+[Q(0)]*12
  force=[Q(0)]*(self.F+12);energy=Q(0);cells={};rigidchecks=0
  bound_numerator=[Q(0)]*self.F
  for coords in product(*(range(len(k)-1) for k in self.overlay)):
   origin=tuple(self.overlay[j][coords[j]] for j in range(3))
   lengths=tuple(self.overlay[j][coords[j]+1]-origin[j] for j in range(3))
   midpoint=[origin[j]+lengths[j]/2 for j in range(3)]
   if all(self.lo[j]<midpoint[j]<self.hi[j] for j in range(3)):continue
   points=[tuple(origin[j]+lengths[j]*bit[j] for j in range(3)) for bit in BITS]
   maps=[self.node(i,p) for i,p in product(range(3),points)]
   values=[sum((w*U[k] for k,w in v.items()),Q(0)) for v in maps]
   # Global common rigid motion is an exact coefficient identity at every node.
   for i,p,v in zip([i for i in range(3) for _ in BITS],points*3,maps):
    actual=[Q(0)]*6
    for k,w in v.items():
     if k<self.F:
      a=raw['active'][k];coeff=self.rigid(tuple(map(exact,a['position'])),a['axis'],0)
      for t,c in coeff.items():actual[t-self.F]+=w*c
     else:actual[(k-self.F)%6]+=w
    expected=[Q(0)]*6
    for k,w in self.rigid(p,i,0).items():expected[k-self.F]=w
    if actual!=expected:raise ValueError('common rigid lift is not reproduced')
    rigidchecks+=6
   K=element_matrix(lengths,self.mu)
   local=[-sum((K[a][b]*values[b] for b in range(24)),Q(0)) for a in range(24)]
   phi=-sum((a*b for a,b in zip(values,local)),Q(0))/2
   if phi<0:raise ValueError('positive energy violated')
   energy+=phi
   for a,v in enumerate(maps):
    for k,w in v.items():force[k]+=w*local[a]
   # Conservative exact rational |H| row bound, not a stepping authorization.
   sizes=[sum((abs(w) for k,w in v.items() if k<self.F),Q(0)) for v in maps]
   local_bound=[sum((abs(K[a][b])*sizes[b] for b in range(24)),Q(0)) for a in range(24)]
   for a,v in enumerate(maps):
    for k,w in v.items():
     if k<self.F:bound_numerator[k]+=abs(w)*local_bound[a]
   polys=[interpolate_poly(values[8*i:8*i+8]) for i in range(3)]
   grad=[[scale(diff(polys[i],j),1/lengths[j]) for j in range(3)] for i in range(3)]
   stress=[[scale(plus(grad[i][j],grad[j][i]),self.mu) for j in range(3)] for i in range(3)]
   volume=lengths[0]*lengths[1]*lengths[2]
   phi_direct=sum((self.mu*volume*integ(mul(grad[i][i],grad[i][i])) for i in range(3)),Q(0))
   for i,j in [(0,1),(0,2),(1,2)]:
    s=plus(grad[i][j],grad[j][i]);phi_direct+=self.mu*volume*integ(mul(s,s))/2
   if phi_direct!=phi:raise ValueError('Gram action and polynomial energy differ')
   extension=[]
   for offset in (0,6):
    extension.append([[interpolate_poly([v.get(self.F+offset+k,Q(0)) for v in maps[8*i:8*i+8]]) for i in range(3)] for k in range(6)])
   cells[coords]={'origin':origin,'lengths':lengths,'stress':stress,'extension':extension,'velocity':polys}
  if sum((u*f for u,f in zip(U,force)),Q(0))!=-2*energy:raise ValueError('transpose dissipation work failed')
  fw=[Q(0)]*6
  for a,f in zip(raw['active'],force[:self.F]):
   v=[Q(0)]*3;v[a['axis']]=f
   fw=padd(fw,v+cross([exact(t)-c for t,c in zip(a['position'],self.c)],v))
  gs,go=force[self.F:self.F+6],force[self.F+6:]
  if padd(padd(fw,gs),go)!=[Q(0)]*6:raise ValueError('common rigid force/torque closure failed')
  wall=[[Q(0)]*6 for _ in range(2)];wall_normal=[[Q(0)]*6 for _ in range(2)];bulk=[[Q(0)]*6 for _ in range(2)];jump=[[Q(0)]*6 for _ in range(2)]
  interfaces=0;solid_faces=0;outer_faces=0
  for coords,cell in cells.items():
   l=cell['lengths'];o=cell['origin'];vol=l[0]*l[1]*l[2];s=cell['stress'];ext=cell['extension']
   div=[{} for _ in range(3)]
   for i,j in product(range(3),repeat=2):div[i]=plus(div[i],scale(diff(s[i][j],j),1/l[j]))
   for boundary,k,i in product(range(2),range(6),range(3)):
    bulk[boundary][k]+=vol*integ(mul(div[i],ext[boundary][k][i]))
   for n in range(3):
    area=vol/l[n]
    neighbor=list(coords);neighbor[n]+=1;neighbor=tuple(neighbor)
    if neighbor in cells:
     other=cells[neighbor];interfaces+=1
     for i in range(3):
      flux=plus(face(s[i][n],n,1),scale(face(other['stress'][i][n],n,0),-1))
      for boundary,k in product(range(2),range(6)):
       b=face(ext[boundary][k][i],n,1)
       if b!=face(other['extension'][boundary][k][i],n,0):raise ValueError('interface extension not continuous')
       jump[boundary][k]-=area*integ(mul(flux,b))
    for side in (0,1):
     nb=list(coords);nb[n]+=(-1 if side==0 else 1);nb=tuple(nb)
     if nb in cells:continue
     exterior=o[n]+side*l[n] in (self.o[n],self.end[n])
     if not exterior:
      # Missing neighbor must be a solid overlay box, never an unexplained gap.
      if not all(0<=nb[j]<len(self.overlay[j])-1 for j in range(3)):raise ValueError('unexplained external face')
      center=[(self.overlay[j][nb[j]]+self.overlay[j][nb[j]+1])/2 for j in range(3)]
      if not all(self.lo[j]<center[j]<self.hi[j] for j in range(3)):raise ValueError('unexplained absent neighbor')
     boundary=1 if exterior else 0
     if exterior:outer_faces+=1
     else:solid_faces+=1
     sign=1 if side==0 else -1 # wall outward normal points INTO fluid
     traction=[scale(face(s[i][n],n,side),sign) for i in range(3)]
     if not exterior:
      if any(face(v,n,side) for v in cell['velocity']):raise ValueError('stationary no-slip trace not exact')
     elif any(traction[i] for i in range(3) if i!=n):raise ValueError('stationary free-slip traction not exact')
     lever=[]
     for j in range(3):
      t=list(ZERO);t[j]=1
      lever.append(face({ZERO:o[j]-self.c[j],tuple(t):l[j]},n,side))
     moment=[plus(mul(lever[1],traction[2]),scale(mul(lever[2],traction[1]),-1)),
             plus(mul(lever[2],traction[0]),scale(mul(lever[0],traction[2]),-1)),
             plus(mul(lever[0],traction[1]),scale(mul(lever[1],traction[0]),-1))]
     wall[boundary]=padd(wall[boundary],[area*integ(p) for p in traction+moment])
     normal_traction=[traction[i] if i==n else {} for i in range(3)]
     nm=[plus(mul(lever[1],normal_traction[2]),scale(mul(lever[2],normal_traction[1]),-1)),
         plus(mul(lever[2],normal_traction[0]),scale(mul(lever[0],normal_traction[2]),-1)),
         plus(mul(lever[0],normal_traction[1]),scale(mul(lever[1],normal_traction[0]),-1))]
     wall_normal[boundary]=padd(wall_normal[boundary],[area*integ(p) for p in normal_traction+nm])
  residual=[padd(a,b) for a,b in zip(bulk,jump)]
  for weak,physical,r in zip([gs,go],wall,residual):
   if weak!=padd(physical,r):raise ValueError('independent element Green identity failed')
  # Connectivity is checked on integration boxes, not assumed from cell count.
  seen=set();todo=[next(iter(cells))]
  while todo:
   point=todo.pop()
   if point in seen:continue
   seen.add(point)
   for n,sign in product(range(3),(-1,1)):
    nb=list(point);nb[n]+=sign;nb=tuple(nb)
    if nb in cells and nb not in seen:todo.append(nb)
  if len(seen)!=len(cells):raise ValueError('fluid overlay disconnected')
  B=max((q/exact(a['mass']) for q,a in zip(bound_numerator,raw['active'])),default=Q(0))
  target=[Q(0),Q(-1,2),Q(0),Q(0),Q(0),Q(-1)]
  return {'active_faces':self.F,'fluid_integration_boxes':len(cells),'internal_interfaces':interfaces,
   'solid_subfaces':solid_faces,'outer_subfaces':outer_faces,'canonical_positive_strain_rows':len(cells)*27*6,
   'rigid_node_coefficient_checks':rigidchecks,'energy':energy,'force_work':-2*energy,
   'fluid_wrench':fw,'weak_solid_reaction':gs,'weak_outer_reaction':go,
   'physical_reconstructed_solid_traction':wall[0],'solid_normal_traction_part':wall_normal[0],
   'solid_tangential_traction_part':[a-b for a,b in zip(wall[0],wall_normal[0])],'physical_reconstructed_outer_traction':wall[1],
   'solid_bulk_divergence_work':bulk[0],'solid_internal_jump_work':jump[0],
   'outer_bulk_divergence_work':bulk[1],'outer_internal_jump_work':jump[1],
   'solid_green_residual':residual[0],'outer_green_residual':residual[1],
   'unchanged_target':target,'solid_traction_error':[a-b for a,b in zip(wall[0],target)],
   'weak_solid_reaction_error':[a-b for a,b in zip(gs,target)],
   'old_native_wrench':list(map(exact,data['solid_wrench'])),
   'exact_conservative_fluid_gram_row_bound_per_mass':B,
   'bound_authorizes_stepping':False,'finite_matrix_bound_includes_mu':True,'fluid_overlay_connected':True,'physical_accuracy_qualified':False,
   'energy_transpose_work_exact':True,'positive_simpson_gram_matches_polynomial_integration':True,'global_rigid_closure_exact':True,'independent_green_exact':True,
   'normal_traction_omitted':False,'residual_defined_by_subtraction':False}

def local_obstruction():
 a,d,mu,omega=Q(3,7),Q(2,5),Q(5,11),Q(7,13)
 U=d*omega;D=U/d
 penalty=mu*a*d*D*D/2
 assert penalty>0
 # Symmetric shear repairs rigid reproduction; generic stationary U exposes its dipole.
 U=Q(11,17);gamma=U/d
 force=mu*a*gamma;extra_moment=d*force
 return {'area':a,'distance':d,'mu':mu,'common_rotation':omega,
         'scalar_ray_common_rotation_energy':penalty,'scalar_ray_rigid_kernel':False,
         'symmetric_row_rigid_kernel':True,'stationary_surface_force':force,
         'symmetric_boundary_extra_moment':extra_moment,'wall_only_moment_about_patch_center':'0',
         'applies_to':'isolated scalar wall-ray row and restored symmetric trace row, not arbitrary richer volume reconstructions'}

def run(executable,output):
 if output.exists():raise ValueError('new external output required')
 if subprocess.run(['git','-C',str(output.parent),'rev-parse','--is-inside-work-tree'],capture_output=True).returncode==0:raise ValueError('external output required')
 fingerprint=sha(executable)
 if fingerprint not in BINS:raise ValueError('source-identified frozen binary required')
 subprocess.run(['git','diff','--exit-code',BASE,'--','src','proofs','tests','examples','Cargo.toml','Cargo.lock','tools/check_viscous_boundary_wrench.py'],cwd=ROOT,check=True,capture_output=True)
 if sha(ROOT/'tools/check_viscous_boundary_wrench.py')!='d25b2316e2ffe27ee9943a475a3878770f58a054d3186c05ff7bd09d3b30310c':raise ValueError('native checker changed')
 output.mkdir(parents=True);cases={}
 for name in ('unit-center','polynomial-bounded'):
  path=output/(name+'.json');subprocess.run([str(executable),name,str(path)],check=True,capture_output=True,timeout=30)
  native_checker.verify(path,name);raw=json.loads(path.read_text())
  if raw['meta']['counts'] not in ([3,3,3],[6,6,6]):raise ValueError('existing bounded fixtures only')
  print('Captured unchanged native '+name,flush=True)
  cases[name]={'native_record_sha256':sha(path),'candidate':Candidate(raw).run(raw)}
  print('Exact candidate/Green checks complete '+name,flush=True)
 if fingerprint!=sha(executable):raise ValueError('binary changed')
 result={'schema':'rheon-common-q1-energy-green-candidate-v1','base':BASE,'source_sha256':sha(Path(__file__)),
  'native_binary_sha256':fingerprint,'native_flavor':BINS[fingerprint],'cases':cases,
  'local_obstruction':local_obstruction(),'same_active_unknowns':True,'stored_mass_unchanged':True,
  'old_strain_action_replaced_in_candidate_only':True,'production_change':False,
  'pressure':False,'stepping':False,'coupling':False,'publication':False,'physical_accuracy_qualified':False}
 path=output/'research.json';path.write_text(json.dumps(serial(result),indent=2)+'\n')
 print(json.dumps({'file':str(path),'sha256':sha(path),'bounded_physical':serial(cases['polynomial-bounded']['candidate']['physical_reconstructed_solid_traction']),
                   'bounded_weak':serial(cases['polynomial-bounded']['candidate']['weak_solid_reaction']),'physical_accuracy_qualified':False}),flush=True)

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--executable',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
 a=p.parse_args();run(a.executable.resolve(),a.output.resolve())
