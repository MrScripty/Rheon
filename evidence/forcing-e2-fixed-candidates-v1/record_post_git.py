"""Record exact published E2 result tree and fresh read-only evidence checks."""
from pathlib import Path
import hashlib,json,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
RESULT='0f6ff5312d25cf96cf1a9316a4cbe1edc2618d60';TREE='e9c6fcad8a43b110640e66b965b3ac65b925e0f4';RECEIPT='b9f651ba042abfc1189efa7b63c8dc1213f221823113d9521dc252aecbcfe9ac'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
def load(path):return json.loads(path.read_text())
output=P/'post-git-verification.json';require(not output.exists(),'never overwrite post-Git evidence');require(git('rev-parse','HEAD')==RESULT and git('rev-parse',RESULT+'^{tree}')==TREE,'actual tested result commit/tree');require(sha((P/'receipt.json').read_bytes())==RECEIPT,'frozen actual result receipt')
require(git('show','-s','--format=%an <%ae>%n%cn <%ce>',RESULT)=='MrScripty <TheEnvironmentGuy@protonmail.com>\nMrScripty <TheEnvironmentGuy@protonmail.com>','author and committer')
changes=[]
for base,count in [('c2ee6acc8a783161a844e3dee0566af71732628d',35),('c210d9ff630504748fad129337aaa84e013522a6',104)]:
 rows=git('diff','--name-status',base,RESULT).splitlines();require(len(rows)==count and all(x.startswith('A\tevidence/forcing-e2-fixed-candidates-v1/')for x in rows),'additive E2 evidence only since '+base);changes.append({'base':base,'additive_files':count,'all_previous_source_evidence_unchanged':True})
raw=subprocess.check_output(['git','ls-tree','-rz',RESULT],cwd=ROOT);entries=raw.split(b'\0')[:-1];require(len(entries)==7920,'exact result inventory')
for entry in entries:
 meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();path=ROOT/name.decode();require(kind==b'blob','no submodule');data=path.readlink().as_posix().encode()if mode==b'120000'else path.read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'exact result Git bytes '+name.decode())
 if mode!=b'120000':require(bool(path.stat().st_mode&0o111)==(mode==b'100755'),'result file mode')
prior=load(P/'verification-commands.json');require(len(prior)==2,'two actual closed verifiers')
for command in prior:
 require(command['exit']==0 and not command['native_execution'],'actual successful read-only verification')
 for channel in ['stdout','stderr']:require(sha((P/command[channel]).read_bytes())==command[channel+'_sha256'],'actual precommit verifier logs')
 require(not(P/command['stderr']).read_bytes(),'no hidden verification failure')
commands=[]
for mode in ['normal','optimized']:
 argv=[sys.executable]+(['-O']if mode=='optimized'else[])+[str(P/'verify.py')];stdout=P/f'post-git-{mode}.json';stderr=P/f'post-git-{mode}-stderr.log';start=time.monotonic()
 with stdout.open('wb')as out,stderr.open('wb')as err:r=subprocess.run(argv,cwd=ROOT,stdout=out,stderr=err)
 commands.append({'argv':argv,'exit':r.returncode,'seconds':time.monotonic()-start,'stdout':stdout.name,'stderr':stderr.name,'stdout_sha256':sha(stdout.read_bytes()),'stderr_sha256':sha(stderr.read_bytes()),'native_execution':False});require(r.returncode==0 and not stderr.read_bytes(),'post-Git read-only verifier '+mode)
require((P/'post-git-normal.json').read_bytes()==(P/'post-git-optimized.json').read_bytes()==(P/'verification-normal.json').read_bytes(),'precommit/post-Git normal/optimized parity')
binding=load(P/'binary-binding.json');require(sha(Path(binding['binary']).read_bytes())==binding['binary_sha256'],'actual same native ELF');metadata={}
for name,argv in [('symbols',['nm','-S','-nC',binding['binary']]),('headers',['readelf','-hW',binding['binary']]),('relocations',['readelf','-rW',binding['binary']])]:
 data=subprocess.check_output(argv);require(data==(P/('native-'+name+'.txt')).read_bytes(),'actual same ELF metadata');metadata[name]={'argv':argv,'sha256':sha(data)}
receipt={'status':'PASS_POST_GIT_E2_FIXED_RESULTS_ONE_CASE_STILL_ABOVE_TARGET','result_source':RESULT,'result_tree':TREE,'result_tree_files':7920,'result_tree_inventory_sha256':sha(raw),'results_receipt_sha256':RECEIPT,'native_source':binding['prototype'],'native_source_tree':binding['prototype_tree'],'native_binary':binding['binary'],'native_binary_sha256':binding['binary_sha256'],'native_ELF_metadata':metadata,'preflight_source':'c2ee6acc8a783161a844e3dee0566af71732628d','preflight_tree':'a59508d87e97e2fb97f5d52d34d187924bbf0d92','preflight_receipt_sha256':sha((P/'preflight-receipt.json').read_bytes()),'review_and_amended_contract_source':'c210d9ff630504748fad129337aaa84e013522a6','review_and_amended_contract_tree':'412bcae8cf87b79b73a995e6f47d377859c6cebc','source_scope':changes,'recording_script_sha256':sha(Path(__file__).read_bytes()),'precommit_verification_commands_sha256':sha((P/'verification-commands.json').read_bytes()),'commands':commands,'verification':load(P/'post-git-normal.json'),'native_execution_in_post_git_checks':False,'trajectory_execution_in_post_git_checks':False,'case41_still_above_target':True,'case47_fixed_planar_eligible_only':True,'original_runtime_refusals_unresolved':2,'original_geometry_band_failures_unresolved':8,'production_adoption':False,'parent_controls_PRs_reviews_and_merges':True}
output.write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n');print(json.dumps({'post_git_receipt_sha256':sha(output.read_bytes()),'result_source':RESULT,'result_tree':TREE,'status':receipt['status']}))
