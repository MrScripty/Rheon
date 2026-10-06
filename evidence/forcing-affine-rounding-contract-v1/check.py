"""Design-only policy and exact toy algebra checks; no Rheon numerical imports."""
from pathlib import Path
from fractions import Fraction as Q
from copy import deepcopy
import hashlib,json,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='b46d26c8fe06f9c331c5cee4e06af6e34fa9190c';TREE='96045fb873ad9f1e00134a46e6e3d6efcda69567'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def validate(p):
 require(p['protocol_id']=='E2-deferred-working-affine-known-stored-endpoint-contract-v1'and p['status']=='DESIGN_ONLY_BLOCKED_PENDING_DIAGNOSTIC_REVIEW','fixed contract identity')
 require(p['base']==BASE and p['base_tree']==TREE and p['base_tree_files']==7803,'exact inherited source')
 review=p['diagnostic_review'];require(review['disposition']is None and not review['execution_enabled']and review['reviewed_source']==BASE and review['reviewed_tree']==TREE,'unavailable review disposition; execution remains blocked')
 boundary=p['semantic_boundary'];require(boundary['reconciliation']=='derive native stored endpoint differences, never use an unrounded solve increment for qualified inertia'and boundary['stored_known_eta_consistent'],'stored endpoint reconciliation')
 require(not any(boundary[x]for x in ['unrounded_endpoint_qualifies','latent_increment_in_qualified_inertia','accepted_initial_projection','retained_tails','new_state_owner','new_clock']),'no latent authority or additional state')
 c=p['construction'];require(c['only_change']=='working affine known targets before existing chart factorization'and c['scalar_significand_bits']==106 and c['scalar_slot_bytes']==32 and c['accepted_constraint_defect_retained'],'minimal existing chart policy')
 require(c['native_inertia_graph']=='unchanged stored binary64 difference and matvec; unrounded working increment excluded'and c['native_accumulation_and_norm']=='unchanged existing E1 graph'and not c['exact_oracle_can_replace_gate']and not c['full_higher_precision_force_or_momentum_evaluator'],'unchanged downstream residual arithmetic')
 r=p['resources'];require(r['reuse_existing_workspace_loan']and r['scalar_slots']==1934 and r['scalar_slot_bytes']==32 if 'scalar_slot_bytes'in r else r['reuse_existing_workspace_loan']and r['scalar_slots']==1934,'existing loan and slots')
 require(r['scalar_storage_bytes']==61888 and r['existing_workspace_bytes']==62096 and r['existing_descriptor_bytes']==8 and r['additional_live_cap_bytes']==67584,'no memory expansion')
 require(not any(r[x]for x in ['new_high_precision_array','new_accepted_snapshot','high_precision_endpoint_copy_live_during_quadrature'])and r['initialize_before_each_read']and r['native_streaming_only']and r['fresh_actual_elf_live_memory_audit_required']and r['old_memory_bound_is_not_E2_qualification'],'fresh complete live-memory proof required')
 gates=p['gates'];expected={'newton_target':1e-13,'physical_momentum_and_full_constraints':1e-11,'quadrature_limit':1e-15,'work_gcl_epsilon_factor':128,'correction_limit':7,'counted_equation_limit':200,'final_acceptance_equations_not_counted':2,'geometry_ratio_band':[1.7,2.3],'full_velocity_ratio_band':[1.8,2.2],'nonconstant_third_ratio_band':[1.8,2.2],'preserve_original_native_budget_graph':True,'threshold_changes':False,'parameter_search':False,'third_solver_unchanged':True};require(gates==expected,'all original numerical gates and budget')
 fixed=p['conditional_fixed_comparison'];require([x['index']for x in fixed['cases']]==[41,47]and fixed['orders']==[16,32]and fixed['new_fixed_equations']==4,'fixed conditional roster')
 require(fixed['accepted_owners_constructed']==fixed['accepted_owners_advanced']==fixed['controller_corrections']==0 and not fixed['third_solve']and not fixed['publication']and fixed['comparison_roster']==['C0_frozen_E1','C1_exact_same_stored_fields','C2_E2_changed_stored_fields'],'no fixed-comparison trajectory or acceptance')
 m=p['measurements'];require(m['full_planar_components']==22 and m['precision_digits']==[80,120]and m['python_modes']==['normal','optimized']and 'rho'in m['endpoint']and 'positive_negative_transfers'in m['path'],'whole independent equation and rounding/path diagnostics')
 require(m['independent_chart_targets']==['exact_continuous_affine_knowns','actual_captured_H_knowns','rounded_stored_knowns'],'separate continuous/H/stored chart comparisons')
 require(set(m['groups'])=={'stored_chart_inertia','direct_nodal_inertia','mass_change','strain_force','pressure_force','body_force','endpoint_donor_transport','physical_path_convection'},'all term groups')
 q=p['qualification_boundary'];require(q['native_refusals_retained']==2 and q['original_geometry_band_failures_retained']==8 and q['geometry_metric']=='unchanged original maximum error at actual common-final-time endpoint'and q['signed_geometry_components_supplement_only']and q['missing_endpoints_not_fabricated']and q['public_prefix_change_is_different_trajectory']and q['no_arithmetic_floor_claim']and q['no_success_prediction'],'failures, metric, state and proof scope retained')
 require(p['executed_in_this_packet']=={'implementation':False,'native_candidate_equations':0,'native_owner_constructions':0,'owner_advances':0,'new_trajectories':0,'new_reference_integrations':0,'production_adoption':False},'no implementation or native execution')
 require(len(p['execution_preconditions'])==6 and p['execution_preconditions'][0].startswith('coordinator-recorded independent diagnostic review disposition'),'review before implementation/execution milestones')
def algebra():
 # A scalar counterexample, not a Rheon candidate or integration.
 h=Q(1,2**54);old=Q(1);latent=old+h;stored=Q(float(latent));dl=latent-old;ds=stored-old;rho=ds-dl
 latent_rate=dl/h-1;stored_rate=ds/h-1
 require(stored==old and ds==dl+rho and latent_rate==0 and stored_rate==-1 and stored_rate-latent_rate==rho/h,'stored reconciliation changes the qualified residual')
 # Binary operands, 105-bit exact product representable in the proposed H.
 s=Q(1)+Q(1,2**52);alpha=Q(1)-Q(1,2**52);eta=Q(-1);product=s*alpha;working=eta+product;native_product=Q(float(float(s)*float(alpha)));native_affine=Q(float(float(eta)+float(native_product)));deferred_store=Q(float(working))
 require(product.numerator.bit_length()<=106 and working==-Q(1,2**104) and native_product==1 and native_affine==0 and deferred_store==working and deferred_store!=native_affine,'same parameters can produce different stored known fields')
 # Independent absolute and increment formulas retain an off-chart start defect.
 z0=[Q(1),-Q(1,2)+Q(1,2**52)];d0=[Q(1),Q(2)];d1=[Q(3),Q(5)];known=Q(1)+h;dk=known-z0[0];defect0=sum(a*b for a,b in zip(d0,z0));rhs=-sum(a*b for a,b in zip(d1,z0))-d1[0]*dk;du=rhs/d1[1];absolute=-d1[0]*known/d1[1]
 require(z0[1]+du==absolute and d1[0]*known+d1[1]*absolute==0,'same targets exact absolute/increment re-expression')
 dropped_rhs=-sum((a-b)*v for a,b,v in zip(d1,d0,z0))-d1[0]*dk;dropped=z0[1]+dropped_rhs/d1[1];require(d1[0]*known+d1[1]*dropped==defect0!=0,'initial defect cannot be dropped')
 return {'lost_increment':{'h':str(h),'stored_increment':str(ds),'latent_increment':str(dl),'rho':str(rho),'latent_rate':str(latent_rate),'stored_rate':str(stored_rate),'units':'increments velocity; rho/h and residual rate acceleration in this unit-mass scalar example','native_acceptance':False},'changed_affine_rounding':{'native_stored_known':str(native_affine),'deferred_stored_known':str(deferred_store),'same_binary_parameters':True,'same_stored_fields':False,'H_product_representable_in_106_bits':True,'scope':'analytic representable example; no scalar implementation or Rheon candidate'},'moving_constraint':{'absolute_endpoint':str(absolute),'increment_endpoint':str(z0[1]+du),'accepted_constraint_defect':str(defect0),'dropped_defect_endpoint_residual':str(d1[0]*known+d1[1]*dropped),'same_known_target_and_D':True},'latent_residual_cannot_qualify_stored_endpoint':True,'no_Rheon_candidate_or_integration':True}
def reject(label,p,path,value):
 q=deepcopy(p);target=q
 for key in path[:-1]:target=target[key]
 target[path[-1]]=value
 try:validate(q)
 except ValueError:return label
 raise ValueError('negative policy control escaped '+label)
def run():
 policy=json.loads((P/'policy.json').read_text());validate(policy)
 raw=subprocess.check_output(['git','ls-tree','-rz',BASE],cwd=ROOT);entries=raw.split(b'\0')[:-1];require(len(entries)==7803 and subprocess.check_output(['git','rev-parse',BASE+'^{tree}'],cwd=ROOT,text=True).strip()==TREE,'complete frozen base identity')
 for entry in entries:
  meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();path=ROOT/name.decode();require(kind==b'blob','no hidden submodule');data=path.readlink().as_posix().encode()if mode==b'120000'else path.read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'base bytes preserved '+name.decode())
  if mode!=b'120000':require(bool(path.stat().st_mode&0o111)==(mode==b'100755'),'base mode preserved')
 for path,digest in policy['baseline_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'exact inherited evidence '+path)
 inputs=json.loads((ROOT/'evidence/forcing-e1-fixed-candidates-v1/inputs.json').read_text())
 for pinned,c in zip(policy['conditional_fixed_comparison']['cases'],inputs['cases']):
  require(pinned['index']==c['index']and pinned['h']==c['h']==.00078125 and pinned['accepted_version']==c['accepted_publication']['stamp']['version']and pinned['expected_original_native_norm']==c['expected_terminal_norm']and pinned['unknown_sha256']==sha(json.dumps(c['final_authorized_unknowns'],separators=(',',':')).encode()),'actual final authorized state and unknowns')
 roster=json.loads((ROOT/'evidence/forcing-e1-trajectories-v2/protocol.json').read_text());require(roster['ordinary_trajectories']==48 and roster['lifecycle_trajectories']==8 and roster['geometry_ratio_band']==[1.7,2.3],'inherited full trajectory qualification roster')
 controls=[]
 for label,path,value in [('latent endpoint qualified',['semantic_boundary','unrounded_endpoint_qualifies'],True),('latent increment enters inertia',['semantic_boundary','latent_increment_in_qualified_inertia'],True),('retained tail',['semantic_boundary','retained_tails'],True),('initial projection',['semantic_boundary','accepted_initial_projection'],True),('second owner',['semantic_boundary','new_state_owner'],True),('stale known eta',['semantic_boundary','stored_known_eta_consistent'],False),('new norm graph',['construction','native_accumulation_and_norm'],'exact oracle'),('omitted accepted defect',['construction','accepted_constraint_defect_retained'],False),('old memory bound reused',['resources','old_memory_bound_is_not_E2_qualification'],False),('extra array',['resources','new_high_precision_array'],True),('expanded allowance',['resources','additional_live_cap_bytes'],131072),('retained high endpoint copy',['resources','high_precision_endpoint_copy_live_during_quadrature'],True),('relaxed Newton',['gates','newton_target'],2e-13),('extra correction',['gates','correction_limit'],8),('hidden geometry failure',['qualification_boundary','original_geometry_band_failures_retained'],7),('new geometry metric',['qualification_boundary','geometry_metric'],'signed-component ratio'),('invented endpoint',['qualification_boundary','missing_endpoints_not_fabricated'],False),('unrecorded review execution',['diagnostic_review','execution_enabled'],True),('invented review disposition',['diagnostic_review','disposition'],'approved'),('executed candidate',['executed_in_this_packet','native_candidate_equations'],1),('advanced owner',['executed_in_this_packet','owner_advances'],1),('new trajectory',['executed_in_this_packet','new_trajectories'],1),('missing transfer group',['measurements','groups'],['stored_chart_inertia']),('production adoption',['executed_in_this_packet','production_adoption'],True)]:controls.append(reject(label,policy,path,value))
 return {'status':'PASS_DESIGN_ONLY_CONTRACT_CHECKS_EXECUTION_BLOCKED','base':BASE,'base_tree':TREE,'preserved_base_files':7803,'base_inventory_sha256':sha(raw),'policy_sha256':sha((P/'policy.json').read_bytes()),'contract_sha256':sha((ROOT/'docs/research-book/implementation/forcing-affine-rounding-contract.md').read_bytes()),'analytical_examples':algebra(),'negative_policy_controls_rejected':controls,'native_candidate_equations_executed':0,'new_trajectories':0,'new_reference_integrations':0,'accepted_owners_constructed_or_advanced':0,'implementation':False,'production_adoption':False,'diagnostic_review_disposition':None,'execution_authorized':False,'fresh_E2_memory_qualification':False,'original_refusals_retained':2,'original_geometry_band_failures_retained':8}
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
