"""Verify frozen comparison and reject mismatched archived-data scenarios."""
from pathlib import Path
import copy,importlib.util,json
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('archived_endpoint_reader',P/'analyze.py');a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
def require(ok,msg):
 if not ok:raise ValueError(msg)
normal=(P/'analysis-normal.json').read_bytes();require(normal==(P/'analysis-optimized.json').read_bytes(),'normal/optimized comparison parity')
r=json.loads(normal);require(a.main()==r,'actual archived comparison reproduced')
require(r['new_reference_integrations']==r['new_trajectories']==r['new_native_invocations']==r['new_FD_equations']==r['new_instantaneous_equations']==0,'bounded scope')
require(not r['temporal_order_qualified'] and not r['broader_geometry_qualified'] and not r['memory_review_resolved'] and r['original_geometry_band_failures_preserved']==8,'limitations retained')
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
   initial=next(row for row in x if 'model'in row);initial['velocity'][0][0]=__import__('math').nextafter(initial['velocity'][0][0],float('inf'))
  return x
 a.data=changed;a.rows=changedrows
 try:a.main()
 except ValueError as error:controls.append(dict(case=kind,status='REJECTED_BEFORE_COMPARISON',reason=str(error)))
 else:raise ValueError('missed archived-data mismatch '+kind)
 finally:a.data=olddata;a.rows=oldrows
print(json.dumps(dict(status='PASS_ARCHIVED_ENDPOINT_READER_AND_FOUR_MISMATCH_CONTROLS',controls=controls,new_reference_integrations=0,new_trajectories=0,new_native_invocations=0,new_FD_equations=0,new_instantaneous_equations=0,temporal_order_qualified=False,broader_geometry_qualified=False),indent=2,sort_keys=True))
