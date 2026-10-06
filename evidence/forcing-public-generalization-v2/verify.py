"""Bind frozen preflight and complete once-only roster; no native execution."""
from pathlib import Path
import copy,hashlib,importlib.util,json,subprocess,sys,tempfile
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='876e78c46503a16232ecd0233df9934f081e8db0';TREE='3180d8341c24983844a8c36e7d5e3fb0e310e5c0'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def preserved():
 require(subprocess.check_output(['git','rev-parse',BASE+'^{tree}'],cwd=ROOT).decode().strip()==TREE,'exact frozen preflight tree')
 count=0;digest=hashlib.sha256()
 for entry in subprocess.check_output(['git','ls-tree','-rz',BASE],cwd=ROOT).split(b'\0'):
  if not entry:continue
  meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();require(mode in (b'100644',b'100755') and kind==b'blob','prior source modes')
  data=(ROOT/name.decode()).read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'all prior evidence/source preserved '+name.decode())
  digest.update(name+b'\0'+hashlib.sha256(data).digest());count+=1
 require(count==7269,'full preflight inventory')
 return {'files':count,'path_content_sha256':digest.hexdigest()}
def checks():
 preflight=json.loads((P/'preflight-receipt.json').read_text());require(preflight['generalization_executed'] is False and preflight['status']=='FROZEN_PASS_NEW_BINARY_PREFLIGHT_BEFORE_GENERALIZATION','historical pre-execution preflight')
 for path,digest in preflight['artifacts'].items():require(sha((ROOT/path).read_bytes())==digest,'all frozen preflight artifacts '+path)
 qualification=module('generalization_preflight',P/'qualify.py').run();require(qualification==preflight['qualification']==json.loads((P/'qualification-normal.json').read_text()),'actual scalar/source/binary/baseline/memory preflight replay')
 require((P/'qualification-normal.json').read_bytes()==(P/'qualification-optimized.json').read_bytes(),'preflight normal/optimized parity')
 # Preserve and replay both earlier resource protocols, without launching either.
 earlier=ROOT/'evidence/forcing-public-call-66k';old=module('old_fixed_result',earlier/'verify.py');r=json.loads((earlier/'receipt.json').read_text());require(old.preserved()==r['preserved_inventory'] and old.artifacts()==r['artifacts'] and old.checks()==r['checks'],'old one-step success, failed 64KiB certificate and controls remain unchanged')
 auth=json.loads((P/'execution-authorization.json').read_text());binding=json.loads((P/'binary-binding.json').read_text());policy=json.loads((P/'protocol.json').read_text());cases=json.loads((P/'cases.json').read_text())['cases']
 require(auth['source']==binding['prototype'] and auth['protocol_sha256']==sha((P/'protocol.json').read_bytes()) and auth['binary_sha256']==binding['binary_sha256'] and auth['qualification_sha256']==sha((P/'qualification-normal.json').read_bytes()) and auth['runner_sha256']==sha((P/'run.py').read_bytes()) and auth['case_roster_sha256']==sha((P/'cases.json').read_bytes()),'pre-launch authorization binds frozen source/binary/roster/runner')
 require(auth['case_indices']==list(range(5)) and auth['main_calls_per_case']==1 and auth['generalization_executed_at_authorization'] is False and all(auth[k] is False for k in ['parameter_search','continued_trajectory','production_adoption']),'no hidden additional experiment')
 require(sorted(p.name for p in P.glob('case-*-command-before.json'))==[f'case-{i}-command-before.json' for i in range(5)],'closed five command roster')
 for c in cases:
  i=c['index'];before=json.loads((P/f'case-{i}-command-before.json').read_text());after=json.loads((P/f'case-{i}-command-completed.json').read_text())
  require(all(after[k]==v for k,v in before.items()),'unchanged prescribed command through completion')
  require(before['argv']==[binding['binary'],'--exact','research_public_call::research_generalization_candidate_one','--nocapture'] and before['env']=={'RHEON_PUBLIC_CALL_ALLOW':policy['protocol_id'],'RHEON_CASE_INDEX':str(i)} and before['case_index']==i and before['case_id']==c['id'],'exact once-only public call invocation')
  require(before['authorization_sha256']==sha((P/'execution-authorization.json').read_bytes()) and before['binary_sha256']==binding['binary_sha256'] and before['main_calls_requested']==1 and all(before[k] is False for k in ['parameter_search','continued_trajectory','production_adoption']),'actual command scope')
  require(after['exit']==0 and after['stdout_sha256']==sha((P/f'case-{i}-native.log').read_bytes()) and after['stderr_sha256']==sha((P/f'case-{i}-native-stderr.log').read_bytes()),'complete actual native result logs')
 reader=module('every_native_result',P/'analyze.py');out=reader.run();require(out==json.loads((P/'outcome.json').read_text()),'every full result and exact state-change replay')
 require(out['successes']==5 and out['refusals']==[] and out['passing_control_full_report_matches_frozen_one_step'] is True and out['passing_control_accepted_bits_match_frozen_one_step'] is True,'observed five successes with exact passing control reproduction')
 # Passing values above the Newton target are deliberately accepted by the
 # original physical gate. A stricter invented physical criterion would fail.
 require(out['cases'][2]['report']['step']['third']['finite_momentum_rate_norm']>1e-13 and out['cases'][4]['report']['step']['planar']['direct_momentum_rate_norm']>1e-13,'distinct physical/Newton criteria preserved')
 with tempfile.TemporaryDirectory(prefix='rheon-generalization-render-') as temporary:
  render=Path(temporary)/'replay.svg';result=subprocess.run([sys.executable,str(P/'render.py'),str(render)],stdout=subprocess.PIPE,stderr=subprocess.PIPE);require(result.returncode==0 and render.read_bytes()==(P/'replay.svg').read_bytes(),'actual deterministic captured-data render')
 mutations=[('case omitted',lambda x:x['cases'].pop()),('success-only reordering',lambda x:x['cases'].reverse()),('new parameter search',lambda x:x.update(parameter_search=True)),('trajectory continued',lambda x:x.update(continued_trajectory=True)),('production adoption',lambda x:x.update(production_adoption=True)),('physical criterion replaced by Newton target',lambda x:x.update(physical_momentum_limit=1e-13)),('Newton target loosened',lambda x:x.update(newton_target=1e-11)),('unapproved memory allowance',lambda x:x.update(additional_cap=70000)),('final equation checks omitted',lambda x:x['cases'][0].update(total_completed_equations=x['cases'][0]['counted_newton_equations'])),('third report invented as absent',lambda x:x['cases'][0].update(third_result='NOT_REACHED')),('failed third physical gate promoted',lambda x:x['cases'][2]['report']['step']['third'].update(finite_momentum_rate_norm=1.01e-11)),('mutation to original prefix',lambda x:x['cases'][1].update(original_owner_unchanged=False)),('clock bit mutation',lambda x:x['cases'][0]['after_snapshot'].update(time=x['cases'][0]['after_snapshot']['time']^1)),('signed-zero prefix mutation',lambda x:x['cases'][0]['before_snapshot']['velocity'][0].__setitem__(1,1<<63)),('field change suppressed',lambda x:x['cases'][0]['changes'].pop()),('failure hidden',lambda x:x.update(refusals=['invented failure'])),('publication hidden',lambda x:x['cases'][0].update(published=False))]
 rejected=[]
 for name,mutate in mutations:
  x=copy.deepcopy(out);mutate(x)
  try:reader.valid(x)
  except ValueError:rejected.append(name)
  else:raise ValueError('missed evidence corruption '+name)
 return {'status':'PASS_COMPLETE_BOUNDED_GENERALIZATION_BINDING','case_count':5,'once_only_main_calls':5,'successes':5,'remaining_prescribed_refusals':[],'counted_newton_equations':[x['counted_newton_equations'] for x in out['cases']],'total_completed_equations':[x['total_completed_equations'] for x in out['cases']],'corrections':[x['corrections'] for x in out['cases']],'exact_field_changes_per_case':[len(x['changes']) for x in out['cases']],'full_third_and_combined_work_gates_passed':5,'single_isolated_publications':5,'passing_control_report_and_accepted_bits_exact':True,'native_original_refusals_reproduced':5,'original_owners_unchanged':True,'binary_sha256':binding['binary_sha256'],'additional_bound':65536,'authorized_additional_cap':67584,'remaining_memory':2048,'newton_target':1e-13,'physical_momentum_limit':1e-11,'old_one_step_64KiB_stopped_protocol_unchanged':True,'old_one_step_66KiB_success_and_controls_unchanged':True,'failed_v1_compile_preserved':True,'parameter_search':False,'continued_trajectory':False,'production_adoption':False,'negative_controls':rejected}
def artifacts():return {str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in sorted(P.iterdir()) if p.is_file() and p.name not in ['receipt.json','record_post_git.py'] and not p.name.startswith(('root-verification','post-git'))}
if __name__=='__main__':
 if '--freeze' in sys.argv:
  r={'base':BASE,'base_tree':TREE,'preserved_inventory':preserved(),'checks':checks(),'artifacts':artifacts()};(P/'receipt.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n');print(json.dumps({'status':'FROZEN_COMPLETE_FIVE_CALL_GENERALIZATION','receipt_sha256':sha((P/'receipt.json').read_bytes()),'artifacts':len(r['artifacts'])}))
 else:
  r=json.loads((P/'receipt.json').read_text());require(r['base']==BASE and r['base_tree']==TREE,'frozen qualification identity');require(r['preserved_inventory']==preserved(),'full prior inventory');require(r['artifacts']==artifacts(),'closed complete result packet');require(r['checks']==checks(),'all actual result and preflight replays');print(json.dumps(dict(**r['checks'],preserved_files=r['preserved_inventory']['files'],artifacts=len(r['artifacts'])),indent=2,sort_keys=True))
