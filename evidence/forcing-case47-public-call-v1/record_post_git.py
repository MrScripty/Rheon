"""Bind published result tree and both receipts; never execute native code."""
from pathlib import Path
import hashlib,importlib.util,json,subprocess,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='2f23503d9bf74842e39f9fe0760edf4bfeb41b4c';BASE_TREE='d178347673f026cd9336f5909318208416d9e783'
def require(ok,msg):
 if not ok:raise ValueError(msg)
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT).decode().strip()
def sha(b):return hashlib.sha256(b).hexdigest()
def inventory(commit):
 digest=hashlib.sha256();count=0
 for entry in subprocess.check_output(['git','ls-tree','-rz',commit],cwd=ROOT).split(b'\0'):
  if not entry:continue
  meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();require(mode in [b'100644',b'100755']and kind==b'blob','regular frozen evidence files');data=(ROOT/name.decode()).read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'preserved tree file '+name.decode());digest.update(name+b'\0'+hashlib.sha256(data).digest());count+=1
 return {'files':count,'path_content_sha256':digest.hexdigest()}
def run(result):
 require(git('rev-parse',BASE+'^{tree}')==BASE_TREE,'unchanged accepted fixed-result tree');old=inventory(BASE);require(old['files']==7926,'full frozen prior inventory');published=inventory(result);receipts={}
 for name in ['forcing-case41-scale-diagnosis-v1','forcing-case47-public-call-v1']:
  folder=ROOT/'evidence'/name;receipt=json.loads((folder/'receipt.json').read_text());artifacts=receipt.get('artifact_sha256',receipt.get('artifacts'))
  for path,digest in artifacts.items():require(sha((folder/path).read_bytes())==digest,'frozen evidence artifact '+name+'/'+path)
  receipts[name]=sha((folder/'receipt.json').read_bytes())
 s=importlib.util.spec_from_file_location('newcase47',P/'verify.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m);checks=m.run();require(checks==json.loads((P/'verification-normal.json').read_text()),'fresh readonly result verification');return {'status':'PASS_PUBLISHED_BOUNDED_FOLLOWUP_TREE','base':BASE,'base_tree':BASE_TREE,'preserved_base':old,'result_commit':result,'result_tree':git('rev-parse',result+'^{tree}'),'result_inventory':published,'receipt_sha256':receipts,'checks':checks,'new_native_executions':0,'new_owner_advances':0,'original_failures_retained':True}
if __name__=='__main__':print(json.dumps(run(sys.argv[1]),indent=2,sort_keys=True))
