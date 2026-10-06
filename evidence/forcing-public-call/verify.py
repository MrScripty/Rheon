"""Verify the stopped public qualification without any native execution."""
from pathlib import Path
import copy,hashlib,importlib.util,json,subprocess,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='77962729f8de389c25b3f595708ab50d4b3362c0';TREE='b578331d1dd8e768b2ec794689f5906acb71bb56'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def preserved():
 require(subprocess.check_output(['git','rev-parse',BASE+'^{tree}'],cwd=ROOT).decode().strip()==TREE,'frozen public prototype tree')
 count=0;digest=hashlib.sha256()
 for entry in subprocess.check_output(['git','ls-tree','-rz',BASE],cwd=ROOT).split(b'\0'):
  if not entry:continue
  meta,name=entry.split(b'\t',1);mode,kind,oid=meta.split();require(mode in (b'100644',b'100755') and kind==b'blob','prior source file modes')
  data=(ROOT/name.decode()).read_bytes();require(hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest().encode()==oid,'prior source/evidence preserved '+name.decode())
  digest.update(name+b'\0'+hashlib.sha256(data).digest());count+=1
 require(count==7131,'full prior public prototype inventory')
 return {'files':count,'path_content_sha256':digest.hexdigest()}
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def checks():
 require((P/'memory-v2-normal.json').read_bytes()==(P/'memory-v2-optimized.json').read_bytes(),'actual static normal/optimized parity')
 audit=module('new_controller_memory',P/'audit_memory.py');memory=audit.run()
 require(memory==json.loads((P/'memory-v2-normal.json').read_text()),'actual linked new memory reader replay')
 scalar=module('exact_scalar',ROOT/'evidence/forcing-increment-comparison/check_scalar.py')
 require(scalar.run(P/'scalar-v2-native.log')==json.loads((P/'scalar-v2-oracle.json').read_text()),'all exact scalar probes replay')
 for name in ['scalar-v2-native.log','type-layout-v2.log','baseline-v2-native.log']:
  require('1 passed; 0 failed' in (P/name).read_text(),'actual native preflight pass '+name)
 require('FAILED. 0 passed; 1 failed' in (P/'baseline-native.log').read_text(),'first clone-accounting failure preserved')
 rows=[json.loads(l) for l in (P/'baseline-v2-native-stderr.log').read_text().splitlines() if l.startswith('{')]
 old=json.loads((ROOT/'evidence/forcing-increment-contract-supplement/selected-capture.json').read_text())
 terminal=[r for r in rows if r.get('event')=='terminal_validation']
 require(terminal==[old['terminal_validation']]*2,'two exact original native public refusal records')
 first=[r for r in rows if r.get('event')=='newton_check' and r['accepted_version']==1]
 require(len(first)==14 and first[0]==first[7]==old['first_second_step_check'],'two identical first fixed controller checks')
 stop=json.loads((P/'qualification-stop.json').read_text())
 def valid(m,s):
  require(m['status']=='BLOCKED_ADDITIONAL_MEMORY_CAP' and m['cap']==65536 and m['aligned_additional_bound']==65552 and m['excess_bytes']==16,'cap failed and unchanged')
  require(m['candidate_execution_permitted'] is False and m['candidate_E1_executed'] is False and m['candidate_owner_advances']==0 and m['cancellation_retry_controls_executed'] is False,'blocked before E1/control execution')
  require(m['matched_frame_deltas']['Work::point']['positive_delta']==1872 and m['matched_frame_deltas']['public_call']['positive_delta']==32,'all changed caller frames charged')
  require(m['old_frozen_comparison_certificate_reused'] is False and m['no_baseline_slack_credit'] is True and m['no_shrink_or_replaced_solver_credit'] is True,'new binary additional accounting')
  require(s['status']=='STOPPED_BEFORE_E1_PUBLIC_CALL_MEMORY_GATE' and s['candidate_E1_executed'] is False and s['candidate_owner_publications']==0 and s['cancellation_retry_controls']=='NOT_RUN_MEMORY_GATE_BLOCKED','honest stopped full-call outcome')
  require(s['parameter_search'] is False and s['production_adoption'] is False and s['native_original_failure_preserved'] is True,'original scope preserved')
 valid(memory,stop)
 mutations=[('failed gate falsely passed',lambda m,s:m.update(status='PASS_NEW_BINARY_MEMORY_PREFLIGHT')),
  ('cap enlarged',lambda m,s:m.update(cap=65552)),('public caller omitted',lambda m,s:m['matched_frame_deltas']['public_call'].update(positive_delta=0)),
  ('candidate secretly executed',lambda m,s:m.update(candidate_E1_executed=True)),('invented candidate publication',lambda m,s:s.update(candidate_owner_publications=1)),
  ('unrun cancellation controls promoted',lambda m,s:s.update(cancellation_retry_controls='PASS')),('old binary certificate reused',lambda m,s:m.update(old_frozen_comparison_certificate_reused=True)),
  ('hidden parameter search',lambda m,s:s.update(parameter_search=True)),('original refusal promoted',lambda m,s:s.update(native_original_failure_preserved=False))]
 rejected=[]
 for name,mutation in mutations:
  m,s=copy.deepcopy(memory),copy.deepcopy(stop);mutation(m,s)
  try:valid(m,s)
  except ValueError:rejected.append(name)
  else:raise ValueError('missed result corruption '+name)
 return {'status':'PASS_STOPPED_PUBLIC_QUALIFICATION_BINDING_ONLY','new_memory_gate':'BLOCKED_ADDITIONAL_MEMORY_CAP','additional_bound':65552,'cap':65536,'excess_bytes':16,'kernel_peak':1536,'point_delta':1872,'public_call_delta':32,'native_original_fixed_call_norm':terminal[0]['rate_norm'],'native_corrections':7,'native_equation_evaluations':50,'scalar_groups':23,'exact_scalar_probes':36,'negative_controls':rejected,'candidate_E1_executed':False,'candidate_publications':0,'cancellation_retry_controls':'NOT_RUN_MEMORY_GATE_BLOCKED','parameter_search':False,'production_changes':0,'original_numerical_evidence_changed':False}
def artifacts():
 return {str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in sorted(P.iterdir()) if p.is_file() and p.name!='receipt.json' and p.name!='record_post_git.py' and not p.name.startswith(('root-verification','post-git'))}
if __name__=='__main__':
 if '--freeze' in sys.argv:
  r={'base':BASE,'base_tree':TREE,'preserved_inventory':preserved(),'checks':checks(),'artifacts':artifacts()}
  (P/'receipt.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n');print(json.dumps({'status':'FROZEN_STOPPED_PUBLIC_QUALIFICATION','receipt_sha256':sha((P/'receipt.json').read_bytes()),'artifacts':len(r['artifacts'])}))
 else:
  r=json.loads((P/'receipt.json').read_text());require(r['base']==BASE and r['base_tree']==TREE,'exact source identity');require(r['preserved_inventory']==preserved(),'complete prior file binding');require(r['artifacts']==artifacts(),'closed stopped-result artifacts');require(r['checks']==checks(),'complete actual stopped outcome replay')
  print(json.dumps(dict(**r['checks'],preserved_files=r['preserved_inventory']['files'],artifacts=len(r['artifacts'])),indent=2,sort_keys=True))
