"""Run the exact two fixed E1 candidates once, after frozen preflight."""
from pathlib import Path
import hashlib,importlib.util,json,os,subprocess,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
spec=importlib.util.spec_from_file_location('qualification',P/'qualify.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);qualification=m.run();require(qualification==json.loads((P/'qualification-normal.json').read_text()),'fresh frozen preflight')
receipt=json.loads((P/'preflight-receipt.json').read_text())
for name,digest in receipt['artifacts'].items():require(sha((P/name).read_bytes())==digest,'frozen preflight '+name)
require(not(P/'execution-authorization.json').exists()and not(P/'capture-native.log').exists(),'never rerun or overwrite fixed candidate capture')
binding=json.loads((P/'binary-binding.json').read_text());argv=[binding['binary'],'--exact','research_public_call::fixed_capture_e1_diagnostic','--nocapture'];env=os.environ.copy();env.pop('RHEON_PUBLIC_CALL_ALLOW',None);env['RHEON_FIXED_CAPTURE_ALLOW']='E1-two-terminal-fixed-candidates-v1'
command={'argv':argv,'env':{'RHEON_FIXED_CAPTURE_ALLOW':env['RHEON_FIXED_CAPTURE_ALLOW']},'source':binding['prototype'],'binary_sha256':binding['binary_sha256'],'protocol_sha256':binding['protocol_sha256'],'preflight_receipt_sha256':sha((P/'preflight-receipt.json').read_bytes()),'runner_sha256':sha(Path(__file__).read_bytes()),'accepted_owners_constructed':0,'accepted_owners_advanced':0,'controller_iterations':0,'candidate_equations_requested':4,'published':False}
(P/'execution-authorization.json').write_text(json.dumps(command,indent=2,sort_keys=True)+'\n');start=time.monotonic()
with(P/'capture-native.log').open('wb')as out,(P/'capture-native-stderr.log').open('wb')as err:result=subprocess.run(argv,cwd=ROOT,env=env,stdout=out,stderr=err)
command.update(exit=result.returncode,seconds=time.monotonic()-start,stdout_sha256=sha((P/'capture-native.log').read_bytes()),stderr_sha256=sha((P/'capture-native-stderr.log').read_bytes()));(P/'capture-command-completed.json').write_text(json.dumps(command,indent=2,sort_keys=True)+'\n')
require(result.returncode==0,'capture assertion/infrastructure failure: preserve and stop without retry')
print(json.dumps({'status':'COMPLETE_FIXED_CANDIDATE_CAPTURE_ONCE','accepted_owners_advanced':0,'controller_iterations':0,'published':False}))
