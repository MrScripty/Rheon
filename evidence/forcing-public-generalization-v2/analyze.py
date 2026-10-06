"""Read every prescribed once-only result and exact field changes; never run it."""
from pathlib import Path
import copy,importlib.util,json,math,struct,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
spec=importlib.util.spec_from_file_location('frozen_debug_reader',ROOT/'evidence/forcing-public-call-66k/analyze.py');old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
def require(ok,message):
 if not ok:raise ValueError(message)
def rows(path):return [json.loads(l) for l in path.read_text().splitlines() if l.startswith('{')]
def bits(x):return struct.unpack('<Q',struct.pack('<d',x))[0]
def f64(x):return struct.unpack('<d',struct.pack('<Q',x))[0]
def changes(before,after,path=''):
 answer=[]
 if isinstance(before,dict):
  require(before.keys()==after.keys(),'exact snapshot schema')
  for k in before:answer.extend(changes(before[k],after[k],path+'.'+k if path else k))
 elif isinstance(before,list):
  require(len(before)==len(after),'exact snapshot shape')
  for i,(a,b) in enumerate(zip(before,after)):answer.extend(changes(a,b,f'{path}[{i}]'))
 elif before!=after:
  item={'path':path,'before_bits' if path.split('[')[0] in ['velocity','positions','mass','pressure','time'] else 'before':before,'after_bits' if path.split('[')[0] in ['velocity','positions','mass','pressure','time'] else 'after':after}
  if 'before_bits' in item:item.update(before_value=f64(before),after_value=f64(after))
  answer.append(item)
 return answer
def valid(r):
 require(r['case_count']==5 and r['main_calls_per_case']==1 and r['parameter_search'] is False and r['continued_trajectory'] is False and r['production_adoption'] is False,'fixed complete roster and stopping rule')
 require(r['physical_momentum_limit']==1e-11 and r['newton_target']==1e-13 and r['additional_cap']==67584,'unchanged criteria and authorized memory')
 cases=json.loads((P/'cases.json').read_text())['cases'];require(len(r['cases'])==5 and [x['index'] for x in r['cases']]==list(range(5)),'every original prescribed input included')
 for c,x in zip(cases,r['cases']):
  require(x['id']==c['id'] and x['h']==c['h'] and x['acceleration']==c['acceleration'] and x['before_snapshot']==c['accepted_snapshot'],'exact fixed accepted input')
  require(x['original_owner_unchanged'] is True and x['native_test_exit']==0,'actual native state assertions')
  require(x['changes']==changes(x['before_snapshot'],x['after_snapshot']),'every exact field change captured')
  if x['status']=='SUCCESS':
   report=x['report'];step=report['step'];planar=step['planar'];third=step['third']
   require(x['error'] is None and x['published'] is True and x['third_result']=='PASS' and x['combined_work_result']=='PASS','full successful public result')
   require(planar['before']==c['accepted_snapshot']['stamp'] and planar['after']==x['after_snapshot']['stamp']=={'id':131,'version':c['prefix_version']+1},'single coherent publication')
   require(planar['dt']==c['h'] and planar['time_after']==c['prefix_time']+c['h'] and x['after_snapshot']['time']==bits(planar['time_after']),'fixed time increment')
   require(x['counted_newton_equations']==planar['equation_evaluations']==x['main_checks'][-1]['calls'] and x['final_acceptance_equations']==2 and x['total_completed_equations']==planar['equation_evaluations']+2,'honest Newton/acceptance equation counts')
   require(x['corrections']==len(x['main_checks'])-1 and len(x['main_checks'])==planar['iterations'] and x['main_checks'][-1]['rate_norm']<=1e-13,'actual Newton convergence')
   for a in [planar,third]:
    for k in ['finite_momentum_rate_norm','direct_momentum_rate_norm']:require(math.isfinite(a[k]) and 0<=a[k]<=1e-11,'original physical momentum criterion')
    for k in ['ledger_error','residual_work']:require(abs(a[k])<=a['work_allowance'],'original work criterion')
    for k in ['backward_euler_loss','mixing_loss','viscous_loss']:require(a[k]>=0,'nonnegative reported losses')
   require(abs(planar['pressure_work'])<=planar['work_allowance'] and planar['quadrature_error']<=1e-15 and planar['full_constraints']<=1e-11 and abs(planar['mass_after']-planar['mass_before'])<=1e-11,'unchanged physical constraints/quadrature/mass gates')
   require(abs(step['ledger_error'])<=step['work_allowance'] and abs(step['residual_work'])<=step['work_allowance'],'combined work qualification')
   require(abs(third['momentum_after']-third['momentum_before']-report['forces']['third_impulse'])<=1e-11,'third force impulse balance')
   require(report['forces']['acceleration']==c['acceleration'] and report['forces']['force_count']==1,'fixed force loading')
   require(all(x['seen'][i]>0 for i in [6,7,8,9,10]) and x['seen'][11]==0,'actual third/work/publication barriers reached')
   require(x['after_snapshot']['triangles']==x['before_snapshot']['triangles'] and x['after_snapshot']['periodic']==x['before_snapshot']['periodic'],'fixed topology and periodic mapping')
  else:
   require(x['status']=='REFUSED' and x['published'] is False and x['report'] is None and x['error'] is not None and x['after_snapshot']==x['before_snapshot'],'honest retained refusal and exact accepted-state preservation')
 require(r['successes']==sum(x['status']=='SUCCESS' for x in r['cases']) and r['refusals']==[x['id'] for x in r['cases'] if x['status']=='REFUSED'],'no case selection or failure suppression')
def run():
 cases=json.loads((P/'cases.json').read_text())['cases'];answer=[]
 for c in cases:
  i=c['index'];command=json.loads((P/f'case-{i}-command-completed.json').read_text());stdout=P/f'case-{i}-native.log';out=rows(stdout);err=rows(P/f'case-{i}-native-stderr.log')
  require(command['exit']==0 and 'test result: ok. 1 passed; 0 failed' in stdout.read_text(),'actual prescribed Rust test passed')
  prefix=[x for x in out if x['event']=='verified_prefix'];result=[x for x in out if x['event']=='generalization_public_result'];begins=[x for x in out if x['event']=='main_call_begin'];done=[x for x in out if x['event']=='generalization_complete']
  require(len(prefix)==len(result)==len(begins)==len(done)==1 and all(x['case']==i for x in [prefix[0],result[0],begins[0],done[0]]),'one fixed case/prefix/main call/result')
  before=old.debug(prefix[0]['accepted']);after=old.debug(result[0]['accepted']);require(before==c['accepted_snapshot'],'bit-exact accepted prefix')
  require(begins[0]['allocated_bytes']==542352 and done[0]=={'event':'generalization_complete','case':i,'original_unchanged':True,'main_calls':1,'continued_trajectory':False,'parameter_search':False},'authorized reservation and actual completed fixed scope')
  text=result[0]['result'];succeeded=text.startswith('Ok(');report=old.debug(text[3:-1]) if succeeded else None
  start=next(j for j,x in enumerate(err) if x.get('accepted_version')==c['prefix_version'] and x.get('event')=='newton_check');main_events=err[start:]
  main=[x for x in main_events if x.get('event')=='newton_check'];require(main and [x['iteration'] for x in main]==list(range(1,len(main)+1)),'one actual main Newton window')
  seen=result[0]['seen'];native_counter=report['step']['planar']['equation_evaluations'] if succeeded else next((x['calls'] for x in reversed(main_events) if x['event'] in ['terminal_validation','refusal'] and 'calls' in x),main[-1]['calls'])
  a={'index':i,'id':c['id'],'role':c['role'],'h':c['h'],'acceleration':c['acceleration'],'status':'SUCCESS' if succeeded else 'REFUSED','error':None if succeeded else text,'report':report,'before_snapshot':before,'after_snapshot':after,'changes':changes(before,after),'seen':seen,'main_checks':main,'corrections':sum(x['event']=='correction' for x in main_events),'counted_newton_equations':native_counter,'final_acceptance_equations':2 if succeeded else 0 if seen[6]==0 else None,'total_completed_equations':native_counter+2 if succeeded else native_counter if seen[6]==0 else None,'third_result':'PASS' if succeeded else 'NOT_REACHED' if seen[7]==0 else 'NO_PUBLIC_REPORT_REFUSED','combined_work_result':'PASS' if succeeded else 'NOT_REACHED' if seen[9]==0 else 'NO_PUBLIC_REPORT_REFUSED','published':succeeded,'original_owner_unchanged':True,'native_test_exit':command['exit'],'seconds':command['seconds']}
  answer.append(a)
 old_result=rows(ROOT/'evidence/forcing-public-call-66k/candidate-native.log');control=next(x for x in old_result if x['event']=='E1_public_result');new_control=answer[3]
 r={'status':'COMPLETE_BOUNDED_FIVE_CALL_GENERALIZATION','case_count':5,'main_calls_per_case':1,'successes':sum(x['status']=='SUCCESS' for x in answer),'refusals':[x['id'] for x in answer if x['status']=='REFUSED'],'cases':answer,'passing_control_full_report_matches_frozen_one_step':new_control['report']==old.debug(control['result'][3:-1]),'passing_control_accepted_bits_match_frozen_one_step':new_control['after_snapshot']==old.debug(control['accepted']),'newton_target':1e-13,'physical_momentum_limit':1e-11,'additional_cap':67584,'parameter_search':False,'continued_trajectory':False,'production_adoption':False,'limitations':['five fixed terminal-repair refusal inputs; not all earlier pre-repair trajectory cases','one E1 step from each preserved prefix, no successful trajectory continued','new binary observer records first-stage visits; no new cancellation controls run','ephemeral chart arithmetic and stored authoritative endpoint; no persistent compensation','original physical/material/2.5D/surface/numerical/Lean limitations unchanged']}
 valid(r);return r
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
