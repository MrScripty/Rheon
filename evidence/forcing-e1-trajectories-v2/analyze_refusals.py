"""Bind the actual two refusals and complete repeated controller traces."""
from pathlib import Path
import json
P=Path(__file__).resolve().parent
def require(ok,message):
 if not ok:raise ValueError(message)
def run():
 native=json.loads((P/'native-outcome.json').read_text());answer=[]
 for case in native['rows']:
  if case['mode']!='trajectory' or case['status']!='REFUSED':continue
  index=case['index'];data=[json.loads(l)for l in(P/f'trajectory-{index}-native-stderr.log').read_text().splitlines()if l.startswith('{')];groups=[]
  for row in data:
   if row['event']=='newton_check' and row['iteration']==1:groups.append([])
   if groups:groups[-1].append(row)
  require(len(groups)==case['accepted_steps']+2,'accepted prefix plus original and repeated refusal')
  first,repeat=groups[-2:];require(first==repeat,'every complete repeated refusal trace exactly identical')
  checks=[q for q in first if q['event']=='newton_check'];corrections=[q for q in first if q['event']=='correction'];terminal=[q for q in first if q['event']=='terminal_validation'];refusals=[q for q in first if q['event']=='refusal']
  require(len(checks)==len(corrections)==7 and len(terminal)==len(refusals)==1 and [q['calls']for q in checks]==[1,8,15,22,29,36,43],'original seven-correction budget')
  t=terminal[0];require(t['calls']==50 and t['corrections']==7 and t['newton_threshold']==1e-13 and t['rate_norm']>1e-13 and t['converged'] is False,'genuine failed final authorized validation')
  require(all(q['accepted_version']==case['accepted_steps'] and q['h']==case['h'] and q['newton_threshold']==1e-13 and q['iteration_limit']==7 for q in checks),'same owner and original criteria')
  answer.append({'index':index,'kind':case['kind'],'field':case['field'],'load':case['load'],'h':case['h'],'accepted_steps':case['accepted_steps'],'attempted_step':case['attempted_step'],'expected_steps':case['expected_steps'],'accepted_time':checks[0]['accepted_time'],'corrections':7,'counted_equations':50,'final_authorized_Newton_norm':t['rate_norm'],'Newton_signed_margin':1e-13-t['rate_norm'],'same_owner_repeat_full_trace_exact':True,'accepted_state_preserved':True,'checks':checks,'corrections_trace':corrections,'terminal_validation':t})
 require([x['index']for x in answer]==[41,47],'all actual refusals retained')
 return {'status':'PASS_EXACT_REPEATED_REFUSAL_EVIDENCE','refused_trajectories':2,'no_fabricated_final_endpoints':True,'original_target':1e-13,'rows':answer}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
