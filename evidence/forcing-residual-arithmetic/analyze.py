"""Distinguish summation, chart solve and endpoint-coordinate rounding.

All variants evaluate frozen candidates only. None is a native acceptance or
an owner advance. High precision promotes captured IEEE inputs exactly; it is
not an independent exact geometry/flux/pressure assembly or a floor proof.
"""
from pathlib import Path
from fractions import Fraction as Q
import hashlib,json,math,struct
import mpmath as mp
P=Path(__file__).resolve().parent
KNOWN=[2,3,12,0,1,4,13];COORD=[0,1,2,3,4,5,2]
UNKNOWN=[5,6,7,8,9,10,11,14,15,16,17,18,19,20,21]
ROWS=[0,2,3,4,5,6,8,10,11,12,14,16,17,18,20]
def require(ok,message):
 if not ok:raise ValueError(message)
def bits(x):return struct.pack('>d',x)
def sequential_norm(x):
 a=0.
 for v in x:a=a+v*v
 return math.sqrt(a)
def native_rate(c,r):
 dz=[a-b for a,b in zip(r['end_z'],r['start_z'])];out=[0.]*22
 for i in range(16):
  for d in range(2):
   increment=0.
   for j in range(22):increment=increment+r['r'][i][d][j]*dz[j]
   stable=r['end_mass'][i]*increment+(r['end_mass'][i]-c['mass'][i])*c['old'][i][d]
   force=c['h']*r['end_force'][i][d]
   if c['acceleration'][d]!=0.:force=force-(c['h']*c['mass'][i])*c['acceleration'][d]
   for v in range(22):out[v]=out[v]+r['r'][i][d][v]*(stable+force)
 for f,(i,j)in enumerate(r['pairs']):
  for d in range(2):
   transported=r['plus'][f]*r['end_velocity'][i][d]-r['minus'][f]*r['end_velocity'][j][d]
   for v in range(22):out[v]=out[v]+(r['r'][i][d][v]-r['r'][j][d][v])*transported
 return [v/c['h']for v in out]
def exact_rate(c,r,dz=None):
 # Fraction is exact binary-input arithmetic, including products and sums.
 if dz is None:dz=[Q(a)-Q(b)for a,b in zip(r['end_z'],r['start_z'])]
 out=[Q(0)]*22;h=Q(c['h'])
 for i in range(16):
  for d in range(2):
   weights=list(map(Q,r['r'][i][d]));increment=sum(a*b for a,b in zip(weights,dz))
   stable=Q(r['end_mass'][i])*increment+(Q(r['end_mass'][i])-Q(c['mass'][i]))*Q(c['old'][i][d])
   force=h*(Q(r['end_force'][i][d])-Q(c['mass'][i])*Q(c['acceleration'][d]))
   for v in range(22):out[v]+=weights[v]*(stable+force)
 for f,(i,j)in enumerate(r['pairs']):
  for d in range(2):
   transported=Q(r['plus'][f])*Q(r['end_velocity'][i][d])-Q(r['minus'][f])*Q(r['end_velocity'][j][d])
   for v in range(22):out[v]+=(Q(r['r'][i][d][v])-Q(r['r'][j][d][v]))*transported
 return [x/h for x in out]
def mpq(x):
 x=Q(x);return mp.mpf(x.numerator)/x.denominator
def chart_delta(c,r,continuous_known):
 D=[[mpq(x)for x in row]for row in r['end_d']];z0=list(map(mpq,r['start_z']))
 dz=[mp.mpf(0)]*22
 for k,col in enumerate(KNOWN):
  dz[col]=((-1 if k==6 else 1)*mpq(c['h'])*mpq(c['unknown'][COORD[k]]))if continuous_known else mpq(r['end_z'][col])-z0[col]
 A=mp.matrix([[D[i][j]for j in UNKNOWN]for i in ROWS])
 rhs=mp.matrix([-sum(D[i][j]*z0[j]for j in range(22))-sum(D[i][j]*dz[j]for j in KNOWN)for i in ROWS])
 answer=mp.lu_solve(A,rhs)
 for j,x in zip(UNKNOWN,answer):dz[j]=x
 full_constraint=max(abs(sum(D[i][j]*(z0[j]+dz[j])for j in range(22)))for i in range(24))
 # This promotes native D, not the exact geometry. Its nominally redundant
 # rows need not obey exact symbolic identities after double assembly.
 return [Q(str(x))for x in dz],float(full_constraint)
def run(dps):
 mp.mp.dps=dps
 packet=json.loads((P/'inputs.json').read_text());cases=packet['rows']
 captures=[json.loads(x)for x in (P/'native-probe-start-chart.log').read_text().splitlines()if x.startswith('{')]
 require(len(cases)==len(captures)==5,'all five native captured refusals')
 result=[]
 for c,r in zip(cases,captures):
  c=dict(c,acceleration=[.0625,-.125,.03125]if c['load']=='forward'else[-.0625,.125,-.03125])
  rate=native_rate(c,r)
  require(all(bits(a)==bits(b)for a,b in zip(rate,r['rate'])),'exact native sequential arithmetic reconstruction')
  require(bits(sequential_norm(rate))==bits(r['norm']) and r['norm']>1e-13,'actual unchanged Newton refusal')
  exact=list(map(float,exact_rate(c,r)))
  rounded_dz,rounded_constraint=chart_delta(c,r,False)
  continuous_dz,continuous_constraint=chart_delta(c,r,True)
  rounded=list(map(float,exact_rate(c,r,rounded_dz)))
  continuous=list(map(float,exact_rate(c,r,continuous_dz)))
  delta_errors=[]
  for k,j in enumerate(KNOWN):
   true_increment=(-1 if k==6 else 1)*Q(c['h'])*Q(c['unknown'][COORD[k]])
   stored_increment=Q(r['end_z'][j])-Q(r['start_z'][j])
   delta_errors.append(float(stored_increment-true_increment))
  result.append(dict(kind=c['kind'],load=c['load'],h=c['h'],accepted_version=c['accepted_version'],native_norm=r['norm'],exact_sum_same_native_endpoints_norm=math.dist(exact,[0.]*22),exact_sum_max_component_gap=max(abs(a-b)for a,b in zip(exact,rate)),isolated_inertia_chart_with_stored_known_endpoints_norm=math.dist(rounded,[0.]*22),isolated_inertia_chart_with_continuous_known_increment_norm=math.dist(continuous,[0.]*22),stored_known_increment_errors=delta_errors,max_stored_known_increment_error_per_h=max(map(abs,delta_errors))/c['h'],rounded_chart_full_constraint=rounded_constraint,continuous_chart_full_constraint=continuous_constraint,accepted_to_rebuilt_start_max_gap=max(abs(a-b)for a,b in zip(r['start_z'],r['rebuilt_start_z'])),accepted_to_rebuilt_start_max_gap_per_h=max(abs(a-b)for a,b in zip(r['start_z'],r['rebuilt_start_z']))/c['h'],native_planar_qualification_pass=r['planar_qualification_pass'],native_work_fraction=r['work_fraction'],native_ledger_fraction=r['ledger_fraction'],native_gcl=r['gcl'],native_quadrature_difference=r['quadrature'],published=False))
 return dict(precision_digits=dps,source='dad53b4054034fe0c2ff6240464df7441ab4e6a9 plus archived test-only overlay',native_newton_target=1e-13,original_temporal_band='FAIL_ORIGINAL_TEMPORAL_BAND',remaining_native_refusals=5,owner_advances=0,high_precision_scope='Exact sum promotes captured native binary operators/masses/fluxes/forces. Chart variants change inertia alone while retaining captured forces/convection: sensitivity probes, NOT coherent equation residuals or acceptance candidates. No exact-geometry or general floating-point-floor theorem.',rows=result)
if __name__=='__main__':print(json.dumps(run(80),indent=2))
