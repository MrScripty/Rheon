"""Closed additive packet binding; static replay only, no native execution."""
from pathlib import Path
import copy,hashlib,importlib.util,json,subprocess,sys,tempfile
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='94818f15e7d9148a079a4797392a8b5e356fd59b';TREE='23a9ae79b0880d3f3c900f548aff828a65898883'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def preserved():
 require(subprocess.check_output(['git','rev-parse',BASE+'^{tree}'],cwd=ROOT).decode().strip()==TREE,'frozen revised protocol tree')
 count=0;digest=hashlib.sha256()
 for entry in subprocess.check_output(['git','ls-tree','-rz',BASE],cwd=ROOT).split(b'\0'):
  if not entry:continue
  meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();require(mode in (b'100644',b'100755') and kind==b'blob','prior file modes')
  data=(ROOT/name.decode()).read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'preserved file '+name.decode())
  digest.update(name+b'\0'+hashlib.sha256(data).digest());count+=1
 require(count==7166,'complete protocol/source/evidence inventory')
 return {'files':count,'path_content_sha256':digest.hexdigest()}
def checks():
 old=ROOT/'evidence/forcing-public-call';prior=module('old_packet',old/'verify.py');receipt=json.loads((old/'receipt.json').read_text())
 require(receipt['preserved_inventory']==prior.preserved() and receipt['artifacts']==prior.artifacts() and receipt['checks']==prior.checks(),'unchanged old failed protocol and unexecuted historical E1')
 policy=json.loads((P/'protocol.json').read_text());m=json.loads((P/'memory-authorization.json').read_text())
 for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen protocol source')
 binding=json.loads((old/'binary-v2-binding.json').read_text());require(sha(Path(binding['binary']).read_bytes())==binding['binary_sha256']==policy['binary_sha256']==m['binary_sha256'],'same exact binary')
 require(m['protocol_sha256']==sha((P/'protocol.json').read_bytes()),'authorization binds new protocol')
 require(m['full_observed_memory_audit']==json.loads((old/'memory-v2-normal.json').read_text()),'fresh actual linked memory audit matches frozen instruction certificate')
 require(m['status']=='PASS_REVISED_66K_PROTOCOL_MEMORY_GATE' and m['observed_aligned_additional_bound']==65552 and m['new_protocol']=={'cap':67584,'status':'PASS','remaining':2032},'new resource qualification')
 require(m['old_protocol']=={'cap':65536,'status':'BLOCKED_ADDITIONAL_MEMORY_CAP','excess':16,'old_E1_classification':'UNEXECUTED'},'old result stays failed')
 require(m['legacy_owner_registered_additional_bytes']==65536 and m['external_supplemental_reservation']==2048 and m['new_full_single_owner_reservation']==542352 and m['binary_rebuilt'] is False and m['production_memory_limits_changed'] is False and m['candidate_E1_executed_at_authorization'] is False,'honest external reservation before execution')
 before=json.loads((P/'candidate-command-before.json').read_text());after=json.loads((P/'candidate-command-completed.json').read_text())
 require(all(after[k]==v for k,v in before.items()),'command immutable through execution')
 require(before['argv']==[binding['binary'],'--exact','research_public_call::research_public_candidate_and_controls','--nocapture'] and before['memory_authorization_sha256']==sha((P/'memory-authorization.json').read_bytes()) and before['protocol_id']==policy['protocol_id'] and before['compatibility_token']==policy['binary_compatibility_token'],'one specified command binds authorization')
 require(before['candidate_main_calls_requested']==1 and before['maximum_cancel_retry_pairs']==12 and before['parameter_search'] is False and before['continued_trajectory'] is False and before['production_adoption'] is False,'command fixed scope')
 require(after['exit']==0 and after['stdout_sha256']==sha((P/'candidate-native.log').read_bytes()) and after['stderr_sha256']==sha((P/'candidate-native-stderr.log').read_bytes()),'actual successful Rust command logs')
 reader=module('native_result',P/'analyze.py');r=reader.run();require(r==json.loads((P/'outcome.json').read_text()),'complete captured native outcome replay')
 with tempfile.TemporaryDirectory(prefix='rheon-66k-render-') as temporary:
  render=Path(temporary)/'replay.svg'
  rendered=subprocess.run([sys.executable,str(P/'render.py'),str(render)],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  require(rendered.returncode==0 and render.read_bytes()==(P/'replay.svg').read_bytes(),'actual captured-data render replay')
 mutations=[('invented resource budget',lambda x:x.update(new_cap=65536)),('old failed gate promoted',lambda x:x.update(old_cap_status='PASS')),('original owner mutation',lambda x:x.update(original_unchanged=False)),('parameter search',lambda x:x.update(parameter_search=True)),('continued trajectory',lambda x:x.update(continued_trajectory=True)),('production adoption',lambda x:x.update(production_adoption=True)),('unreached stage promoted',lambda x:x['controls'][11].update(status='CANCEL_PRESERVES_AND_RETRY_REPRODUCES')),('missing retry control',lambda x:x['controls'].pop()),('refused result promoted',lambda x:x.update(status='REFUSAL')),('failed momentum gate',lambda x:x['report']['step']['third'].update(finite_momentum_rate_norm=1.1e-13)),('publication clock bit changed',lambda x:x['accepted_snapshot'].update(time=x['accepted_snapshot']['time']^1)),('correction count misreported',lambda x:x.update(main_corrections=4))]
 rejected=[]
 for name,mutate in mutations:
  x=copy.deepcopy(r);mutate(x)
  try:reader.valid(x)
  except ValueError:rejected.append(name)
  else:raise ValueError('missed corruption '+name)
 return {'status':'PASS_FIXED_PUBLIC_CALL_66K_EVIDENCE_BINDING','native_test_exit':0,'same_binary_sha256':binding['binary_sha256'],'memory_bound':65552,'new_cap':67584,'new_memory_margin':2032,'old_cap':65536,'old_cap_status':'BLOCKED_ADDITIONAL_MEMORY_CAP','old_protocol_E1':'UNEXECUTED','new_protocol_E1':'SUCCESS','isolated_main_publications':1,'version':2,'time':0.0015625,'newton_checks':4,'corrections':3,'equation_evaluations':22,'planar_finite_norm':r['report']['step']['planar']['finite_momentum_rate_norm'],'third_finite_norm':r['report']['step']['third']['finite_momentum_rate_norm'],'reached_controls_passed':11,'unused_stage':'NOT_REACHED_BY_FIXED_CALL','original_unchanged':True,'parameter_search':False,'continued_trajectory':False,'production_adoption':False,'negative_controls':rejected}
def artifacts():
 return {str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in sorted(P.iterdir()) if p.is_file() and p.name not in ('receipt.json','record_post_git.py') and not p.name.startswith(('root-verification','post-git'))}
if __name__=='__main__':
 if '--freeze' in sys.argv:
  r={'base':BASE,'base_tree':TREE,'preserved_inventory':preserved(),'checks':checks(),'artifacts':artifacts()};(P/'receipt.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n');print(json.dumps({'status':'FROZEN_FIXED_PUBLIC_CALL_66K','receipt_sha256':sha((P/'receipt.json').read_bytes()),'artifacts':len(r['artifacts'])}))
 else:
  r=json.loads((P/'receipt.json').read_text());require(r['base']==BASE and r['base_tree']==TREE,'frozen protocol identity');require(r['preserved_inventory']==preserved(),'full preserved prior inventory');require(r['artifacts']==artifacts(),'closed result artifacts');require(r['checks']==checks(),'actual result replay');print(json.dumps(dict(**r['checks'],preserved_files=r['preserved_inventory']['files'],artifacts=len(r['artifacts'])),indent=2,sort_keys=True))
