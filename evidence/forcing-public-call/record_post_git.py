"""Bind the stopped public-call evidence to Git and its actual linked binary."""
from pathlib import Path
import gzip,hashlib,json,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='77962729f8de389c25b3f595708ab50d4b3362c0'
EVIDENCE='87a27d5c0d63173ea3617ab25deb21ed8989290d'
TREE='fcfa1693d29cb813105a557aead061f9aea8f190'
RECEIPT='a282fb204c3470fcabae5c025e2fb843db3e78755e7da7e0263dcd021f9fbbd0'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
require(git('rev-parse',EVIDENCE+'^{tree}').decode().strip()==TREE,'frozen stopped-result tree')
require(sha((P/'receipt.json').read_bytes())==RECEIPT,'frozen stopped-result receipt')
blobs={};changes=git('diff','--name-status',BASE,EVIDENCE).decode().splitlines()
require(len(changes)==26,'closed additive 26-file stopped-result packet')
for line in changes:
 status,path=line.split('\t');require(status=='A' and path.startswith('evidence/forcing-public-call/'),'additive stopped-result files only')
 data=git('show',EVIDENCE+':'+path);require(data==(ROOT/path).read_bytes(),'actual committed evidence blob '+path);blobs[path]=sha(data)
binding=json.loads((P/'binary-v2-binding.json').read_text());binary=binding['binary']
require(sha(Path(binary).read_bytes())==binding['binary_sha256'],'new binary unchanged')
raw=subprocess.check_output(['objdump','-d','-C',binary])
require(raw==gzip.decompress((P/'native-v2-disassembly.txt.gz').read_bytes()),'archived full instructions match actual new binary')
for name,argv in [('symbols',['nm','-S','-nC',binary]),('relocations',['readelf','-rW',binary]),('headers',['readelf','-hW',binary])]:
 require(subprocess.check_output(argv)==(P/('native-v2-'+name+'.txt')).read_bytes(),'actual ELF metadata '+name)
commands=[]
for label,extra in [('normal',[]),('optimized',['-O'])]:
 argv=[sys.executable,*extra,'evidence/forcing-public-call/verify.py'];start=time.monotonic()
 result=subprocess.run(argv,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE);seconds=time.monotonic()-start
 stdout,stderr='post-git-'+label+'.log','post-git-'+label+'-stderr.log'
 (P/stdout).write_bytes(result.stdout);(P/stderr).write_bytes(result.stderr)
 commands.append({'argv':argv,'exit':result.returncode,'seconds':seconds,'stdout':stdout,'stderr':stderr,'stdout_sha256':sha(result.stdout),'stderr_sha256':sha(result.stderr)})
 require(result.returncode==0,'actual stopped-result verification pass')
require(commands[0]['stdout_sha256']==commands[1]['stdout_sha256'],'post-Git mode parity')
r={'status':'PASS_STOPPED_PUBLIC_QUALIFICATION_BINDING_ONLY','base':BASE,'evidence':EVIDENCE,'evidence_tree':TREE,'committed_stopped_result_blobs':blobs,'receipt_sha256':RECEIPT,'binary_sha256':binding['binary_sha256'],'actual_disassembly_sha256':sha(raw),'commands':commands,'recorder_sha256':sha(Path(__file__).read_bytes()),'preserved_prior_files':7131,'candidate_E1_executed':False,'candidate_owner_publications':0,'memory_gate':'BLOCKED_ADDITIONAL_MEMORY_CAP','cap':65536,'aligned_additional_bound':65552,'cancellation_retry_controls':'NOT_RUN_MEMORY_GATE_BLOCKED','production_changes':0}
(P/'post-git-verification.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n')
print(json.dumps({'status':r['status'],'evidence':EVIDENCE,'evidence_tree':TREE,'receipt_sha256':RECEIPT,'committed_stopped_result_blobs':len(blobs)},sort_keys=True))
