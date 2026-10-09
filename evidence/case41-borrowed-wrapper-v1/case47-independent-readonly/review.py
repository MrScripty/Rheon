"""Independent immutable packet replay; no native or integration execution."""
from pathlib import Path
import copy, hashlib, importlib.util, json, math, subprocess, sys
P=Path(__file__).resolve().parent
A=P/'archived-comparison';F=P/'inputs'
spec=importlib.util.spec_from_file_location('immutable_case47_comparison',A/'analyze.py')
a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
a.F=F;a.N=F/'evidence/case47-pure-e2-128-v2';a.O=F/'evidence/forcing-e1-trajectories-v2'
result=a.main()
expected=json.loads((A/'analysis-normal.json').read_text())
if result!=expected:raise ValueError('independent current environment replay differs from frozen analysis')
olddata=a.data;oldrows=a.rows;controls=[]
for kind in ['wrong-reference-fixture','wrong-physical-load','wrong-endpoint','wrong-native-initial-bit']:
 def changed(path,kind=kind):
  x=copy.deepcopy(olddata(path))
  if path.name=='reference-pressure_state-nonconstant-reversed.json':
   if kind=='wrong-reference-fixture':x['kind']='initial'
   elif kind=='wrong-physical-load':x['rows'][0]['result']['acceleration'][2]=.03125
   elif kind=='wrong-endpoint':x['rows'][0]['result']['duration']=.2
  return x
 def changedrows(path,kind=kind):
  x=copy.deepcopy(oldrows(path))
  if kind=='wrong-native-initial-bit' and path.name=='main-native.log':
   initial=next(row for row in x if 'model' in row);initial['velocity'][0][0]=math.nextafter(initial['velocity'][0][0],float('inf'))
  return x
 a.data=changed;a.rows=changedrows
 try:a.main()
 except ValueError as error:controls.append({'case':kind,'status':'REJECTED_BEFORE_COMPARISON','reason':str(error)})
 else:raise ValueError('mismatch guard failed '+kind)
 finally:a.data=olddata;a.rows=oldrows
outer=json.loads((P/'outer-frame/outer-frame.json').read_text())
residual=81152-2*8848-62096
bound=(65760-8+residual+304+15)//16*16
if residual!=1360 or bound!=67424 or bound>67584:raise ValueError('outer additional-scope arithmetic')
# Verify exact original metric formula independently of the archived helper.
import numpy as np
native=oldrows(a.N/'main-native.log');pubs=[r for r in native if 'model' in r]
refs=olddata(a.O/'reference-pressure_state-nonconstant-reversed.json')
v=np.array(pubs[-1]['velocity']);q=np.array(pubs[-1]['end_q']);x=np.array([q[0],q[2],q[1],2.25-q[2]])
w=np.array(result['fine_endpoint_mass'])
for label,row in zip(['coarse','fine'],refs['rows']):
 r=row['result'];delta=v-np.array(r['final_velocity'])
 metric={'full_velocity_lumped_L2_error':float(np.sqrt(np.sum(w[:,None]*delta**2))), 'third_velocity_lumped_L2_error':float(np.sqrt(np.sum(w*delta[:,2]**2))), 'geometry_max_error':float(np.max(np.abs(x-np.array(r['final_x']))))}
 if any(metric[k]!=result[label+'_metrics'][k] for k in metric):raise ValueError('original metrics independently disagree '+label)
print(json.dumps({'status':'PASS_INDEPENDENT_ARCHIVED_CASE47_ENDPOINT_REVIEW', 'comparison_publication':'24e231d8544bc1f981704acdafe090f24d7687f4', 'comparison_source': 'b7af5b24fb020a0b6eafd0f6d546976f6845be66', 'comparison_reader_sha256':hashlib.sha256((A/'analyze.py').read_bytes()).hexdigest(), 'frozen_analysis_sha256':hashlib.sha256((A/'analysis-normal.json').read_bytes()).hexdigest(), 'original_metric_recalculation_identical':True,'archived_result_exact_reproduction':True,'controls':controls,'case47_numeric_acceptance_publication':'26154d9a32e193b35d9cda0b02b2d7c5f770cfeb', 'supplement_publication':'30a312eb77b3f3a5b5aebf8af1cfb848b7257d82','outer_frame_residual':residual,'supplemented_additional_bound':bound,'cap':67584,'headroom':67584-bound,'fresh_elf_disassembly_performed':False,'elf_not_available_locally':True,'new_native_invocations':0,'new_reference_integrations':0,'new_trajectories':0,'new_FD_equations':0,'new_instantaneous_equations':0,'temporal_order_qualified':False,'eight_original_geometry_band_failures_unresolved':True,'retry_reproducibility_unqualified':True,'endpoint_comparison':result},indent=2,sort_keys=True))
