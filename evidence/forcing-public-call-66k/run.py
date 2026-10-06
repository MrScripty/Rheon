"""Freeze authorization, then run exactly the unchanged specified Rust test."""
from pathlib import Path
import hashlib,importlib.util,json,os,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
policy=json.loads((P/'protocol.json').read_text())
for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen revised protocol source')
old=ROOT/'evidence/forcing-public-call'
# Verifier replays metadata and static instructions only; no native E1 run.
prior=module('old_stopped_verifier',old/'verify.py')
receipt=json.loads((old/'receipt.json').read_text())
require(receipt['preserved_inventory']==prior.preserved(),'all prior prototype paths unchanged')
require(receipt['artifacts']==prior.artifacts() and receipt['checks']==prior.checks(),'old failed result stays unchanged and unexecuted in its own packet')
binding=json.loads((old/'binary-v2-binding.json').read_text());binary=Path(binding['binary'])
require(sha(binary.read_bytes())==policy['binary_sha256']==binding['binary_sha256'],'same exact frozen binary')
require(policy['new_additional_cap']==67584 and policy['old_additional_cap']==65536,'explicit revised resource allowance')
audit=module('new_same_binary_static_audit',old/'audit_memory.py');observed=audit.run()
require(observed['status']=='BLOCKED_ADDITIONAL_MEMORY_CAP' and observed['aligned_additional_bound']==65552,'old status and observed bound retained')
require(observed['aligned_additional_bound']<=policy['new_additional_cap'],'new memory gate must pass before candidate')
qualification={'status':'PASS_REVISED_66K_PROTOCOL_MEMORY_GATE','protocol_id':policy['protocol_id'],'protocol_sha256':sha((P/'protocol.json').read_bytes()),'binary':str(binary),'binary_sha256':sha(binary.read_bytes()),'observed_aligned_additional_bound':65552,'old_protocol':{'cap':65536,'status':'BLOCKED_ADDITIONAL_MEMORY_CAP','excess':16,'old_E1_classification':'UNEXECUTED'},'new_protocol':{'cap':67584,'status':'PASS','remaining':2032},'legacy_owner_registered_additional_bytes':65536,'external_supplemental_reservation':2048,'new_full_single_owner_reservation':542352,'production_memory_limits_changed':False,'binary_rebuilt':False,'native_scalar_groups_bound':23,'exact_scalar_probes_bound':36,'native_baseline_bound':True,'candidate_E1_executed_at_authorization':False,'full_observed_memory_audit':observed}
require(not (P/'memory-authorization.json').exists() and not (P/'candidate-native.log').exists(),'never overwrite/repeat an authorized run')
(P/'memory-authorization.json').write_text(json.dumps(qualification,indent=2,sort_keys=True)+'\n')
argv=[str(binary),'--exact','research_public_call::research_public_candidate_and_controls','--nocapture']
env=os.environ.copy();env['RHEON_PUBLIC_CALL_ALLOW']=policy['binary_compatibility_token']
command={'argv':argv,'protocol_id':policy['protocol_id'],'binary_sha256':policy['binary_sha256'],'memory_authorization_sha256':sha((P/'memory-authorization.json').read_bytes()),'compatibility_token':policy['binary_compatibility_token'],'new_cap':67584,'original_cap_stays_failed':True,'candidate_main_calls_requested':1,'maximum_cancel_retry_pairs':12,'parameter_search':False,'continued_trajectory':False,'production_adoption':False}
(P/'candidate-command-before.json').write_text(json.dumps(command,indent=2,sort_keys=True)+'\n')
start=time.monotonic()
with (P/'candidate-native.log').open('wb') as out,(P/'candidate-native-stderr.log').open('wb') as err:
 result=subprocess.run(argv,cwd=ROOT,env=env,stdout=out,stderr=err)
command.update(exit=result.returncode,seconds=time.monotonic()-start,stdout_sha256=sha((P/'candidate-native.log').read_bytes()),stderr_sha256=sha((P/'candidate-native-stderr.log').read_bytes()))
(P/'candidate-command-completed.json').write_text(json.dumps(command,indent=2,sort_keys=True)+'\n')
print(json.dumps({'event':'fixed_public_experiment_finished','exit':result.returncode,'memory_gate':'PASS_REVISED_66K_PROTOCOL_MEMORY_GATE','observed_bound':65552,'new_cap':67584,'old_cap_status':'BLOCKED','no_rebuild':True}))
