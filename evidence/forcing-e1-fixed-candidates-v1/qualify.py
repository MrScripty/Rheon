"""Bind frozen native source, actual preflight and capture memory before E1."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def rows(path):return [json.loads(x)for x in path.read_text().splitlines()if x.startswith('{')]
def run():
 policy=json.loads((P/'protocol.json').read_text());binding=json.loads((P/'binary-binding.json').read_text());commands=json.loads((P/'preflight-commands.json').read_text());inputs=json.loads((P/'inputs.json').read_text())
 require(binding['prototype']==commands['source']=='00d7de13b82396836e8f1efad6e3049034f5c2e7' and binding['prototype_tree']==commands['source_tree']=='e6487beb81407818a0280513fd2a1bbbc986d883','frozen compiled native source')
 for field in ['source_sha256','numerical_body_sha256']:
  for name,digest in policy[field].items():require(sha((ROOT/name).read_bytes())==digest,'all frozen source '+name)
 for name in ['candidate.rs','reference.rs','scalar.rs']:require((P/name).read_bytes().startswith((ROOT/'evidence/forcing-e1-trajectories-v2'/name).read_bytes()),'accepted numerical body exact prefix')
 for name,digest in inputs['source_sha256'].items():require(sha((ROOT/name).read_bytes())==digest,'actual failed input source '+name)
 require(binding['protocol_sha256']==sha((P/'protocol.json').read_bytes()) and binding['target']==policy['target'] and binding['rustflags']==policy['rustflags'],'frozen build policy')
 require([Path(c['stdout']).stem for c in commands['commands']]==['format','install','build','clippy','scalar-native','type-layout','reference-fixed-native','restore'],'exact preflight command roster')
 for command in commands['commands']:
  require(command['exit']==0,'actual preflight command pass')
  for channel in ['stdout','stderr']:require(sha((P/command[channel]).read_bytes())==command[channel+'_sha256'],'actual preflight logs')
 for name in ['scalar-native.log','type-layout.log','reference-fixed-native.log']:require('1 passed; 0 failed'in(P/name).read_text(),'native assertions '+name)
 scalar=module('exact_scalar',P/'check_scalar.py').run(P/'scalar-native.log');require(scalar==json.loads((P/'scalar-oracle.json').read_text()) and scalar['native_assertion_groups']==23 and scalar['exact_scalar_probes']==36,'actual scalar/exact oracle')
 captured=rows(P/'reference-fixed-native.log');complete=[x for x in captured if x['event']=='fixed_capture_complete'];equations=[x for x in captured if x['event']=='fixed_equation'];require([x['index']for x in complete]==[41,47] and [(x['index'],x['order'])for x in equations]==[(41,16),(41,32),(47,16),(47,32)] and all(x['mode']=='native-reference' and x['owner_advances']==0 and x['published'] is False for x in complete+equations),'native reference fixed-candidate captures without owners')
 binary=binding['binary'];require(sha(Path(binary).read_bytes())==binding['binary_sha256'],'actual linked binary');raw=subprocess.check_output(['objdump','-d','-C',binary]);require(raw==gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()) and sha(raw)==(P/'native-disassembly.txt.sha256').read_text().strip(),'actual linked instructions')
 for name,argv in [('symbols',['nm','-S','-nC',binary]),('relocations',['readelf','-rW',binary]),('headers',['readelf','-hW',binary])]:require(subprocess.check_output(argv)==(P/('native-'+name+'.txt')).read_bytes(),'actual ELF metadata')
 require((P/'memory-normal.json').read_bytes()==(P/'memory-optimized.json').read_bytes(),'memory mode parity');memory=module('fresh_memory',P/'audit_memory.py').run();require(memory==json.loads((P/'memory-normal.json').read_text()) and memory['status']=='PASS_FIXED_CANDIDATE_MEMORY_PREFLIGHT' and memory['aligned_additional_bound']==65616 and memory['remaining']==1968,'fresh actual linked memory proof')
 return {'status':'PASS_FIXED_CANDIDATE_PREFLIGHT','source':binding['prototype'],'source_tree':binding['prototype_tree'],'binary_sha256':binding['binary_sha256'],'protocol_sha256':binding['protocol_sha256'],'case_indices':[41,47],'native_scalar_groups':23,'exact_scalar_probes':36,'native_reference_equations':4,'native_accepted_owners_constructed':0,'native_accepted_owners_advanced':0,'additional_memory_bound':65616,'additional_memory_cap':67584,'remaining_memory':1968,'newton_target':1e-13,'fixed_candidate_e1_not_executed':True,'new_controller_iterations':0,'third_solve_executed':False,'parameter_search':False,'production_adoption':False}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
