"""Bind actual committed result blobs, ELF instructions and replay commands."""
from pathlib import Path
import gzip,hashlib,json,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='94818f15e7d9148a079a4797392a8b5e356fd59b'
EVIDENCE='34ba87d95ffc46fbd1c7029d332e999fbfed1c79'
TREE='43f60e58cf4384a733099a117ca345e33a535192'
RECEIPT='32f5471040fd70e4bde56f17f00901b023e04b82cbe2443444e3cd49f8173f39'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
require(git('rev-parse',EVIDENCE+'^{tree}').decode().strip()==TREE,'frozen numerical result tree')
require(sha((P/'receipt.json').read_bytes())==RECEIPT,'frozen result receipt')
changes=git('diff','--name-status',BASE,EVIDENCE).decode().splitlines();require(len(changes)==16,'closed additive sixteen-file result packet');blobs={}
for line in changes:
 status,path=line.split('\t');require(status=='A' and path.startswith('evidence/forcing-public-call-66k/'),'additive new packet only')
 data=git('show',EVIDENCE+':'+path);require(data==(ROOT/path).read_bytes(),'actual committed result blob '+path);blobs[path]=sha(data)
require(git('diff','4d60e9637431a8d9792944bcd9ff00979ead237e',EVIDENCE,'--','src','research','evidence/forcing-public-call')==b'','all old source and stopped result untouched')
old=ROOT/'evidence/forcing-public-call';binding=json.loads((old/'binary-v2-binding.json').read_text());binary=binding['binary']
require(sha(Path(binary).read_bytes())==binding['binary_sha256'],'same selected actual binary')
raw=subprocess.check_output(['objdump','-d','-C',binary]);require(raw==gzip.decompress((old/'native-v2-disassembly.txt.gz').read_bytes()),'actual linked instructions match certificate')
for name,argv in [('symbols',['nm','-S','-nC',binary]),('relocations',['readelf','-rW',binary]),('headers',['readelf','-hW',binary])]:
 require(subprocess.check_output(argv)==(old/('native-v2-'+name+'.txt')).read_bytes(),'actual ELF metadata '+name)
commands=[]
for label,extra in [('normal',[]),('optimized',['-O'])]:
 argv=[sys.executable,*extra,'evidence/forcing-public-call-66k/verify.py'];start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE);seconds=time.monotonic()-start
 stdout,stderr='post-git-'+label+'.log','post-git-'+label+'-stderr.log';(P/stdout).write_bytes(result.stdout);(P/stderr).write_bytes(result.stderr)
 commands.append({'argv':argv,'exit':result.returncode,'seconds':seconds,'stdout':stdout,'stderr':stderr,'stdout_sha256':sha(result.stdout),'stderr_sha256':sha(result.stderr)})
 require(result.returncode==0,'actual post-Git verification pass')
require(commands[0]['stdout_sha256']==commands[1]['stdout_sha256'],'post-Git normal/optimized parity')
r={'status':'PASS_COMMITTED_FIXED_PUBLIC_CALL_66K_BINDING','source':BASE,'source_tree':'23a9ae79b0880d3f3c900f548aff828a65898883','evidence':EVIDENCE,'evidence_tree':TREE,'committed_result_blobs':blobs,'receipt_sha256':RECEIPT,'protocol_sha256':sha((P/'protocol.json').read_bytes()),'binary_sha256':binding['binary_sha256'],'actual_disassembly_sha256':sha(raw),'commands':commands,'recorder_sha256':sha(Path(__file__).read_bytes()),'preserved_prior_files':7166,'new_memory_cap':67584,'observed_memory_bound':65552,'memory_margin':2032,'old_memory_cap':65536,'old_memory_gate':'BLOCKED_ADDITIONAL_MEMORY_CAP','old_protocol_E1':'UNEXECUTED','new_protocol_E1':'SUCCESS','isolated_main_publications':1,'reached_cancellation_retry_controls':11,'unused_stage':'NOT_REACHED_BY_FIXED_CALL','original_owner_unchanged':True,'parameter_search':False,'continued_trajectory':False,'production_adoption':False,'production_changes':0}
(P/'post-git-verification.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n')
print(json.dumps({'status':r['status'],'evidence':EVIDENCE,'evidence_tree':TREE,'receipt_sha256':RECEIPT,'committed_result_blobs':len(blobs)},sort_keys=True))
