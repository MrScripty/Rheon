"""Fresh actual-ELF FD route accounting; no native equation execution."""
from pathlib import Path
import gzip, hashlib, importlib.util, json, re, subprocess
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
PREP=ROOT/'evidence/case41-scaled-fd-preparation-v1'
def require(ok,msg):
 if not ok: raise ValueError(msg)
def sha(b): return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 b=json.loads((P/'compiled-input.json').read_text())
 require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'actual frozen ELF')
 raw=subprocess.check_output(['objdump','-d','-C',b['binary']])
 require(raw==gzip.decompress((PREP/'native-disassembly.txt.gz').read_bytes()),'actual complete linked instructions')
 h=load('frames',ROOT/'evidence/forcing-increment-memory-audit/audit.py')
 selected=''.join(part for part in re.split(r'(?m)(?=^[a-f0-9]+ <)',raw.decode()) if part.splitlines() and 'research_public_call::' in part.splitlines()[0])
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
 # FD shell performs the same fixed geometry/embedding setup as reference
 # fixed_capture. FD equation journal performs the same returned-equation
 # observation as capture_equations (one order, no capture_point/qualifier).
 # Clamp each difference separately; never use the smaller FD journal as credit.
 mapping={'case41_fd_capture':'fixed_capture','Work::case41_fd_equation':'Work::capture_equations','Work::equation':'Work::equation','Work::partition':'Work::partition','Work::point':'Work::point'}
 roots={};graphs={};frames={}
 for kind,root in [('candidate','case41_fd_capture'),('reference','fixed_capture')]:
  start=next(a for a,f in fs.items() if f['name'].endswith(kind+'::'+root));roots[kind]=start;pending=[start];rows={};graph={}
  while pending:
   a=pending.pop()
   if a in rows:continue
   f=fs[a];edges=h.transfers(f);targets=[]
   for e in edges:
    target=e['target']
    if e['kind']=='outgoing_jump':require(target in fs,'resolved outgoing tail')
    if e['kind'] in ['call','outgoing_jump'] and target in fs and ('::'+kind+'::') in fs[target]['name'] and fs[target]['name'].endswith(tuple(mapping if kind=='candidate' else mapping.values())):targets.append(target)
   rows[a]={'name':f['name'],**h.fixed_frame(f),'transfers':edges};graph[a]=sorted(set(targets));pending.extend(targets)
  graphs[kind]=graph;frames[kind]=rows
 deltas={}
 for new,old in mapping.items():
  x=[r['frame_bound'] for r in frames['candidate'].values() if r['name'].endswith(new)]
  y=[r['frame_bound'] for r in frames['reference'].values() if r['name'].endswith(old)]
  require(x and y,'matched source operation '+new)
  deltas[new]={'candidate':x,'reference':y,'positive_delta':max(0,max(x)-min(y)),'reference_operation':old}
 def peak(a,path=(),absolute=False):
  require(a not in path,'acyclic capture route');r=frames['candidate'][a];category=next(s for s in mapping if r['name'].endswith(s))
  children=[peak(c,path+(a,),absolute) for c in graphs['candidate'][a]]
  if any(e['kind']=='call' and e['target'] in kernel for e in r['transfers']):children.append((kernel_peak,['linked scalar kernel']))
  value,route=max(children or [(0,[])],key=lambda x:x[0])
  return (r['frame_bound'] if absolute else deltas[category]['positive_delta'])+value,[r['name'],*route]
 added,route=peak(roots['candidate']);absolute,_=peak(roots['candidate'],absolute=True)
 arrays=3*22*8 # retained baseline, perturbed unknown, streamed native column;
 # Conservative duplicate charge even where their bytes are already reflected
 # in matched positive frame differences. No original snapshot is retained.
 bound=(62096+8+added+arrays+15)//16*16
 return {'status':'PASS_FD_MEMORY_PREFLIGHT' if bound<=67584 else 'BLOCKED_FD_MEMORY_CAP','bound':bound,'cap':67584,'remaining':67584-bound,'workspace_bytes':62096,'descriptor_bytes':8,'explicit_arrays_bytes':arrays,'added_frame_peak':added,'added_route':route,'absolute_FD_route_stack':absolute,'deltas':deltas,'graphs':graphs,'frames':frames,'kernel_peak':kernel_peak,'kernel_records':{str(a):r for a,r in records.items()},'libc_leaf':leaf,'binary_sha256':b['binary_sha256'],'native_equations':0,'owner_advances':0,'no_shrink_credit':True,'no_baseline_slack_credit':True,'scope':'Fixed source-operation frame comparisons plus explicit new arrays and fresh scalar Result paths. Common bounded journal formatting/immutable input fixture are external diagnostics; no whole-process RSS or physical root certificate.'}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
