"""Fresh actual-ELF FD route accounting; no native equation execution."""
from pathlib import Path
import gzip, hashlib, importlib.util, json, re, subprocess
P=Path(__file__).resolve().parent; ROOT=Path('/workspace/Rheon-fd-pure-e2')
PREP=ROOT/'evidence/case41-scaled-fd-preparation-v1'
def require(ok,msg):
 if not ok: raise ValueError(msg)
def sha(b): return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 b=json.loads((ROOT/'evidence/case41-six-fd-columns-v1/compiled-input.json').read_text())
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
 def edges(f):
  out=[]
  for e in h.transfers(f):
   if e['kind']=='internal_branch':continue
   e=dict(e)
   if e['target'] is None:
    bnd=indirect_binding(e['instruction']);e['binding']=bnd
    if bnd:e['target']=bnd['address']
   if e['target'] in fs:e['classification']='linked_crate'
   elif e['target'] is not None:e['classification']='external_linked_function'
   elif e.get('binding'):e['classification']='external_dynamic_binding'
   else:e['classification']='unresolved_indirect'
   out.append(e)
  return out
 graphs={};frames={};roots={};unresolved={}
 for kind,root in [('candidate','case41_fd_capture'),('reference','fixed_capture')]:
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
 mapping={'case41_fd_capture':'fixed_capture','Work::case41_fd_equation':'Work::capture_equations','Work::equation':'Work::equation','Work::partition':'Work::partition','Work::point':'Work::point','coefficients':'coefficients','cross':'cross','linear':'linear'}
 deltas={};absorbed=[]
 for new,oldname in mapping.items():
  x=[r['frame_bound'] for r in frames['candidate'].values() if '::candidate::' in r['name'] and r['name'].endswith('::'+new)]
  y=[r['frame_bound'] for r in frames['reference'].values() if '::reference::' in r['name'] and r['name'].endswith('::'+oldname)]
  if not x and not y:
   require(new=='Work::equation','only inspected absorbed equation may be jointly absent')
   for kind,observer in [('candidate','Work::case41_fd_equation'),('reference','Work::capture_equations')]:
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
 def peak(a,path=(),absolute=False):
  require(a not in path,'acyclic capture route')
  if a in kernel:return kp(a),[fs[a]['name'],'closed kernel Result paths']
  r=frames['candidate'][a];children=[peak(c,path+(a,),absolute) for c in graphs['candidate'][a]]
  value,route=max(children or [(0,[])],key=lambda z:z[0])
  return (r['frame_bound'] if absolute else weights[a])+value,[r['name'],*route]
 added,route=peak(roots['candidate']);absolute,absroute=peak(roots['candidate'],absolute=True)
 outer=next(f for f in fs.values() if f['name'].endswith('::case41_six_columns_only'))
 outer_frame=h.fixed_frame(outer)
 require(any(e['target']==roots['candidate'] for e in edges(outer)),'actual outer invokes FD shell')
 workspace=62096;require(outer_frame['frame_bound']>=workspace,'outer contains full workspace')
 outer_residual=outer_frame['frame_bound']-workspace
 arrays=3*22*8 # duplicate charge: baseline, perturbation, streamed column
 subtotal=(workspace+outer_residual+added+arrays+15)//16*16
 # External callees have NOT been silently assigned zero. A numerical memory
 # certificate needs their simultaneous additional cost closed or proven common.
 return {'status':'BLOCKED_EXTERNAL_TRANSFER_ACCOUNTING','known_crate_and_storage_subtotal':subtotal,'cap':67584,'unallocated_headroom':67584-subtotal,'workspace_bytes':workspace,'outer_frame':outer_frame,'outer_residual_bytes':outer_residual,'explicit_arrays_bytes':arrays,'added_crate_frame_peak':added,'added_route':route,'absolute_crate_route_stack':absolute,'absolute_route':absroute,'deltas':deltas,'absorbed_operations':absorbed,'graphs':graphs,'frames':frames,'kernel_peak':kernel_peak,'kernel_records':{str(a):r for a,r in records.items()},'libc_leaf':leaf,'external_transfer_ledger':unresolved,'unmatched_crate_callees':missing,'binary_sha256':b['binary_sha256'],'native_equations':0,'owner_advances':0,'no_shrink_credit':True,'no_baseline_slack_credit':True,'scope':'Complete directly and GOT-reached crate graph; all unknown, dynamic and non-crate transfers remain an explicit blocker. Subtotal is NOT a complete bound or permission to execute FD.'}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
