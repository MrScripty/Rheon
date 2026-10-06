"""Read the one frozen native execution; never launch numerical work."""
from pathlib import Path
import copy,json,math,re,struct,sys
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
STAGES=['BeforeForceInputs','ForceInput','BeforeGeometry','Quadrature','BeforeSolve','Iteration','BeforeAcceptance','BeforeThirdSolve','ThirdAssembly','AfterThirdSolve','BeforePublish','unused index']
def require(ok,message):
 if not ok:raise ValueError(message)
def debug(text):
 text=re.sub(r'\b[A-Za-z_][A-Za-z_0-9]*\s*\{','{',text)
 text=re.sub(r'\b([A-Za-z_][A-Za-z_0-9]*)\s*:',r'"\1":',text)
 return json.loads(text)
def rows(path):return [json.loads(l) for l in path.read_text().splitlines() if l.startswith('{')]
def bits(x):return struct.unpack('<Q',struct.pack('<d',x))[0]
def valid(result):
 require(result['status']=='SUCCESS_FIXED_PUBLIC_CALL_ONLY','actual success classification')
 require(result['memory_gate']=='PASS_REVISED_66K_PROTOCOL_MEMORY_GATE' and result['memory_bound']==65552 and result['new_cap']==67584 and result['old_cap_status']=='BLOCKED_ADDITIONAL_MEMORY_CAP','separate old/new resource results')
 require(result['original_unchanged'] is True and result['parameter_search'] is False and result['continued_trajectory'] is False and result['production_adoption'] is False,'fixed scope and original preserved')
 r=result['report'];p=r['step']['planar'];t=r['step']['third'];s=result['accepted_snapshot']
 require(p['before']=={'id':131,'version':1} and p['after']==s['stamp']=={'id':131,'version':2},'single isolated owner publication')
 require(p['dt']==0.00078125 and p['time_after']==0.0015625 and s['time']==bits(0.0015625),'fixed time and stored bits')
 require(p['iterations']==4 and p['equation_evaluations']==22 and result['main_corrections']==3,'four checks, three corrections, 22 calls')
 require(len(s['velocity'])==16 and all(len(v)==3 for v in s['velocity']) and len(s['pressure'])==16 and len(s['positions'])==19 and len(s['mass'])==16 and len(s['triangles'])==24 and len(s['periodic'])==19,'complete accepted field capture')
 for a in [p,t]:
  for k in ['finite_momentum_rate_norm','direct_momentum_rate_norm']:
   require(math.isfinite(a[k]) and 0<=a[k]<=1e-13,'ordinary reported momentum gate '+k)
  for k in ['ledger_error','residual_work']:require(abs(a[k])<=a['work_allowance'],'reported work gate '+k)
  for k in ['backward_euler_loss','mixing_loss','viscous_loss']:require(a[k]>=0,'nonnegative loss '+k)
 require(p['quadrature_error']<=1e-15 and p['full_constraints']<=1e-13 and p['mass_before']==p['mass_after']==3.375,'reported geometry/quadrature/mass gates')
 require(abs(p['pressure_work'])<=p['work_allowance'],'pressure work gate')
 require(abs(r['step']['ledger_error'])<=r['step']['work_allowance'] and abs(r['step']['residual_work'])<=r['step']['work_allowance'],'combined work gates')
 require(r['forces']['acceleration']==[0.0625,-0.125,0.03125] and r['forces']['force_count']==1,'unchanged load')
 require(result['seen']==[1,1,1,1738,1,4,1,1,24,1,1,0],'actual fixed-call barrier observations')
 require(result['controls']==[{'stage':i,'name':STAGES[i],'status':'CANCEL_PRESERVES_AND_RETRY_REPRODUCES' if i<11 else 'NOT_REACHED_BY_FIXED_CALL'} for i in range(12)],'each reached control and honest unused stage')
 require(len(result['main_newton_checks'])==4 and [x['iteration'] for x in result['main_newton_checks']]==[1,2,3,4] and [x['calls'] for x in result['main_newton_checks']]==[1,8,15,22],'fixed main trace')
 require(result['main_newton_checks'][-1]['rate_norm']<=1e-13,'main Newton convergence')
def run():
 out=rows(P/'candidate-native.log');err=rows(P/'candidate-native-stderr.log')
 require('test result: ok. 1 passed; 0 failed' in (P/'candidate-native.log').read_text(),'actual Rust assertions passed')
 require([x['phase'] for x in out if x['event']=='public_phase']==['original_prefix','E1_fixed_public_call'],'one original prefix and one main call')
 e=[x for x in out if x['event']=='E1_public_result'];require(len(e)==1,'exactly one fixed main result');e=e[0]
 require(e['result'].startswith('Ok(') and e['result'].endswith(')'),'actual public success')
 prefix=[x for x in out if x['event']=='verified_prefix'];require(len(prefix)==1 and prefix[0]['version']==1 and prefix[0]['time']==0.00078125,'verified native prefix')
 prior_prefix=[x for x in rows(ROOT/'evidence/forcing-public-call/baseline-v2-native.log') if x['event']=='verified_prefix']
 require(prior_prefix==prefix,'exact original successful prefix report reproduced')
 # The first version-1 trace precedes all cancellation controls. Stop at the
 # next iteration-1 reset, rather than attributing retries to the main call.
 i=next(i for i,x in enumerate(err) if x.get('event')=='newton_check' and x.get('accepted_version')==1)
 trace=err[i:];j=next((j for j,x in enumerate(trace[1:],1) if x.get('event')=='newton_check' and x['iteration']==1),len(trace));trace=trace[:j]
 done=[x for x in out if x['event']=='public_experiment_complete'];require(len(done)==1 and done[0]=={'event':'public_experiment_complete','new_cases':0,'parameters_changed':False,'original_unchanged':True},'completed original-state assertion and fixed scope')
 controls=[{'stage':x['stage'],'name':STAGES[x['stage']],'status':x['status']} for x in out if x['event']=='cancellation_control']
 r={'status':'SUCCESS_FIXED_PUBLIC_CALL_ONLY','memory_gate':'PASS_REVISED_66K_PROTOCOL_MEMORY_GATE','memory_bound':65552,'new_cap':67584,'old_cap_status':'BLOCKED_ADDITIONAL_MEMORY_CAP','report':debug(e['result'][3:-1]),'accepted_snapshot':debug(e['accepted']),'seen':e['seen'],'controls':controls,'main_corrections':sum(x['event']=='correction' for x in trace),'main_newton_checks':[x for x in trace if x['event']=='newton_check'],'original_unchanged':True,'parameter_search':False,'continued_trajectory':False,'production_adoption':False,'prefix_report':debug(prefix[0]['report']),'limitations':['one fixed case and one isolated step','no continued trajectory qualification','research chart remains private; no production adoption','stored endpoint remains authoritative; no persistent compensation','2.5D model; no full 3D, density, adhesion or surface reconstruction qualification','original arithmetic failures and all proof limitations retained']}
 valid(r);return r
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
