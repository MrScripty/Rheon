"""Exactly two authorized jobs after fresh scalar/full-memory qualification."""
from pathlib import Path
import hashlib,importlib.util,json,os,subprocess,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def preflight():
 policy=json.loads((P/'protocol.json').read_text());b=json.loads((P/'binary-binding.json').read_text());commands=json.loads((P/'preflight-commands.json').read_text());require(b['protocol_sha256']==sha((P/'protocol.json').read_bytes()),'frozen protocol');require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'same actual ELF')
 for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen source '+path)
 require([Path(c['stdout']).stem for c in commands['commands']]==['format','install','build','clippy','scalar-native','type-layout','affine-native','selector-native','restore'],'closed preflight roster')
 for c in commands['commands']:
  require(c['exit']==0,'passed actual preflight')
  for channel in ['stdout','stderr']:require(sha((P/c[channel]).read_bytes())==c[channel+'_sha256'],'actual log binding')
 oracle=load('scalar',P/'check_scalar.py').run(P/'scalar-native.log');require(oracle['native_assertion_groups']==23 and oracle['exact_scalar_probes']==36,'scalar oracle counts');affine=load('affine',P/'check_affine.py').run();memory=json.loads(json.dumps(load('fullmemory',P/'audit_full.py').run()));require(memory==json.loads((P/'memory-normal.json').read_text())and(P/'memory-normal.json').read_bytes()==(P/'memory-optimized.json').read_bytes(),'fresh memory replay/mode parity');require(memory['status']=='PASS_FULL_PUBLIC_MEMORY_PREFLIGHT','full native memory gates');receipt={'status':'PASS_BEFORE_ANY_CASE47_OWNER','source':b['prototype'],'source_tree':b['prototype_tree'],'binary_sha256':b['binary_sha256'],'protocol_sha256':b['protocol_sha256'],'scalar':oracle,'affine':affine,'memory_bound':memory['bound'],'cap':memory['cap'],'owner_advances':0};return receipt,b
if __name__=='__main__':
 amend=json.loads((P/'reader-amendment.json').read_text());require(sha(Path(__file__).read_bytes())==amend['driver_sha256'],'frozen reader fix');receipt,b=preflight();require(not(P/'preflight-receipt.json').exists(),'never repeat authorized native jobs');(P/'preflight-receipt.json').write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
 for mode in ['E1','E2']:
  argv=[b['binary'],'--exact','research_public_call::case47_single_public_comparison','--nocapture'];env=os.environ.copy();env['RHEON_CASE47_PUBLIC_ALLOW']='case47-E1-prefix25-one-call-v1';env['RHEON_CASE47_MODE']=mode;record={'source':b['prototype'],'source_tree':b['prototype_tree'],'binary_sha256':b['binary_sha256'],'protocol_sha256':b['protocol_sha256'],'preflight_sha256':sha((P/'preflight-receipt.json').read_bytes()),'argv':argv,'environment':{k:env[k]for k in ['RHEON_CASE47_PUBLIC_ALLOW','RHEON_CASE47_MODE']},'comparison_calls':1,'prefix_steps':25};(P/(mode+'-command-before.json')).write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE);(P/(mode+'-native.log')).write_bytes(result.stdout);(P/(mode+'-native-stderr.log')).write_bytes(result.stderr);record.update(exit=result.returncode,seconds=time.monotonic()-start,stdout_sha256=sha(result.stdout),stderr_sha256=sha(result.stderr));(P/(mode+'-command-completed.json')).write_text(json.dumps(record,indent=2,sort_keys=True)+'\n');require(result.returncode==0,'actual one-call qualification failure '+mode)
 print(json.dumps({'status':'TWO_AUTHORIZED_JOBS_COMPLETED','reference_integrations':0,'trajectory_extension':False}))
