"""Qualify the new linked controller binary, never invoke the candidate."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,re
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
spec=importlib.util.spec_from_file_location('static_helpers',ROOT/'evidence/forcing-increment-memory-audit/audit.py')
helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
def run():
 binding=json.loads((P/'binary-binding.json').read_text())
 require(sha(Path(binding['binary']).read_bytes())==binding['binary_sha256'],'actual new binary identity')
 policy=json.loads((P/'protocol.json').read_text())
 for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen v2 source')
 raw=gzip.decompress((P/'native-disassembly.txt.gz').read_bytes())
 require(sha(raw)==(P/'native-disassembly.txt.sha256').read_text().strip(),'lossless new linked disassembly')
 selected=''.join(part for part in re.split(r'(?m)(?=^[a-f0-9]+ <)',raw.decode()) if 'research_public_call::' in (part.splitlines()[0] if part.splitlines() else '') and any(x in (part.splitlines()[0] if part.splitlines() else '') for x in ('candidate::','reference::','scalar::')))
 fs=helper.functions(selected,(P/'native-symbols.txt').read_text())
 kernel={a:f for a,f in fs.items() if 'research_public_call::scalar::' in f['name'] and not f['name'].endswith('::conformance')}
 require(len(kernel)==7,'all new binary kernel functions captured')
 symbols=(P/'native-symbols.txt').read_text();relocs=(P/'native-relocations.txt').read_text()
 panic=re.search(r'^([a-f0-9]+) [a-f0-9]+ T core::panicking::panic_bounds_check$',symbols,re.M).group(1).lstrip('0')
 def resolve(asm,target):
  if target in kernel:return target,'bounded_kernel'
  slot=re.search(r'# ([a-f0-9]+) ',asm)
  if slot and '<memset@' in asm:return 'memset','new_binary_default_libc_leaf'
  if slot and re.search(r'^0*'+slot[1]+r' .*R_X86_64_RELATIVE\s+'+panic+r'$',relocs,re.M):return 'panic_bounds_check','excluded_private_invariant_panic'
  raise ValueError('unresolved new kernel transfer '+asm)
 records={a:helper.kernel_cfg(f,resolve) for a,f in kernel.items()}
 # Recheck the installed default leaf's entire graph; no prior stack bound is
 # imported. Shared libc instructions are inputs, not a controller certificate.
 old=ROOT/'evidence/forcing-increment-comparison';libc=json.loads((old/'memset-binding.json').read_text())
 require(sha(Path(libc['library']).read_bytes())==json.loads((old/'memory-preflight-staged.json').read_text())['libc_sha256'],'same actual installed libc')
 code={}
 for line in (old/'memset-leaf-disassembly.txt').read_text().splitlines():
  fields=line.split('\t')
  if len(fields)>=3 and re.match(r'^\s*[a-f0-9]+:$',fields[0]):code[int(fields[0].strip()[:-1],16)]=fields[-1].strip()
 def external(asm,target):raise ValueError('unresolved libc transfer '+asm)
 leaf=helper.kernel_cfg({'start':libc['resolved_file_address'],'code':code},external)
 require(leaf['frame_bound']==8 and not leaf['edges'],'closed actual default memset leaf')
 def peak(a,path=()):
  if a=='memset':return 8,['default memset']
  require(a not in path,'no recursive new kernel');r=records[a];best=r['frame_bound'];names=[fs[a]['name']]
  for edge in r['edges']:
   if edge['disposition']=='excluded_private_invariant_panic':continue
   child,route=peak(edge['callee'],path+(a,));value=8+edge['stack_depth']+child
   if value>best:best=value;names=[fs[a]['name'],*route]
  return best,names
 solve=next(a for a,f in kernel.items() if f['name'].endswith('ChartWorkspace::solve'))
 kernel_peak,kernel_route=peak(solve)
 # Match only variants actually reached by each public wrapper. Constructor
 # points with a different cancel closure cannot supply the native comparison.
 starts={kind:next(a for a,f in fs.items() if f['name'].endswith(kind+'::public_call')) for kind in ['reference','candidate']}
 graphs={};frames={};reachable={}
 for kind,start in starts.items():
  pending=[start];seen=set();graph={};rows={}
  while pending:
   a=pending.pop()
   if a in seen:continue
   seen.add(a);f=fs[a];edges=helper.transfers(f)
   outgoing=[e for e in edges if e['kind']=='outgoing_jump']
   require(not outgoing,'no omitted outgoing tail jumps in relevant controller callers')
   rows[a]={'name':f['name'],**helper.fixed_frame(f),'transfers':edges}
   targets=[e['target'] for e in edges if e['kind']=='call' and e['target'] in fs and ('::'+kind+'::') in fs[e['target']]['name'] and fs[e['target']]['name'].endswith(('Work::seed','Work::equation','Work::partition','Work::point'))]
   graph[a]=sorted(set(targets));pending.extend(targets)
  graphs[kind]=graph;frames[kind]=rows;reachable[kind]=seen
 deltas={}
 for suffix in ['public_call','Work::seed','Work::equation','Work::partition','Work::point']:
  pair={kind:[(a,r) for a,r in frames[kind].items() if r['name'].endswith(suffix)] for kind in starts}
  require(all(len(rows)==1 for rows in pair.values()),'one matched reachable public-call variant '+suffix)
  native_a,native=pair['reference'][0];new_a,new=pair['candidate'][0]
  deltas[suffix]={'reference_address':native_a,'candidate_address':new_a,'reference_frame':native['frame_bound'],'candidate_frame':new['frame_bound'],'positive_delta':max(0,new['frame_bound']-native['frame_bound'])}
 def extra_path(a,path=()):
  require(a not in path,'acyclic controller caller chain')
  name=fs[a]['name'];suffix=next(s for s in deltas if name.endswith(s))
  child=[extra_path(c,path+(a,)) for c in graphs['candidate'][a]]
  if suffix=='Work::point':
   require(sum(e['kind']=='call' and e['target']==solve for e in helper.transfers(fs[a]))==1,'one chart solve per point')
   child.append((kernel_peak,['chart kernel']))
  value,route=max(child or [(0,[])],key=lambda v:v[0])
  return deltas[suffix]['positive_delta']+value,[name,*route]
 added_stack,route=extra_path(starts['candidate'])
 layout=json.loads((P/'scalar-oracle.json').read_text())['layout']
 # Also audit direct E1 construction, conservatively charging any linked chart
 # path although construction initializes its private research borrow to None.
 ctor_suffixes=['public_constructor','CoupledDiscreteFlow::initialize','CoupledDiscreteFlow::new_forced_extruded','Work::seed','Work::equation','Work::partition','Work::point']
 ctor_graphs={};ctor_frames={};ctor_starts={}
 for kind in ['reference','candidate']:
  start=next(a for a,f in fs.items() if f['name'].endswith(kind+'::public_constructor'));ctor_starts[kind]=start
  pending=[start];seen=set();graph={};rows={}
  while pending:
   a=pending.pop()
   if a in seen:continue
   seen.add(a);f=fs[a];edges=helper.transfers(f)
   rows[a]={'name':f['name'],**helper.fixed_frame(f),'transfers':edges};targets=[]
   for edge in edges:
    target=edge['target']
    if edge['kind']=='outgoing_jump':require(target in fs and ('::'+kind+'::') in fs[target]['name'],'resolved internal constructor tail transfer')
    if edge['kind'] in ['call','outgoing_jump'] and target in fs and ('::'+kind+'::') in fs[target]['name'] and fs[target]['name'].endswith(tuple(ctor_suffixes)):targets.append(target)
   graph[a]=sorted(set(targets));pending.extend(targets)
  ctor_graphs[kind]=graph;ctor_frames[kind]=rows
 ctor_deltas={}
 for suffix in ctor_suffixes:
  pair={kind:[r for r in ctor_frames[kind].values() if r['name'].endswith(suffix)] for kind in ['reference','candidate']}
  if not pair['candidate'] and not pair['reference']:continue
  require(bool(pair['candidate']) and bool(pair['reference']),'matched constructor category '+suffix)
  native_min=min(r['frame_bound'] for r in pair['reference']);new_max=max(r['frame_bound'] for r in pair['candidate'])
  ctor_deltas[suffix]={'reference_frames':[r['frame_bound'] for r in pair['reference']],'candidate_frames':[r['frame_bound'] for r in pair['candidate']],'positive_delta':max(0,new_max-native_min),'comparison':'max candidate minus min native across constructor closure variants; no shrink credit'}
 def ctor_extra(a,path=()):
  require(a not in path,'acyclic constructor chain');suffix=next(s for s in ctor_deltas if fs[a]['name'].endswith(s))
  children=[ctor_extra(c,path+(a,)) for c in ctor_graphs['candidate'][a]]
  if suffix=='Work::point' and any(e['kind']=='call' and e['target']==solve for e in helper.transfers(fs[a])):children.append((kernel_peak,['possible constructor chart kernel']))
  value,names=max(children or [(0,[])],key=lambda v:v[0]);return ctor_deltas[suffix]['positive_delta']+value,[fs[a]['name'],*names]
 ctor_added_stack,ctor_route=ctor_extra(ctor_starts['candidate'])
 public_added_stack=added_stack;added_stack=max(added_stack,ctor_added_stack)
 extra=layout['workspace_bytes']+8+added_stack;aligned=(extra+15)//16*16
 def absolute_path(a,path=()):
  require(a not in path,'acyclic absolute caller chain')
  kind='candidate' if a in frames['candidate'] else 'reference'
  children=[absolute_path(c,path+(a,)) for c in graphs[kind][a]]
  if kind=='candidate' and fs[a]['name'].endswith('Work::point'):children.append(kernel_peak)
  return frames[kind][a]['frame_bound']+max(children or [0])
 required_prefix=absolute_path(starts['candidate'])
 status='PASS_REPEATED_TRAJECTORY_MEMORY_PREFLIGHT' if aligned<=67584 else 'BLOCKED_REPEATED_TRAJECTORY_ADDITIONAL_MEMORY_CAP'
 return {'constructor_graphs':ctor_graphs,'constructor_frames':ctor_frames,'constructor_frame_deltas':ctor_deltas,'constructor_added_stack_peak':ctor_added_stack,'constructor_added_stack_route':ctor_route,'public_added_stack_peak':public_added_stack,'status':status,'prototype':binding['prototype'],'prototype_tree':binding['prototype_tree'],'policy_sha256':sha((P/'protocol.json').read_bytes()),'binary':binding['binary'],'binary_sha256':binding['binary_sha256'],'kernel_peak':kernel_peak,'kernel_peak_route':kernel_route,'kernel_records':{format(a,'x'):{'name':fs[a]['name'],**r} for a,r in records.items()},'libc_leaf':leaf,'caller_frames':{kind:{format(a,'x'):r for a,r in rows.items()} for kind,rows in frames.items()},'caller_graphs':{kind:{format(a,'x'):[format(c,'x') for c in cs] for a,cs in graph.items()} for kind,graph in graphs.items()},'matched_frame_deltas':deltas,'added_stack_peak':added_stack,'added_stack_peak_route':route,'workspace_bytes':layout['workspace_bytes'],'owner_descriptor_bytes':8,'subtotal':extra,'aligned_additional_bound':aligned,'cap':67584,'excess_bytes':max(0,aligned-67584),'ordinary_native_reservation':474768,'two_test_owner_native_reservations':2*474768,'actual_candidate_arithmetic_prefix':required_prefix,'unchanged_baseline_step_stack_reservation':392416,'required_single_owner_nominal_plus_this_bound':474768+aligned,'allowed_single_owner_nominal_plus_cap':474768+67584,'candidate_execution_permitted':aligned<=67584,'generalization_candidates_executed':False,'candidate_owner_advances':0,'cancellation_retry_controls_executed':False,'no_shrink_or_replaced_solver_credit':True,'no_baseline_slack_credit':True,'old_frozen_comparison_certificate_reused':False,'scope':'new frozen binary successful/defined Result chart paths and actually reached public caller variants; private invariant panic excluded on frozen source basis; independent conservative added-object accounting, not a measurement of actual overrun or a complete controller/lifecycle acceptance proof. A cap failure stops before E1.'}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
