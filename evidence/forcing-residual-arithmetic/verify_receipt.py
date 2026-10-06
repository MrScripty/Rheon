"""Verify unchanged source and archived arithmetic diagnosis; no remote reads."""
from pathlib import Path
import copy,hashlib,json,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def verify(r):
 require(r['base']=='dad53b4054034fe0c2ff6240464df7441ab4e6a9' and r['base_tree']=='57648befc4e108bf8d727b3367adf7e119cdebc1','frozen base identity')
 paths=subprocess.check_output(['git','ls-tree','-r','--name-only',r['base']],cwd=ROOT,text=True).splitlines()
 require(len(paths)==r['preserved_tracked_paths'],'frozen tracked inventory')
 # One archive read avoids thousands of subprocesses. Comparison includes all
 # original source, book, proof and evidence bytes; new diagnosis is additive.
 import io,tarfile
 archive=subprocess.check_output(['git','archive',r['base']],cwd=ROOT)
 with tarfile.open(fileobj=io.BytesIO(archive))as tar:
  for path in paths:
   member=tar.extractfile(path)
   require(member is not None and (ROOT/path).read_bytes()==member.read(),'original tracked bytes: '+path)
 for path,digest in r['file_sha256'].items():require(hashlib.sha256((P/path).read_bytes()).hexdigest()==digest,'diagnosis artifact hash: '+path)
 require(r['production_changes']==[] and r['owner_advances']==0 and r['newton_target']==1e-13 and r['remaining_frozen_refusals']==5 and r['original_temporal_band']=='FAIL_ORIGINAL_TEMPORAL_BAND','unchanged source/decisions/scope')
 require(r['production_solver_defect_established']is False and r['arithmetic_floor_proved']is False,'no unestablished defect/floor theorem')
 original=(P/'frozen-coupled-source.rs').read_bytes();built=(P/'overlay-coupled-source.rs').read_bytes()
 require((ROOT/'src/coupled_discrete.rs').read_bytes()==original,'restored controller')
 require(built==original+b'// BEGIN ARITHMETIC PROBE TEST OVERLAY\n#[cfg(test)]\nmod arithmetic_probe {\n    include!("../evidence/forcing-residual-arithmetic/native_probe.rs");\n}\n// END ARITHMETIC PROBE TEST OVERLAY\n','test-only overlay without numerical edit')
 for probe in r['native_probes']:
  require(probe['exit']==0 and 'test result: ok. 1 passed; 0 failed;'in(P/probe['stdout']).read_text(),'actual native probe completion')
 for mode in ['normal','optimized']:
  packet=json.loads((P/('verification-'+mode+'.json')).read_text())
  require(packet['status']=='PASS_CAPTURED_REFUSAL_ANALYSIS' and len(packet['corruptions_rejected'])==7 and packet['accepted_endpoints_added']==0,'actual analysis qualification')
 require((P/'analysis-normal.json').read_bytes()==(P/'analysis-optimized.json').read_bytes(),'normal/optimized analysis byte parity')
 neighbors=json.loads((P/'neighborhood-summary.json').read_text())
 require(neighbors['native_equation_evaluations']==220 and neighbors['published_endpoints']==0 and all(x['neighbors_meeting_target']==0 for x in neighbors['rows']),'bounded native neighborhood; no accepted endpoint')
r=json.loads((P/'receipt.json').read_text());verify(r);controls=[]
for name,mutation in [('base',lambda x:x.update(base='0'*40)),('artifact hash',lambda x:x['file_sha256'].update({'analysis-normal.json':'0'*64})),('source mutation claim',lambda x:x.update(production_changes=['src/coupled_discrete.rs'])),('new publication',lambda x:x.update(owner_advances=1)),('changed target',lambda x:x.update(newton_target=1e-12)),('temporal promotion',lambda x:x.update(original_temporal_band='PASS')),('floor theorem',lambda x:x.update(arithmetic_floor_proved=True))]:
 bad=copy.deepcopy(r);mutation(bad)
 try:verify(bad)
 except ValueError:controls.append(name)
 else:raise ValueError('accepted corrupt receipt '+name)
print('PASS restored frozen source/evidence and diagnostic artifact bindings; rejected '+repr(controls))
