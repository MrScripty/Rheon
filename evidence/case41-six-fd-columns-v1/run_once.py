"""One authorized seven-equation invocation after the frozen passing preflight."""
from pathlib import Path
import hashlib,json,os,resource,subprocess,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
require(not(P/'capture-before.json').exists(),'never repeat this experiment')
b=json.loads((P/'compiled-input.json').read_text());r=json.loads((P/'preflight-receipt.json').read_text());policy=json.loads((P/'protocol.json').read_text())
require(r['status']=='PASS_SCALAR_AFFINE_AND_FRESH_FD_MEMORY' and r['memory_bound']<=67584 and r['binary_sha256']==b['binary_sha256'],'qualified exact ELF')
for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen experiment source')
for path,digest in r['artifact_sha256'].items():require(sha((P/path).read_bytes())==digest,'frozen passing preflight artifact '+path)
require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'same ELF')
env=dict(os.environ,RHEON_CASE41_FD_ALLOW='case41-fixed-seventh-candidate-six-original-columns-v1')
argv=[b['binary'],'--exact','research_public_call::case41_six_columns_only','--nocapture']
before={'argv':argv,'environment':{'RHEON_CASE41_FD_ALLOW':env['RHEON_CASE41_FD_ALLOW']},'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip(),'source_tree':subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=ROOT).decode().strip(),'binary_sha256':b['binary_sha256'],'preflight_receipt_sha256':sha((P/'preflight-receipt.json').read_bytes()),'equations_max':7,'corrections':0,'owners':0,'stdout_cap':16*1024*1024,'stderr_cap':1024*1024}
(P/'capture-before.json').write_text(json.dumps(before,indent=2,sort_keys=True)+'\n')
def limits():resource.setrlimit(resource.RLIMIT_FSIZE,(16*1024*1024,16*1024*1024))
start=time.monotonic()
with (P/'native.log').open('wb') as out,(P/'native-stderr.log').open('wb') as err: result=subprocess.run(argv,cwd=ROOT,env=env,stdout=out,stderr=err,preexec_fn=limits)
after={**before,'exit':result.returncode,'seconds':time.monotonic()-start,'stdout_sha256':sha((P/'native.log').read_bytes()),'stderr_sha256':sha((P/'native-stderr.log').read_bytes())}
(P/'capture-completed.json').write_text(json.dumps(after,indent=2,sort_keys=True)+'\n')
require((P/'native.log').stat().st_size<=before['stdout_cap'] and (P/'native-stderr.log').stat().st_size<=before['stderr_cap'],'journal bound')
require(result.returncode==0,'actual native capture test failure: retain logs and stop')
rows=[json.loads(x) for x in (P/'native.log').read_text().splitlines() if x.startswith('{')];terminal=[x for x in rows if x['event']=='fd_complete'];require(len(terminal)==1,'one honest terminal')
print(json.dumps({'status':'CAPTURE_COMPLETED_READER_PENDING','terminal':terminal[0],'rows':len(rows),'native_invocations':1,'corrections':0,'owners':0},indent=2,sort_keys=True))
