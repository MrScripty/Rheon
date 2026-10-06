"""Fresh actual native/scalar/memory preflight; never executes an E2 point."""
from pathlib import Path
import hashlib,gzip,importlib.util,json,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def run():
 policy=json.loads((P/'protocol.json').read_text());binding=json.loads((P/'binary-binding.json').read_text());commands=json.loads((P/'preflight-commands.json').read_text());require(binding['prototype']==commands['source']=='cbeaa72b403cb1f3af4f95727cc29ec660c99ca3'and binding['prototype_tree']==commands['source_tree']=='2539d960f279d5f50aa213f3374651d7c51b5957','actual frozen compiled source')
 for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'all frozen source '+path)
 original=(ROOT/'evidence/forcing-e1-fixed-candidates-v1/candidate.rs').read_text();require((P/'candidate.rs').read_text()==original.replace('research.solve(&p.d, &p.z)?','research.solve(&p.d, &eta0, unknown, t)?'),'only chart-call expression differs; native known/inertia/downstream body retained')
 require(binding['protocol_sha256']==sha((P/'protocol.json').read_bytes())and binding['rustflags']==policy['rustflags']and binding['target']==policy['target'],'actual frozen build policy')
 require([Path(x['stdout']).stem for x in commands['commands']]==['format','install','build','clippy','scalar-native','type-layout','affine-native','reference-fixed-native','restore'],'exact preflight roster with no E2 execution')
 for command in commands['commands']:
  require(command['exit']==0,'actual preflight command pass')
  for channel in ['stdout','stderr']:require(sha((P/command[channel]).read_bytes())==command[channel+'_sha256'],'actual preflight log binding')
 for name in ['scalar-native.log','type-layout.log','affine-native.log','reference-fixed-native.log']:require('1 passed; 0 failed'in(P/name).read_text(),'actual assertions pass')
 scalar=module('fresh_scalar_oracle',P/'check_scalar.py').run(P/'scalar-native.log');require(scalar==json.loads((P/'scalar-oracle.json').read_text())and scalar['native_assertion_groups']==23 and scalar['exact_scalar_probes']==36,'actual primitive scalar conformance')
 affine=module('fresh_affine_oracle',P/'check_affine.py').run();require(affine==json.loads((P/'affine-oracle.json').read_text()),'all six actual working-affine scalar probes')
 binary=binding['binary'];require(sha(Path(binary).read_bytes())==binding['binary_sha256'],'actual linked ELF')
 raw=subprocess.check_output(['objdump','-d','-C',binary]);require(raw==gzip.decompress((P/'native-disassembly.txt.gz').read_bytes())and sha(raw)==(P/'native-disassembly.txt.sha256').read_text().strip(),'actual linked instructions')
 for name,argv in [('symbols',['nm','-S','-nC',binary]),('relocations',['readelf','-rW',binary]),('headers',['readelf','-hW',binary])]:require(subprocess.check_output(argv)==(P/('native-'+name+'.txt')).read_bytes(),'actual ELF metadata')
 require((P/'memory-normal.json').read_bytes()==(P/'memory-optimized.json').read_bytes(),'fresh memory mode parity');memory=module('fresh_E2_memory',P/'audit_memory_v2.py').run();require(memory==json.loads((P/'memory-normal.json').read_text())and memory['status']=='PASS_FIXED_CANDIDATE_MEMORY_PREFLIGHT'and memory['aligned_additional_bound']==65840 and memory['remaining']==1744 and memory['kernel_peak']==1696,'fresh actual matched kernel/capture/public/constructor/planar-gate memory paths')
 review=ROOT/'evidence/forcing-affine-rounding-contract-v1/review-authorization.json';amend=ROOT/'evidence/forcing-affine-rounding-contract-v1/native-known-amendment-policy.json';require(sha(review.read_bytes())==policy['review_authorization_sha256']and sha(amend.read_bytes())==policy['amendment_policy_sha256'],'actual coordinator ACK and stricter public-known boundary')
 return {'status':'PASS_E2_PREFLIGHT_BEFORE_FIXED_EXECUTION','native_source':binding['prototype'],'native_source_tree':binding['prototype_tree'],'binary_sha256':binding['binary_sha256'],'protocol_sha256':binding['protocol_sha256'],'native_scalar_groups':23,'exact_primitive_scalar_probes':36,'actual_affine_scalar_probes':6,'kernel_peak':1696,'additional_memory_bound':65840,'additional_memory_cap':67584,'remaining_memory':1744,'candidate_equations_executed_in_preflight':0,'accepted_owners_constructed_or_advanced':0,'controller_iterations':0,'trajectory_reruns':False,'reference_integrations':False,'production_adoption':False,'public_known_coordinate_graph_unchanged':True,'stored_inertia_graph_unchanged':True,'fresh_actual_ELF_audit':True}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
