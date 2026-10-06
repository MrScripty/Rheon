"""Three exact scalar tests, fresh memory audit, zero equation/owner execution."""
from pathlib import Path
import hashlib, importlib.util, json, os, subprocess, sys, time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
require(not(P/'preflight-receipt.json').exists() and not(P/'preflight-commands.json').exists(),'run this preflight once')
b=json.loads((P/'compiled-input.json').read_text());policy=json.loads((P/'protocol.json').read_text())
for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen experiment source '+path)
require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'actual compiled ELF')
source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip();tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=ROOT).decode().strip()
env=dict(os.environ);env.pop('RHEON_CASE41_FD_ALLOW',None);commands=[]
def run(label,argv):
 start=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True)
 (P/(label+'.log')).write_bytes(r.stdout);(P/(label+'-stderr.log')).write_bytes(r.stderr)
 commands.append({'argv':argv,'exit':r.returncode,'seconds':time.monotonic()-start,'stdout':label+'.log','stderr':label+'-stderr.log','stdout_sha256':sha(r.stdout),'stderr_sha256':sha(r.stderr),'FD_allow_present':False})
 (P/'preflight-commands.json').write_text(json.dumps(commands,indent=2,sort_keys=True)+'\n');require(r.returncode==0,'actual preflight failure '+label)
for label,test in [('scalar-native','research_public_scalar_preflight'),('type-layout','research_public_type_layout'),('affine-native','scalar_affine_preflight')]:run(label,[b['binary'],'--exact','research_public_call::'+test,'--nocapture'])
scalar=load('fd_scalar',P/'check_scalar.py').run(P/'scalar-native.log');affine=load('fd_affine',P/'check_affine.py').run()
for label,flags in [('memory-normal',[]),('memory-optimized',['-O'])]:run(label,[sys.executable,*flags,str(P/'audit_memory.py')])
require((P/'memory-normal.log').read_bytes()==(P/'memory-optimized.log').read_bytes(),'audit mode parity')
memory=json.loads((P/'memory-normal.log').read_text());require(memory['status']=='PASS_FD_MEMORY_PREFLIGHT','stop before capture: memory preflight failure')
r={'status':'PASS_SCALAR_AFFINE_AND_FRESH_FD_MEMORY','source':source,'source_tree':tree,'compiled_source':b['source'],'compiled_source_tree':b['source_tree'],'binary_sha256':b['binary_sha256'],'protocol_sha256':sha((P/'protocol.json').read_bytes()),'scalar':scalar,'affine':affine,'memory_bound':memory['bound'],'cap':67584,'native_equations':0,'owner_advances':0,'new_corrections':0,'artifact_sha256':{str(x.relative_to(P)):sha(x.read_bytes()) for x in sorted(P.rglob('*')) if x.is_file() and '__pycache__' not in x.parts}}
(P/'preflight-receipt.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n');print(json.dumps(r,indent=2,sort_keys=True))
