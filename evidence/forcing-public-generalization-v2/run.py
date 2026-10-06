"""After frozen qualification, execute each fixed public call exactly once."""
from pathlib import Path
import hashlib,importlib.util,json,os,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
spec=importlib.util.spec_from_file_location('preflight',P/'qualify.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
qualification=m.run();require(qualification==json.loads((P/'qualification-normal.json').read_text()),'frozen preflight replay before any launch')
policy=json.loads((P/'protocol.json').read_text());binding=json.loads((P/'binary-binding.json').read_text());cases=json.loads((P/'cases.json').read_text())['cases']
require(not (P/'execution-authorization.json').exists(),'never overwrite/repeat the prescribed experiment')
authorization={'source':binding['prototype'],'protocol_sha256':sha((P/'protocol.json').read_bytes()),'binary_sha256':binding['binary_sha256'],'qualification_sha256':sha((P/'qualification-normal.json').read_bytes()),'runner_sha256':sha(Path(__file__).read_bytes()),'case_roster_sha256':sha((P/'cases.json').read_bytes()),'case_indices':[c['index'] for c in cases],'main_calls_per_case':1,'parameter_search':False,'continued_trajectory':False,'production_adoption':False,'generalization_executed_at_authorization':False}
(P/'execution-authorization.json').write_text(json.dumps(authorization,indent=2,sort_keys=True)+'\n')
for case in cases:
 index=case['index'];label=f'case-{index}';require(not (P/(label+'-native.log')).exists(),'case never overwritten or retried')
 argv=[binding['binary'],'--exact','research_public_call::research_generalization_candidate_one','--nocapture'];env=os.environ.copy();env['RHEON_PUBLIC_CALL_ALLOW']=policy['protocol_id'];env['RHEON_CASE_INDEX']=str(index)
 command={'argv':argv,'case_index':index,'case_id':case['id'],'authorization_sha256':sha((P/'execution-authorization.json').read_bytes()),'binary_sha256':binding['binary_sha256'],'env':{'RHEON_PUBLIC_CALL_ALLOW':policy['protocol_id'],'RHEON_CASE_INDEX':str(index)},'main_calls_requested':1,'parameter_search':False,'continued_trajectory':False,'production_adoption':False}
 (P/(label+'-command-before.json')).write_text(json.dumps(command,indent=2,sort_keys=True)+'\n');start=time.monotonic()
 with (P/(label+'-native.log')).open('wb') as out,(P/(label+'-native-stderr.log')).open('wb') as err:result=subprocess.run(argv,cwd=ROOT,env=env,stdout=out,stderr=err)
 command.update(exit=result.returncode,seconds=time.monotonic()-start,stdout_sha256=sha((P/(label+'-native.log')).read_bytes()),stderr_sha256=sha((P/(label+'-native-stderr.log')).read_bytes()));(P/(label+'-command-completed.json')).write_text(json.dumps(command,indent=2,sort_keys=True)+'\n')
 print(json.dumps({'event':'prescribed_case_finished','case':index,'exit':result.returncode}),flush=True)
 require(result.returncode==0,'assertion/infrastructure failure; preserve logs and stop without retry')
print(json.dumps({'status':'ALL_FIVE_PRESCRIBED_CALLS_EXECUTED_ONCE','normal_solver_refusals_retained':True,'no_retries':True,'parameter_search':False,'continued_trajectory':False}))
