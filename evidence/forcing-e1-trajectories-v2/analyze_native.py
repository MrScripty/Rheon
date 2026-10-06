"""Read all actual journals; bind commands, state bits, reports and lifecycle."""
from pathlib import Path
import hashlib,importlib.util,json,math,struct
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
spec=importlib.util.spec_from_file_location('debug_reader',ROOT/'evidence/forcing-public-call-66k/analyze.py');reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def rows(path):return [json.loads(l)for l in path.read_text().splitlines()if l.startswith('{')]
def bits(x):return struct.unpack('<Q',struct.pack('<d',x))[0]
def snapshot(row):return {'velocity':[[bits(q)for q in v]for v in row['velocity']],'positions':[[bits(q)for q in v]for v in row['positions']],'mass':[bits(q)for q in row['mass']],'pressure':[bits(q)for q in row['pressure_coefficients']],'triangles':row['triangles'],'periodic':row['periodic_indices'],'time':bits(row['time']),'stamp':row['stamp']}
def bit_tree(value):
 if isinstance(value,float):return ('float',bits(value))
 if isinstance(value,list):return [bit_tree(v)for v in value]
 if isinstance(value,dict):return {k:bit_tree(v)for k,v in value.items()}
 return value
def run():
 roster=json.loads((P/'roster.json').read_text());binding=json.loads((P/'binary-binding.json').read_text());authorization=json.loads((P/'execution-authorization.json').read_text())
 require(authorization['source']==binding['prototype'] and authorization['binary_sha256']==sha(Path(binding['binary']).read_bytes())==binding['binary_sha256'],'executed exact source/binary')
 for key,name in [('protocol_sha256','protocol.json'),('qualification_sha256','qualification-normal.json'),('preflight_receipt_sha256','preflight-receipt.json'),('runner_sha256','run.py'),('roster_sha256','roster.json')]:require(authorization[key]==sha((P/name).read_bytes()),'frozen launch '+name)
 require(authorization['ordinary_indices']==list(range(48)) and authorization['lifecycle_controls']==roster['lifecycle_controls'] and not authorization['parameter_search'] and not authorization['production_adoption'],'exact experiment scope')
 outputs=[];journals={};publications={};metrics=[];cancellations=[]
 jobs=[('trajectory',i)for i in range(48)]+[('lifecycle',c['trajectory_index'])for c in roster['lifecycle_controls']]
 for mode,index in jobs:
  label=f'{mode}-{index}';case=roster['trajectories'][index];command=json.loads((P/(label+'-command-completed.json')).read_text());before=json.loads((P/(label+'-command-before.json')).read_text())
  require({k:v for k,v in command.items()if k not in ['exit','seconds','stdout_sha256','stderr_sha256']}==before,'actual completed command '+label)
  require(command['argv']==[binding['binary'],'--exact','research_public_call::research_repeated_trajectory_one','--nocapture'] and command['mode']==mode and command['trajectory']==case and command['exit']==0 and command['binary_sha256']==binding['binary_sha256'] and command['authorization_sha256']==sha((P/'execution-authorization.json').read_bytes()),'actual fixed native launch '+label)
  env={'RHEON_PUBLIC_CALL_ALLOW':'E1-original-complete-trajectory-66KiB-v2','RHEON_TRAJECTORY_INDEX':str(index)}
  if mode=='lifecycle':env['RHEON_LIFECYCLE_CONTROL']='1'
  require(command['env']==env and not command['parameter_search'] and not command['production_adoption'],'exact native command environment')
  for channel in ['stdout','stderr']:
   name=label+('-native.log'if channel=='stdout'else'-native-stderr.log');require(command[channel+'_sha256']==sha((P/name).read_bytes()),'actual native journal '+name)
  data=rows(P/(label+'-native.log'));require('1 passed; 0 failed' in(P/(label+'-native.log')).read_text(),'native assertions passed '+label)
  trace_groups=[]
  for trace in rows(P/(label+'-native-stderr.log')):
   if trace.get('event')=='newton_check' and trace['iteration']==1:trace_groups.append([])
   if trace_groups:trace_groups[-1].append(trace)
  begins=[q for q in data if q.get('event')=='trajectory_begin'];require(begins==[{'event':'trajectory_begin','index':index,'lifecycle_control':mode=='lifecycle','expected_steps':case['steps']}],'one same-owner trajectory')
  pubs=[q for q in data if 'model'in q];attempts=[q for q in data if q.get('event')=='trajectory_attempt'];results=[q for q in data if q.get('event')=='trajectory_result'];terminal=[q for q in data if q.get('event')=='trajectory_terminal'];require(len(terminal)==1,'one actual terminal');terminal=terminal[0]
  require(terminal['index']==index and terminal['expected_steps']==case['steps'] and len(pubs)==terminal['accepted_steps']+1,'actual retained prefix length')
  original=roster['constructor_inputs'][case['input_index']]['publication'];require(bit_tree(snapshot(pubs[0]))==bit_tree(snapshot(original)),'original exact constructor fields')
  clock=0.
  for step,row in enumerate(pubs):
   require((row['kind'],row['field'],row['load'],row['h'])==(case['kind'],case['field'],case['load'],case['h']) and row['step']==step and row['stamp']=={'id':131,'version':step} and row['allocated_bytes']==542352,'actual published identity and reservation')
   if step:clock+=case['h']
   require(bits(row['time'])==bits(clock),'original repeated-addition clock')
  require(len(attempts)==len(results)==terminal['accepted_steps']+(terminal['status']=='REFUSED'),'actual attempted advance count')
  for step,(attempt,result) in enumerate(zip(attempts,results),1):
   require(attempt['index']==result['index']==index and attempt['step']==result['step']==step and reader.debug(attempt['before'])==snapshot(pubs[step-1]),'all accepted fields before call')
   accepted=reader.debug(result['accepted'])
   if step<len(pubs):
    require(result['result'].startswith('Ok(') and accepted==snapshot(pubs[step]),'all accepted fields belong to successful publication')
    report=reader.debug(result['result'][3:-1]);planar=report['step']['planar'];third=report['step']['third'];row=pubs[step];previous=pubs[step-1]
    require(planar['before']==previous['stamp'] and planar['after']==row['stamp'] and bits(planar['dt'])==bits(case['h']) and bits(planar['time_after'])==bits(clock if step==len(pubs)-1 else row['time']),'public report lineage')
    require(bit_tree(planar['unknowns'])==bit_tree(row['unknowns']) and bit_tree(planar['end_q'])==bit_tree(row['end_q']) and bit_tree(planar['end_eta'])==bit_tree(row['end_eta']) and bit_tree(third['coefficients'])==bit_tree(row['third_coefficients']) and bit_tree(report['forces'])==bit_tree(row['forcing']),'public report and actual emitted fields')
    p=row['report']['planar'];t=row['report']['third'];require(1<=p['iterations']<=7 and 0<p['equation_evaluations']<=200 and result['seen'][10]==1,'bounded successful native controller')
    groups=[g for g in trace_groups if g[0]['accepted_version']==step-1];require(groups,'actual Newton trace for every successful step');group=groups[-1];checks=[g for g in group if g['event']=='newton_check'];corrections=[g for g in group if g['event']=='correction'];last=group[-1]
    require(len(corrections)<=7 and len(checks)==p['iterations'] and [q['iteration']for q in checks]==list(range(1,p['iterations']+1)) and [q['calls']for q in checks]==[1+7*j for j in range(p['iterations'])],'actual original counted controller equations')
    terminal_checks=[q for q in group if q['event']=='terminal_validation'];final=terminal_checks[-1]if terminal_checks else checks[-1]
    require(final['rate_norm']<=1e-13 and final['calls']==p['equation_evaluations'] and all(q['h']==case['h'] and q['accepted_time']==previous['time'] and q['newton_threshold']==1e-13 and q['iteration_limit']==7 for q in checks),'unchanged actual Newton target and terminal count')
    metric={'mode':mode,'index':index,'step':step,'corrections':len(corrections),'counted_equations':p['equation_evaluations'],'total_equations':p['equation_evaluations']+2,'final_Newton_rate_norm':final['rate_norm'],'Newton_signed_margin':1e-13-final['rate_norm'],'finite_full_norm':math.hypot(p['finite_momentum_rate_norm'],t['finite_momentum_rate_norm']),'direct_full_norm':math.hypot(p['direct_momentum_rate_norm'],t['direct_momentum_rate_norm']),'full_constraints':p['full_constraints'],'quadrature_error':p['quadrature_error'],'reported_GCL_max':p['gcl_max'],'mass_before_error':abs(planar['mass_before']-3.375),'mass_after_error':abs(planar['mass_after']-3.375),'third_impulse_error':abs(t['momentum_after']-t['momentum_before']-row['forcing']['third_impulse'])}
    for key in ['finite_full_norm','direct_full_norm','full_constraints','mass_before_error','mass_after_error','third_impulse_error']:require(math.isfinite(metric[key]) and 0<=metric[key]<=1e-11,'original native physical gate '+key);metric[key+'_signed_margin']=1e-11-metric[key]
    require(metric['quadrature_error']<=1e-15,'original quadrature gate');metric['quadrature_signed_margin']=1e-15-metric['quadrature_error']
    metric['strict_frozen_reported_planar_margin']=1e-13-p['finite_momentum_rate_norm'];metric['work']={}
    for part in ['planar','third','total']:
     w=row['report'][part];allow=w['fixed_work_allowance'];errors=[abs(w['discrete_residual_work']),abs(w['energy_ledger_error'])]+([abs(w['pressure_work'])]if part=='planar'else[])
     require(allow>0 and max(errors)<=allow and min(w[k]for k in ['backward_Euler_loss','mixing_loss','viscous_loss'])>=0,'original separate/combined work gates')
     metric['work'][part]={'allowance':allow,'maximum_error':max(errors),'signed_margin':allow-max(errors),'ratio':max(errors)/allow}
    if case['field']=='constant':
     deviation=max(abs(v[2]-(.25+row['time']*row['forcing']['acceleration'][2]))for v in row['velocity']);limit=128*2.220446049250313e-16*.25;require(deviation<=limit,'original constant third gate');metric.update(constant_deviation=deviation,constant_signed_margin=limit-deviation)
    metrics.append(metric)
   else:require(result['result'].startswith('Err(') and accepted==snapshot(pubs[-1]),'refusal preserves every accepted bit')
  if terminal['status']=='COMPLETE':require(terminal['accepted_steps']==case['steps'] and bits(terminal['actual_time'])==bits(clock),'actual complete prescribed endpoint')
  else:require(terminal['status']=='REFUSED' and terminal['accepted_steps']<case['steps'] and terminal['attempted_step']==len(pubs) and terminal['same_owner_repeat']=='EXACT_REFUSAL_AND_STATE' and terminal['state_preserved'] is True and results[-1]['result']=='Err('+terminal['error']+')','actual identical repeated refusal')
  cancels=[q for q in data if q.get('event')=='lifecycle_cancel']
  if mode=='trajectory':require(not cancels,'ordinary run has no interruptions')
  else:
   target=next(c for c in roster['lifecycle_controls']if c['trajectory_index']==index);expected=[(t['step'],t['stage'],t['visit'],rep)for t in target['interruptions']if t['step']<=len(attempts)for rep in range(2)]
   if terminal['status']=='COMPLETE':require([(c['step'],c['stage'],c['visit'],c['repetition'])for c in cancels]==expected,'all scheduled repeated cancellations')
   for c in cancels:
    require(c['index']==index and (c['step'],c['stage'],c['visit'],c['repetition'])in expected,'prescribed control target')
    if c['status']=='CANCELLED_EXACT_STATE_PRESERVED':require(c['seen'][c['stage']]==c['visit'] and reader.debug(c['accepted'])==snapshot(pubs[c['step']-1]),'repeated cancellation exact accepted state')
    else:require(c['status']=='TARGET_NOT_REACHED_REFUSAL' and c['result'].startswith('Err('),'honest unreachable control')
   require(bit_tree(pubs)==bit_tree(publications[index]) and results==journals[index]['results'] and terminal==journals[index]['terminal'],'interrupted retries reproduce all reports and accepted states exactly')
   cancellations.extend(cancels)
  if mode=='trajectory':publications[index]=pubs;journals[index]={'results':results,'terminal':terminal}
  outputs.append({'mode':mode,**case,**terminal,'publication_count':len(pubs),'cancellations':len(cancels),'command_seconds':command['seconds']})
 ordinary=[o for o in outputs if o['mode']=='trajectory'];lifecycle=[o for o in outputs if o['mode']=='lifecycle']
 return {'status':'PASS_NATIVE_JOURNAL_AND_LIFECYCLE_AUDIT','source':binding['prototype'],'binary_sha256':binding['binary_sha256'],'original_trajectories':48,'ordinary_completed':sum(o['status']=='COMPLETE'for o in ordinary),'ordinary_refused':sum(o['status']=='REFUSED'for o in ordinary),'ordinary_accepted_steps':sum(o['accepted_steps']for o in ordinary),'lifecycle_completed':sum(o['status']=='COMPLETE'for o in lifecycle),'lifecycle_accepted_steps':sum(o['accepted_steps']for o in lifecycle),'actual_repeated_cancellations':len(cancellations),'cancellation_targets_not_reached':sum(c['status']!='CANCELLED_EXACT_STATE_PRESERVED'for c in cancellations),'memory_bound':65536,'memory_remaining':2048,'production_adoption':False,'parameter_search':False,'rows':outputs,'metrics':metrics,'cancellations':cancellations}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
