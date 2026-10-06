"""Run the frozen original trajectories once, then interrupted lifecycle controls."""
from pathlib import Path
import hashlib,importlib.util,json,os,subprocess,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
spec=importlib.util.spec_from_file_location('preflight',P/'qualify.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
qualification=m.run();require(qualification==json.loads((P/'qualification-normal.json').read_text()),'fresh preflight before launch')
receipt=json.loads((P/'preflight-receipt.json').read_text())
for name,digest in receipt['artifact_sha256'].items():require(sha((P/name).read_bytes())==digest,'frozen preflight artifact '+name)
policy=json.loads((P/'protocol.json').read_text());binding=json.loads((P/'binary-binding.json').read_text());roster=json.loads((P/'roster.json').read_text())
require(not (P/'execution-authorization.json').exists(),'never overwrite or repeat experiment')
authorization={'source':binding['prototype'],'source_tree':binding['prototype_tree'],'protocol_sha256':sha((P/'protocol.json').read_bytes()),'binary_sha256':binding['binary_sha256'],'qualification_sha256':sha((P/'qualification-normal.json').read_bytes()),'preflight_receipt_sha256':sha((P/'preflight-receipt.json').read_bytes()),'runner_sha256':sha(Path(__file__).read_bytes()),'roster_sha256':sha((P/'roster.json').read_bytes()),'ordinary_indices':list(range(48)),'lifecycle_controls':roster['lifecycle_controls'],'parameter_search':False,'production_adoption':False}
(P/'execution-authorization.json').write_text(json.dumps(authorization,indent=2,sort_keys=True)+'\n')
jobs=[('trajectory',i,False)for i in range(48)]+[('lifecycle',c['trajectory_index'],True)for c in roster['lifecycle_controls']]
for mode,index,lifecycle in jobs:
 case=roster['trajectories'][index];label=f'{mode}-{index}'
 require(not (P/(label+'-native.log')).exists(),'never overwrite or repeat a trajectory')
 argv=[binding['binary'],'--exact','research_public_call::research_repeated_trajectory_one','--nocapture'];env=os.environ.copy();env['RHEON_PUBLIC_CALL_ALLOW']=policy['protocol_id'];env['RHEON_TRAJECTORY_INDEX']=str(index);env.pop('RHEON_LIFECYCLE_CONTROL',None)
 if lifecycle:env['RHEON_LIFECYCLE_CONTROL']='1'
 command={'argv':argv,'mode':mode,'trajectory':case,'authorization_sha256':sha((P/'execution-authorization.json').read_bytes()),'binary_sha256':binding['binary_sha256'],'env':{k:env[k]for k in ['RHEON_PUBLIC_CALL_ALLOW','RHEON_TRAJECTORY_INDEX','RHEON_LIFECYCLE_CONTROL']if k in env},'parameter_search':False,'production_adoption':False}
 (P/(label+'-command-before.json')).write_text(json.dumps(command,indent=2,sort_keys=True)+'\n');start=time.monotonic()
 with (P/(label+'-native.log')).open('wb') as out,(P/(label+'-native-stderr.log')).open('wb') as err:result=subprocess.run(argv,cwd=ROOT,env=env,stdout=out,stderr=err)
 command.update(exit=result.returncode,seconds=time.monotonic()-start,stdout_sha256=sha((P/(label+'-native.log')).read_bytes()),stderr_sha256=sha((P/(label+'-native-stderr.log')).read_bytes()))
 (P/(label+'-command-completed.json')).write_text(json.dumps(command,indent=2,sort_keys=True)+'\n')
 print(json.dumps({'event':'trajectory_process_finished','mode':mode,'index':index,'exit':result.returncode}),flush=True)
 require(result.returncode==0,'assertion/infrastructure failure: preserve all evidence and stop without retry')
print(json.dumps({'status':'ALL_48_TRAJECTORIES_AND_8_CONTROLS_EXECUTED_ONCE','numerical_refusals_retained':True,'parameter_search':False,'production_adoption':False}))
