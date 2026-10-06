"""Reuse all frozen physical gates for bounded diagnostic native trajectories."""
import json,sys
from pathlib import Path
import numpy as np
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'evidence/forced-extruded-liquid';P=Path(__file__).resolve().parent;sys.path.insert(0,str(BASE));import replay as frozen
r=frozen.r;strict=frozen.strict
H=(.0015625,.00078125)
def require(ok,message):
 if not ok:raise ValueError(message)
def schema(rows):
 require(1<=len(rows)<=129,'bounded actual diagnostic publications')
 first=rows[0];key=(first['kind'],first['field'],first['load'],first['h']);kind,field,load,h=key
 require(kind in frozen.KINDS and field=='nonconstant'and load in frozen.LOADS and h in H,'declared diagnostic case')
 for index,row in enumerate(rows):
  strict.keys(row,['model','kind','field','load','forcing','h','step','stamp','unknowns','end_q','end_eta','third_coefficients','velocity','pressure_coefficients','time','positions','triangles','periodic_indices','mass','physical_pressure','report','allocated_bytes'],'actual fine publication')
  require(row['model']=='periodic-z-invariant-forced'and (row['kind'],row['field'],row['load'],row['h'])==key,'one actual fine case')
  strict.keys(row['stamp'],['id','version'],'fine identity');require(type(row['step'])is int and row['step']==index and type(row['stamp']['id'])is int and row['stamp']['id']==131 and type(row['stamp']['version'])is int and row['stamp']['version']==index,'fine actual step identity')
  require(type(row['allocated_bytes'])is int and row['allocated_bytes']==frozen.BUDGET,'unchanged native memory budget')
  for name,shape in [('end_q',(3,)),('end_eta',(6,)),('third_coefficients',(12,)),('velocity',(16,3)),('pressure_coefficients',(16,)),('time',()),('positions',(19,2)),('triangles',(24,3)),('periodic_indices',(19,)),('mass',(16,)),('physical_pressure',(24,))]:strict.array(row[name],shape,name)
  require(np.array_equal(row['periodic_indices'],r.m.IDS)and all(type(i)is int for i in row['periodic_indices'])and all(type(i)is int for tri in row['triangles']for i in tri),'native topology and IDs')
  if index==0:require(row['unknowns']is None and row['report']is None and row['forcing']is None,'actual fine constructor')
  else:
   strict.array(row['unknowns'],(22,),'actual fine unknowns');strict.keys(row['report'],['planar','third','total'],'fine full report')
   for part,keys in [('planar',frozen.PLANAR),('third',frozen.THIRD),('total',frozen.REPORT)]:
    strict.keys(row['report'][part],keys,part+' report')
    for name in keys:strict.array(row['report'][part][name],(12,)if name=='coefficients'else(),name)
   for name,maximum in [('iterations',7),('equation_evaluations',200),('sign_roots',64)]:require(type(row['report']['planar'][name])is int and 0<=row['report']['planar'][name]<=maximum,'unchanged solve bound '+name)
   strict.keys(row['forcing'],['acceleration','force_count','planar_work','third_work','total_work','horizontal_impulse','third_impulse'],'fine actual forcing');strict.array(row['forcing']['acceleration'],(3,),'fine acceleration')
   require(np.array_equal(row['forcing']['acceleration'],r.A*(1 if load=='forward'else-1))and type(row['forcing']['force_count'])is int and row['forcing']['force_count']==1,'actual canonical fine force')
   for name in ['planar_work','third_work','total_work','horizontal_impulse','third_impulse']:strict.array(row['forcing'][name],(),name)
 return {key:rows}
def run(path):
 rows=[json.loads(line)for line in Path(path).read_text().splitlines()];case=[];results=[];native_steps=0;constructors=0
 references=json.loads((P/'first-reference-diagnosis.json').read_text())['rows'];original=frozen.schema
 # Only diagnostic input grouping is replaced. Every frozen physical equation,
 # work/GCL, constraint, Newton and quadrature gate executes without alteration.
 frozen.schema=schema
 try:
  for row in rows:
   if 'model'in row:case.append(row);continue
   require(row['probe_status']in ['COMPLETE','REFUSED']and case,'actual terminal probe result')
   first=case[0];key=(first['kind'],first['field'],first['load'],first['h'])
   require((row['kind'],row['field'],row['load'],row['h'])==key and row['accepted_steps']==len(case)-1 and row['expected_steps']==round(.1/key[3]),'actual bounded case counts')
   if row['probe_status']=='COMPLETE':require(row['accepted_steps']==row['expected_steps'],'actual complete endpoint')
   else:require(row['state_preserved']is True and row['attempted_step']==len(case)and row['error']=='IterationLimit','unchanged gate refusal and accepted state')
   physical=frozen.replay(case,references=False);native_steps+=len(case)-1;constructors+=1
   result=dict(**row,physical_equation_replay='PASS_UNCHANGED_FROZEN_GATES',maxima=physical['maxima'],last_actual_clock=case[-1]['time'],last_actual_stamp=case[-1]['stamp'],last_actual_q=case[-1]['end_q'])
   if row['probe_status']=='COMPLETE':
    ref=next(x for x in references if x['kind']==key[0]and x['load']==key[2]);error=r.m.q_to_x(np.array(case[-1]['end_q']))-ref['tight_DOP853']['final_x'];maximum=float(np.max(abs(error)));previous=ref['errors'][-1]
    require(previous['h']==2*key[3],'actual additional halving pair');ratio=previous['geometry_max_error']/maximum
    result.update(error_components=error.tolist(),geometry_max_error=maximum,additional_halving_ratio=ratio,original_geometry_band=[1.7,2.3],additional_pair_in_original_band=1.7<ratio<2.3,original_five_interval_band_pass=ref['original_band_pass'])
   results.append(result);case=[]
  require(not case and constructors==8 and [(x['kind'],x['load'],x['h'])for x in results]==[(k,l,h)for k in frozen.KINDS for l in frozen.LOADS for h in H],'all eight actual fine probe results')
 finally:frozen.schema=original
 return dict(status='PASS_PHYSICAL_PROBE_REPLAY_WITH_NATIVE_REFUSALS_RETAINED',actual_publications=native_steps+constructors,actual_steps=native_steps,actual_constructors=constructors,completed_cases=sum(x['probe_status']=='COMPLETE'for x in results),refused_cases=sum(x['probe_status']=='REFUSED'for x in results),original_reversed_five_interval_qualification='FAIL_ORIGINAL_TEMPORAL_BAND',rows=results)
if __name__=='__main__':print(json.dumps(run(sys.argv[1]),indent=2))
