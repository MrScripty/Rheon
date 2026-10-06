"""Apply every frozen physical gate to each actual complete or partial prefix.

Only input grouping and the explicitly charged research reservation differ.
Neither equations nor tolerance checks are replaced. A failed gate stays failed.
"""
from pathlib import Path
import json,sys
import numpy as np
P=Path(__file__).resolve().parent;ROOT=P.parents[1];BASE=ROOT/'evidence/forced-extruded-liquid'
sys.path.insert(0,str(BASE));import replay as frozen
strict=frozen.strict;r=frozen.r
def require(ok,message):
 if not ok:raise ValueError(message)
def schema(rows):
 require(1<=len(rows)<=129,'bounded actual prefix')
 first=rows[0];key=(first['kind'],first['field'],first['load'],first['h']);kind,field,load,h=key
 require(kind in frozen.KINDS and field in frozen.FIELDS and load in frozen.LOADS and h in tuple(r.H)+(.0015625,.00078125),'original declared family and interval')
 for index,row in enumerate(rows):
  strict.keys(row,['model','kind','field','load','forcing','h','step','stamp','unknowns','end_q','end_eta','third_coefficients','velocity','pressure_coefficients','time','positions','triangles','periodic_indices','mass','physical_pressure','report','allocated_bytes'],'actual trajectory publication')
  require(row['model']=='periodic-z-invariant-forced' and (row['kind'],row['field'],row['load'],row['h'])==key,'single actual family')
  strict.keys(row['stamp'],['id','version'],'actual identity');require(type(row['step'])is int and row['step']==index and type(row['stamp']['id'])is int and row['stamp']['id']==131 and type(row['stamp']['version'])is int and row['stamp']['version']==index,'actual step/stamp')
  require(type(row['allocated_bytes'])is int and row['allocated_bytes']==542352,'explicit original plus research allowance')
  for name,shape in [('end_q',(3,)),('end_eta',(6,)),('third_coefficients',(12,)),('velocity',(16,3)),('pressure_coefficients',(16,)),('time',()),('positions',(19,2)),('triangles',(24,3)),('periodic_indices',(19,)),('mass',(16,)),('physical_pressure',(24,))]:strict.array(row[name],shape,name)
  require(np.array_equal(row['periodic_indices'],r.m.IDS) and all(type(i)is int for i in row['periodic_indices']) and all(type(i)is int for tri in row['triangles']for i in tri),'actual native topology/IDs')
  if index==0:require(row['unknowns']is None and row['report']is None and row['forcing']is None,'actual constructor')
  else:
   strict.array(row['unknowns'],(22,),'actual unknowns');strict.keys(row['report'],['planar','third','total'],'full native report')
   for part,keys in [('planar',frozen.PLANAR),('third',frozen.THIRD),('total',frozen.REPORT)]:
    strict.keys(row['report'][part],keys,part+' report')
    for name in keys:strict.array(row['report'][part][name],(12,)if name=='coefficients'else(),name)
   for name,maximum in [('iterations',7),('equation_evaluations',200),('sign_roots',64)]:require(type(row['report']['planar'][name])is int and 0<=row['report']['planar'][name]<=maximum,'original solve bound '+name)
   strict.keys(row['forcing'],['acceleration','force_count','planar_work','third_work','total_work','horizontal_impulse','third_impulse'],'actual forcing');strict.array(row['forcing']['acceleration'],(3,),'actual acceleration')
   require(np.array_equal(row['forcing']['acceleration'],r.A*(1 if load=='forward'else -1)) and type(row['forcing']['force_count'])is int and row['forcing']['force_count']==1,'original applied load')
   for name in ['planar_work','third_work','total_work','horizontal_impulse','third_impulse']:strict.array(row['forcing'][name],(),name)
 return {key:rows}
def run():
 roster=json.loads((P/'roster.json').read_text())['trajectories'];results=[];original_schema=frozen.schema;original_cell=frozen.cell;frozen.schema=schema
 try:
  for case in roster:
   index=case['index'];pubs=[json.loads(l)for l in(P/f'trajectory-{index}-native.log').read_text().splitlines()if l.startswith('{') and '"model"'in l];visited=0
   def observed_cell(*args):
    nonlocal visited
    visited+=1
    return original_cell(*args)
   frozen.cell=observed_cell
   try:
    replay=frozen.replay(pubs,references=False);result={'status':'PASS_ALL_FROZEN_PHYSICAL_GATES','maxima':replay['maxima'],'actual_steps_replayed':len(pubs)-1}
   except ValueError as error:result={'status':'FAIL_UNCHANGED_FROZEN_REPLAY_GATE','error':str(error),'first_failed_step':visited,'actual_steps_fully_replayed_before_failure':max(0,visited-1),'all_later_steps_unqualified_by_this_replay':True}
   results.append({'index':index,'kind':case['kind'],'field':case['field'],'load':case['load'],'h':case['h'],'actual_accepted_steps':len(pubs)-1,**result})
   print(json.dumps({'event':'physical_replay_finished','index':index,'status':result['status']}),file=sys.stderr,flush=True)
 finally:frozen.schema=original_schema;frozen.cell=original_cell
 failed=[x for x in results if x['status'].startswith('FAIL')]
 return {'status':'FAIL_UNCHANGED_FROZEN_REPLAY_GATES'if failed else'PASS_ALL_FROZEN_PHYSICAL_GATES','passed_trajectories':len(results)-len(failed),'failed_trajectories':len(failed),'physical_source_unchanged':True,'schema_adapter_only':True,'research_owner_reservation':542352,'reported_planar_finite_gate':1e-13,'other_physical_momentum_gate':1e-11,'tolerance_relaxation':False,'rows':results}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
