"""Compute original endpoint errors, strict bands and signed margins."""
from pathlib import Path
import json,sys
import numpy as np
P=Path(__file__).resolve().parent;ROOT=P.parents[1];sys.path.insert(0,str(ROOT/'evidence/forced-extruded-liquid'));import reference as r
def require(ok,message):
 if not ok:raise ValueError(message)
def run():
 roster=json.loads((P/'roster.json').read_text())['trajectories'];families=[]
 for kind in ['initial','pressure_state']:
  for field in ['nonconstant','constant']:
   for load in ['forward','reversed']:
    ref=json.loads((P/f'reference-{kind}-{field}-{load}.json').read_text());require(len(ref['rows'])==2 and all(x['status']=='COMPLETE'for x in ref['rows']),'original paired references complete')
    coarse,fine=[x['result']for x in ref['rows']];require(coarse['duration']==fine['duration']==.1 and coarse['rtol']==1e-10 and fine['rtol']==2e-12 and coarse['max_step']==.00625 and fine['max_step']==.003125 and max(coarse['calls'],fine['calls'])<=5000,'original reference parameters/cap')
    mass=r.m.spatial(np.array(fine['final_x']))['mass'];vf=np.array(fine['final_velocity']);gap=float(np.sqrt(np.sum(mass[:,None]*(np.array(coarse['final_velocity'])-vf)**2)));errors=[]
    cases=sorted([c for c in roster if(c['kind'],c['field'],c['load'])==(kind,field,load)],key=lambda c:-c['h'])
    for case in cases:
     data=[json.loads(l)for l in(P/f"trajectory-{case['index']}-native.log").read_text().splitlines()if l.startswith('{')];pubs=[x for x in data if 'model'in x];terminal=next(x for x in data if x.get('event')=='trajectory_terminal');result={'index':case['index'],'h':case['h'],'status':terminal['status'],'accepted_steps':terminal['accepted_steps'],'expected_steps':case['steps'],'actual_clock':pubs[-1]['time']}
     if terminal['status']=='COMPLETE':
      vel=np.array(pubs[-1]['velocity']);result.update(full_velocity_lumped_L2_error=float(np.sqrt(np.sum(mass[:,None]*(vel-vf)**2))),third_velocity_lumped_L2_error=float(np.sqrt(np.sum(mass*(vel[:,2]-vf[:,2])**2))),geometry_max_error=float(np.max(np.abs(r.m.q_to_x(np.array(pubs[-1]['end_q']))-fine['final_x']))))
     errors.append(result)
    checks=[]
    quantities=[('full_velocity_lumped_L2_error',1.8,2.2),('geometry_max_error',1.7,2.3)]+([('third_velocity_lumped_L2_error',1.8,2.2)]if field=='nonconstant'else[])
    for first,second in zip(errors,errors[1:]):
     for quantity,lo,hi in quantities:
      original=first['h']in r.H and second['h']in r.H;check={'quantity':quantity,'coarser_h':first['h'],'finer_h':second['h'],'lower':lo,'upper':hi,'original_five_interval_check':original}
      if first['status']==second['status']=='COMPLETE':
       ratio=first[quantity]/second[quantity];check.update(ratio=ratio,signed_lower_margin=ratio-lo,signed_upper_margin=hi-ratio,signed_margin=min(ratio-lo,hi-ratio),status='PASS_ORIGINAL_STRICT_BAND'if lo<ratio<hi else'FAIL_ORIGINAL_STRICT_BAND')
      else:check.update(status='INCOMPLETE_NO_ENDPOINT_RATIO')
      checks.append(check)
    completed=[x for x in errors if x['status']=='COMPLETE'];resolution_limit=min(x['full_velocity_lumped_L2_error']for x in completed)/1000 if completed else 0.;original_checks=[x for x in checks if x['original_five_interval_check']];original_status='INCOMPLETE_ORIGINAL_ENDPOINTS'if any(x['status']=='INCOMPLETE_NO_ENDPOINT_RATIO'for x in original_checks)else('FAIL_ORIGINAL_TEMPORAL_BAND'if any(x['status'].startswith('FAIL')for x in original_checks)else'PASS_ORIGINAL_BANDS')
    families.append({'kind':kind,'field':field,'load':load,'reference_calls':[coarse['calls'],fine['calls']],'reference_full_velocity_gap':gap,'reference_resolution_limit':resolution_limit,'reference_resolution_signed_margin':resolution_limit-gap,'reference_resolution_status':'PASS'if gap<resolution_limit else'FAIL','original_temporal_qualification':original_status,'errors':errors,'checks':checks})
 failed=[{'kind':f['kind'],'field':f['field'],'load':f['load'],**c}for f in families for c in f['checks']if c['status'].startswith('FAIL')]
 return {'status':'FAIL_ORIGINAL_TEMPORAL_BAND'if any(f['original_temporal_qualification']=='FAIL_ORIGINAL_TEMPORAL_BAND'for f in families)else('INCOMPLETE_ORIGINAL_ENDPOINTS'if any(f['original_temporal_qualification'].startswith('INCOMPLETE')for f in families)else'PASS_ORIGINAL_BANDS'),'reference_integrations':16,'reference_resolution_all_pass':all(f['reference_resolution_status']=='PASS'for f in families),'failed_original_checks':sum(c['original_five_interval_check']for c in failed),'failed_finer_checks':sum(not c['original_five_interval_check']for c in failed),'production_adoption':False,'tolerance_relaxation':False,'families':families,'failed_checks':failed}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
