"""Bind actual new binary, scalar, five native baselines and memory preflight."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,re,subprocess,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def rows(path):return [json.loads(l) for l in path.read_text().splitlines() if l.startswith('{')]
def run():
 policy=json.loads((P/'protocol.json').read_text());binding=json.loads((P/'binary-binding.json').read_text());commands=json.loads((P/'preflight-commands.json').read_text())
 require(binding['prototype']==commands['source']=='050c0a86d71a32e0e9392bd37d1d74045d291b5a' and binding['prototype_tree']==commands['source_tree']=='3942fa171b8c97ff890734540283e970d532fa51','frozen new source identity')
 for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen numerical source '+path)
 require(binding['protocol_sha256']==sha((P/'protocol.json').read_bytes()) and binding['target']==policy['target'] and binding['rustflags']==policy['rustflags'],'frozen build protocol')
 require(binding['generalization_executed'] is False and commands['generalization_executed'] is False,'preflight did not execute E1')
 require([Path(c['stdout']).stem for c in commands['commands']]==['format','install','build','clippy','scalar-native','type-layout','baseline-native','constructor-native','restore'],'complete actual command roster')
 for c in commands['commands']:
  require(c['exit']==0,'actual native preflight pass')
  for k in ['stdout','stderr']:require(sha((P/c[k]).read_bytes())==c[k+'_sha256'],'actual command log '+c[k])
 scalar=module('scalar_preflight',P/'check_scalar.py').run(P/'scalar-native.log');require(scalar==json.loads((P/'scalar-oracle.json').read_text()) and scalar['native_assertion_groups']==23 and scalar['exact_scalar_probes']==36,'new native scalar/exact oracle')
 for name in ['scalar-native.log','type-layout.log','baseline-native.log','constructor-native.log']:require('1 passed; 0 failed' in (P/name).read_text(),'native Rust assertions '+name)
 baseline=rows(P/'baseline-native.log');results=[r for r in baseline if r['event']=='baseline_conformance'];require(len(results)==5 and [r['case'] for r in results]==list(range(5)),'all prescribed baselines')
 require(all(r['native_result']==r['disabled_result']=='Err(IterationLimit)' and r['generalization_executed'] is False for r in results),'five original refusals reproduced')
 reader=module('old_debug_reader',ROOT/'evidence/forcing-public-call-66k/analyze.py');cases=json.loads((P/'cases.json').read_text())
 for path,digest in cases['input_sources_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'original preserved input source')
 prefixes=[r for r in baseline if r['event']=='verified_prefix'];require(len(prefixes)==5,'all baseline prefix controls')
 require(all(reader.debug(prefixes[i]['accepted'])==cases['cases'][i]['accepted_snapshot'] for i in range(5)),'all exact preserved accepted prefixes')
 original=rows(ROOT/'evidence/solver-terminal-validation/trace-finer-stderr.log');actual=rows(P/'baseline-native-stderr.log')
 terminal=lambda data:[r for r in data if r.get('event')=='terminal_validation' and not r['converged']]
 require(terminal(actual)==terminal(original) and len(terminal(actual))==10,'native and disabled exact failing terminal records for all five calls')
 # Compare every original Newton check/correction in the failing windows.
 def failing_paths(data):
  current=[];answer=[]
  for r in data:
   if r.get('event')=='newton_check' and r['iteration']==1:current=[]
   if r.get('event') in ['newton_check','correction','terminal_validation']:current.append(r)
   if r.get('event')=='terminal_validation' and not r['converged']:answer.append(current.copy())
  return answer
 require(failing_paths(actual)==failing_paths(original),'all exact original controller refusal windows reproduced')
 roster=json.loads((P/'roster.json').read_text())
 for path,digest in roster['original_sources_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'original complete protocol source')
 constructors=[x for x in rows(P/'constructor-native.log') if x['event']=='constructor_conformance'];require([x['input']for x in constructors]==list(range(4)),'all direct constructor fixtures')
 import struct
 def bits(x):return struct.unpack('<Q',struct.pack('<d',x))[0]
 for x,c in zip(constructors,roster['constructor_inputs']):
  a=c['publication'];expected={'velocity':[[bits(q)for q in v]for v in a['velocity']],'positions':[[bits(q)for q in v]for v in a['positions']],'mass':[bits(q)for q in a['mass']],'pressure':[bits(q)for q in a['pressure_coefficients']],'triangles':a['triangles'],'periodic':a['periodic_indices'],'time':bits(a['time']),'stamp':a['stamp']}
  require(reader.debug(x['accepted'])==expected and x['candidate_without_chart_bytes']==474776,'exact original constructor state and descriptor charge')
 require(len(roster['trajectories'])==48 and sum(x['steps']for x in roster['trajectories'])==1264 and len(roster['lifecycle_controls'])==8,'complete original roster and controls')
 binary=binding['binary'];require(sha(Path(binary).read_bytes())==binding['binary_sha256'],'actual new binary identity')
 raw=subprocess.check_output(['objdump','-d','-C',binary]);require(raw==gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()) and sha(raw)==(P/'native-disassembly.txt.sha256').read_text().strip(),'actual linked instructions')
 for name,argv in [('symbols',['nm','-S','-nC',binary]),('relocations',['readelf','-rW',binary]),('headers',['readelf','-hW',binary])]:require(subprocess.check_output(argv)==(P/('native-'+name+'.txt')).read_bytes(),'actual ELF metadata '+name)
 require((P/'memory-closed-normal.json').read_bytes()==(P/'memory-closed-optimized.json').read_bytes(),'memory normal/optimized parity')
 memory=module('actual_generalization_memory',P/'audit_memory_closed.py').run();require(memory==json.loads((P/'memory-closed-normal.json').read_text()),'fresh kernel/caller memory replay')
 require(memory['status']=='PASS_REPEATED_TRAJECTORY_MEMORY_PREFLIGHT' and memory['cap']==67584 and memory['aligned_additional_bound']==65536 and memory['kernel_peak']==1536,'new binary within new authorized allowance')
 require(memory['matched_frame_deltas']['public_call']['positive_delta']==16 and memory['matched_frame_deltas']['Work::point']['positive_delta']==1872,'changed actual caller frames included')
 require(memory['no_baseline_slack_credit'] is True and memory['no_shrink_or_replaced_solver_credit'] is True and memory['old_frozen_comparison_certificate_reused'] is False,'independent conservative additional accounting')
 symbols=(P/'native-symbols.txt').read_text();fixture=re.search(r'^[a-f0-9]+ ([a-f0-9]+) [a-zA-Z] rheon::research_public_call::CASES$',symbols,re.M);require(fixture is not None,'actual immutable fixture symbol')
 return {'status':'PASS_NEW_REPEATED_TRAJECTORY_PREFLIGHT','source':binding['prototype'],'source_tree':binding['prototype_tree'],'protocol_sha256':binding['protocol_sha256'],'binary_sha256':binding['binary_sha256'],'native_scalar_groups':23,'exact_scalar_probes':36,'native_baseline_refusals':5,'chart_disabled_refusals':5,'complete_original_refusal_windows_identical':True,'exact_accepted_prefixes':5,'additional_bound':65536,'authorized_additional_cap':67584,'remaining':2048,'kernel_peak':1536,'point_delta':1872,'public_call_delta':16,'constructor_added_stack_peak':memory['constructor_added_stack_peak'],'constructor_shell_frames':memory['constructor_shell_frames'],'constructor_shell_positive_delta':memory['constructor_shell_positive_delta'],'closed_memory_reader_sha256':sha((P/'audit_memory_closed.py').read_bytes()),'first_memory_reader_failure_preserved':True,'direct_constructor_fixtures_bit_exact':4,'original_trajectories':48,'lifecycle_runs':8,'trajectory_execution_completed':False,'new_single_owner_reservation':542352,'maximum_two_native_owner_reservations':949536,'fixed_snapshot_bytes':1696,'fixed_control_bytes':120,'immutable_five_case_fixture_symbol_bytes':int(fixture[1],16),'fixture_and_diagnostic_scope':'bounded immutable test inputs shared by both controllers and fixed host observation snapshots; not hidden numerical chart workspace or a process-wide memory cap','old_one_step_bound':65552,'old_one_step_64KiB_protocol':'still failed; not relabeled by this different binary','controller_count_limit':200,'correction_limit':7,'newton_target':1e-13,'physical_momentum_limit':1e-11,'final_success_acceptance_equations':2,'parameter_search':False,'continued_trajectory':False,'production_adoption':False,'generalization_executed':False}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
