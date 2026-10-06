"""Verify the frozen force experiment without promoting its temporal refusal."""
import copy,hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];P=ROOT/'evidence/forced-extruded-liquid';V=Path(__file__).resolve().parent;RECEIPT=P/'final-receipt.json';BASE='35b00247e68f04e6db539c365298862f82e1c2d0'
EXCLUDED={str(RECEIPT.relative_to(ROOT)),'evidence/forced-extruded-liquid-validation/frozen-normal.log','evidence/forced-extruded-liquid-validation/frozen-optimized.log','evidence/forced-extruded-liquid-validation/frozen-validation-receipt.json'}
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(b):return hashlib.sha256(b).hexdigest()
def git(*a):return subprocess.check_output(['git',*a],cwd=ROOT)
def tree(c):return {l.split(b'\t',1)[1].decode():l.split(b'\t',1)[0].split()[2].decode()for l in git('ls-tree','-r',c).splitlines()}
def verify(r,evidence_commit=None):
 require(r['status']=='FROZEN_FORCING_EXPERIMENT_PHYSICAL_PASS_TEMPORAL_GATE_FAILED','honest frozen experiment status')
 source=r['qualified_source_commit'];s=tree(source);base=tree(BASE);allowed={'src/coupled_discrete.rs','src/lib.rs'}
 require(git('rev-parse',source+'^{tree}').decode().strip()==r['qualified_source_tree'],'source tree')
 require(git('show','-s','--format=%P',source).decode().split()==r['ordered_source_parents'],'ordered source parents')
 require(r['historical_base']==BASE and set(r['legitimate_changed_baseline_paths'])==allowed,'separate reviewed force lane')
 require(r['historical_git_blobs']=={p:b for p,b in base.items()if p not in allowed},'complete preserved baseline')
 require(all(s.get(p)==b for p,b in r['historical_git_blobs'].items()),'rewritten original source/evidence')
 for p,d in r['source_sha256'].items():
  require(p in s and sha(git('show',source+':'+p))==d,'Git source '+p)
  require(sha((ROOT/p).read_bytes())==d,'working source '+p)
 evidence=tree(evidence_commit)if evidence_commit else None
 prefixes=[str(p.relative_to(ROOT))+'/'for p in [P,V]]
 files={p for p in evidence if any(p.startswith(pre)for pre in prefixes)and p not in EXCLUDED}if evidence is not None else {str(p.relative_to(ROOT))for directory in [P,V]for p in directory.rglob('*')if p.is_file()and '__pycache__'not in p.parts and str(p.relative_to(ROOT))not in EXCLUDED}
 require(files==set(r['file_sha256']),'complete source/evidence packet inventory')
 for p,d in r['file_sha256'].items():require(sha(git('show',evidence_commit+':'+p)if evidence else(ROOT/p).read_bytes())==d,'frozen evidence '+p)
 local=json.loads((P/'local-qualification/receipt.json').read_text())
 require(local['status']=='LOCAL_PHYSICAL_REPLAY_COMPLETE_TEMPORAL_GATE_FAILED_IDENTITY_HOLD'and local['qualification_source_is_uncommitted']and local['git_identity_hold'],'preserved precommit execution facts')
 require(local['git_checkpoint']=='6458b6888e59caf8baee2480efff13c5b9ab7f14','actual precommit checkpoint')
 for p,d in local['source_sha256'].items():require(r['source_sha256'].get(p)==d,'unchanged qualified execution source '+p)
 matrix=json.loads((P/'native-preflight/receipt.json').read_text());native='5bb0b5d538ac0b1707142cb9691e61994fd68b8e'
 require(matrix['status']=='PASS'and len(matrix['commands'])==7 and all(c['exit']==0 for c in matrix['commands']),'actual complete Rust matrix')
 for p,d in matrix['source_sha256'].items():require(r['source_sha256'][p]==d and sha(git('show',native+':'+p))==d,'unchanged compiled source '+p)
 for mode in ['normal','optimized']:
  require('independently expected original-band first order initial nonconstant reversed geometry_max_error'in(P/f'local-qualification/{mode}-strict-temporal-refusal-stderr.log').read_text(),'actual unchanged strict temporal refusal '+mode)
 result=json.loads((P/'local-qualification/normal-diagnostic-replay.json').read_text())
 require(result['status']=='FAIL_ORIGINAL_TEMPORAL_BAND'and result['physical_equation_replay']=='PASS'and len(result['failed_temporal_bands'])==4 and len(result['references'])==16 and len(result['actual_corruption_rejections'])==19,'physical replay and retained four temporal failures')
 require((P/'local-qualification/normal-diagnostic-replay.json').read_bytes()==(P/'local-qualification/optimized-diagnostic-replay.json').read_bytes(),'actual mode parity')
 require(all(c['exit']==c['expected_exit']for c in local['actual_commands'])and sum(c['expected_exit']==1 for c in local['actual_commands'])==2,'actual expected command exits')
 require(local['force_native_modes_identical']and local['legacy_planar_publications_byte_identical']and local['legacy_third_publications_byte_identical'],'native and original publication parity')
 require(r['browser_status']==local['browser_status']and not r['hosted_CI_qualified'],'explicit unqualified hosting/browser scope')
 identity=json.loads((V/'identity-verified.json').read_text())
 require(identity['changed_settings']=={'user.name':'MrScripty','user.email':'TheEnvironmentGuy@protonmail.com'}and identity['only_two_identity_settings_changed'],'authorized identity correction')
 require(git('show','-s','--format=%an%n%ae%n%cn%n%ce',source).decode().splitlines()==['MrScripty','TheEnvironmentGuy@protonmail.com','MrScripty','TheEnvironmentGuy@protonmail.com'],'new source author and committer')
 return dict(status=r['status'],source=source,tree=r['qualified_source_tree'],source_paths=len(r['source_sha256']),packet_files=len(files),preserved_baseline_paths=len(r['historical_git_blobs']),actual_publications=536,actual_corruption_rejections=19,independent_references=16,temporal_gate='FAILED_ORIGINAL_BAND',browser_status=r['browser_status'],hosted_CI_qualified=False)
if __name__=='__main__':
 r=json.loads(RECEIPT.read_text());commit=sys.argv[sys.argv.index('--evidence-commit')+1]if'--evidence-commit'in sys.argv else None;print(json.dumps(verify(r,commit),indent=2))
 if '--negative-self-test'in sys.argv:
  for field in ['source_sha256','file_sha256','historical_git_blobs']:
   bad=copy.deepcopy(r);bad[field][next(iter(bad[field]))]='0'*len(next(iter(bad[field].values())))
   try:verify(bad,commit)
   except ValueError as e:print('REJECTED '+field+': '+str(e))
   else:raise ValueError('corrupt binding accepted '+field)
