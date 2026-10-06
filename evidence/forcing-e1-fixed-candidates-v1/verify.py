"""Read-only closure of two separate diagnoses; never executes a native candidate."""
from pathlib import Path
from copy import deepcopy
import hashlib,importlib.util,json,struct,subprocess,tempfile,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1];G=ROOT/'evidence/forcing-e1-signed-geometry-v1';OLD=ROOT/'evidence/forcing-e1-trajectories-v2'
BASE='4ee70e4587cb4ce5f5f164bcfea59427905e1565';TREE='c1254024e893049c7290341916350b56ca262d8c'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def load(path):return json.loads(path.read_text())
def records(path):return [json.loads(x)for x in path.read_text().splitlines()if x.startswith('{')]
def inventory():return subprocess.check_output(['git','ls-tree','-rz',BASE],cwd=ROOT)
def verify_inventory(raw):
 entries=raw.split(b'\0')[:-1];require(len(entries)==7768,'frozen predecessor inventory count')
 for entry in entries:
  metadata,name=entry.split(b'\t',1);mode,kind,oid=metadata.split();path=ROOT/name.decode();require(kind==b'blob','no hidden submodule')
  data=path.readlink().as_posix().encode()if mode==b'120000'else path.read_bytes()
  require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'predecessor bytes preserved '+name.decode())
  if mode!=b'120000':require(bool(path.stat().st_mode&0o111)==(mode==b'100755'),'predecessor file mode')
def artifact_paths():
 paths=[]
 for folder in [P,G]:
  for path in folder.iterdir():
   if not path.is_file():continue
   if folder==P and(path.name=='receipt.json'or path.name=='record_post_git.py'or path.name.startswith('root-verification-')or path.name.startswith('post-git-')):continue
   paths.append(path)
 return sorted(paths)
def same_bytes(data,digest,label):require(sha(data)==digest,label)
def validate_arithmetic(out):
 require(out['status']=='PASS_TWO_FIXED_FAILED_CANDIDATE_DIAGNOSIS'and out['cases']==[41,47]and out['complete_fixed_equations']==4 and out['native_terminal_failures_retained']==2,'full fixed-candidate outcome')
 require(out['accepted_owners_constructed']==out['accepted_owners_advanced']==out['controller_iterations']==0 and out['original_newton_target']==1e-13,'frozen owner/controller/target scope')
 require(not any(out[k]for k in ['arithmetic_floor_assumed','production_adoption','memory_cap_expanded','trajectory_remedy_established']),'no promotion to repair/floor/production')
 require([(r['index'],r['order'])for r in out['rows']]==[(41,16),(41,32),(47,16),(47,32)],'exact fixed equation roster')
 policy=load(P/'protocol.json');below={'exact_continuous_known_chart_inertia_only','fixed_operator_chart_endpoint_substitution_continuous_known'}
 for row in out['rows']:
  require(row['h']==.00078125 and row['complete_planar_equations']==22 and row['owner_advances']==row['controller_iterations']==0 and not row['third_solve_executed']and not row['published'],'full unchanged fixed planar scope')
  require(row['native_norm']>1e-13 and row['native_signed_margin']<0 and row['exact_stable_direct_defect_identity_pass'],'failures and exact stored defect retained')
  require(set(row['variants'])==set(policy['arithmetic_variants']),'all frozen variants')
  require(set(row['native_exact_term_groups'])=={'chart_increment_inertia','mass_change_inertia','direct_inertia','captured_combined_force','body_force','endpoint_donor_transport'},'whole term groups')
  for v in [*row['variants'].values(),*row['isolated_changes'].values(),*row['native_exact_term_groups'].values()]:require(len(v['rate_components'])==22,'all equation components retained')
  for name,v in row['variants'].items():
   require(not v['native_acceptance']and v['below_original_target_exactly']==(name in below),'counterfactual classification remains diagnostic')
   if 'inertia_only'in name:require('not a coherent acceptance'in v['scope'],'isolated sensitivity limitation')
   if name.startswith('fixed_operator'):require('Counterfactual endpoint'in v['scope'],'fixed operator limitation')
def validate_geometry(out):
 require(out['status']=='PASS_SIGNED_COMPONENT_DIAGNOSIS_WITH_ORIGINAL_BAND_FAILURES'and out['coordinate_labels']==['left_cap_x','left_cap_height','middle_cap_x','middle_cap_height'],'correct primary coordinate order')
 require(out['existing_complete_endpoints']==46 and out['missing_finer_endpoints']==2 and out['original_geometry_failures']==len(out['failed_checks'])==8,'endpoints and failures retained')
 require(out['failed_checks_with_dominant_mode_switch']==4 and out['failed_checks_without_mode_switch']==4 and out['failed_checks_fixed_finer_coordinate_in_band']==0,'dominance explains only part')
 require(out['new_reference_integrations']==out['new_trajectory_steps']==0 and out['reference_resolution_already_passed']and not out['thresholds_changed']and not out['remedy_established'],'existing-only geometry scope')
 require(len(out['families'])==8,'all families')
 missing=0
 for family in out['families']:
  for row in family['errors']:
   if row['status']=='COMPLETE':
    require(all(len(row[k])==4 for k in ['signed_error_components','signed_error_over_h','exact_error_over_h','paired_reference_difference_over_h']),'whole signed component vectors')
    require(row['height_pair_signed_error_sum']==row['signed_error_components'][1]+row['signed_error_components'][3],'actual paired heights')
   else:missing+=1;require(row['endpoint_missing']and 'signed_error_components'not in row,'missing endpoint remains missing')
  for check in family['checks']:require(check['original_band']==[1.7,2.3],'original band frozen')
 require(missing==2 and all(x['status']=='FAIL_ORIGINAL_BAND'and x['signed_margin']<0 and not x['fixed_coordinate_in_original_band']for x in out['failed_checks']),'failed same-coordinate bands retained')
def validate_inputs(packet):
 reader=module('frozen_debug',ROOT/'evidence/forcing-public-call-66k/analyze.py')
 require([c['index']for c in packet['cases']]==[41,47],'two actual failed states')
 for c in packet['cases']:
  rows=records(OLD/f"trajectory-{c['index']}-native.log");traces=records(OLD/f"trajectory-{c['index']}-native-stderr.log");terminal=rows[-1];attempt=next(x for x in reversed(rows)if x.get('event')=='trajectory_attempt');result=next(x for x in reversed(rows)if x.get('event')=='trajectory_result');publication=next(x for x in reversed(rows)if 'model'in x)
  require(terminal['status']=='REFUSED'and terminal['state_preserved']and terminal['same_owner_repeat']=='EXACT_REFUSAL_AND_STATE','original refusal authority')
  require(result['accepted']==attempt['before']and reader.debug(attempt['before'])==c['accepted_snapshot']and publication==c['accepted_publication'],'full accepted frozen state')
  starts=[i for i,x in enumerate(traces)if x.get('event')=='newton_check'and x['iteration']==1];last=traces[starts[-1]:];previous=traces[starts[-2]:starts[-1]]
  require(last==previous==c['original_final_trace']and c['same_owner_repeat_full_trace_exact'],'complete two original failed traces')
  correction=next(x for x in reversed(last)if x['event']=='correction');validation=next(x for x in last if x['event']=='terminal_validation')
  require(correction['iteration']==7 and correction['unknown_after']==c['final_authorized_unknowns']and validation['calls']==50 and validation['corrections']==7 and not validation['converged']and validation['rate_norm']==c['expected_terminal_norm'],'final authorized seventh correction and exact terminal norm')
  require(c['q']==last[0]['q']==publication['end_q']and c['eta']==last[0]['eta']==publication['end_eta']and c['h']==last[0]['h']==.00078125,'actual accepted coordinates/knowns/time step')
def reject(label,validator,value,mutate):
 candidate=deepcopy(value);mutate(candidate)
 try:validator(candidate)
 except(ValueError,KeyError,IndexError):return label
 raise ValueError('negative control escaped: '+label)
def run():
 receipt=load(P/'receipt.json');raw=inventory();require(subprocess.check_output(['git','rev-parse',BASE+'^{tree}'],cwd=ROOT,text=True).strip()==TREE and sha(raw)==receipt['predecessor_inventory_sha256'],'exact predecessor source tree');verify_inventory(raw)
 paths=artifact_paths();require(set(receipt['artifacts'])=={str(p.relative_to(ROOT))for p in paths},'closed complete artifact roster')
 for path in paths:same_bytes(path.read_bytes(),receipt['artifacts'][str(path.relative_to(ROOT))],'frozen artifact '+path.name)
 for folder,name in [(P,'preflight-receipt.json'),(G,'receipt.json')]:
  for name,digest in load(folder/name)['artifacts'].items():same_bytes((folder/name).read_bytes(),digest,'original separately frozen receipt')
 preflight=module('fresh_preflight',P/'qualify.py').run();require(preflight==load(P/'preflight-receipt.json')['qualification'],'fresh native linked preflight remains valid')
 auth=load(P/'execution-authorization.json');completed=load(P/'capture-command-completed.json');require(all(completed[k]==v for k,v in auth.items()),'actual exact authorized capture');require(completed['exit']==0 and completed['candidate_equations_requested']==4 and completed['accepted_owners_constructed']==completed['accepted_owners_advanced']==completed['controller_iterations']==0 and not completed['published'],'actual capture scope')
 require(completed['argv']==[load(P/'binary-binding.json')['binary'],'--exact','research_public_call::fixed_capture_e1_diagnostic','--nocapture']and completed['env']=={'RHEON_FIXED_CAPTURE_ALLOW':'E1-two-terminal-fixed-candidates-v1'},'exact diagnostic gate')
 for key,path in [('binary_sha256',Path(completed['argv'][0])),('stdout_sha256',P/'capture-native.log'),('stderr_sha256',P/'capture-native-stderr.log'),('runner_sha256',P/'run_capture.py'),('protocol_sha256',P/'protocol.json'),('preflight_receipt_sha256',P/'preflight-receipt.json')]:same_bytes(path.read_bytes(),completed[key],'actual capture binding '+key)
 packet=load(P/'inputs.json');validate_inputs(packet);arithmetic=module('fresh_arithmetic',P/'analyze.py').run(80);require(arithmetic==load(P/'analysis-80-normal.json'),'fresh complete exact arithmetic replay');validate_arithmetic(arithmetic)
 commands=load(P/'analysis-commands.json');require([(x['precision_digits'],x['optimized'])for x in commands]==[(80,False),(80,True),(120,False),(120,True)],'actual reader command roster')
 for command in commands:
  require(command['exit']==0 and command['accepted_owner_advances']==0 and command['reader_sha256']==sha((P/'analyze.py').read_bytes())and command['capture_sha256']==sha((P/'capture-native.log').read_bytes()),'actual arithmetic executions')
  for channel in ['stdout','stderr']:same_bytes((P/command[channel]).read_bytes(),command[channel+'_sha256'],'actual reader log')
  require(not(P/command['stderr']).read_bytes(),'no hidden reader failure');output=load(P/command['stdout']);require(output['precision_digits']==command['precision_digits'],'precision display binding');output.pop('precision_digits');base=deepcopy(arithmetic);base.pop('precision_digits');require(output==base,'normal/optimized and 80/120 outcome parity')
 geometry=module('fresh_geometry',G/'analyze_v3.py').run();require(geometry==load(G/'outcome-v3-normal.json')and(G/'outcome-v3-normal.json').read_bytes()==(G/'outcome-v3-optimized.json').read_bytes(),'fresh existing endpoint replay and mode parity');validate_geometry(geometry)
 require(not(G/'outcome-normal.json').read_bytes()and b'not JSON serializable'in(G/'outcome-normal-stderr.log').read_bytes(),'original failed JSON attempt retained')
 # Rendering is an external-storage view of existing results, not a solver call.
 with tempfile.TemporaryDirectory(prefix='rheon-fixed-render-')as temporary:
  for folder,name in [(P,'arithmetic.svg'),(G,'signed-components.svg')]:
   output=Path(temporary)/name;subprocess.run([sys.executable,str(folder/'render.py'),str(output)],cwd=ROOT,check=True,capture_output=True);require(output.read_bytes()==(folder/name).read_bytes(),'deterministic archived render')
 controls=[]
 for label,key,value in [('owner advanced','accepted_owners_advanced',1),('extra iteration','controller_iterations',1),('relaxed Newton threshold','original_newton_target',2e-13),('floor asserted','arithmetic_floor_assumed',True),('production adoption','production_adoption',True),('memory expansion','memory_cap_expanded',True),('diagnosis promoted to remedy','trajectory_remedy_established',True)]:controls.append(reject(label,validate_arithmetic,arithmetic,lambda x,k=key,v=value:x.__setitem__(k,v)))
 controls.append(reject('invented third solve',validate_arithmetic,arithmetic,lambda x:x['rows'][0].__setitem__('third_solve_executed',True)))
 controls.append(reject('missing force term',validate_arithmetic,arithmetic,lambda x:x['rows'][0]['native_exact_term_groups'].pop('captured_combined_force')))
 controls.append(reject('missing equation component',validate_arithmetic,arithmetic,lambda x:x['rows'][0]['variants']['exact_products_sums_stored_stable']['rate_components'].pop()))
 controls.append(reject('sensitivity promoted to native acceptance',validate_arithmetic,arithmetic,lambda x:x['rows'][0]['variants']['exact_continuous_known_chart_inertia_only'].__setitem__('native_acceptance',True)))
 controls.append(reject('wrong final unknown',validate_inputs,packet,lambda x:x['cases'][0]['final_authorized_unknowns'].__setitem__(0,__import__('math').nextafter(x['cases'][0]['final_authorized_unknowns'][0],float('inf')))))
 controls.append(reject('changed accepted state bit',validate_inputs,packet,lambda x:x['cases'][0]['accepted_snapshot']['mass'].__setitem__(0,x['cases'][0]['accepted_snapshot']['mass'][0]+1)))
 for label,key,value in [('new reference integration','new_reference_integrations',1),('new trajectory step','new_trajectory_steps',1),('hidden geometry failure','original_geometry_failures',7),('swapped physical labels','coordinate_labels',['left_cap_x','middle_cap_x','left_cap_height','middle_cap_height']),('invented missing endpoint','missing_finer_endpoints',1),('altered geometry threshold','thresholds_changed',True),('geometry diagnosis promoted to remedy','remedy_established',True)]:controls.append(reject(label,validate_geometry,geometry,lambda x,k=key,v=value:x.__setitem__(k,v)))
 controls.append(reject('relaxed geometry band',validate_geometry,geometry,lambda x:x['families'][0]['checks'][0].__setitem__('original_band',[1.4,2.7])))
 for name,data,digest in [('omitted original reader failure',b'',sha((G/'outcome-normal-stderr.log').read_bytes())),('changed native capture',b'changed',sha((P/'capture-native.log').read_bytes())),('changed known rounding error',b'changed',sha((P/'analysis-80-normal.json').read_bytes()))]:
  try:same_bytes(data,digest,name)
  except ValueError:controls.append(name)
  else:raise ValueError('negative hash control escaped')
 return {'status':'PASS_SEPARATE_FIXED_CANDIDATE_AND_EXISTING_GEOMETRY_DIAGNOSES','predecessor':BASE,'predecessor_tree':TREE,'preserved_predecessor_files':7768,'receipt_sha256':sha((P/'receipt.json').read_bytes()),'native_source':completed['source'],'native_binary_sha256':completed['binary_sha256'],'complete_fixed_equations':4,'terminal_native_failures_retained':2,'geometry_band_failures_retained':8,'geometry_failures_with_switch':4,'geometry_failures_without_switch':4,'same_coordinate_failures_restored':0,'accepted_owners_constructed':0,'accepted_owners_advanced':0,'new_controller_iterations':0,'new_reference_integrations':0,'new_trajectory_steps':0,'third_solve_executed':False,'production_adoption':False,'memory_cap_expanded':False,'trajectory_remedy_established':False,'arithmetic_floor_assumed':False,'additional_memory_bound':65616,'additional_memory_cap':67584,'remaining_memory':1968,'negative_controls_rejected':controls,'arithmetic_precision_and_mode_parity':True,'geometry_mode_parity':True,'native_reexecution_in_this_verifier':False,'deterministic_renders':True}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
