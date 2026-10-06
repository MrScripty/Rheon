"""Audit actual corrections/counters/publications, not a toy controller model."""
from pathlib import Path
import json,math,copy,struct
P=Path(__file__).resolve().parent;T=1e-13
def require(ok,msg):
 if not ok:raise ValueError(msg)
def norm(v):
 x=0.
 for y in v:x=x+y*y
 return math.sqrt(x)
def bits(v):
 if isinstance(v,list):return [bits(x)for x in v]
 return struct.pack('>d',float(v)).hex()
def attempts(events):
 result=[];current=None;case=None;retry=False
 for e in events:
  event=e['event']
  if event=='case':case=(e['kind'],e['load'],e['h']);retry=False
  elif event=='retry_begin':retry=True
  elif event=='retry_end':require(e['state_preserved']is True and e['error']=='IterationLimit','actual retry refusal');retry=False
  elif event=='newton_check'and e['iteration']==1:
   current={'case':case,'retry':retry,'checks':[],'corrections':[],'terminal':[],'refusals':[],'observations':[]};result.append(current);current['checks'].append(e)
  else:
   require(current is not None,'actual attempt before event')
   names={'newton_check':'checks','correction':'corrections','terminal_validation':'terminal','refusal':'refusals','post_window_observation':'observations'}
   require(event in names,'known actual trace event');current[names[event]].append(e)
 return result
def validate(a,budget):
 checks=a['checks'];corr=a['corrections'];terminal=a['terminal']
 require(1<=len(checks)<=min(budget,7) and len(corr)<=min(budget,7) and len(terminal)<=1,'unchanged correction cap')
 for i,e in enumerate(checks):
  require(e['iteration']==i+1 and e['calls']==1+7*i and e['newton_threshold']==T and e['iteration_limit']==min(budget,7),'original check/counter/threshold')
  require(bits(norm(e['rate']))==bits(e['rate_norm']),'native sequential residual norm')
  if i<len(corr):
   c=corr[i];require(e['rate_norm']>T and c['iteration']==i+1 and c['calls']==7*(i+1),'authorized correction only after failed check')
   require(bits(c['unknown_after'])==bits([x+y for x,y in zip(e['unknown'],c['correction'])]),'actual correction arithmetic')
   if i+1<len(checks):require(bits(c['unknown_after'])==bits(checks[i+1]['unknown']),'next check of actual computed correction')
 if terminal:
  e=terminal[0];require(len(corr)==len(checks)==min(budget,7) and e['corrections']==len(corr) and e['calls']==7*len(corr)+1<=50 and e['newton_threshold']==T and e['converged']==(e['rate_norm']<=T),'one counted terminal validation, no new correction')
  passed=e['converged'];unknown=corr[-1]['unknown_after'];calls=e['calls']
 else:
  require(len(corr)==len(checks)-1 and checks[-1]['rate_norm']<=T,'ordinary converged check');passed=True;unknown=checks[-1]['unknown'];calls=checks[-1]['calls']
 require(bool(a['refusals'])==(not passed) and calls<=200,'refusal decision and work bound')
 if not passed:
  require(len(a['refusals'])==1 and a['refusals'][0]['reason']=='correction_budget_exhausted'and a['refusals'][0]['calls']==calls,'exact exhaustion reason')
  require(len(a['observations'])==2 and [x['order']for x in a['observations']]==[16,32],'bounded refused observations')
  require(bits(a['observations'][0]['rate_norm'])==bits(terminal[0]['rate_norm']),'same refused terminal rate')
 return passed,unknown,calls
def audit(boundary,finer):
 stdout=[json.loads(l)for l in(P/'trace-boundary.jsonl').read_text().splitlines()];bs=attempts(boundary)
 require(len(bs)==len(stdout)==14,'actual fourteen boundary attempts')
 boundary_rows=[]
 for a,row in zip(bs,stdout):
  passed,unknown,calls=validate(a,row.get('correction_budget',1));require(passed==(row['status']=='ACCEPTED'),'actual boundary publication')
  if 'unknowns'in row:require(bits(unknown)==bits(row['unknowns']) and row['equation_calls']==calls,'actual boundary state and calls')
  boundary_rows.append({'case':row['case'],'h':row.get('h'),'budget':row.get('correction_budget',1),'corrections':len(a['corrections']),'terminal_validation':bool(a['terminal']),'calls':calls,'status':row['status']})
 public=[json.loads(l)for l in(P/'trace-finer.jsonl').read_text().splitlines()];published={};terminal_public={}
 for row in public:
  key=(row['kind'],row['load'],row['h'])
  if 'model'in row:published.setdefault(key,[]).append(row)
  else:terminal_public[key]=row
 fs=attempts(finer);accepted=0;refused=0;retries=0;terminal_accepts=[];primaries={}
 for a in fs:
  passed,unknown,calls=validate(a,7);key=a['case'];version=a['checks'][0]['accepted_version'];index=(key,version)
  if a['retry']:
   require(not passed and index in primaries,'refused retry only');old=primaries[index]
   for part in ['checks','corrections','terminal','refusals','observations']:require(a[part]==old[part],'bit-identical same-owner retry '+part)
   retries+=1;continue
  require(index not in primaries,'unique original attempt');primaries[index]=a
  old=published[key][version];require(bits(a['checks'][0]['accepted_time'])==bits(old['time']),'actual accepted clock')
  if passed:
   row=published[key][version+1];require(bits(unknown)==bits(row['unknowns']) and row['report']['planar']['equation_evaluations']==calls,'actual fully qualified publication after Newton')
   accepted+=1
   if a['terminal']:terminal_accepts.append({'kind':key[0],'load':key[1],'h':key[2],'step':version+1,'rate_norm':a['terminal'][0]['rate_norm'],'calls':calls,'time_after':row['time']})
  else:
   row=terminal_public[key];require(row['probe_status']=='REFUSED'and row['state_preserved']is True and row['accepted_steps']==version and row['error']=='IterationLimit','actual state-preserving final refusal');refused+=1
 require(accepted==sum(len(v)-1 for v in published.values())and refused==retries==5 and len(terminal_accepts)==2,'all new publications/refusals/retries accounted')
 return {'status':'PASS_ACTUAL_TERMINAL_CHECK_ACCOUNTING','boundary':boundary_rows,'finer_actual_accepted_steps':accepted,'finer_original_attempts_refused':refused,'finer_same_owner_refusal_retries':retries,'fully_published_terminal_acceptances':terminal_accepts,'maximum_computed_corrections':max(len(a['corrections'])for a in fs),'maximum_counted_newton_equations':max(validate(a,7)[2]for a in fs),'unchanged_newton_threshold':T,'original_temporal_band_pass':False,'arithmetic_floor_proved':False}
if __name__=='__main__':
 b=[json.loads(l)for l in(P/'trace-boundary-stderr.log').read_text().splitlines()];f=[json.loads(l)for l in(P/'trace-finer-stderr.log').read_text().splitlines()];answer=audit(b,f);controls=[]
 for name,mutation in [('false convergence',lambda x:next(e for e in x if e['event']=='terminal_validation'and not e['converged']).__setitem__('converged',True)),('over-budget call',lambda x:next(e for e in x if e['event']=='terminal_validation').__setitem__('calls',201)),('extra correction',lambda x:x.insert(next(i for i,e in enumerate(x)if e['event']=='terminal_validation'),copy.deepcopy(next(e for e in x if e['event']=='correction')))),('altered last unknown',lambda x:next(e for e in x if e['event']=='correction')['unknown_after'].__setitem__(0,123.))]:
  broken=copy.deepcopy(f);mutation(broken)
  try:audit(b,broken)
  except ValueError:controls.append({'control':name,'status':'REJECTED'})
  else:raise ValueError('accepted broken actual trace '+name)
 answer['actual_corruption_rejections']=controls;print(json.dumps(answer,indent=2))
