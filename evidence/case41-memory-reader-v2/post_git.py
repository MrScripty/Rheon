"""Readonly Git/source binding of the tested tooling diagnosis."""
from pathlib import Path
import hashlib,json,subprocess,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
IDENTITY='MrScripty <TheEnvironmentGuy@protonmail.com>'
def req(v,msg):
 if not v:raise ValueError(msg)
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT).decode().strip()
def sha(x):return hashlib.sha256(x).hexdigest()
def tree(commit):
 result={}
 for row in subprocess.check_output(['git','ls-tree','-r','-z',commit],cwd=ROOT).split(b'\0'):
  if row:
   meta,path=row.split(b'\t',1);result[path.decode()]=meta.decode()
 return result
def run(result):
 req(result=='6599a5c234643ad3d5f53c67e63797777b93212e','exact frozen tested diagnosis')
 r=json.loads((P/'receipt.json').read_text())
 for path,digest in r['artifact_sha256'].items():req(sha((P/path).read_bytes())==digest,'receipt artifact '+path)
 req(r['FD_execution_allowed'] is False and r['native_equations']==r['owner_advances']==r['FD_test_invocations']==0 and r['native_test_invocations']==3 and r['build_invocations']==0 and r['complete_memory_bound'] is None,'limited guard-only scope')
 base='26154d9a32e193b35d9cda0b02b2d7c5f770cfeb';before=tree(base);after=tree(result)
 req(len(before)==8392 and all(after.get(path)==blob for path,blob in before.items()),'all frozen predecessor Git objects/modes retained')
 for path in set(after)-set(before):req(path.startswith(('evidence/case41-memory-reader-v2/','evidence/case47-outer-frame-clarification-v1/')) or path=='docs/research-book/implementation/case41-memory-reader-v2.md','only authorized tooling/evidence/book additions')
 commits=[r['tooling_source'],'30a312eb77b3f3a5b5aebf8af1cfb848b7257d82','ee9f5332512ca2230ecf16d705e6dd64b3597ee1',result]
 for commit in commits:req(git('show','-s','--format=%an <%ae>%n%cn <%ce>',commit).splitlines()==[IDENTITY,IDENTITY],'identity '+commit)
 remote=git('ls-remote','origin','refs/heads/research/case41-memory-reader-repair').split()[0];req(remote==result,'published tested milestone')
 return dict(status='PASS_PUBLISHED_TOOLING_DIAGNOSIS__FD_MEMORY_STILL_BLOCKED',tested_result=result,tested_result_tree=git('rev-parse',result+'^{tree}'),tooling_source=r['tooling_source'],tooling_source_tree=r['tooling_source_tree'],compiled_source=r['compiled_source'],compiled_source_tree=r['compiled_source_tree'],binary_sha256=r['binary_sha256'],receipt_sha256=sha((P/'receipt.json').read_bytes()),case47_clarification_commit=commits[1],preserved_predecessor_files=len(before),tested_tree_files=len(after),native_test_invocations=3,native_equations=0,owner_advances=0,FD_test_invocations=0,build_invocations=0,complete_memory_bound=None,remote_tested_result=remote)
if __name__=='__main__':print(json.dumps(run(sys.argv[1]),indent=2,sort_keys=True))
