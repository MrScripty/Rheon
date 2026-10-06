"""Single archived endpoint comparison; no integration or native invocation."""
from pathlib import Path
from fractions import Fraction
import hashlib,importlib.util,json,math,struct,sys
import numpy as np
P=Path(__file__).resolve().parent; ROOT=P.parents[1];F=Path('/workspace/Rheon-fd-pure-e2')
N=F/'evidence/case47-pure-e2-128-v2';O=F/'evidence/forcing-e1-trajectories-v2'
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(data):return hashlib.sha256(data).hexdigest()
def data(path):return json.loads(path.read_text())
def rows(path):return [json.loads(line)for line in path.read_text().splitlines()if line.startswith('{')]
def bits(value):
 if isinstance(value,float):return ['f64',struct.unpack('>Q',struct.pack('>d',value))[0]]
 if isinstance(value,list):return [bits(x)for x in value]
 if isinstance(value,dict):return {k:bits(v)for k,v in value.items()}
 return value
def main():
 policy=data(P/'protocol.json')
 for path,digest in policy['input_sha256'].items():require(sha((F/path).read_bytes())==digest,'frozen input '+path)
 for path,digest in policy['reader_sha256'].items():require(sha((P/path).read_bytes())==digest,'frozen reader '+path)
 # Verify original evidence bindings without executing their runners/readers.
 oldreceipt=data(O/'receipt.json');newreceipt=data(N/'receipt.json')
 for path in ['reference-pressure_state-nonconstant-reversed.json','reference-authorization.json','run_references.py','roster.json','protocol.json','convergence.py','convergence-outcome.json','trajectory-47-native.log']:
  require(sha((O/path).read_bytes())==oldreceipt['artifacts'][path],'original reference/native publication '+path)
 for path in ['main-native.log','main-completed.json','initial-bits.json','binary-binding.json','pure_e2.rs']:
  require(sha((N/path).read_bytes())==newreceipt['artifact_sha256'][path],'pure-E2 frozen publication '+path)
 for path in ['FittedHeightWorkspace-overlay.rs','TranslatedViscousFlow-overlay.rs','candidate_helpers.rs','inputs.rs','trajectory_inputs.rs']:
  require((N/path).read_bytes()==(O/path).read_bytes(),'same fixture/geometry/public physical boundary '+path)
 auth=data(O/'reference-authorization.json');oldpolicy=data(O/'protocol.json');roster=data(O/'roster.json');case=roster['trajectories'][47]
 require(auth['driver_sha256']==sha((O/'run_references.py').read_bytes()) and auth['protocol_sha256']==sha((O/'protocol.json').read_bytes()) and auth['original_sources_sha256']==roster['original_sources_sha256'],'original reference provenance')
 require(auth['parameters']==oldpolicy['reference_integrations'] and auth['rhs_cap']==5000 and auth['parameter_search'] is False,'original integration policy')
 for path,digest in auth['original_sources_sha256'].items():require(sha((F/path).read_bytes())==digest,'original physical source '+path)
 family=('pressure_state','nonconstant','reversed');require(tuple(case[k]for k in ['kind','field','load'])==family and case['h']==.00078125 and case['steps']==128 and case['final_time']==.1,'identical prescribed endpoint fixture')
 refs=data(O/'reference-pressure_state-nonconstant-reversed.json');require(tuple(refs[k]for k in ['kind','field','load'])==family and len(refs['rows'])==2 and all(row['status']=='COMPLETE'for row in refs['rows']),'identical completed reference family')
 coarse,fine=[row['result']for row in refs['rows']]
 for row,params in zip(refs['rows'],oldpolicy['reference_integrations']):
  ref=row['result'];require(row['parameters']==params and params['atol']==params['rtol']/100 and ref['rtol']==params['rtol'] and ref['max_step']==params['max_step'],'unaltered DOP853 parameters')
  require(ref['kind']==family[0] and ref['field']==family[1] and ref['acceleration']==[-.0625,.125,-.03125] and ref['duration']==.1 and 0<ref['calls']<=5000,'same reference model/load/endpoint and original RHS cap')
 mainrows=rows(N/'main-native.log');pubs=[x for x in mainrows if 'model'in x];oldpubs=[x for x in rows(O/'trajectory-47-native.log')if 'model'in x]
 terminal=[x for x in mainrows if x.get('event')=='pure_e2_terminal'];require(len(terminal)==1 and terminal[0]['status']=='COMPLETE' and terminal[0]['accepted_steps']==terminal[0]['expected_steps']==128 and terminal[0]['retries']==0,'actual frozen completed endpoint')
 require(len(pubs)==129 and [p['step']for p in pubs]==list(range(129)),'actual existing publication sequence')
 initial,last=pubs[0],pubs[-1]
 fields=['model','kind','field','load','h','time','stamp','end_q','end_eta','third_coefficients','velocity','pressure_coefficients','positions','triangles','periodic_indices','mass','physical_pressure']
 require(all(bits(initial[k])==bits(oldpubs[0][k])for k in fields),'pure E2 starts from identical original native accepted state bits')
 require(all(tuple(p[k]for k in ['kind','field','load'])==family and p['model']=='periodic-z-invariant-forced' and p['h']==case['h']for p in pubs),'unchanged published physical model and fixture')
 require(initial['time']==0. and initial['end_q']==[0.,.5,1.] and initial['third_coefficients']==[.25,.5,-.125,.375,.625,-.25,.1875,.4375,-.0625,.5625,.3125,.125],'identical canonical initial geometry/third fixture')
 clock=0.
 for p in pubs[1:]:
  clock+=case['h'];require(bits(clock)==bits(p['time']),'original repeated-addition clock')
  require(p['forcing']['acceleration']==[-.0625,.125,-.03125],'same acceleration units/body load')
 require(bits(clock)==bits(terminal[0]['actual_time']) and 128*case['h']==fine['duration']==case['final_time'],'same nominal physical endpoint; no retiming')
 completed=data(N/'main-completed.json');require(completed['stdout_sha256']==sha((N/'main-native.log').read_bytes()) and completed['exit']==0,'actual archived run log')
 # Import only the original static geometry/embedding algebra used by the
 # original error metric. Every integration/equation/trajectory entry is barred.
 import scipy.integrate
 forbidden_calls=[]
 def forbidden(*args,**kwargs):
  forbidden_calls.append(True);raise ValueError('new integration, equation or trajectory call forbidden')
 scipy.integrate.solve_ivp=forbidden
 sys.path.insert(0,str(F/'evidence/forced-extruded-liquid'))
 spec=importlib.util.spec_from_file_location('endpoint_original_reference',F/'evidence/forced-extruded-liquid/reference.py');r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
 for module,names in [(r,['reference','instantaneous','evaluate','solve','seed','finite_third']),(r.third,['reference','rate','finite']),(r.t,['initial','point','evaluate','solve','seed']),(r.m,['evaluate','cell_point'])]:
  for name in names:
   if hasattr(module,name):setattr(module,name,forbidden)
 require(Path(r.m.__file__)==F/'evidence/fitted-coupled-temporal/model.py' and Path(r.t.__file__)==F/'evidence/fitted-discrete-refinement/temporal.py','exact original metric dependency resolution')
 # The reference initializer obtains eta from exact force_solve().z. Reuse its
 # preserved exact rational output; do not solve a new instantaneous equation.
 seed=data(F/'evidence/fitted-pressure-path/research-qualification/normal-geometry.json');z=np.array([float(Fraction(x))for x in seed['reduced_velocity']]);q=np.array([0.,.5,1.]);eta=np.r_[r.m.C_CAP@z,z[r.m.SELECT]]
 require(bits(eta.tolist())==bits(initial['end_eta']) and bits(q.tolist())==bits(initial['end_q']),'reference and native canonical q/eta are identical bits')
 require(bits(r.third.initial_xi('nonconstant').tolist())==bits(initial['third_coefficients']),'same canonical third coefficients')
 s0=r.m.spatial(r.m.q_to_x(q));J,_=r.m.cap_chart(s0);z0=J@eta
 v0=np.einsum('ndj,j->nd',r.m.R_NODE,z0)+np.c_[np.zeros((16,2)),r.third.RW@r.third.XI]
 initial_gap=float(np.max(np.abs(v0-np.array(initial['velocity']))));position_gap=float(np.max(np.abs(s0['pos']-np.array(initial['positions']))))
 require(initial_gap<=1e-11 and position_gap<=1e-11,'original independent constructor velocity/geometry consistency checks')
 require(np.array_equal(initial['third_coefficients'],r.third.XI) and np.array_equal(np.array(initial['velocity'])[:,2],r.third.RW@r.third.XI),'original exact third constructor check')
 require(np.array_equal(s0['mass']>0,np.ones(16,bool)) and np.array_equal(r.m.TRI,np.array(initial['triangles'])) and np.array_equal(r.m.IDS,np.array(initial['periodic_indices'])),'same positive periodic microgeometry/connectivity')
 require(np.array_equal(r.A*-1,np.array(fine['acceleration'])),'same reversed physical acceleration')
 # Exactly the original convergence.py metrics: fine endpoint lumped masses
 # are used for BOTH candidate/reference velocity errors and the coarse/fine gap.
 sf=r.m.spatial(np.array(fine['final_x']));mass=sf['mass'];velocity=np.array(last['velocity']);x=r.m.q_to_x(np.array(last['end_q']));vf=np.array(fine['final_velocity']);vc=np.array(coarse['final_velocity'])
 require(mass.shape==(16,) and velocity.shape==vf.shape==vc.shape==(16,3) and np.all(np.isfinite(mass)) and np.all(mass>0),'original metric shapes/positive finite weights')
 def metrics(ref):
  vr=np.array(ref['final_velocity']);dx=x-np.array(ref['final_x'])
  return dict(full_velocity_lumped_L2_error=float(np.sqrt(np.sum(mass[:,None]*(velocity-vr)**2))),third_velocity_lumped_L2_error=float(np.sqrt(np.sum(mass*(velocity[:,2]-vr[:,2])**2))),geometry_max_error=float(np.max(np.abs(dx))),x_difference=dx.tolist())
 e_fine=metrics(fine);e_coarse=metrics(coarse);gap=float(np.sqrt(np.sum(mass[:,None]*(vc-vf)**2)))
 oldfamily=next(row for row in data(O/'convergence-outcome.json')['families']if tuple(row[k]for k in ['kind','field','load'])==family)
 require(gap==oldfamily['reference_full_velocity_gap'] and oldfamily['reference_resolution_status']=='PASS','original reference-consistency gap reproduced exactly')
 limit=e_fine['full_velocity_lumped_L2_error']/1000;resolved=gap<limit
 require(resolved,'unchanged strict reference-consistency check: gap < endpoint full error / 1000')
 require(not forbidden_calls,'zero forbidden numerical entry calls')
 binding=data(N/'binary-binding.json')
 return dict(status='PASS_ONE_ARCHIVED_PURE_E2_ENDPOINT_COMPARISON',compiled_source=binding['prototype'],compiled_source_tree=binding['prototype_tree'],binary_sha256=binding['binary_sha256'],native_execution_source=completed['source'],native_execution_source_tree=completed['source_tree'],pure_E2_tested_result='c20aa84e2de27a6537d4685c102c640d8f87dba4',pure_E2_tested_result_tree='a45d25d85d634fd18a14f95f14e3fe2ff9f798af',reference_publication='40c25f9eca37b66e7e24bee5a8d5f6aa995e2196',reference_publication_tree='3da767af9d6dcbed92760b049b917e85457b7d04',reference_driver_sha256=auth['driver_sha256'],reference_authorization_sha256=sha((O/'reference-authorization.json').read_bytes()),reference_data_sha256=sha((O/'reference-pressure_state-nonconstant-reversed.json').read_bytes()),reference_execution_commit_not_recorded=True,family=list(family),h=case['h'],steps=128,nominal_endpoint=.1,actual_native_clock=clock,native_clock_minus_nominal=clock-.1,no_retiming=True,initial_native_fields_bitwise_identical=fields,canonical_initial_q_eta_xi_bitwise_identical=True,independent_initial_velocity_max_difference=initial_gap,independent_initial_position_max_difference=position_gap,independent_nodal_reconstruction_bitwise_identical=False,endpoint_q=last['end_q'],endpoint_x=x.tolist(),reference_coarse_x=coarse['final_x'],reference_fine_x=fine['final_x'],fine_metrics=e_fine,coarse_metrics=e_coarse,metric_mass_source='original static spatial(fine.final_x).mass; reused unchanged for all velocity comparisons',fine_endpoint_mass=mass.tolist(),fine_endpoint_mass_sum=float(np.sum(mass)),reference_consistency=dict(status='PASS_ORIGINAL_STRICT_CHECK',full_velocity_gap=gap,endpoint_resolution_limit=limit,signed_margin=limit-gap,coarse_fine_geometry_max_gap=float(np.max(np.abs(np.array(coarse['final_x'])-fine['final_x']))),coarse_fine_third_lumped_L2_gap=float(np.sqrt(np.sum(mass*(vc[:,2]-vf[:,2])**2))),original_stored_full_velocity_gap_identical=True),original_reference_parameters=[row['parameters']for row in refs['rows']],original_reference_calls=[coarse['calls'],fine['calls']],new_trajectories=0,new_reference_integrations=0,new_native_invocations=0,new_FD_equations=0,new_instantaneous_equations=0,new_Lean_claims=0,temporal_order_qualified=False,broader_geometry_qualified=False,original_geometry_band_failures_preserved=8,memory_review_resolved=False,production_adoption=False,scope='One completed archived pure-E2 endpoint against the original paired DOP853 references for the identical prescribed canonical fixture/model/nominal endpoint; original independent representation consistency checks retained. No bitwise identity claim for independent nodal reconstruction, new temporal band/order claim, broader geometry qualification, or changed gate.')
if __name__=='__main__':print(json.dumps(main(),indent=2,sort_keys=True))
