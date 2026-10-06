"""Run ONLY three authorized scalar/layout tests; never enable or invoke FD."""
from pathlib import Path
import hashlib, importlib.util, json, os, subprocess, sys, time
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
F=Path('/workspace/Rheon-fd-pure-e2')
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(data):return hashlib.sha256(data).hexdigest()
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
require(not(P/'preflight-receipt.json').exists() and not(P/'preflight-commands.json').exists(),'run this guard-only preflight once')
policy=json.loads((P/'protocol.json').read_text())
require(subprocess.check_output(['git','rev-parse','HEAD'],cwd=F).decode().strip()==policy['frozen_history'],'unchanged frozen history')
for path,digest in policy['tooling_sha256'].items():require(sha((P/path).read_bytes())==digest,'frozen new tooling '+path)
old=json.loads((F/'evidence/case41-six-fd-columns-v1/protocol.json').read_text())
for path,digest in old['source_sha256'].items():require(sha((F/path).read_bytes())==digest,'frozen original source '+path)
b=json.loads((F/'evidence/case41-six-fd-columns-v1/compiled-input.json').read_text())
require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'exact existing ELF')
source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip();tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=ROOT).decode().strip()
env=dict(os.environ);env.pop('RHEON_CASE41_FD_ALLOW',None);env['PYTHONDONTWRITEBYTECODE']='1';commands=[]
def run(label,argv):
 start=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True)
 (P/(label+'.log')).write_bytes(r.stdout);(P/(label+'-stderr.log')).write_bytes(r.stderr)
 commands.append(dict(argv=argv,exit=r.returncode,seconds=time.monotonic()-start,stdout=label+'.log',stderr=label+'-stderr.log',stdout_sha256=sha(r.stdout),stderr_sha256=sha(r.stderr),FD_allow_present=False))
 (P/'preflight-commands.json').write_text(json.dumps(commands,indent=2,sort_keys=True)+'\n');require(r.returncode==0,'guard-only preflight failed '+label)
for label,test in [('scalar-native','research_public_scalar_preflight'),('type-layout','research_public_type_layout'),('affine-native','scalar_affine_preflight')]:run(label,[b['binary'],'--exact','research_public_call::'+test,'--nocapture'])
scalar=load('scalar_oracle_v2',P/'check_scalar.py').run(P/'scalar-native.log');affine=load('affine_oracle_v2',P/'check_affine.py').run()
for label,flags in [('memory-normal',[]),('memory-optimized',['-O'])]:run(label,[sys.executable,*flags,str(P/'audit_memory.py')])
require((P/'memory-normal.log').read_bytes()==(P/'memory-optimized.log').read_bytes(),'normal/optimized fail-closed reader parity')
memory=json.loads((P/'memory-normal.log').read_text());require(memory['status']=='BLOCKED_EXTERNAL_TRANSFER_ACCOUNTING','never grant numerical execution permission')
r=dict(status='PASS_SCALAR_LAYOUT_AFFINE__MEMORY_STILL_BLOCKED',source=source,source_tree=tree,compiled_source=b['source'],compiled_source_tree=b['source_tree'],binary=b['binary'],binary_sha256=b['binary_sha256'],protocol_sha256=sha((P/'protocol.json').read_bytes()),scalar=scalar,affine=affine,known_crate_and_storage_subtotal=memory['known_crate_and_storage_subtotal'],complete_memory_bound=None,cap=67584,native_equations=0,owner_advances=0,new_corrections=0,native_test_invocations=3,FD_execution_allowed=False,FD_test_invocations=0,build_invocations=0)
(P/'preflight-receipt.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n');print(json.dumps(r,indent=2,sort_keys=True))
