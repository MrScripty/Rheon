"""Bind the final candidate to exact source, preserved failures and actual checks."""
from pathlib import Path
import hashlib,json,subprocess,copy,sys,re
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
SOURCE='922466bd9efdf4612462288fbe2df5a679ef4ccb';BASE='a2fcd29a927177500bcb57659c520d71e0ed0f12';TREE='914d1313534e05f7fb9a15cbd15fce7124dd6d48'
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(data):return hashlib.sha256(data).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
def tree(commit):return {r.split(b'\t',1)[1].decode():r.split()[2].decode()for r in git('ls-tree','-r',commit).splitlines()}
def load(name):return json.loads((P/name).read_text())
def inventory():return {str(p.relative_to(ROOT))for p in P.rglob('*')if p.is_file()and '__pycache__'not in p.parts and p.name!='final-receipt.json'}
def verify(r,evidence_commit=None):
 require(r['source']==SOURCE and r['source_tree']==TREE and r['baseline']==BASE,'exact source/tree/baseline')
 require(git('rev-parse',SOURCE+'^{tree}').decode().strip()==TREE and git('show','-s','--format=%P',SOURCE).decode().split()==[BASE],'ordinary additive source commit')
 require(r['newton_threshold']==1e-13 and r['correction_cap']==7 and r['residual_check_cap']==8 and r['separate_qualification_equations']==2 and r['counted_equation_budget']==200 and r['nominal_forced_bytes']==474768,'unchanged numerical and memory limits')
 require(r['original_temporal_band_pass']is False and r['arithmetic_floor_proved']is False and r['general_rate_proved']is False and r['hosted_CI_qualified']is False and r['active_workflow_changed']is False,'honest limitations')
 for commit in [SOURCE,BASE]:require(git('show','-s','--format=%an%n%ae%n%cn%n%ce',commit).decode().splitlines()==['MrScripty','TheEnvironmentGuy@protonmail.com']*2,'authorized author and committer')
 preserved=load('preservation.json');expected={p:b for p,b in tree(BASE).items()if p!='src/coupled_discrete.rs'}
 require(preserved['preserved_git_blobs']==expected and preserved['preserved_path_count']==r['preserved_baseline_paths']==len(expected),'complete frozen baseline')
 source_tree=tree(SOURCE)
 require(all(source_tree.get(p)==b for p,b in expected.items()),'source rewrote frozen evidence')
 for p,b in expected.items():
  data=(ROOT/p).read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()==b,'live frozen baseline '+p)
 source_tree=tree(SOURCE)
 require(set(r['source_sha256'])=={p for p in source_tree if p.startswith(('src/','tests/','examples/'))or p in ['Cargo.toml','Cargo.lock','rust-toolchain.toml']},'complete compiled Rust source inventory')
 for p,digest in r['source_sha256'].items():require(sha(git('show',SOURCE+':'+p))==digest==sha((ROOT/p).read_bytes()),'exact qualified source '+p)
 for p,digest in r['supporting_document_sha256'].items():require(sha((ROOT/p).read_bytes()if evidence_commit is None else git('show',evidence_commit+':'+p))==digest,'source-bound research clarification '+p)
 files=inventory()if evidence_commit is None else {p for p in tree(evidence_commit)if p.startswith(str(P.relative_to(ROOT))+'/')and not p.endswith('/final-receipt.json')}
 require(files==set(r['file_sha256']),'complete qualification packet inventory')
 for p,digest in r['file_sha256'].items():require(sha((ROOT/p).read_bytes()if evidence_commit is None else git('show',evidence_commit+':'+p))==digest,'frozen actual evidence '+p)
 matrix=load('matrix-receipt.json');captures=load('captures-receipt.json');trace=load('trace-receipt.json');common=load('common-receipt.json')
 for record,n in [(matrix,10),(captures,6),(trace,3),(common,5)]:require(record['source_commit']==SOURCE and record['source_sha256']==r['source_sha256'] and record['status']=='PASS'and len(record['commands'])==n and all(c['exit']==0 for c in record['commands']),'actual exact-source complete commands')
 expected_commands=[['cargo','fmt','--all','--check']]
 for label,flags in [('default',[]),('core-only',['--no-default-features']),('desktop',['--features','desktop'])]:
  expected_commands += [['cargo','clippy','--locked',*flags,'--all-targets','--','-D','warnings'],['cargo','test','--locked',*flags,'--all-targets'],['cargo','test','--locked',*flags,'--doc']]
  text=(P/('tests-'+label+'.log')).read_text();count=sum(map(int,re.findall(r'test result: ok\. (\d+) passed;',text)));require(count==r['actual_Rust_tests_passed'][label]=={'default':251,'core-only':245,'desktop':257}[label] and 'test result: FAILED'not in text,'all original and five boundary tests '+label)
 require([c['argv']for c in matrix['commands']]==expected_commands,'exact full feature matrix and doc tests')
 release=load('release-focused-receipt.json');require(release['source_commit']==SOURCE and release['source_sha256']==r['source_sha256']and release['status']=='PASS'and release['command']['exit']==0 and 'test result: ok. 5 passed;'in(P/'release-focused.log').read_text(),'actual optimized budget/cancellation/publication contracts')
 require(trace['RUSTFLAGS']=='--cfg rheon_newton_trace','only observational trace configuration')
 for stem in ['solver_iteration_boundary','forcing_temporal_probe','forced_extruded']:require((P/(stem+'-default.jsonl')).read_bytes()==(P/(stem+'-core-only.jsonl')).read_bytes(),'native mode parity '+stem)
 require((P/'trace-boundary.jsonl').read_bytes()==(P/'solver_iteration_boundary-default.jsonl').read_bytes()and(P/'trace-finer.jsonl').read_bytes()==(P/'forcing_temporal_probe-default.jsonl').read_bytes(),'tracing does not change publications')
 require((P/'forced_extruded-default.jsonl').read_bytes()==(ROOT/'evidence/forced-extruded-liquid/native-trials/first-native.jsonl').read_bytes(),'536 frozen original publications')
 for a,b in [('audit-normal.json','audit-optimized.json'),('replay-first.json','replay-optimized.json'),('hosted-normal.log','hosted-optimized.log'),('prefix-normal.json','prefix-optimized.json'),('budget-normal.json','budget-optimized.json')]:require((P/a).read_bytes()==(P/b).read_bytes(),'normal optimized parity '+a)
 prefix=load('prefix-normal.json');require(prefix['original_newton_check_records_compared']==611 and prefix['original_newton_check_records_byte_identical']is True and prefix['original_actual_finer_publications_compared']==143 and prefix['original_actual_finer_publications_byte_identical']is True,'all original native arithmetic/publication prefixes')
 budget=load('budget-normal.json');require(budget['original_maximum_unperturbed_residual_checks']==7 and budget['candidate_maximum_unperturbed_residual_checks']==8 and budget['maximum_computed_Newton_correction_solves']==7 and budget['maximum_counted_Newton_equation_evaluations']==50 and budget['maximum_accepted_total_work_equation_evaluations_excluding_seed']==52 and budget['equation_evaluation_performs_existing_15_by_15_chart_solves']is True,'honest solve/check/qualification accounting')
 audit=load('audit-normal.json');replay=load('replay-first.json')
 require(audit['status']=='PASS_ACTUAL_TERMINAL_CHECK_ACCOUNTING'and audit['finer_actual_accepted_steps']==194 and audit['finer_original_attempts_refused']==audit['finer_same_owner_refusal_retries']==5 and audit['maximum_computed_corrections']==7 and audit['maximum_counted_newton_equations']==50 and len(audit['actual_corruption_rejections'])==4,'actual bounded controller and retries')
 require([(x['kind'],x['load'],x['h'],x['step'],x['calls'])for x in audit['fully_published_terminal_acceptances']]==[('pressure_state','reversed',.0015625,7,50),('pressure_state','reversed',.00078125,1,50)],'two actually published terminal candidates')
 require(replay['actual_publications']==202 and replay['actual_steps']==194 and replay['completed_cases']==3 and replay['refused_cases']==5 and replay['original_physical_verifier_changed']is False and len(replay['new_capture_corruption_rejections'])==8 and all(x['status']=='REJECTED'for x in replay['new_capture_corruption_rejections']),'all new actual physical replay and controls')
 require(replay['original_reversed_five_interval_qualification']=='FAIL_ORIGINAL_TEMPORAL_BAND','original failure retained')
 lifecycle=(P/'lifecycle-tests-stderr.log').read_text();require(re.search(r'Ran 37 tests in ',lifecycle)is not None and lifecycle.rstrip().endswith('OK'),'all original hosted lifecycle methods')
 paths=load('current-column-path-ledger.json');require(len(paths['tracked_examples'])==25 and len(paths['controls'])==56,'actual current tracked path ledger')
 require('PASS executable completion manifest, all-step divergence and PNG dimensions'in(P/'executable-smoke-assertions.log').read_text(),'actual original smoke checks')
 timings=load('hosted-timing-proposal.json');require(timings['active_workflow_changed']is False and timings['hosted_timing_qualified']is False and timings['measurement_receipt_sha256']==sha((ROOT/'evidence/forcing-main-integration/receipt.json').read_bytes()),'inert measured hosted proposal')
 renders=load('render-data.json');require(renders['actual_new_states']==65 and renders['actual_original_prefix_states']==7 and renders['coordinates']==['x0','H0','x1','H1']and renders['browser_rasterization_claimed']is False,'scientific plots of actual states')
 return {'status':'FROZEN_TESTED_TERMINAL_VALIDATION_CANDIDATE','source':SOURCE,'tree':TREE,'Rust_tests_passed':r['actual_Rust_tests_passed'],'preserved_baseline_paths':len(expected),'new_actual_steps':194,'remaining_refusals':5,'original_temporal_band_pass':False,'hosted_CI_qualified':False,'packet_files':len(files)}
if __name__=='__main__':
 r=load('final-receipt.json');e=sys.argv[sys.argv.index('--evidence-commit')+1]if '--evidence-commit'in sys.argv else None;result=verify(r,e);controls=[]
 if '--negative-self-test'in sys.argv:
  for key in ['source','source_sha256','file_sha256','newton_threshold','actual_Rust_tests_passed','original_temporal_band_pass','hosted_CI_qualified']:
   bad=copy.deepcopy(r)
   if key in ['source_sha256','file_sha256']:bad[key][next(iter(bad[key]))]='0'*64
   elif key=='source':bad[key]='0'*40
   elif key=='newton_threshold':bad[key]=2e-13
   elif key=='actual_Rust_tests_passed':bad[key]['desktop']=0
   else:bad[key]=True
   try:verify(bad,e)
   except ValueError:controls.append({'control':key,'status':'REJECTED'})
   else:raise ValueError('accepted forged candidate '+key)
 result['actual_binding_corruption_rejections']=controls;print(json.dumps(result,indent=2))
