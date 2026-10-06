"""Bind committed complete trajectories to exact binary and actual verification."""
from pathlib import Path
import gzip,hashlib,json,os,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='ac75dbccea47a9b5f43e2aad9fb33e885dcae741'
EVIDENCE='40c25f9eca37b66e7e24bee5a8d5f6aa995e2196';TREE='3da767af9d6dcbed92760b049b917e85457b7d04'
RECEIPT='c327b1e0e718c6aa9056d053abe3b6b11b012a8ec9e1e119a970aa61e0d486ea'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
require(not(P/'post-git-verification.json').exists(),'never overwrite post-Git certificate')
require(git('rev-parse',EVIDENCE+'^{tree}').decode().strip()==TREE and git('rev-parse','HEAD').decode().strip()==EVIDENCE,'exact committed result source')
require(sha((P/'receipt.json').read_bytes())==RECEIPT,'exact committed receipt')
changes=git('diff','--name-status',BASE,EVIDENCE).decode().splitlines();require(len(changes)==273,'complete additive result packet');blobs={}
for line in changes:
 status,path=line.split('\t');require(status=='A' and path.startswith('evidence/forcing-e1-trajectories-v2/'),'additive results only');data=git('show',EVIDENCE+':'+path);require(data==(ROOT/path).read_bytes(),'exact committed result blob '+path);blobs[path]=sha(data)
for line in git('diff','--name-status','676c6feaf906a965920f37792e07e2d4d623b3d2',EVIDENCE).decode().splitlines():
 status,path=line.split('\t');require(status=='A' and path.startswith(('evidence/forcing-e1-trajectories/','evidence/forcing-e1-trajectories-v2/')),'all original source, book, proofs, fixtures and old packets unchanged')
binding=json.loads((P/'binary-binding.json').read_text());binary=binding['binary'];require(sha(Path(binary).read_bytes())==binding['binary_sha256'],'exact actual native binary')
raw=subprocess.check_output(['objdump','-d','-C',binary]);require(raw==gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()),'exact actual linked instruction certificate')
for name,argv in [('symbols',['nm','-S','-nC',binary]),('relocations',['readelf','-rW',binary]),('headers',['readelf','-hW',binary])]:require(subprocess.check_output(argv)==(P/('native-'+name+'.txt')).read_bytes(),'actual ELF metadata '+name)
commands=[]
for label,extra in [('normal',[]),('optimized',['-O'])]:
 argv=[sys.executable,*extra,'evidence/forcing-e1-trajectories-v2/verify.py'];env=os.environ.copy();env.update(OPENBLAS_NUM_THREADS='1',OMP_NUM_THREADS='1');start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 stdout,stderr='post-git-'+label+'.log','post-git-'+label+'-stderr.log';(P/stdout).write_bytes(result.stdout);(P/stderr).write_bytes(result.stderr)
 commands.append({'argv':argv,'exit':result.returncode,'seconds':time.monotonic()-start,'resource_env':{'OPENBLAS_NUM_THREADS':'1','OMP_NUM_THREADS':'1'},'stdout':stdout,'stderr':stderr,'stdout_sha256':sha(result.stdout),'stderr_sha256':sha(result.stderr),'native_execution':False})
 require(result.returncode==0,'actual committed evidence verification pass')
require(commands[0]['stdout_sha256']==commands[1]['stdout_sha256'],'post-Git normal/optimized exact parity')
checks=json.loads((P/'receipt.json').read_text())['checks']
record={'status':'PASS_COMMITTED_TRAJECTORY_PACKET_WITH_FAILURES_RETAINED','source':binding['prototype'],'source_tree':binding['prototype_tree'],'preflight':BASE,'preflight_tree':'e254b76974cdc0ed3a5dd6984de8750b5c0d291f','evidence':EVIDENCE,'evidence_tree':TREE,'committed_result_blobs':blobs,'receipt_sha256':RECEIPT,'protocol_sha256':sha((P/'protocol.json').read_bytes()),'binary_sha256':binding['binary_sha256'],'actual_disassembly_sha256':sha(raw),'recorder_sha256':sha(Path(__file__).read_bytes()),'commands':commands,'checks':checks,'preserved_prior_files':7408,'production_source_changes':0,'old_evidence_rewritten':False,'new_native_runs_during_verification':0,'prose_margin_precision_note':'RESULTS.md refusal margins have last-digit transcription differences; exact subtraction values are refusal-outcome.json and receipt.json: -7.244587773183377e-15, -3.1448968474775056e-14.'}
(P/'post-git-verification.json').write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
print(json.dumps({'status':record['status'],'evidence':EVIDENCE,'evidence_tree':TREE,'receipt_sha256':RECEIPT,'committed_result_blobs':len(blobs)},sort_keys=True))
