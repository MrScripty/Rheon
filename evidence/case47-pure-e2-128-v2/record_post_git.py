"""Bind tested result tree, both stopped experiments and frozen worktrees."""
from pathlib import Path
import hashlib,importlib.util,json,subprocess,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1];BASE='8fc718ff603e7a7123037e592b4f40f59762e011';IDENT='MrScripty <TheEnvironmentGuy@protonmail.com>'
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def git(*args,cwd=ROOT):return subprocess.check_output(['git',*args],cwd=cwd).decode().strip()
def run(result):
 v=load('tree_inventory',P/'verify_v2.py');preserved=v.inventory(BASE);published=v.inventory(result);require(preserved['files']==8134,'all inherited bytes')
 receipts={}
 for folder,name in [('case41-six-fd-columns-v1','blocked-receipt.json'),('case47-pure-e2-128-v1','blocked-receipt.json'),('case47-pure-e2-128-v2','receipt.json')]:
  d=ROOT/'evidence'/folder;r=json.loads((d/name).read_text())
  for path,digest in r['artifact_sha256'].items():require(sha((d/path).read_bytes())==digest,'frozen experiment artifact '+folder+'/'+path)
  receipts[folder]=sha((d/name).read_bytes())
 require((P/'verification-v2-normal.json').read_bytes()==(P/'verification-v2-optimized.json').read_bytes(),'final verification mode parity');checks=json.loads((P/'verification-v2-normal.json').read_text());require(checks['preserved_base']==preserved and checks['actual_main_steps']==128 and checks['actual_controls']==list(range(11)) and checks['case41_candidate_equations']==0,'honest qualified/stopped results')
 parent=load('old_frozen_worktrees',ROOT/'evidence/case41-scaled-fd-preparation-v1/record_post_git.py');frozen={**parent.FROZEN,'Rheon-case41-next-plan':BASE};worktrees={}
 for name,expected in frozen.items():
  cwd=Path('/workspace')/name;head=git('rev-parse','HEAD',cwd=cwd);status=git('status','--porcelain',cwd=cwd);require(head==expected and not status,'frozen worktree '+name);worktrees[name]={'head':head,'status':status}
 identities={}
 for c in ['f2d2183346fcab7f6a825b831e3f15c194ed4c96','e0b5343c865c9389d4b59ed7bfd60c272d53a4c2','e37b0bb34ace62f16dc931488146d6a396b27be0','529707429fb092beb4df9a278d978bf72e8612cf',result]:
  values=git('show','-s','--format=%an <%ae>%n%cn <%ce>',c).splitlines();require(values==[IDENT,IDENT],'author/committer identity');identities[c]=values
 b=json.loads((P/'binary-binding.json').read_text());require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'same tested ELF')
 return {'status':'PASS_PUBLISHED_SEPARATE_CASE41_STOP_AND_CASE47_PURE_E2_RESULT','base':BASE,'base_tree':git('rev-parse',BASE+'^{tree}'),'preserved_base':preserved,'result_commit':result,'result_tree':git('rev-parse',result+'^{tree}'),'result_inventory':published,'receipt_sha256':receipts,'binary_sha256':b['binary_sha256'],'frozen_worktrees':worktrees,'identities':identities,'case41_new_candidate_equations':0,'case47_actual_main_steps':128,'case47_cancelled_controls':11,'step_retries':0,'new_native_executions_by_this_reader':0,'production_adoption':False,'all_original_failures_retained':True}
if __name__=='__main__':print(json.dumps(run(sys.argv[1]),indent=2,sort_keys=True))
