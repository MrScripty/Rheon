"""Exact linear algebra on existing case41 captures; never run a candidate."""
from pathlib import Path
from fractions import Fraction as Q
import hashlib, importlib.util, json, sys
import mpmath as mp
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
def require(ok,msg):
 if not ok: raise ValueError(msg)
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); return m
a=load('equation',ROOT/'evidence/forcing-e1-fixed-candidates-v1/analyze.py')
def transpose(A):return list(map(list,zip(*A)))
def mul(A,B):return [[sum(x*y for x,y in zip(row,col))for col in transpose(B)]for row in A]
def mv(A,v):return [sum(x*y for x,y in zip(row,v))for row in A]
def dot(x,y):return sum(a*b for a,b in zip(x,y))
def inverse(A):
 n=len(A); B=[list(row)+[Q(i==j)for j in range(n)]for i,row in enumerate(A)]
 for j in range(n):
  pivot=max(range(j,n),key=lambda i:abs(B[i][j])); require(B[pivot][j]!=0,'full exact rank'); B[j],B[pivot]=B[pivot],B[j]; scale=B[j][j];B[j]=[x/scale for x in B[j]]
  for i in range(n):
   if i!=j:
    scale=B[i][j];B[i]=[x-scale*y for x,y in zip(B[i],B[j])]
 inv=[row[n:]for row in B];require(mul(A,inv)==[[Q(i==j)for j in range(n)]for i in range(n)],'exact inverse identity');return inv
 def_unreachable=0

def infnorm(A):return max(sum(map(abs,row))for row in A)
def condition(A):return {'norm_inf':float(infnorm(A)),'inverse_norm_inf':float(infnorm(inverse(A))),'condition_inf':float(infnorm(A)*infnorm(inverse(A))),'scope':'exact rational matrix/inverse; displayed ratio rounded; captured basis and units only, not full Newton Jacobian'}
def summary(v):return {'norm':a.normq(v),'squared_norm_exact':str(dot(v,v)),'components':[float(x)for x in v]}
def run(dps):
 mp.mp.dps=dps; policy=json.loads((P/'protocol.json').read_text())
 for path,digest in policy['inputs'].items(): require(hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,'pinned input '+path)
 C=ROOT/'evidence/forcing-e2-fixed-candidates-v1';c=next(x for x in json.loads((C/'inputs.json').read_text())['cases']if x['index']==41);c=dict(c,acceleration=[.0625,-.125,.03125]);records=a.records(C/'candidate-fixed.log');results=[]
 for order in [16,32]:
  e=next(x for x in records if x['event']=='fixed_equation'and x['index']==41 and x['order']==order);end=next(x for x in records if x['event']=='fixed_point'and x['index']==41 and x['order']==order and x['label']=='end');r,groups=a.exact(c,e);latent=[a.scalar(x)for x in end['chart_endpoint_parts']];rho=[Q(x)-y for x,y in zip(e['end_z'],latent)];rl,_=a.exact(c,e,z=latent)
  G=[[sum(Q(e['end_mass'][i])*Q(e['r'][i][d][j])*Q(e['r'][i][d][k])/Q(c['h'])for i in range(16)for d in range(2))for k in range(22)]for j in range(22)]
  vk=mv(G,[x if j in a.KNOWN else Q(0)for j,x in enumerate(rho)]);vu=mv(G,[x if j in a.UNKNOWN else Q(0)for j,x in enumerate(rho)]);v=[x+y for x,y in zip(vk,vu)];require(r==[x+y for x,y in zip(rl,v)],'exact stored endpoint reconciliation');require(dot(r,r)-dot(rl,rl)==2*dot(rl,v)+dot(v,v),'exact squared norm cross term')
  B=transpose([[Q(x)for x in row]for row in e['end_b']]);Bt=transpose(B);N=mul(Bt,B);coeff=mv(inverse(N),mv(Bt,r));parallel=mv(B,coeff);perp=[x-y for x,y in zip(r,parallel)];require(all(x==0 for x in mv(Bt,perp)),'exact complementary orthogonality');require(dot(r,r)==dot(parallel,parallel)+dot(perp,perp),'exact Pythagoras');require(mv(Bt,[x-y for x,y in zip(r,parallel)])==[Q(0)]*16,'pressure span uses 16 columns')
  D=[[Q(end['d'][i][j])for j in a.UNKNOWN]for i in a.ROWS]; M=[[x*Q(c['h'])for x in row]for row in G];native=list(map(Q,e['rate']));err=[x-y for x,y in zip(native,r)];rank=sorted(range(22),key=lambda j:abs(r[j]),reverse=True);active={k:v for k,v in groups.items()if k!='direct_inertia'};scale=[sum(abs(v[j])for v in active.values())for j in range(22)]
  results.append({'order':order,'native_norm':e['norm'],'exact_stored':summary(r),'top_components':[{'index':j,'value':float(r[j]),'squared_share':float(r[j]*r[j]/dot(r,r)),'sum_abs_term_groups':float(scale[j]),'group_cancellation_ratio':float(scale[j]/abs(r[j]))if r[j]else None}for j in rank],'components_2_3_squared_share':float((r[2]**2+r[3]**2)/dot(r,r)),'term_groups':{k:summary(v)for k,v in active.items()},'native_minus_exact':summary(err),'rho':summary(rho),'rho_known_rate':summary(vk),'rho_dependent_rate':summary(vu),'rho_total_rate':summary(v),'latent_inertia_only':summary(rl),'rho_cross_term':str(2*dot(rl,v)),'pressure_range':summary(parallel),'pressure_complement':summary(perp),'pressure_squared_share':float(dot(parallel,parallel)/dot(r,r)),'pressure_coefficients_exact':[str(x)for x in coeff],'selected_chart_condition':condition(D),'mass_gram_condition':condition(M),'pressure_normal_gram_condition':condition(N),'momentum_target':float(Q(c['h'])*Q(1e-13)),'native_target':1e-13,'physical_rate_gate':1e-11,'exact_identities':True})
 require(results[0]['selected_chart_condition']==results[1]['selected_chart_condition'],'same end geometry');return {'status':'PASS_EXISTING_DATA_EXACT_LINEAR_ALGEBRA','precision_digits':dps,'rows':results,'native_executions':0,'owner_advances':0,'new_corrections':0,'arithmetic_floor_claimed':False,'full_Newton_Jacobian_condition_claimed':False,'threshold_changed':False,'original_runtime_refusals':2,'original_geometry_band_failures':8,'scope':'Captured endpoint pressure-column span, chart and mass scaling in existing basis; pressure projection is a diagnostic linear model, not a new native correction or a physical state error bound.'}
if __name__=='__main__':print(json.dumps(run(int(sys.argv[1])if len(sys.argv)>1 else 80),indent=2,sort_keys=True))
