"""Verify the local uncommitted forcing evidence without promoting its limits."""
import copy,hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];P=ROOT/'evidence/forced-extruded-liquid';BASE='35b00247e68f04e6db539c365298862f82e1c2d0'
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(b):return hashlib.sha256(b).hexdigest()
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
def tree(commit):return {l.split(b'\t',1)[1].decode():l.split(b'\t',1)[0].split()[2].decode()for l in git('ls-tree','-r',commit).splitlines()}
def verify(r):
 require(r['status']=='LOCAL_PHYSICAL_REPLAY_COMPLETE_TEMPORAL_GATE_FAILED_IDENTITY_HOLD','honest local milestone status')
 require(r['qualification_source_is_uncommitted']and r['git_identity_hold'],'explicit uncommitted source and identity hold')
 require(r['binding_verifier_path']==str(Path(__file__).relative_to(ROOT)) and sha(Path(__file__).read_bytes())==r['binding_verifier_sha256'],'actual local binding verifier')
 require(r['git_checkpoint']==git('rev-parse','HEAD').decode().strip(),'no new commit during identity hold')
 require(r['historical_base']==BASE,'frozen reviewed baseline')
 base=tree(BASE);allowed={'src/coupled_discrete.rs','src/lib.rs'}
 require(r['historical_git_blobs']=={p:b for p,b in base.items()if p not in allowed},'complete frozen baseline inventory')
 for p,b in r['historical_git_blobs'].items():require(sha((ROOT/p).read_bytes())==sha(git('cat-file','blob',b)),'overwritten original source/evidence '+p)
 for p,d in r['source_sha256'].items():require(sha((ROOT/p).read_bytes())==d,'local source '+p)
 native='5bb0b5d538ac0b1707142cb9691e61994fd68b8e'
 require(r['compiled_native_source']==native and git('rev-parse',native+'^{tree}').decode().strip()=='79bdb50c5f98be9c7c9e86df05bc34f3165d3aeb','compiled source identity')
 matrix=json.loads((P/'native-preflight/receipt.json').read_text())
 require(matrix['status']=='PASS'and len(matrix['commands'])==7 and all(c['exit']==0 for c in matrix['commands']),'actual complete Rust matrix')
 for p,d in matrix['source_sha256'].items():require(r['source_sha256'][p]==d and sha(git('show',native+':'+p))==d,'compiled source bytes '+p)
 for p,d in r['file_sha256'].items():require(sha((ROOT/p).read_bytes())==d,'local packet '+p)
 actual={str(p.relative_to(ROOT))for p in P.rglob('*')if p.is_file()and '__pycache__'not in p.parts and p!=P/'local-root-receipt.json'}
 require(actual==set(r['file_sha256']),'complete local packet')
 qualified=json.loads((P/'local-qualification/receipt.json').read_text())
 require(qualified['status']==r['status']and qualified['source_sha256']==r['source_sha256']and qualified['qualification_source_is_uncommitted'],'actual local command binding')
 result=json.loads((P/'local-qualification/normal-diagnostic-replay.json').read_text())
 require(result['status']=='FAIL_ORIGINAL_TEMPORAL_BAND'and result['physical_equation_replay']=='PASS'and len(result['failed_temporal_bands'])==4 and len(result['references'])==16 and len(result['actual_corruption_rejections'])==19,'unpromoted temporal gate and actual physical replay')
 require((P/'local-qualification/normal-diagnostic-replay.json').read_bytes()==(P/'local-qualification/optimized-diagnostic-replay.json').read_bytes(),'actual mode parity')
 for mode in ['normal','optimized']:
  require('independently expected original-band first order initial nonconstant reversed geometry_max_error'in (P/f'local-qualification/{mode}-strict-temporal-refusal-stderr.log').read_text(),'actual strict refusal '+mode)
 require(all(c['exit']==c['expected_exit']for c in qualified['actual_commands']),'actual expected command results')
 require(sum(c['expected_exit']==1 for c in qualified['actual_commands'])==2,'two actual refused temporal qualifiers')
 require(qualified['legacy_planar_publications_byte_identical']and qualified['legacy_third_publications_byte_identical']and qualified['force_native_modes_identical'],'old and feature parity')
 require(r['browser_status']==qualified['browser_status'],'browser limit')
 return dict(status=r['status'],compiled_native_source=native,git_checkpoint=r['git_checkpoint'],qualification_source_is_uncommitted=True,source_paths=len(r['source_sha256']),packet_files=len(actual),preserved_baseline_paths=len(r['historical_git_blobs']),actual_publications=536,actual_corruption_rejections=19,temporal_gate='FAILED_ORIGINAL_BAND',browser_status=r['browser_status'])
if __name__=='__main__':
 r=json.loads((P/'local-root-receipt.json').read_text());print(json.dumps(verify(r),indent=2))
 if '--negative-self-test'in sys.argv:
  for field in ['source_sha256','file_sha256','historical_git_blobs']:
   bad=copy.deepcopy(r);bad[field][next(iter(bad[field]))]='0'*len(next(iter(bad[field].values())))
   try:verify(bad)
   except ValueError as e:print('REJECTED '+field+': '+str(e))
   else:raise ValueError('corrupt binding accepted '+field)
