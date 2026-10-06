"""Bind published result tree to actual read-only verification and linked ELF."""
from pathlib import Path
import hashlib,importlib.util,json,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
RESULT='247c46b0a65fed0d8578a7df13eff2060af12752';TREE='d4fa5b85d0c97d67a785d45f6b8983a7433e6e03'
RECEIPT='f8de833140df092488a4ef529550820bf31bab8b134070c7562acc9bea78fe98'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
def load(path):return json.loads(path.read_text())
output=P/'post-git-verification.json';require(not output.exists(),'post-Git receipt is immutable; no reexecution')
require(git('rev-parse','HEAD')==RESULT and git('rev-parse',RESULT+'^{tree}')==TREE,'actual published tested result source/tree')
require(sha((P/'receipt.json').read_bytes())==RECEIPT,'immutable result receipt')
require(git('show','-s','--format=%an <%ae>%n%cn <%ce>',RESULT)=='MrScripty <TheEnvironmentGuy@protonmail.com>\nMrScripty <TheEnvironmentGuy@protonmail.com>','result author and committer')
changes=[]
for base,count in [('4ee70e4587cb4ce5f5f164bcfea59427905e1565',29),('008fdd02ca41e88821bfa8999283347bab3455eb',109)]:
 rows=git('diff','--name-status',base,RESULT).splitlines();require(len(rows)==count and all(x.startswith('A\tevidence/forcing-e1-fixed-candidates-v1/')or x.startswith('A\tevidence/forcing-e1-signed-geometry-v1/')for x in rows),'only additive diagnostic evidence since '+base);changes.append({'base':base,'additions':count,'all_previous_source_and_evidence_unchanged':True})
raw=subprocess.check_output(['git','ls-tree','-rz',RESULT],cwd=ROOT);entries=raw.split(b'\0')[:-1];require(len(entries)==7797,'complete result tree inventory')
for entry in entries:
 metadata,name=entry.split(b'\t',1);mode,kind,oid=metadata.split();path=ROOT/name.decode();require(kind==b'blob','no submodule');data=path.readlink().as_posix().encode()if mode==b'120000'else path.read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'result Git bytes '+name.decode())
 if mode!=b'120000':require(bool(path.stat().st_mode&0o111)==(mode==b'100755'),'result Git file modes')
prior=load(P/'root-verification-commands.json');require(len(prior)==2,'two recorded result verifiers')
for command,mode in zip(prior,['normal','optimized']):
 require(command['exit']==0 and command['native_reexecution']is False,'actual precommit result verification')
 for channel in ['stdout','stderr']:require(sha((P/command[channel]).read_bytes())==command[channel+'_sha256'],'actual frozen verification logs')
 require(command['stdout']==f'root-verification-{mode}.json'and command['stderr']==f'root-verification-{mode}-stderr.log'and not(P/command['stderr']).read_bytes(),'precommit modes/no hidden failure')
 require(('-O'in command['argv'])==(mode=='optimized'),'actual interpreter mode')
commands=[]
for mode in ['normal','optimized']:
 argv=[sys.executable]+(['-O']if mode=='optimized'else[])+[str(P/'verify.py')];stdout=P/f'post-git-{mode}.json';stderr=P/f'post-git-{mode}-stderr.log';start=time.monotonic()
 with stdout.open('wb')as out,stderr.open('wb')as err:completed=subprocess.run(argv,cwd=ROOT,stdout=out,stderr=err)
 commands.append({'argv':argv,'exit':completed.returncode,'seconds':time.monotonic()-start,'stdout':stdout.name,'stderr':stderr.name,'stdout_sha256':sha(stdout.read_bytes()),'stderr_sha256':sha(stderr.read_bytes()),'native_reexecution':False})
 require(completed.returncode==0 and not stderr.read_bytes(),'actual post-Git read-only verification '+mode)
require((P/'post-git-normal.json').read_bytes()==(P/'post-git-optimized.json').read_bytes()==(P/'root-verification-normal.json').read_bytes(),'actual precommit and post-Git mode parity')
verification=load(P/'post-git-normal.json');binding=load(P/'binary-binding.json');binary=Path(binding['binary']);require(sha(binary.read_bytes())==binding['binary_sha256'],'actual current linked ELF')
elf={}
for name,argv in [('symbols',['nm','-S','-nC',str(binary)]),('relocations',['readelf','-rW',str(binary)]),('headers',['readelf','-hW',str(binary)])]:
 data=subprocess.check_output(argv);require(data==(P/('native-'+name+'.txt')).read_bytes(),'actual current ELF metadata');elf[name]={'argv':argv,'sha256':sha(data)}
receipt={'status':'PASS_POST_GIT_SEPARATE_DIAGNOSTIC_BINDING_WITH_FAILURES_RETAINED','result_source':RESULT,'result_tree':TREE,'result_tree_files':7797,'result_tree_inventory_sha256':sha(raw),'results_receipt_sha256':RECEIPT,'native_source':binding['prototype'],'native_source_tree':binding['prototype_tree'],'native_binary':str(binary),'native_binary_sha256':binding['binary_sha256'],'native_elf_metadata':elf,'tested_preflight_source':'4ee70e4587cb4ce5f5f164bcfea59427905e1565','tested_preflight_tree':'c1254024e893049c7290341916350b56ca262d8c','preflight_receipt_sha256':sha((P/'preflight-receipt.json').read_bytes()),'geometry_corrected_reader_source':'4c747b8002179ba2b39a9c5a978be5fc8fcad947','geometry_corrected_reader_tree':'088f03fc64acaaad6c74e8243bc19a2347bc6d60','separate_geometry_receipt_sha256':sha((ROOT/'evidence/forcing-e1-signed-geometry-v1/receipt.json').read_bytes()),'source_changes':changes,'recording_script_sha256':sha(Path(__file__).read_bytes()),'precommit_verification_commands_sha256':sha((P/'root-verification-commands.json').read_bytes()),'commands':commands,'verification':verification,'original_native_refusals_retained':2,'original_geometry_band_failures_retained':8,'diagnostic_only':True,'validated_remedy_established':False,'no_new_reference_or_trajectory_execution':True,'no_new_native_execution_in_post_git_check':True,'parent_controls_reviews_PRs_and_merges':True}
output.write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n')
print(json.dumps({'post_git_receipt_sha256':sha(output.read_bytes()),'result_source':RESULT,'result_tree':TREE,'status':receipt['status']}))
