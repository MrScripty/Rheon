"""Bind committed complete roster to exact native binary and actual replays."""
from pathlib import Path
import gzip,hashlib,json,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='876e78c46503a16232ecd0233df9934f081e8db0'
EVIDENCE='95d75a8fcfb5b852a031c7e902dc7105803c5af2'
TREE='02f9ad66647dc2e86fc9090c02ce357a57194d38'
RECEIPT='900fa623b772cb3029bd9c580787a689e7019a9b6b6d1da8b7914016ac14ea59'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
require(git('rev-parse',EVIDENCE+'^{tree}').decode().strip()==TREE,'frozen complete result tree')
require(sha((P/'receipt.json').read_bytes())==RECEIPT,'frozen result receipt')
changes=git('diff','--name-status',BASE,EVIDENCE).decode().splitlines();require(len(changes)==32,'closed additive 32-file complete result packet');blobs={}
for line in changes:
 status,path=line.split('\t');require(status=='A' and path.startswith('evidence/forcing-public-generalization-v2/'),'additive result packet only')
 data=git('show',EVIDENCE+':'+path);require(data==(ROOT/path).read_bytes(),'actual committed result blob '+path);blobs[path]=sha(data)
require(git('diff','e3b6858d2d29fc8837dbf2acd84db0c01784a352',EVIDENCE,'--','src','docs','research','evidence/forcing-public-call','evidence/forcing-public-call-66k')==b'','production/book/proofs and both old resource protocols untouched')
binding=json.loads((P/'binary-binding.json').read_text());binary=binding['binary'];require(sha(Path(binary).read_bytes())==binding['binary_sha256'],'actual new binary unchanged')
raw=subprocess.check_output(['objdump','-d','-C',binary]);require(raw==gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()),'actual linked instruction certificate')
for name,argv in [('symbols',['nm','-S','-nC',binary]),('relocations',['readelf','-rW',binary]),('headers',['readelf','-hW',binary])]:require(subprocess.check_output(argv)==(P/('native-'+name+'.txt')).read_bytes(),'actual ELF metadata '+name)
commands=[]
for label,extra in [('normal',[]),('optimized',['-O'])]:
 argv=[sys.executable,*extra,'evidence/forcing-public-generalization-v2/verify.py'];start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE);seconds=time.monotonic()-start
 stdout,stderr='post-git-'+label+'.log','post-git-'+label+'-stderr.log';(P/stdout).write_bytes(result.stdout);(P/stderr).write_bytes(result.stderr)
 commands.append({'argv':argv,'exit':result.returncode,'seconds':seconds,'stdout':stdout,'stderr':stderr,'stdout_sha256':sha(result.stdout),'stderr_sha256':sha(result.stderr)})
 require(result.returncode==0,'actual post-Git complete-result verification pass')
require(commands[0]['stdout_sha256']==commands[1]['stdout_sha256'],'actual post-Git mode parity')
r={'status':'PASS_COMMITTED_FIVE_CALL_GENERALIZATION_BINDING','source':binding['prototype'],'source_tree':binding['prototype_tree'],'preflight':BASE,'preflight_tree':'3180d8341c24983844a8c36e7d5e3fb0e310e5c0','evidence':EVIDENCE,'evidence_tree':TREE,'committed_complete_result_blobs':blobs,'receipt_sha256':RECEIPT,'protocol_sha256':sha((P/'protocol.json').read_bytes()),'binary_sha256':binding['binary_sha256'],'actual_disassembly_sha256':sha(raw),'commands':commands,'recorder_sha256':sha(Path(__file__).read_bytes()),'preserved_prior_files':7269,'case_roster':'all five terminal-repair refusal inputs including passing pressure-forward control','once_only_candidate_main_calls':5,'successes':5,'remaining_prescribed_refusals':[],'original_owners_unchanged':True,'passing_control_report_and_snapshot_match':True,'additional_bound':65536,'authorized_additional_cap':67584,'remaining_memory':2048,'newton_target':1e-13,'physical_momentum_limit':1e-11,'counted_newton_equations':[22,29,15,22,22],'completed_total_equations':[24,31,17,24,24],'old_one_step_experiments_and_controls_unchanged':True,'failed_first_compile_preserved':True,'parameter_search':False,'continued_trajectory':False,'production_adoption':False,'production_source_changes':0}
(P/'post-git-verification.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n')
print(json.dumps({'status':r['status'],'evidence':EVIDENCE,'evidence_tree':TREE,'receipt_sha256':RECEIPT,'committed_complete_result_blobs':len(blobs)},sort_keys=True))
