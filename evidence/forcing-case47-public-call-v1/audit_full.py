"""Fresh ELF positive-delta accounting through all internal public numerical callees."""
from pathlib import Path
import gzip,importlib.util,json,re
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 base=load('base',P/'audit_base.py').run();h=load('frames',ROOT/'evidence/forcing-increment-memory-audit/audit.py');raw=gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()).decode();selected=''.join(part for part in re.split(r'(?m)(?=^[a-f0-9]+ <)',raw)if part.splitlines()and 'research_public_call::'in part.splitlines()[0]);fs=h.functions(selected,(P/'native-symbols.txt').read_text());graphs={};frames={};roots={}
 for kind in ['reference','candidate']:
  roots[kind]=next(a for a,f in fs.items()if f['name'].endswith(kind+'::public_call'));pending=[roots[kind]];graph={};rows={}
  while pending:
   a=pending.pop()
   if a in rows:continue
   f=fs[a];edges=h.transfers(f);rows[a]={'name':f['name'],**h.fixed_frame(f),'transfers':edges};targets=[]
   for e in edges:
    target=e['target']
    if e['kind']=='outgoing_jump':require(target in fs,'resolved outgoing internal tail')
    if e['kind']in ['call','outgoing_jump']and target in fs and ('::'+kind+'::')in fs[target]['name']:targets.append(target)
   graph[a]=sorted(set(targets));pending.extend(targets)
  graphs[kind]=graph;frames[kind]=rows
 def suffix(name,kind):return name.split('::'+kind+'::',1)[1]
 ref={suffix(r['name'],'reference'):r for r in frames['reference'].values()};delta={}
 for a,r in frames['candidate'].items():
  name=suffix(r['name'],'candidate');require(name in ref,'matched full public operation '+name);delta[a]=max(0,r['frame_bound']-ref[name]['frame_bound'])
 kernel={a for a,f in fs.items()if '::scalar::'in f['name']and not f['name'].endswith('::conformance')}
 def peak(a,path=()):
  require(a not in path,'acyclic full public numerical route');children=[peak(c,path+(a,))for c in graphs['candidate'][a]]
  if any(e['kind']=='call'and e['target']in kernel for e in frames['candidate'][a]['transfers']):children.append((base['kernel_peak'],['actual chart kernel']))
  value,route=max(children or [(0,[])],key=lambda x:x[0]);return delta[a]+value,[frames['candidate'][a]['name'],*route]
 value,route=peak(roots['candidate']);stack=max(value,base['constructor_added_stack_peak']);bound=(62096+8+stack+15)//16*16;require(bound<=67584,'full public additional cap');return {'status':'PASS_FULL_PUBLIC_MEMORY_PREFLIGHT','bound':bound,'cap':67584,'remaining':67584-bound,'added_public_stack':value,'route':route,'frame_deltas':{suffix(r['name'],'candidate'):{'candidate':r['frame_bound'],'reference':ref[suffix(r['name'],'candidate')]['frame_bound'],'positive_delta':delta[a]}for a,r in frames['candidate'].items()},'graphs':graphs,'frames':frames,'base':base,'owner_executions':0,'scope':'All reached linked candidate/reference internal numerical callers, scalar Result paths and constructors. Unchanged geometry/accepted publication callees and common bounded journal formatting are baseline. No shrunk frame/slack credit; not whole-process RSS or arbitrary lifecycle proof.'}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
