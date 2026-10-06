"""Close the complete research packet, without rerunning native trajectories."""
from pathlib import Path
import copy,hashlib,importlib.util,json,os,subprocess,sys,tempfile
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='ac75dbccea47a9b5f43e2aad9fb33e885dcae741';TREE='e254b76974cdc0ed3a5dd6984de8750b5c0d291f'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def preserved():
 require(subprocess.check_output(['git','rev-parse',BASE+'^{tree}'],cwd=ROOT).decode().strip()==TREE,'frozen preflight tree')
 count=0;digest=hashlib.sha256()
 for entry in subprocess.check_output(['git','ls-tree','-rz',BASE],cwd=ROOT).split(b'\0'):
  if not entry:continue
  meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();require(mode in [b'100644',b'100755'] and kind==b'blob','source modes')
  data=(ROOT/name.decode()).read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'all prior source and evidence byte-preserved '+name.decode());digest.update(name+b'\0'+hashlib.sha256(data).digest());count+=1
 require(count==7408,'complete prior inventory')
 return {'files':count,'path_content_sha256':digest.hexdigest()}
def artifacts():return {x.name:sha(x.read_bytes())for x in sorted(P.iterdir())if x.is_file()and x.name not in ['receipt.json','record_post_git.py']and not x.name.startswith(('root-verification','post-git'))}
def validate_outcomes(values,expected):
 require(values==expected,'every outcome exactly matches captured/recomputed evidence')
 n,c,p,r=values
 require(n['ordinary_completed']==46 and n['ordinary_refused']==2 and n['ordinary_accepted_steps']==1034 and n['lifecycle_completed']==8 and n['lifecycle_accepted_steps']==128 and n['actual_repeated_cancellations']==64 and n['cancellation_targets_not_reached']==0,'honest trajectories and controls')
 require(n['memory_bound']==65536 and n['memory_remaining']==2048 and not n['production_adoption'] and not n['parameter_search'],'memory and scope')
 require(c['status']=='FAIL_ORIGINAL_TEMPORAL_BAND' and c['failed_original_checks']==8 and c['failed_finer_checks']==0 and c['reference_resolution_all_pass'] and not c['tolerance_relaxation'] and not c['production_adoption'],'original convergence failures retained')
 require(p['status']=='PASS_ALL_FROZEN_PHYSICAL_GATES' and p['passed_trajectories']==48 and p['failed_trajectories']==0 and p['reported_planar_finite_gate']==1e-13 and p['other_physical_momentum_gate']==1e-11 and p['tolerance_relaxation'] is False and sum(x['actual_steps_replayed']for x in p['rows'])==1034,'all actual accepted prefix physical equations')
 require(r['refused_trajectories']==2 and [x['index']for x in r['rows']]==[41,47] and all(x['Newton_signed_margin']<0 and x['accepted_state_preserved'] and x['same_owner_repeat_full_trace_exact']for x in r['rows']),'all exact refusals retained')
def checks():
 preflight=json.loads((P/'preflight-receipt.json').read_text());require(preflight['trajectory_executed'] is False and preflight['preserved_source_file_count']==7355,'historical preflight before E1')
 for name,digest in preflight['artifact_sha256'].items():require(sha((P/name).read_bytes())==digest,'frozen preflight artifact '+name)
 q=module('qualification',P/'qualify.py').run();require(q==preflight['qualification'] and q['additional_bound']==65536 and q['remaining']==2048,'fresh binary/memory/native preflight binding')
 reader=module('native_reader',P/'analyze_native.py');n=reader.run();c=module('convergence_reader',P/'convergence.py').run();r=module('refusal_reader',P/'analyze_refusals.py').run();p=json.loads((P/'physical-outcome.json').read_text());stored=[json.loads((P/(label+'-outcome.json')).read_text())for label in ['native','convergence','physical','refusal']];expected=[n,c,p,r];validate_outcomes(stored,expected)
 for label in ['native','convergence','physical','refusal']:require((P/(label+'-outcome.json')).read_bytes()==(P/(label+'-optimized.json')).read_bytes(),'normal/optimized outcome parity '+label)
 commands=json.loads((P/'analysis-commands.json').read_text());require(len(commands)==6 and [(Path(x['argv'][-1]).name,'-O'in x['argv'])for x in commands]==[(script,opt)for script in ['analyze_native.py','convergence.py','replay_physics.py']for opt in [False,True]],'actual complete replay command roster')
 for command in commands:
  require(command['exit']==0 and command['native_execution'] is False and command['cwd']==str(ROOT) and command['reader_sha256']==sha(Path(command['argv'][-1]).read_bytes()) and command['resource_env']=={'OPENBLAS_NUM_THREADS':'1','OMP_NUM_THREADS':'1'},'actual unchanged reader commands')
  for channel in ['stdout','stderr']:require(command[channel+'_sha256']==sha((P/command[channel]).read_bytes()),'actual reader logs')
 # The original complete coarse schema is retained, with allocation alone
 # adjusted for the explicitly authorized research reserve.
 frozen=module('physical_adapter',P/'replay_physics.py').frozen;budget=frozen.BUDGET;frozen.BUDGET=542352
 try:
  coarse=[x for i in range(40)for x in reader.rows(P/f'trajectory-{i}-native.log')if 'model'in x];groups=frozen.schema(coarse);require(len(coarse)==536 and len(groups)==40,'original complete forty-case temporal protocol schema')
 finally:frozen.BUDGET=budget
 refauth=json.loads((P/'reference-authorization.json').read_text());roster=json.loads((P/'roster.json').read_text());policy=json.loads((P/'protocol.json').read_text())
 require(refauth['driver_sha256']==sha((P/'run_references.py').read_bytes()) and refauth['protocol_sha256']==sha((P/'protocol.json').read_bytes()) and refauth['original_sources_sha256']==roster['original_sources_sha256'] and refauth['parameters']==policy['reference_integrations'] and refauth['rhs_cap']==5000 and not refauth['parameter_search'],'fixed original reference authorization')
 refcommands=reader.rows(P/'reference-driver.log');require(len(refcommands)==16 and all(x['status']=='COMPLETE'for x in refcommands) and not(P/'reference-driver-stderr.log').read_bytes(),'all actual paired reference completions')
 jobs=reader.rows(P/'execution-driver.log');require(len(jobs)==57 and [x['index']for x in jobs[:48]]==list(range(48)) and [x['index']for x in jobs[48:56]]==[x['trajectory_index']for x in roster['lifecycle_controls']] and all(x['exit']==0 for x in jobs[:56]) and jobs[-1]['status']=='ALL_48_TRAJECTORIES_AND_8_CONTROLS_EXECUTED_ONCE' and not(P/'execution-driver-stderr.log').read_bytes(),'entire once-only launch sequence')
 require(len(list(P.glob('trajectory-*-command-completed.json')))==48 and len(list(P.glob('lifecycle-*-command-completed.json')))==8,'closed native run roster')
 with tempfile.TemporaryDirectory(prefix='rheon-e1-render-')as temporary:
  target=Path(temporary)/'replay.svg';env=os.environ.copy();env.update(OPENBLAS_NUM_THREADS='1',OMP_NUM_THREADS='1');render=subprocess.run([sys.executable,str(P/'render.py'),str(target)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE);require(render.returncode==0 and target.read_bytes()==(P/'replay.svg').read_bytes(),'actual deterministic captured-data render')
 mutations=[('trajectory omitted',lambda x:x[0]['rows'].pop()),('refusal promoted to complete',lambda x:x[0]['rows'][41].update(status='COMPLETE')),('refusal hidden',lambda x:x[0].update(ordinary_refused=0)),('wrong accepted step count',lambda x:x[0].update(ordinary_accepted_steps=1264)),('missing interruption',lambda x:x[0]['cancellations'].pop()),('changed cancellation state bits',lambda x:x[0]['cancellations'][0].update(accepted=x[0]['cancellations'][0]['accepted'].replace('time:','time: 1, wrong:'))),('production adoption',lambda x:x[0].update(production_adoption=True)),('memory cap changed',lambda x:x[0].update(memory_remaining=4096)),('final equations omitted',lambda x:x[0]['metrics'][0].update(total_equations=x[0]['metrics'][0]['counted_equations'])),('failed work promoted',lambda x:x[0]['metrics'][0]['work']['total'].update(signed_margin=-1.)),('band relaxed',lambda x:x[1]['failed_checks'][0].update(upper=3.)),('original failure relabeled',lambda x:x[1].update(status='PASS_ORIGINAL_BANDS')),('reference unresolved',lambda x:x[1]['families'][0].update(reference_resolution_signed_margin=-1.)),('missing finer endpoint invented',lambda x:x[1]['families'][0]['errors'][-1].update(status='COMPLETE',geometry_max_error=1e-8)),('reported gate relaxed',lambda x:x[2].update(reported_planar_finite_gate=1e-11)),('physical source replaced',lambda x:x[2].update(physical_source_unchanged=False)),('repeated refusal changed',lambda x:x[3]['rows'][0].update(same_owner_repeat_full_trace_exact=False)),('refusal norm below target',lambda x:x[3]['rows'][0].update(Newton_signed_margin=1e-15))]
 rejected=[]
 for name,mutate in mutations:
  altered=copy.deepcopy(stored);mutate(altered)
  try:validate_outcomes(altered,expected)
  except ValueError:rejected.append(name)
  else:raise ValueError('missed corruption '+name)
 ordinary=[x for x in n['metrics']if x['mode']=='trajectory'];native_margins={key:min(x[key]for x in ordinary)for key in ['finite_full_norm_signed_margin','direct_full_norm_signed_margin','full_constraints_signed_margin','quadrature_signed_margin','Newton_signed_margin']};physical_maxima={key:max(x['maxima'][key]for x in p['rows'])for key in p['rows'][0]['maxima']}
 return {'status':'PASS_COMPLETE_E1_TRAJECTORY_PACKET_WITH_FAILURES_RETAINED','source':q['source'],'source_tree':q['source_tree'],'binary_sha256':q['binary_sha256'],'ordinary_trajectories':48,'ordinary_completed':46,'ordinary_refused':2,'ordinary_accepted_steps':1034,'coarse_complete':40,'finer_complete':6,'lifecycle_complete':8,'lifecycle_accepted_steps':128,'repeated_cancellations':64,'every_retry_report_and_state_bit_exact':True,'physical_replay':'PASS_ALL_1034_ACCEPTED_STEPS','native_minimum_signed_margins':native_margins,'independent_physical_maxima':physical_maxima,'reference_integrations':16,'reference_resolution_all_pass':True,'temporal_qualification':'FAIL_ORIGINAL_TEMPORAL_BAND','failed_original_geometry_checks':8,'complete_finer_checks_passed':18,'missing_finer_endpoint_checks':6,'memory_additional_bound':65536,'authorized_additional_cap':67584,'memory_remaining':2048,'refusal_indices':[41,47],'final_refusal_Newton_norms':[x['final_authorized_Newton_norm']for x in r['rows']],'refusal_signed_margins':[x['Newton_signed_margin']for x in r['rows']],'production_adoption':False,'parameter_search':False,'tolerance_relaxation':False,'original_source_and_proof_limitations_preserved':True,'negative_controls':rejected}
if __name__=='__main__':
 if '--freeze'in sys.argv:
  receipt={'base':BASE,'base_tree':TREE,'preserved_inventory':preserved(),'checks':checks(),'artifacts':artifacts()};(P/'receipt.json').write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n');print(json.dumps({'status':'FROZEN_COMPLETE_TRAJECTORY_PACKET','receipt_sha256':sha((P/'receipt.json').read_bytes()),'artifacts':len(receipt['artifacts'])}))
 else:
  receipt=json.loads((P/'receipt.json').read_text());require(receipt['base']==BASE and receipt['base_tree']==TREE and receipt['preserved_inventory']==preserved(),'actual frozen prior source tree');require(receipt['artifacts']==artifacts(),'closed actual complete result packet');require(receipt['checks']==checks(),'all current source and numerical replays');print(json.dumps({**receipt['checks'],'preserved_files':receipt['preserved_inventory']['files'],'artifacts':len(receipt['artifacts'])},indent=2,sort_keys=True))
