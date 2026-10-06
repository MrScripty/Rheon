"""Read-only closed E2 evidence verifier. Never reruns any native equation."""
from pathlib import Path
from copy import deepcopy
import hashlib,importlib.util,json,subprocess,sys,tempfile
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='c2ee6acc8a783161a844e3dee0566af71732628d';TREE='a59508d87e97e2fb97f5d52d34d187924bbf0d92'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def load(path):return json.loads(path.read_text())
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def inventory():return subprocess.check_output(['git','ls-tree','-rz',BASE],cwd=ROOT)
def validate_inventory(raw):
 entries=raw.split(b'\0')[:-1];require(len(entries)==7885,'exact frozen source/preflight tree inventory')
 for entry in entries:
  meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();path=ROOT/name.decode();require(kind==b'blob','no hidden submodule');data=path.readlink().as_posix().encode()if mode==b'120000'else path.read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'all prior source/evidence bytes preserved '+name.decode())
  if mode!=b'120000':require(bool(path.stat().st_mode&0o111)==(mode==b'100755'),'prior file mode unchanged')
def artifacts():return sorted(f for f in P.iterdir()if f.is_file()and f.name not in ['receipt.json','record_post_git.py']and not f.name.startswith(('verification-','post-git-')))
def valid(out):
 require(out['status']=='PASS_AUTHORIZED_FIXED_EXPERIMENT_REPLAY_ONLY'and out['native_equations_requested']==out['native_equations_evaluated']==4 and out['native_equation_refusals']==0,'exact actual fixed equation scope')
 require(out['original_E1_failures_reproduced']==out['original_runtime_refusals_unresolved']==2 and out['original_geometry_band_failures_unresolved']==8,'original runtime and geometry failures retained')
 require(out['new_controller_iterations']==out['accepted_owners_constructed']==out['accepted_owners_advanced']==out['new_trajectory_steps']==out['new_reference_integrations']==0 and not any(out[k]for k in ['third_solve_executed','published','production_adoption','trajectory_remedy_established','arithmetic_floor_claimed']),'fixed scope without acceptance or floor promotion')
 require([(x['index'],x['order'])for x in out['rows']]==[(41,16),(41,32),(47,16),(47,32)]and out['native_newton_eligible_equations']==2,'prescribed actual roster')
 for r in out['rows']:
  require(r['native_newton_target']==1e-13 and r['native_newton_eligible']==(r['index']==47)and r['native_newton_eligible']==(r['native_norm']<=1e-13),'native threshold and actual eligibility')
  require(r['h']==.00078125 and not r['native_acceptance']and not r['published']and not r['third_solve_executed'],'no latent or public acceptance')
  require(all(r[k]for k in ['exact_rho_reconciliation_identity','exact_stored_embedding_defect_identity','public_known_and_geometry_bit_policy_preserved','native_stored_inertia_replayed','fresh_native_force_transfer_replayed']),'all complete independent stored-field comparisons')
  require(len(r['native_rate_components'])==len(r['native_direct_components'])==len(r['rho_exact_components'])==22,'whole momentum and reconciliation vectors')
  require(r['bit_changes_vs_original_E1']['end_q']==r['bit_changes_vs_original_E1']['end_eta']==r['bit_changes_vs_original_E1']['end_mass']==0 and r['bit_changes_vs_original_E1']['end_z']==12,'unchanged native public knowns/geometry with explicit changed dependents')
  require(r['exact_stored_stable']['below_original_target_exactly']==(r['index']==47)and r['rho_projected_over_h']['norm']>0,'exact stored threshold classification and nonzero reconciliation')
  for key in ['exact_stored_stable','exact_stored_direct','rho_projected_over_h','latent_inertia_only_counterfactual']:require(not r[key]['native_acceptance']and len(r[key]['rate_components'])==22,'diagnostic norm is not native acceptance')
  require('not an acceptance candidate'in r['latent_inertia_only_counterfactual']['scope'],'latent sensitivity explicitly excluded')
  require(set(r['term_groups'])=={'chart_increment_inertia','mass_change_inertia','direct_inertia','captured_combined_force','body_force','endpoint_donor_transport'}and all(len(g['components'])==22 for g in r['term_groups'].values()),'all original projected term groups')
 require([g['index']for g in out['planar_qualifiers']]==[41,47]and all(g['passed']and not g['third_solve_executed']and not g['public_step_completed']for g in out['planar_qualifiers']),'actual planar eligibility is not public completion')
def reject(label,out,mutate):
 bad=deepcopy(out);mutate(bad)
 try:valid(bad)
 except(ValueError,KeyError):return label
 raise ValueError('corruption control escaped '+label)
def run():
 receipt=load(P/'receipt.json');raw=inventory();require(sha(raw)==receipt['predecessor_inventory_sha256']and subprocess.check_output(['git','rev-parse',BASE+'^{tree}'],cwd=ROOT,text=True).strip()==TREE,'actual immutable preflight tree');validate_inventory(raw)
 require(set(receipt['artifacts'])=={f.name for f in artifacts()},'closed result artifact roster')
 for f in artifacts():require(sha(f.read_bytes())==receipt['artifacts'][f.name],'result artifact hash '+f.name)
 preflight=load(P/'preflight-receipt.json')
 for name,digest in preflight['artifacts'].items():require(sha((P/name).read_bytes())==digest,'frozen pre-execution artifact')
 require(module('fresh_qualification',P/'qualify.py').run()==preflight['qualification'],'fresh same actual ELF/scalar/memory qualification')
 auth=load(P/'execution-authorization.json');binding=load(P/'binary-binding.json');require(auth['source']==binding['prototype']and auth['source_tree']==binding['prototype_tree']and auth['E2_binary_sha256']==binding['binary_sha256']and auth['E2_fixed_equations_requested']==4 and auth['owner_advances']==auth['controller_iterations']==0 and auth['review_disposition']=='ACK_DIAGNOSTICS_ONLY'and not auth['trajectory_reruns']and not auth['new_references'],'exact authorized native execution')
 require(auth['protocol_sha256']==sha((P/'protocol.json').read_bytes())and auth['preflight_receipt_sha256']==sha((P/'preflight-receipt.json').read_bytes())and auth['runner_sha256']==sha((P/'run_fixed_v2.py').read_bytes()),'actual frozen authorization and runner')
 for label,test,gate,binary in [('baseline-reproduced','fixed_capture_e1_diagnostic','RHEON_FIXED_CAPTURE_ALLOW',load(P/'protocol.json')['baseline_binary']),('candidate-fixed','fixed_capture_e2_experiment','RHEON_E2_FIXED_CAPTURE_ALLOW',binding['binary'])]:
  command=load(P/(label+'-command-completed.json'));require(command['exit']==0 and command['argv']==[binary,'--exact','research_public_call::'+test,'--nocapture']and command['accepted_owners_constructed']==command['accepted_owners_advanced']==command['controller_iterations']==0 and not command['third_solve_executed']and not command['published'],'actual exact native command '+label)
  expected='E1-two-terminal-fixed-candidates-v1'if label=='baseline-reproduced'else'E2-two-fixed-candidates-native-known-v1';require(command['env']=={gate:expected},'exact environment gate')
  for channel,suffix in [('stdout','.log'),('stderr','-stderr.log')]:require(sha((P/(label+suffix)).read_bytes())==command[channel+'_sha256'],'actual command journals')
  require(not(P/(label+'-stderr.log')).read_bytes()and '1 passed; 0 failed'in(P/(label+'.log')).read_text(),'actual assertions/no hidden controller')
 out=module('fresh_E2_equation',P/'analyze.py').run(80);require(out==load(P/'analysis-80-normal.json'),'fresh full stored-field and rho replay');valid(out)
 commands=load(P/'analysis-commands.json');require([(x['precision_digits'],x['optimized'])for x in commands]==[(80,False),(80,True),(120,False),(120,True)],'four actual arithmetic readers')
 compare=deepcopy(out);compare.pop('precision_digits')
 for c in commands:
  require(c['exit']==0 and c['native_execution']is False and c['reader_sha256']==sha((P/'analyze.py').read_bytes())and c['capture_sha256']==sha((P/'candidate-fixed.log').read_bytes()),'actual analysis executions and source')
  for channel in ['stdout','stderr']:require(sha((P/c[channel]).read_bytes())==c[channel+'_sha256'],'actual analysis journal hash')
  require(not(P/c['stderr']).read_bytes(),'no hidden analysis failure');candidate=load(P/c['stdout']);require(candidate.pop('precision_digits')==c['precision_digits']and candidate==compare,'80/120 and normal/optimized complete outcome parity')
 work=module('fresh_nodal_work',P/'work_identity.py').run();require(work==load(P/'work-normal.json')and(P/'work-normal.json').read_bytes()==(P/'work-optimized.json').read_bytes()and not(P/'work-normal-stderr.log').read_bytes()and not(P/'work-optimized-stderr.log').read_bytes(),'fresh four independent nodal work/projection/embedding identities')
 with tempfile.TemporaryDirectory(prefix='rheon-e2-render-')as tmp:
  for script,name in [('render.py','fixed-equations.svg'),('render_v2.py','fixed-equations-v2.svg')]:
   target=Path(tmp)/name;subprocess.run([sys.executable,str(P/script),str(target)],cwd=ROOT,check=True,capture_output=True);require(target.read_bytes()==(P/name).read_bytes(),'deterministic presentation-only render')
 controls=[]
 for label,key,value in [('owner advanced','accepted_owners_advanced',1),('new controller iteration','new_controller_iterations',1),('new trajectory','new_trajectory_steps',1),('new reference','new_reference_integrations',1),('invented third solve','third_solve_executed',True),('publication invented','published',True),('production adoption','production_adoption',True),('trajectory remedy claimed','trajectory_remedy_established',True),('floor claimed','arithmetic_floor_claimed',True),('hidden original refusal','original_runtime_refusals_unresolved',1),('hidden geometry failure','original_geometry_band_failures_unresolved',7)]:controls.append(reject(label,out,lambda x,k=key,v=value:x.__setitem__(k,v)))
 controls.append(reject('relaxed native target',out,lambda x:x['rows'][0].__setitem__('native_newton_target',2e-13)))
 controls.append(reject('case41 refusal promoted',out,lambda x:x['rows'][0].__setitem__('native_newton_eligible',True)))
 controls.append(reject('latent norm supplies acceptance',out,lambda x:x['rows'][0]['latent_inertia_only_counterfactual'].__setitem__('native_acceptance',True)))
 controls.append(reject('rho omitted',out,lambda x:x['rows'][0].__setitem__('exact_rho_reconciliation_identity',False)))
 controls.append(reject('zero fake reconciliation',out,lambda x:x['rows'][0]['rho_projected_over_h'].__setitem__('norm',0)))
 controls.append(reject('public knowns changed',out,lambda x:x['rows'][0].__setitem__('public_known_and_geometry_bit_policy_preserved',False)))
 controls.append(reject('captured old transfers reused',out,lambda x:x['rows'][0].__setitem__('fresh_native_force_transfer_replayed',False)))
 controls.append(reject('missing force term',out,lambda x:x['rows'][0]['term_groups'].pop('captured_combined_force')))
 controls.append(reject('missing momentum component',out,lambda x:x['rows'][0]['native_rate_components'].pop()))
 controls.append(reject('planar eligibility promoted to full step',out,lambda x:x['planar_qualifiers'][1].__setitem__('public_step_completed',True)))
 return {'status':'PASS_E2_FIXED_EXPERIMENT_EVIDENCE_WITH_ONE_CASE_STILL_ABOVE_TARGET','predecessor':BASE,'predecessor_tree':TREE,'preserved_predecessor_files':7885,'receipt_sha256':sha((P/'receipt.json').read_bytes()),'native_source':binding['prototype'],'native_source_tree':binding['prototype_tree'],'native_binary_sha256':binding['binary_sha256'],'original_E1_numerical_bytes_reproduced':True,'E2_equations':4,'native_Newton_eligible_equations':2,'case41_still_above_target_at_both_orders':True,'case47_below_target_at_both_orders':True,'actual_planar_qualifiers_passed':2,'rho_accounted_for':True,'public_known_and_native_geometry_policy_unchanged':True,'native_stored_inertia_unchanged_and_replayed':True,'fresh_force_and_transfer_replay':True,'independent_nodal_work_and_projection_identities':4,'negative_controls_rejected':controls,'additional_memory_bound':65840,'additional_memory_cap':67584,'remaining_memory':1744,'accepted_owners_constructed_or_advanced':0,'controller_iterations':0,'new_trajectories':0,'new_reference_integrations':0,'third_solve_executed':False,'published':False,'production_adoption':False,'original_runtime_refusals_unresolved':2,'original_geometry_band_failures_unresolved':8,'trajectory_remedy_established':False,'native_execution_in_this_verifier':False}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
