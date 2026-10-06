"""Fresh linked kernel/caller audit for the fixed-candidate capture paths."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,re
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 base=module('new_binary_base_audit',P/'audit_base.py').run();helper=module('static_helper',ROOT/'evidence/forcing-increment-memory-audit/audit.py');raw=gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()).decode()
 selected=''.join(part for part in re.split(r'(?m)(?=^[a-f0-9]+ <)',raw)if part.splitlines()and 'research_public_call::'in part.splitlines()[0]and any('::'+k+'::'in part.splitlines()[0]for k in ['reference','candidate','scalar']))
 fs=helper.functions(selected,(P/'native-symbols.txt').read_text());categories=['fixed_capture','Work::capture_equations','Work::capture_point','Work::equation','Work::partition','Work::point'];kernel={a for a,f in fs.items()if '::scalar::'in f['name']and not f['name'].endswith('::conformance')};graphs={};frames={};starts={}
 for kind in ['reference','candidate']:
  start=next(a for a,f in fs.items()if f['name'].endswith(kind+'::fixed_capture'));starts[kind]=start;pending=[start];seen=set();graph={};rows={}
  while pending:
   a=pending.pop()
   if a in seen:continue
   seen.add(a);f=fs[a];edges=helper.transfers(f);targets=[]
   for edge in edges:
    target=edge['target']
    if edge['kind']=='outgoing_jump':require(target in fs and ('::'+kind+'::')in fs[target]['name'],'resolved internal capture tail transfers')
    if edge['kind']in ['call','outgoing_jump']and target in fs and ('::'+kind+'::')in fs[target]['name']and fs[target]['name'].endswith(tuple(categories)):targets.append(target)
   rows[a]={'name':f['name'],**helper.fixed_frame(f),'transfers':edges};graph[a]=sorted(set(targets));pending.extend(targets)
  graphs[kind]=graph;frames[kind]=rows
 deltas={}
 for category in categories:
  pair={kind:[f['frame_bound']for f in frames[kind].values()if f['name'].endswith(category)]for kind in starts}
  if not pair['candidate']and not pair['reference']:continue
  require(pair['candidate']and pair['reference'],'matched capture operation '+category)
  deltas[category]={'reference_frames':pair['reference'],'candidate_frames':pair['candidate'],'positive_delta':max(0,max(pair['candidate'])-min(pair['reference']))}
 def peak(a,path=()):
  require(a not in path,'acyclic capture numerical callers');category=next(c for c in deltas if fs[a]['name'].endswith(c));children=[peak(c,path+(a,))for c in graphs['candidate'][a]]
  if any(e['kind']=='call'and e['target']in kernel for e in frames['candidate'][a]['transfers']):children.append((base['kernel_peak'],['actual chart kernel']))
  value,names=max(children or [(0,[])],key=lambda x:x[0]);return deltas[category]['positive_delta']+value,[fs[a]['name'],*names]
 extra,route=peak(starts['candidate']);bound=(62096+8+extra+15)//16*16
 result={'status':'PASS_FIXED_CANDIDATE_MEMORY_PREFLIGHT'if bound<=67584 else'BLOCKED_FIXED_CANDIDATE_MEMORY_CAP','additional_cap':67584,'aligned_additional_bound':bound,'remaining':67584-bound,'workspace_bytes':62096,'descriptor_bytes':8,'capture_added_stack_peak':extra,'capture_peak_route':route,'capture_frame_deltas':deltas,'capture_graphs':graphs,'capture_frames':frames,'fresh_same_binary_public_constructor_audit':base,'kernel_peak':base['kernel_peak'],'no_shrink_credit':True,'no_baseline_slack_credit':True,'accepted_owners_constructed':0,'accepted_owners_advanced':0,'e1_fixed_candidates_executed':False,'scope':'Matched fixed-candidate numerical capture frames, original linked kernel paths and fixed workspace. Common bounded journal formatting and external evidence storage are diagnostics; not a process-wide RSS bound.'}
 return json.loads(json.dumps(result))
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
