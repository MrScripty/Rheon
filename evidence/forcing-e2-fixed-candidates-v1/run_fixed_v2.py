"""Run the exact reproduced E1 baseline, then four authorized E2 fixed equations."""
from pathlib import Path
import hashlib,json,os,subprocess,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def rows(path):return [json.loads(x)for x in path.read_text().splitlines()if x.startswith('{')]
def run(label,argv,env):
 for suffix in ['.log','-stderr.log','-command-completed.json']:require(not(P/(label+suffix)).exists(),'never overwrite a native execution journal')
 start=time.monotonic()
 with(P/(label+'.log')).open('wb')as out,(P/(label+'-stderr.log')).open('wb')as err:result=subprocess.run(argv,cwd=ROOT,env=env,stdout=out,stderr=err)
 receipt={'argv':argv,'exit':result.returncode,'seconds':time.monotonic()-start,'stdout_sha256':sha((P/(label+'.log')).read_bytes()),'stderr_sha256':sha((P/(label+'-stderr.log')).read_bytes()),'env':{k:v for k,v in env.items()if k.startswith('RHEON_')},'accepted_owners_constructed':0,'accepted_owners_advanced':0,'controller_iterations':0,'third_solve_executed':False,'published':False}
 (P/(label+'-command-completed.json')).write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n');require(result.returncode==0,'actual native fixed execution '+label);return receipt
policy=json.loads((P/'protocol.json').read_text());preflight=json.loads((P/'preflight-receipt.json').read_text());binding=json.loads((P/'binary-binding.json').read_text());require(preflight['status']=='PASS_E2_PREFLIGHT_BEFORE_FIXED_EXECUTION','fresh preflight required')
for name,digest in preflight['artifacts'].items():require(sha((P/name).read_bytes())==digest,'immutable actual preflight artifact')
for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen implementation source')
require(sha(Path(binding['binary']).read_bytes())==binding['binary_sha256'],'actual E2 binary')
base=policy['baseline_binary'];require(sha(Path(base).read_bytes())==policy['baseline_binary_sha256'],'actual original E1 binary')
env=os.environ.copy()
for key in list(env):
 if key.startswith('RHEON_'):env.pop(key)
env['RHEON_FIXED_CAPTURE_ALLOW']='E1-two-terminal-fixed-candidates-v1'
authorization={'protocol_sha256':sha((P/'protocol.json').read_bytes()),'preflight_receipt_sha256':sha((P/'preflight-receipt.json').read_bytes()),'runner_sha256':sha(Path(__file__).read_bytes()),'baseline_binary_sha256':policy['baseline_binary_sha256'],'E2_binary_sha256':binding['binary_sha256'],'source':binding['prototype'],'source_tree':binding['prototype_tree'],'E2_fixed_equations_requested':4,'review_disposition':'ACK_DIAGNOSTICS_ONLY','coordinator_authorization_sha256':policy['review_authorization_sha256'],'owner_advances':0,'controller_iterations':0,'trajectory_reruns':False,'new_references':False}
require(not(P/'execution-authorization.json').exists(),'single frozen authorization only');(P/'execution-authorization.json').write_text(json.dumps(authorization,indent=2,sort_keys=True)+'\n')
run('baseline-reproduced',[base,'--exact','research_public_call::fixed_capture_e1_diagnostic','--nocapture'],env)
baseline=rows(P/'baseline-reproduced.log');eq=[x for x in baseline if x['event']=='fixed_equation'];require([(x['index'],x['order'])for x in eq]==[(41,16),(41,32),(47,16),(47,32)],'exact reproduced E1 roster')
for index,expected in zip([41,47],policy['expected_original_norms']):require(next(x['norm']for x in eq if x['index']==index and x['order']==16)==expected and expected>1e-13,'both actual original E1 refusals first')
require([x for x in(P/'baseline-reproduced.log').read_bytes().splitlines()if x.startswith(b'{')]==[x for x in(ROOT/'evidence/forcing-e1-fixed-candidates-v1/capture-native.log').read_bytes().splitlines()if x.startswith(b'{')],'all original E1 capture JSON bytes reproduced; libtest wall time is metadata')
require(not(P/'baseline-reproduced-stderr.log').read_bytes()and '1 passed; 0 failed'in(P/'baseline-reproduced.log').read_text(),'actual baseline assertions and no controller trace')
env.pop('RHEON_FIXED_CAPTURE_ALLOW');env['RHEON_E2_FIXED_CAPTURE_ALLOW']='E2-two-fixed-candidates-native-known-v1'
run('candidate-fixed',[binding['binary'],'--exact','research_public_call::fixed_capture_e2_experiment','--nocapture'],env)
print(json.dumps({'event':'authorized_fixed_roster_finished','candidate_equations_requested':4,'owner_advances':0,'controller_iterations':0,'no_trajectory_execution':True,'no_publication':True}))
