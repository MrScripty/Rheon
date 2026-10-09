"""Fresh actual-ELF FD route accounting; no native equation execution."""
from pathlib import Path
import gzip, hashlib, importlib.util, json, re, subprocess, sys
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; OUT=Path(sys.argv[1]).resolve()
PREP=OUT
def require(ok,msg):
 if not ok: raise ValueError(msg)
def sha(b): return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 b=json.loads((OUT/'compile-binding.json').read_text())
 require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'actual frozen ELF')
 raw=subprocess.check_output(['objdump','-d','-C',b['binary']])
 require(raw==gzip.decompress((PREP/'native-disassembly.txt.gz').read_bytes()),'actual complete linked instructions')
 h=load('frames',ROOT/'evidence/forcing-increment-memory-audit/audit.py')
 selected=''.join(part for part in re.split(r'(?m)(?=^[a-f0-9]+ <)',raw.decode()) if part.splitlines() and 'rheon::' in part.splitlines()[0])
 fs=h.functions(selected,(PREP/'native-symbols.txt').read_text())
 kernel={a:f for a,f in fs.items() if '::scalar::' in f['name'] and not f['name'].endswith('::conformance')}
 require(len(kernel)==7,'all linked scalar Result paths')
 symbols=(PREP/'native-symbols.txt').read_text();relocs=(PREP/'native-relocations.txt').read_text()
 panic=re.search(r'^([a-f0-9]+) [a-f0-9]+ T core::panicking::panic_bounds_check$',symbols,re.M).group(1).lstrip('0')
 def resolve(asm,target):
  if target in kernel:return target,'bounded_kernel'
  slot=re.search(r'# ([a-f0-9]+) ',asm)
  if slot and '<memset@' in asm:return 'memset','actual_default_libc_leaf'
  if slot and re.search(r'^0*'+slot[1]+r' .*R_X86_64_RELATIVE\s+'+panic+r'$',relocs,re.M):return 'panic_bounds_check','excluded_private_invariant_panic'
  raise ValueError('unresolved kernel transfer '+asm)
 records={a:h.kernel_cfg(f,resolve) for a,f in kernel.items()}
 old=ROOT/'evidence/forcing-increment-comparison';libc=json.loads((old/'memset-binding.json').read_text())
 require(sha(Path(libc['library']).read_bytes())==json.loads((old/'memory-preflight-staged.json').read_text())['libc_sha256'],'actual installed libc')
 code={}
 for line in (old/'memset-leaf-disassembly.txt').read_text().splitlines():
  fields=line.split('\t')
  if len(fields)>=3 and re.match(r'^\s*[a-f0-9]+:$',fields[0]):code[int(fields[0].strip()[:-1],16)]=fields[-1].strip()
 def external(asm,target):raise ValueError('unresolved libc transfer '+asm)
 leaf=h.kernel_cfg({'start':libc['resolved_file_address'],'code':code},external)
 require(leaf['frame_bound']==8 and not leaf['edges'],'fresh closed memset leaf')
 def kp(a,path=()):
  if a=='memset':return 8
  require(a not in path,'acyclic kernel');r=records[a]
  return max([r['frame_bound']]+[8+e['stack_depth']+kp(e['callee'],path+(a,)) for e in r['edges'] if e['disposition']!='excluded_private_invariant_panic'])
 kernel_peak=kp(next(a for a,f in kernel.items() if f['name'].endswith('ChartWorkspace::solve')))
 # ALL linked crate callees, including GOT-resolved crate calls. The complete
 # fixed frames are retained; an absent source-level frame never removes bytes.
 aliases={}
 for line in symbols.splitlines():
  m=re.match(r'^([a-f0-9]+) [a-f0-9]+ [tT] (.+)$',line)
  if m:aliases.setdefault(int(m[1],16),[]).append(m[2])
 bindings={}
 for line in relocs.splitlines():
  m=re.match(r'^([a-f0-9]+) .*R_X86_64_RELATIVE\s+([a-f0-9]+)$',line)
  if m:bindings[int(m[1],16)]={'address':int(m[2],16),'names':aliases.get(int(m[2],16),[])}
  m=re.match(r'^([a-f0-9]+) .*R_X86_64_(?:GLOB_DAT|JUMP_SLOT)\s+\S+\s+(\S+)',line)
  if m:bindings[int(m[1],16)]={'address':None,'names':[m[2]]}
 def indirect_binding(asm):
  m=re.search(r'# ([a-f0-9]+) ',asm)
  if not m:return None
  return bindings.get(int(m[1],16))
 # Conservative intraprocedural register-origin dataflow. Unknown entry values,
 # memory reloads, unmodelled writes and mixed paths remain UNKNOWN. We never
 # infer a function pointer from its spelling or nearby instruction alone.
 def register_origins(f):
  regs=['rax','rbx','rcx','rdx','rsi','rdi','rbp','rsp','r8','r9','r10','r11','r12','r13','r14','r15']
  alias={r:r for r in regs}
  for full,short,word,low,high in [('rax','eax','ax','al','ah'),('rbx','ebx','bx','bl','bh'),('rcx','ecx','cx','cl','ch'),('rdx','edx','dx','dl','dh'),('rsi','esi','si','sil',None),('rdi','edi','di','dil',None),('rbp','ebp','bp','bpl',None),('rsp','esp','sp','spl',None)]:
   for part in [short,word,low,high]:
    if part:alias[part]=full
  for r in regs[8:]:alias[r+'d']=r;alias[r+'w']=r;alias[r+'b']=r
  unknown=frozenset({None});entry={r:unknown for r in regs}
  addrs=list(f['code']);following={a:b for a,b in zip(addrs,addrs[1:])}
  states={f['start']:entry};pending=[f['start']];calls={}
  while pending:
   a=pending.pop();state=states[a];out=dict(state);asm=f['code'][a];plain=asm.split('#')[0].strip();op=plain.split()[0]
   if op.startswith('call') or op.startswith('jmp'):
    m=re.match(r'^(?:call\S*|jmp\S*)\s+\*%([a-z0-9]+)$',plain)
    if m:calls[a]=state.get(alias.get(m[1]),unknown)
   dst=re.search(r',%([a-z0-9]+)$',plain)
   if dst and not op.startswith(('cmp','test','bt')):
    reg=alias.get(dst[1]);value=unknown
    if reg:
     src=plain.split(None,1)[1].rsplit(',%',1)[0]
     if op in ('mov','movq') and dst[1]==reg:
      if re.fullmatch(r'%[a-z0-9]+',src):value=state.get(alias.get(src[1:]),unknown)
      elif '(%rip)' in src:
       bnd=indirect_binding(asm)
       if bnd:value=frozenset({int(re.search(r'# ([a-f0-9]+) ',asm)[1],16)})
     out[reg]=value
   elif not op.startswith(('push','cmp','test','bt','call','j','ret')):
    m=re.search(r'%([a-z0-9]+)$',plain)
    if m and alias.get(m[1]):out[alias[m[1]]]=unknown
   if op.startswith(('xchg','xadd','cmpxchg')):
    for m in re.findall(r'%([a-z0-9]+)',plain):
     if alias.get(m):out[alias[m]]=unknown
   if op in ('mul','imul','div','idiv','cqo','cdq','cwd') and (',' not in plain or op in ('cqo','cdq','cwd')):
    out['rax']=unknown;out['rdx']=unknown
   if op.startswith('call'):
    for reg in ['rax','rcx','rdx','rsi','rdi','r8','r9','r10','r11']:out[reg]=unknown
   successors=[]
   direct=re.match(r'^j\S*\s+([a-f0-9]+)\s',plain)
   if direct and int(direct[1],16) in f['code']:successors.append(int(direct[1],16))
   if not op.startswith(('ret','jmp','ud2')) and a in following:successors.append(following[a])
   for dest in successors:
    old=states.get(dest);merged=out if old is None else {r:old[r]|out[r] for r in regs}
    if old!=merged:states[dest]=merged;pending.append(dest)
  return calls
 def edges(f):
  out=[];origins=register_origins(f)
  for e in h.transfers(f):
   if e['kind']=='internal_branch':continue
   e=dict(e)
   if e['target'] is None:
    bnd=indirect_binding(e['instruction']);e['binding']=bnd
    if bnd:e['target']=bnd['address']
    elif e['address'] in origins:
     slots=origins[e['address']];e['register_origin_slots']=sorted(s for s in slots if s is not None);e['register_origin_unknown']=None in slots
     # Only singleton, fully known origins establish a concrete transfer.
     if None not in slots and len(slots)==1:
      bnd=bindings[next(iter(slots))];e['binding']=bnd;e['target']=bnd['address'];e['origin']='CFG_register_dataflow'
     else:e['possible_bindings']=[bindings[s] for s in sorted(v for v in slots if v is not None)]
   if e['target'] in fs:e['classification']='linked_crate'
   elif e['target'] is not None:e['classification']='external_linked_function'
   elif e.get('binding'):e['classification']='external_dynamic_binding'
   else:e['classification']='unresolved_indirect'
   out.append(e)
  return out
 graphs={};frames={};roots={};unresolved={}
 for kind,root in [('candidate','case41_fd_capture'),('reference','case41_fd_capture')]:
  start=next(a for a,f in fs.items() if f['name'].endswith(kind+'::'+root));roots[kind]=start
  pending=[start];rows={};graph={};unknown=[]
  while pending:
   a=pending.pop()
   if a in rows:continue
   f=fs[a];es=edges(f)
   targets=sorted({e['target'] for e in es if e['target'] in fs})
   rows[a]={'name':f['name'],**h.fixed_frame(f),'transfers':es};graph[a]=targets;pending.extend(targets)
   unknown.extend({'caller':f['name'],'caller_address':a,**e} for e in es if e['classification']!='linked_crate')
  graphs[kind]=graph;frames[kind]=rows;unresolved[kind]=unknown
 mapping={'case41_fd_capture':'case41_fd_capture','Work::case41_numerical':'Work::case41_numerical','Work::case41_serialize':'Work::case41_serialize','Work::equation':'Work::equation','Work::partition':'Work::partition','Work::point':'Work::point','coefficients':'coefficients','cross':'cross','linear':'linear'}
 deltas={};absorbed=[]
 for new,oldname in mapping.items():
  x=[r['frame_bound'] for r in frames['candidate'].values() if '::candidate::' in r['name'] and r['name'].endswith('::'+new)]
  y=[r['frame_bound'] for r in frames['reference'].values() if '::reference::' in r['name'] and r['name'].endswith('::'+oldname)]
  if not x and not y:
   require(new=='Work::equation','only inspected absorbed equation may be jointly absent')
   for kind,observer in [('candidate','Work::case41_numerical'),('reference','Work::case41_numerical')]:
    obs=[a for a,r in frames[kind].items() if r['name'].endswith('::'+observer)]
    require(len(obs)==1,'unique actual observer frame')
    actual=[frames[kind][a]['name'] for a in graphs[kind][obs[0]]]
    require(any(n.endswith('::Work::point') for n in actual) and any(n.endswith('::Work::partition') for n in actual),'absorbed body retains actual point and partition calls')
   absorbed.append(new);continue
  require(x and y,'one-sided or missing operation '+new)
  deltas[new]={'candidate':x,'reference':y,'positive_delta':max(0,max(x)-min(y)),'reference_operation':oldname}
 weights={};missing=[]
 for a,r in frames['candidate'].items():
  if a in kernel:continue
  category=next((s for s in mapping if '::candidate::' in r['name'] and r['name'].endswith('::'+s)),None)
  if category:weights[a]=deltas[category]['positive_delta']
  elif a in frames['reference']:weights[a]=0 # identical actual linked body, retained in both graphs
  else:
   weights[a]=r['frame_bound'] # no credit for an unmatched numerical callee
   missing.append({'address':a,'name':r['name'],'charged_full_frame':r['frame_bound']})
 recursion=[]
 def peak(a,path=(),absolute=False):
  if a in path:
   cycle=path[path.index(a):]+(a,)
   require(all(v in frames['reference'] and weights.get(v)==0 for v in cycle),'unbounded nonzero additional recursive route')
   recursion.append([fs[v]['name'] for v in cycle])
   return 0,['identical shared recursive body: no additional crate frame cost']
  if a in kernel:return kp(a),[fs[a]['name'],'closed kernel Result paths']
  r=frames['candidate'][a];children=[peak(c,path+(a,),absolute) for c in graphs['candidate'][a]]
  value,route=max(children or [(0,[])],key=lambda z:z[0])
  return (r['frame_bound'] if absolute else weights[a])+value,[r['name'],*route]
 added,route=peak(roots['candidate']);absolute=None;absroute=[]
 outer=next(f for f in fs.values() if f['name'].endswith('::case41_borrowed_candidate_outer'))
 outer_frame=h.fixed_frame(outer)
 require(any(e['target']==roots['candidate'] for e in edges(outer)),'actual outer invokes FD shell')
 workspace=62096;require(outer_frame['frame_bound']>=workspace,'outer contains full workspace')
 outer_residual=outer_frame['frame_bound']-workspace
 arrays=3*22*8 # duplicate charge: baseline, perturbation, streamed column
 subtotal=(workspace+outer_residual+added+arrays+15)//16*16
 # Retain complete outer transfers as well as all phase-specific ledgers.
 outer_records={}
 phase_records={}
 for kind in ('candidate','reference'):
  of=next(f for f in fs.values() if f['name'].endswith('::case41_borrowed_'+kind+'_outer'))
  outer_records[kind]={'frame':h.fixed_frame(of),'transfers':edges(of),'name':of['name']}
  phase_records[kind]={}
  for phase,suffix in [('numerical','Work::case41_numerical'),('serialization','Work::case41_serialize')]:
   root=next(a for a,r in frames[kind].items() if r['name'].endswith('::'+suffix))
   pending=[root];seen=set()
   while pending:
    address=pending.pop()
    if address in seen:continue
    seen.add(address);pending.extend(graphs[kind][address])
   phase_records[kind][phase]={'root':root,'crate_addresses':sorted(seen),'external_transfers':[e for address in sorted(seen) for e in frames[kind][address]['transfers'] if e['classification']!='linked_crate']}
 # A partial absolute crate frame route can be bounded with source-qualified
 # seam recursion depth two. It does NOT include non-crate children or heap.
 def partial_absolute(a,path=()):
  if a in path:
   require(fs[a]['name'].endswith('FittedHeightWorkspace::split_position'),'only fixed topology seam recursion')
   if path.count(a)>=2:return 0
  children=[partial_absolute(c,path+(a,)) for c in graphs['candidate'][a]]
  return frames['candidate'][a]['frame_bound']+max(children or [0])
 absolute=partial_absolute(roots['candidate'])
 # ALL external callees, including outer calls, remain retained. Shared symbol
 # names alone do not qualify common memory, and unknown costs are never zero.
 # External callees have NOT been silently assigned zero. A numerical memory
 # certificate needs their simultaneous additional cost closed or proven common.
 return {'status':'BLOCKED_LINKED_PATH_ACCOUNTING','known_crate_and_storage_subtotal':subtotal,'cap':67584,'unallocated_headroom':67584-subtotal,'workspace_bytes':workspace,'outer_frame':outer_frame,'outer_residual_bytes':outer_residual,'explicit_arrays_bytes':arrays,'added_crate_frame_peak':added,'added_route':route,'absolute_crate_route_stack':None,'absolute_route':absroute,'shared_recursions':sorted({tuple(v) for v in recursion}),'absolute_stack_bound_not_established':True,'partial_absolute_crate_frame_peak':absolute,'fixed_topology_recursion_active_frames':2,'common_split_frame_bytes_each':next(r['frame_bound'] for r in frames['candidate'].values() if r['name'].endswith('FittedHeightWorkspace::split_position')),'outer_records':outer_records,'phase_records':phase_records,'deltas':deltas,'absorbed_operations':absorbed,'graphs':graphs,'frames':frames,'kernel_peak':kernel_peak,'kernel_records':{str(a):r for a,r in records.items()},'libc_leaf':leaf,'external_transfer_ledger':unresolved,'unmatched_crate_callees':missing,'binary_sha256':b['binary_sha256'],'native_equations':0,'owner_advances':0,'no_shrink_credit':True,'no_baseline_slack_credit':True,'scope':'Paired outer/capture/numerical/borrowed serialization. Common geometry frame differences are zero only under separately recorded fixed arguments/lifetimes; absolute memory is nonzero. ALL linked/dynamic/unknown costs remain blockers. Subtotal cannot authorize FD.', 'complete_memory_bound':None, 'independent_acceptance':False, 'FD_execution_allowed':False}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
