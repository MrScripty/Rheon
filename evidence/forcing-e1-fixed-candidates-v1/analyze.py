"""Fixed-candidate arithmetic diagnosis; no native execution or owner advance."""
from pathlib import Path
from fractions import Fraction as Q
from functools import lru_cache
import hashlib,importlib.util,json,math,struct,sys
import mpmath as mp
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
spec=importlib.util.spec_from_file_location('debug_reader',ROOT/'evidence/forcing-public-call-66k/analyze.py');debug_reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(debug_reader)
KNOWN=[2,3,12,0,1,4,13];COORD=[0,1,2,3,4,5,2];UNKNOWN=[5,6,7,8,9,10,11,14,15,16,17,18,19,20,21];ROWS=[0,2,3,4,5,6,8,10,11,12,14,16,17,18,20]
TARGET=Q(1e-13)
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def bits(x):return struct.pack('>d',x)
def records(path):return [json.loads(l)for l in path.read_text().splitlines()if l.startswith('{')]
def sequential(values):
 total=0.
 for value in values:total+=value
 return total
def mpq(x):return mp.mpf(x.numerator)/x.denominator
def normq(vector):return float(mp.sqrt(sum(mpq(x*x)for x in vector)))
def summarize(vector,scope):
 square=sum(x*x for x in vector);norm=normq(vector)
 return {'norm':norm,'signed_margin':float(TARGET)-norm,'exact_squared_norm':str(square),'below_original_target_exactly':square<=TARGET*TARGET,'rate_components':[float(x)for x in vector],'scope':scope,'native_acceptance':False}
def native(c,e,direct=False):
 old=c['accepted_publication']['velocity'];m0=e['start_mass'];m1=e['end_mass'];h=c['h'];terms=[[]for _ in range(22)];out=[0.]*22
 for i in range(16):
  for d in range(2):
   dz=[a-b for a,b in zip(e['end_z'],e['start_z'])];increment=sequential(a*b for a,b in zip(e['r'][i][d],dz));inertia=m1[i]*e['end_velocity'][i][d]-m0[i]*old[i][d]if direct else m1[i]*increment+(m1[i]-m0[i])*old[i][d]
   force=h*e['end_force'][i][d];force-=h*m0[i]*c['acceleration'][d]
   for j in range(22):value=e['r'][i][d][j]*(inertia+force);out[j]+=value;terms[j].append(Q(value))
 for f,(i,k)in enumerate(e['pairs']):
  for d in range(2):
   transported=e['plus'][f]*e['end_velocity'][i][d]-e['minus'][f]*e['end_velocity'][k][d]
   for j in range(22):value=(e['r'][i][d][j]-e['r'][k][d][j])*transported;out[j]+=value;terms[j].append(Q(value))
 return [x/h for x in out],[sum(t)/Q(h)for t in terms]
def exact(c,e,z=None,u=None,force=None,plus=None,minus=None,direct=False):
 old=c['accepted_publication']['velocity'];h=Q(c['h']);z=list(map(Q,e['end_z']))if z is None else z;u=[[Q(v)for v in row]for row in e['end_velocity']]if u is None else u;force=[[Q(v)for v in row]for row in e['end_force']]if force is None else force;plus=list(map(Q,e['plus']))if plus is None else plus;minus=list(map(Q,e['minus']))if minus is None else minus
 groups={key:[Q(0)]*22 for key in ['chart_increment_inertia','mass_change_inertia','direct_inertia','captured_combined_force','body_force','endpoint_donor_transport']}
 for i in range(16):
  for d in range(2):
   inc=sum(Q(w)*(a-Q(b))for w,a,b in zip(e['r'][i][d],z,e['start_z']));chart=Q(e['end_mass'][i])*inc/h;mass=(Q(e['end_mass'][i])-Q(e['start_mass'][i]))*Q(old[i][d])/h;di=(Q(e['end_mass'][i])*u[i][d]-Q(e['start_mass'][i])*Q(old[i][d]))/h;body=-Q(e['start_mass'][i])*Q(c['acceleration'][d])
   for j in range(22):
    w=Q(e['r'][i][d][j]);groups['chart_increment_inertia'][j]+=w*chart;groups['mass_change_inertia'][j]+=w*mass;groups['direct_inertia'][j]+=w*di;groups['captured_combined_force'][j]+=w*force[i][d];groups['body_force'][j]+=w*body
 for f,(i,k)in enumerate(e['pairs']):
  for d in range(2):
   transported=(plus[f]*u[i][d]-minus[f]*u[k][d])/h
   for j in range(22):groups['endpoint_donor_transport'][j]+=(Q(e['r'][i][d][j])-Q(e['r'][k][d][j]))*transported
 keys=['direct_inertia']if direct else['chart_increment_inertia','mass_change_inertia'];keys+=['captured_combined_force','body_force','endpoint_donor_transport'];return [sum(groups[key][j]for key in keys)for j in range(22)],groups
def forces(c,point,u,exact_arithmetic):
 convert=Q if exact_arithmetic else float;zero=convert(0);triangles=debug_reader.debug(point['triangles']);pressure=debug_reader.debug(point['pressure_terms']);ids=c['accepted_publication']['periodic_indices'];p=c['final_authorized_unknowns'][6:];image=[zero]*24;strain=[[zero]*3 for _ in range(16)];normal=[[zero]*3 for _ in range(16)]
 for term in pressure:image[term['triangle']]+=convert(term['value'])*convert(p[term['mode']])
 for t,tri in enumerate(triangles):
  local=[ids[i]for i in tri['nodes']];g=[[zero]*2 for _ in range(3)]
  for j in range(3):
   for d in range(3):
    for k in range(2):g[d][k]+=u[local[j]][d]*convert(tri['gradients'][j][k])
  sx=g[0][0];sy=g[1][1];cross=g[0][1]+g[1][0];weight=(convert(.05)*convert(1.))*convert(tri['area']);qw=(convert(1.)*convert(tri['area']))*image[t]
  for j,node in enumerate(local):
   gx,gy=map(convert,tri['gradients'][j]);value=[weight*((convert(2.)*(gx*sx))+(gy*cross)),weight*((convert(2.)*(gy*sy))+(gx*cross)),weight*((gx*g[2][0])+(gy*g[2][1]))]
   for d in range(3):strain[node][d]+=value[d]
   normal[node][0]+=-(qw*gx);normal[node][1]+=-(qw*gy)
 return [[a+b for a,b in zip(x,y)]for x,y in zip(strain,normal)],strain,normal
def transfers(points,faces,exact_arithmetic):
 convert=Q if exact_arithmetic else float;plus=[convert(0)]*faces;minus=[convert(0)]*faces
 for point in points:
  for f,flux in enumerate(point['flux']):plus[f]+=convert(point['factor'])*convert(max(flux,0.));minus[f]+=convert(point['factor'])*convert(max(-flux,0.))
 return plus,minus
def embedding(e,z):return [[sum(Q(w)*v for w,v in zip(row,z))for row in node]+[Q(0)]for node in e['r']]
def scalar(parts):
 sig,exponent,negative=parts;value=Q(int(sig))*Q(2)**(exponent-105);return -value if negative else value
@lru_cache(maxsize=16)
def chart(matrix,known):
 d=[[Q(v)for v in row]for row in matrix];z=[Q(0)]*22
 for j,value in zip(KNOWN,known):z[j]=value
 a=[[d[i][j]for j in UNKNOWN]+[-sum(d[i][j]*z[j]for j in KNOWN)]for i in ROWS]
 for j in range(15):
  pivot=max(range(j,15),key=lambda i:abs(a[i][j]));require(a[pivot][j]!=0,'nonsingular captured chart');a[j],a[pivot]=a[pivot],a[j]
  for i in range(j+1,15):
   factor=a[i][j]/a[j][j]
   for k in range(j+1,16):a[i][k]-=factor*a[j][k]
   a[i][j]=Q(0)
 answer=[Q(0)]*15
 for j in range(14,-1,-1):answer[j]=(a[j][15]-sum(a[j][k]*answer[k]for k in range(j+1,15)))/a[j][j]
 for j,value in zip(UNKNOWN,answer):z[j]=value
 require(all(sum(d[i][j]*z[j]for j in range(22))==0 for i in ROWS),'exact captured selected chart equations')
 return tuple(z)
def run(dps):
 mp.mp.dps=dps;packet=json.loads((P/'inputs.json').read_text());policy=json.loads((P/'protocol.json').read_text());data=records(P/'capture-native.log');require('1 passed; 0 failed'in(P/'capture-native.log').read_text(),'actual capture assertions pass');require(not(P/'capture-native-stderr.log').read_bytes(),'no hidden Newton controller trace')
 for path,digest in packet['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'original failed state source')
 complete=[x for x in data if x['event']=='fixed_capture_complete'];require([x['index']for x in complete]==[41,47] and all(x['equation_evaluations']==2 and x['controller_iterations']==x['owner_advances']==0 and not x['published']for x in complete),'exact two fixed candidates, no iterations or advances')
 output=[]
 for c in packet['cases']:
  c=dict(c,acceleration=[.0625,-.125,.03125]if c['load']=='forward'else[-.0625,.125,-.03125])
  for order in [16,32]:
   eq=next(x for x in data if x['event']=='fixed_equation'and x['index']==c['index']and x['order']==order);points=[x for x in data if x['event']=='fixed_point'and x['index']==c['index']and x['order']==order];end=next(x for x in points if x['label']=='end');start=next(x for x in points if x['label']=='start');quad=[x for x in points if x['label']=='quadrature'];partition=next(x for x in data if x['event']=='fixed_partition'and x['index']==c['index']and x['order']==order)
   require(len(quad)==order*(len(partition['partition'])-1)<=65*order and all(x['mode']=='E1'and x['owner_advances']==0 and not x['published']for x in [eq,*points]),'complete bounded unchanged quadrature path')
   require(eq['start_mass']==c['accepted_publication']['mass'] and eq['end_z']==end['z'] and eq['end_velocity']==end['velocity'] and eq['end_force']==end['force'] and eq['end_mass']==end['mass'],'actual frozen state and endpoint equation data')
   rate,rounded=native(c,eq);direct_rate,_=native(c,eq,True);require(all(bits(a)==bits(b)for a,b in zip(rate,eq['rate'])) and all(bits(a)==bits(b)for a,b in zip(direct_rate,eq['direct_rate'])),'bit-exact full native arithmetic replay');require(bits(math.sqrt(sequential(x*x for x in rate)))==bits(eq['norm']),'native sequential norm')
   if order==16:require(bits(eq['norm'])==bits(c['expected_terminal_norm'])and eq['norm']>1e-13,'exact original failed terminal norm')
   f64force,f64strain,f64pressure=forces(c,end,end['velocity'],False);require(all(bits(a)==bits(b)for x,y in zip(f64force,eq['end_force'])for a,b in zip(x,y)),'bit-exact complete native force assembly');f64plus,f64minus=transfers(quad,len(eq['pairs']),False);require(all(bits(a)==bits(b)for x,y in [(f64plus,eq['plus']),(f64minus,eq['minus'])]for a,b in zip(x,y)),'bit-exact complete native transfer quadrature')
   base,groups=exact(c,eq);direct,_=exact(c,eq,direct=True);u=[[Q(x)for x in row]for row in end['velocity']];force,visc,pressure=forces(c,end,u,True);plus,minus=transfers(quad,len(eq['pairs']),True);force_only,_=exact(c,eq,force=force);transfer_only,_=exact(c,eq,plus=plus,minus=minus);combined,_=exact(c,eq,force=force,plus=plus,minus=minus)
   native_q=list(map(Q,rate));variants={'native_sequential':summarize(native_q,'Observed binary64 equation; archived failure.'),'exact_sum_of_rounded_contributions':summarize(rounded,'Exact final accumulation/division of already rounded native contributions.'),'exact_products_sums_stored_stable':summarize(base,'Exact arithmetic on captured stored endpoints, masses, forces and transfers; original stable inertia.'),'exact_products_sums_stored_direct':summarize(direct,'Exact arithmetic on captured stored nodal endpoints with direct mass momentum difference.'),'exact_native_force_assembly':summarize(force_only,'Exact force assembly on captured native triangles/gradients/pressure terms and stored velocity; fixed masses/transfers.'),'exact_captured_flux_quadrature':summarize(transfer_only,'Exact accumulation of captured native flux samples/factors; fixed native forces and stored endpoints.'),'combined_exact_force_transfer':summarize(combined,'Exact force assembly plus exact captured-flux accumulation; stored endpoints and masses retained.')}
   matrix=tuple(tuple(row)for row in end['d']);stored_known=tuple(Q(end['z'][j])for j in KNOWN);continuous_known=tuple((-1 if k==6 else 1)*(Q(c['eta'][COORD[k]])+Q(c['h'])*Q(c['final_authorized_unknowns'][COORD[k]]))for k in range(7));stored=chart(matrix,stored_known);continuous=chart(matrix,continuous_known);actual106=tuple(scalar(x)for x in end['chart_endpoint_parts']);require(len(actual106)==22 and all(bits(float(actual106[j]))==bits(end['z'][j])for j in UNKNOWN),'actual E1 106-bit endpoint rounds to captured stored chart')
   for name,z in [('actual_106_chart_inertia_only',actual106),('exact_stored_known_chart_inertia_only',stored),('exact_continuous_known_chart_inertia_only',continuous)]:
    v,_=exact(c,eq,z=z);variants[name]=summarize(v,'Inertia-only chart sensitivity: captured stored donor velocity/forces are unchanged; not a coherent acceptance candidate.')
   for name,z in [('fixed_operator_chart_endpoint_substitution_stored_known',stored),('fixed_operator_chart_endpoint_substitution_continuous_known',continuous)]:
    highu=embedding(eq,z);highforce,_,_=forces(c,end,highu,True);v,_=exact(c,eq,z=z,u=highu,force=highforce,plus=plus,minus=minus);variants[name]=summarize(v,'All 22 terms use substituted endpoint velocity consistently on fixed captured binary geometry/fluxes/masses. Counterfactual endpoint, no native acceptance or new geometry/flux integration.')
   require(list(variants)==policy['arithmetic_variants'],'exact frozen variant roster')
   defect=[Q(0)]*22;old=c['accepted_publication']['velocity'];old_embedding=embedding(eq,list(map(Q,eq['start_z'])));new_embedding=embedding(eq,list(map(Q,eq['end_z'])))
   for i in range(16):
    for d in range(2):
     value=Q(eq['end_mass'][i])*((new_embedding[i][d]-Q(eq['end_velocity'][i][d]))-(old_embedding[i][d]-Q(old[i][d])))/Q(c['h'])
     for j in range(22):defect[j]+=Q(eq['r'][i][d][j])*value
   require([a-b for a,b in zip(base,direct)]==defect,'exact stable/direct stored embedding defect identity')
   group_output={k:{'norm':normq(v),'rate_components':[float(x)for x in v]}for k,v in groups.items()};changes={name:summarize([Q(a)-Q(b)for a,b in zip(v,base)],'Isolated exact vector contribution change; not a separate solve.')for name,v in [('force_assembly_change',force_only),('transfer_accumulation_change',transfer_only),('stored_direct_inertia_change',direct)]}
   output.append({'index':c['index'],'kind':c['kind'],'load':c['load'],'h':c['h'],'accepted_version':c['accepted_publication']['stamp']['version'],'order':order,'native_norm':eq['norm'],'native_signed_margin':1e-13-eq['norm'],'native_direct_norm':eq['direct_norm'],'complete_planar_equations':22,'diagnostic_quadrature_points':len(quad),'sign_segments':len(partition['partition'])-1,'variants':variants,'native_exact_term_groups':group_output,'isolated_changes':changes,'exact_stable_direct_defect_identity_pass':True,'stored_known_increment_rounding_errors':[float(a-b)for a,b in zip(stored_known,continuous_known)],'actual_106_vs_exact_stored_chart_max_gap':float(max(abs(a-b)for a,b in zip(actual106,stored))),'stored_endpoint_vs_actual106_max_gap':float(max(abs(Q(a)-b)for a,b in zip(end['z'],actual106))),'accepted_to_rebuilt_start_chart_max_gap':max(abs(a-b)for a,b in zip(eq['start_z'],start['z'])),'exact_stored_chart_full_constraint_max':float(max(abs(sum(Q(d)*z for d,z in zip(row,stored)))for row in end['d'])),'exact_continuous_chart_full_constraint_max':float(max(abs(sum(Q(d)*z for d,z in zip(row,continuous)))for row in end['d'])),'owner_advances':0,'controller_iterations':0,'third_solve_executed':False,'published':False})
 return {'status':'PASS_TWO_FIXED_FAILED_CANDIDATE_DIAGNOSIS','precision_digits':dps,'complete_fixed_equations':4,'cases':[41,47],'accepted_owners_constructed':0,'accepted_owners_advanced':0,'controller_iterations':0,'native_terminal_failures_retained':2,'original_newton_target':1e-13,'arithmetic_floor_assumed':False,'production_adoption':False,'memory_cap_expanded':False,'trajectory_remedy_established':False,'scope':'Exact arithmetic on captured native binary inputs and fixed declared sensitivities; no exact continuum assembly, native acceptance, continued trajectory or arithmetic-floor theorem.','rows':output}
if __name__=='__main__':print(json.dumps(run(int(sys.argv[1])if len(sys.argv)>1 else 80),indent=2,sort_keys=True))
