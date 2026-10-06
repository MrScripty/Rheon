"""Read actual pure-E2 publications, refusals and controls; never execute native code."""
from pathlib import Path
import copy,hashlib,importlib.util,json,math,struct
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def rows(path):return [json.loads(x) for x in path.read_text().splitlines() if x.startswith('{')]
def bt(x):
 if isinstance(x,float):return ['f64',struct.pack('>d',x).hex()]
 if isinstance(x,list):return [bt(y) for y in x]
 if isinstance(x,dict):return {k:bt(v) for k,v in x.items()}
 return x
def fields(r):return {k:v for k,v in r.items() if k not in ['event','label','step']}
def trace_groups(trace):
 groups={};current=None
 for r in trace:
  if r['event']=='newton_check':current=r['accepted_version'];groups.setdefault(current,[])
  require(current is not None,'trace belongs to a finite call');groups[current].append(r)
 return groups
def validate_publication(previous,row,result,trace):
 p=row['report']['planar'];t=row['report']['third'];w=row['report']['total'];step=previous['step']+1
 require(row['step']==step and row['stamp']=={'id':131,'version':step} and bt(row['time'])==bt(previous['time']+row['h']),'accepted time/stamp lineage')
 require(bt(row['pressure_coefficients'])==bt(row['unknowns'][6:]) and bt(row['third_coefficients'])==bt(t['coefficients']),'pressure and third published consistently')
 require(result['result'].startswith('Ok(') and result['step']==step,'public result success')
 parsed=load('debug_result',ROOT/'evidence/forcing-public-call-66k/analyze.py').debug(result['result'][3:-1])
 require(parsed['step']['planar']['before']==previous['stamp'] and parsed['step']['planar']['after']==row['stamp'] and bt(parsed['step']['planar']['unknowns'])==bt(row['unknowns']) and bt(parsed['step']['third']['coefficients'])==bt(row['third_coefficients']) and bt(parsed['forces'])==bt(row['forcing']),'ordinary result/publication binding')
 checks=[x for x in trace if x['event']=='newton_check'];corrections=[x for x in trace if x['event']=='correction'];terminal=[x for x in trace if x['event']=='terminal_validation']
 require(1<=len(checks)<=7 and len(corrections)<=7 and len(terminal)<=1,'original correction/check bounds')
 require(all(x['accepted_version']==step-1 and x['newton_threshold']==1e-13 and x['iteration_limit']==7 and x['h']==0.00078125 and x['calls']<=200 for x in checks),'unchanged native gates and input')
 last=terminal[-1] if terminal else checks[-1]
 require(last['rate_norm']<=1e-13 and p['iterations']==len(checks) and p['equation_evaluations']==last['calls']<=200,'actual convergence and counted equation report')
 require(all(result['seen'][i]>0 for i in range(11)) and result['seen'][11]==0,'all actual publication barriers reached')
 for r in [p,t]:
  for name in ['finite_momentum_rate_norm','direct_momentum_rate_norm']:require(math.isfinite(r[name]) and 0<=r[name]<=1e-11,'original physical momentum gate')
 for r in [p,t,w]:require(min(r[k] for k in ['backward_Euler_loss','mixing_loss','viscous_loss'])>=0 and max(abs(r[k]) for k in ['energy_ledger_error','discrete_residual_work'])<=r['fixed_work_allowance'],'original separate/combined work gates')
 require(p['finite_momentum_rate_norm']<=1e-13 and p['quadrature_error']<=1e-15 and p['full_constraints']<=1e-11 and abs(p['pressure_work'])<=p['fixed_work_allowance'],'original planar/quad/constraint/pressure gates')
 require(t['shear_x_loss']>0 and t['shear_y_loss']>0,'both third shears exercised')
 require(max(math.hypot(p[k],t[k]) for k in ['finite_momentum_rate_norm','direct_momentum_rate_norm'])<=1e-11,'original full-vector physical gate')
 require(row['forcing']['acceleration']==[-.0625,.125,-.03125] and row['forcing']['force_count']==1,'original applied force')
 return {'checks':len(checks),'corrections':len(corrections),'counted_equations':last['calls'],'Newton_norm':last['rate_norm']}
def run():
 b=json.loads((P/'binary-binding.json').read_text());policy=json.loads((P/'protocol.json').read_text());require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'actual dedicated ELF');require(b['prototype']=='e37b0bb34ace62f16dc931488146d6a396b27be0' and b['prototype_tree']=='5e3fef4bdb87087acbeb7bdf56303fa9669b2b2a','compiled source identity')
 for name,digest in policy['source_sha256'].items():require(sha((ROOT/name).read_bytes())==digest,'frozen experiment source')
 pre=json.loads((P/'preflight-receipt.json').read_text());require(pre['status']=='PASS_PURE_E2_SCALAR_AND_FULL_PUBLIC_MEMORY' and pre['memory_bound']==67088,'qualified source before owner')
 for name,digest in pre['artifact_sha256'].items():require(sha((P/name).read_bytes())==digest,'actual preflight artifacts')
 jobs=json.loads((P/'jobs-completed.json').read_text());require(jobs and jobs[0]['label']=='main' and len(jobs)<=12,'one main and bounded controls');alljobs={};traces={}
 for job in jobs:
  label=job['label'];before=json.loads((P/(label+'-before.json')).read_text());after=json.loads((P/(label+'-completed.json')).read_text());require({k:v for k,v in after.items() if k not in ['exit','seconds','stdout_sha256','stderr_sha256']}==before,'completed exact command');require(after['exit']==0 and after['binary_sha256']==b['binary_sha256'] and after['preflight_receipt_sha256']==sha((P/'preflight-receipt.json').read_bytes()) and after['retries']==0,'actual qualified invocation')
  expected_env={'RHEON_CASE47_PURE_E2_ALLOW':'case47-pure-E2-original128-first-visit-cancel-v1'}
  if label!='main':expected_env['RHEON_CASE47_CANCEL_STAGE']=label.split('-')[1]
  require(after['environment']==expected_env and after['argv']==[b['binary'],'--exact','research_public_call::case47_pure_e2_original_128_or_cancel','--nocapture'],'only authorized exact native test')
  for channel in ['stdout','stderr']:
   name=label+'-native'+('' if channel=='stdout' else '-stderr')+'.log';require(sha((P/name).read_bytes())==after[channel+'_sha256'],'closed actual native journal')
  data=rows(P/(label+'-native.log'));state=[x for x in data if x.get('event')=='accepted_bits'];result=[x for x in data if x.get('event')=='pure_e2_result'];pubs=[x for x in data if 'model' in x];terminal=[x for x in data if x.get('event')=='pure_e2_terminal'];require(len(terminal)==1 and terminal[0]==job['terminal'] and len(state)==1+2*len(result) and len(pubs)==1+terminal[0]['accepted_steps'],'complete incremental observations')
  for i,r in enumerate(result):
   beforebits,afterbits=state[1+2*i:3+2*i];require(beforebits['step']==afterbits['step']==r['step']==i+1,'exact ordered attempts')
   if r['result'].startswith('Err('):require(fields(beforebits)==fields(afterbits),'all accepted fields preserved on refusal/cancellation')
  alljobs[label]={'data':data,'state':state,'results':result,'pubs':pubs,'terminal':terminal[0]};traces[label]=trace_groups(rows(P/(label+'-native-stderr.log')))
 main=alljobs['main'];initial=next(x for x in rows(ROOT/'evidence/forcing-case47-public-call-v1/E2-native.log') if 'model' in x);require(bt(main['pubs'][0])==bt(initial),'exact original constructor publication')
 declared=json.loads((P/'initial-bits.json').read_text());declared['stamp']=[declared['stamp']['id'],declared['stamp']['version']];require(fields(main['state'][0])==declared,'correct frozen initial bits')
 details=[]
 for i,row in enumerate(main['pubs'][1:]):details.append(validate_publication(main['pubs'][i],row,main['results'][i],traces['main'][i]))
 accepted=main['terminal']['accepted_steps'];status=main['terminal']['status'];require(status in ['COMPLETE','REFUSED'],'honest main terminal')
 if status=='COMPLETE':require(accepted==128 and len(main['results'])==128,'one complete original trajectory')
 else:require(len(main['results'])==accepted+1 and len(jobs)==1,'stop at first refusal with no retry or cancellation extension')
 controls=[]
 for stage in range(len(jobs)-1):
  label='cancel-'+str(stage);require(jobs[stage+1]['label']==label,'exact stage roster');c=alljobs[label]
  require(status=='COMPLETE' and c['terminal']['status']=='CANCELLED' and c['terminal']['accepted_steps']==25 and c['terminal']['attempted_step']==26,'only declared first-visit cancellation')
  require(bt(c['pubs'])==bt(main['pubs'][:26]) and bt(c['results'][:25])==bt(main['results'][:25]) and c['state'][:51]==main['state'][:51],'all prefix publication/result/state bits reproduce main')
  require(c['results'][-1]['seen'][stage]==1,'first actually reached cancellation visit')
  require({k:v for k,v in traces[label].items() if k<25}=={k:v for k,v in traces['main'].items() if k<25},'entire pure-E2 prefix trace reproduced')
  controls.append(stage)
 physical=json.loads((P/'physical-normal.json').read_text());require((P/'physical-normal.json').read_bytes()==(P/'physical-optimized.json').read_bytes(),'independent replay mode parity')
 negative=[]
 if details:
  previous,row=main['pubs'][0],main['pubs'][1];res=main['results'][0];trace=traces['main'][0]
  for name,mutate in [('wrong version',lambda x:x['stamp'].update(version=99)),('wrong pressure',lambda x:x['pressure_coefficients'].__setitem__(0,0.)),('omitted third shear',lambda x:x['report']['third'].update(shear_y_loss=0.)),('false work',lambda x:x['report']['total'].update(energy_ledger_error=1.)),('relaxed planar gate',lambda x:x['report']['planar'].update(finite_momentum_rate_norm=1e-12)),('wrong third publication',lambda x:x['third_coefficients'].__setitem__(0,0.)),('wrong force',lambda x:x['forcing']['acceleration'].__setitem__(2,0.))]:
   bad=copy.deepcopy(row);mutate(bad)
   try:validate_publication(previous,bad,res,trace)
   except ValueError:negative.append(name)
   else:raise ValueError('missed corruption '+name)
 qualified=status=='COMPLETE' and physical['status']=='PASS_UNCHANGED_PHYSICAL_REPLAY' and controls==list(range(11))
 return {'status':'PASS_ONE_ORIGINAL_PURE_E2_TRAJECTORY_AND_DECLARED_CANCELLATIONS' if qualified else 'BOUNDED_RESULT_WITH_REMAINING_GAPS','native_source':b['prototype'],'native_source_tree':b['prototype_tree'],'binary_sha256':b['binary_sha256'],'preflight_commit':'529707429fb092beb4df9a278d978bf72e8612cf','main_terminal':main['terminal'],'actual_main_accepted_steps':accepted,'main_public_attempts':len(main['results']),'published_time':main['pubs'][-1]['time'],'pure_E2_before_first_advance':True,'cancellation_stages_passed':controls,'cancellation_retry_behavior_qualified':False,'step_retries':0,'owners_in_this_v2':len(jobs),'max_simultaneous_owners':1,'total_public_attempts':sum(len(x['results']) for x in alljobs.values()),'total_accepted_advances':sum(x['terminal']['accepted_steps'] for x in alljobs.values()),'physical_replay':physical,'maximum_Newton_norm':max([x['Newton_norm'] for x in details] or [0]),'maximum_corrections':max([x['corrections'] for x in details] or [0]),'maximum_counted_equations':max([x['counted_equations'] for x in details] or [0]),'main_solver_details':details,'negative_controls':negative,'memory_bound':67088,'memory_cap':67584,'reference_integrations':0,'original_geometry_band_failures_preserved':8,'original_runtime_failures_preserved':2,'production_adoption':False,'scope':'Only original case47 pure-E2 128-step main and scheduled first-visit cancellation preservation. No retry behavior, broader refinement roster, endpoint reference error/temporal order, geometry-band repair, smooth-root/IEEE/Lean proof or production adoption.'}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
