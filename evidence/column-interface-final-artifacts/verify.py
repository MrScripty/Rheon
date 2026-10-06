"""Bind the narrow final-artifact/CI repair and preserve all earlier evidence."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

P=Path(__file__).resolve().parent
ROOT=P.parents[1]
RECEIPT=P/'final-receipt.json'
BASE='35b00247e68f04e6db539c365298862f82e1c2d0'
ALLOWED={'.github/workflows/rust-rheon.yml','tools/verify_column_interface.py','tools/test_column_interface_verifier.py'}
def require(ok,message):
    if not ok:raise ValueError(message)
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT)
def sha(data):return hashlib.sha256(data).hexdigest()
def tree(commit):return {l.split(b'\t',1)[1].decode():l.split(b'\t',1)[0].split()[2].decode() for l in git('ls-tree','-r',commit).splitlines()}
def verify(r,evidence_commit=None):
    require(r['status']=='PASS_NARROW_REPAIR_LOCAL_QUALIFICATION','milestone status')
    source=r['qualified_source_commit'];actual=tree(source);base=tree(BASE)
    require(git('rev-parse',source+'^{tree}').decode().strip()==r['qualified_source_tree'],'source tree')
    require(git('show','-s','--format=%P',source).decode().split()==r['ordered_source_parents'],'source parents')
    require(r['historical_base']==BASE and set(r['legitimate_changed_baseline_paths'])==ALLOWED,'exact repair scope')
    require(r['historical_git_blobs']=={p:b for p,b in base.items() if p not in ALLOWED},'complete frozen baseline')
    require(all(actual.get(p)==b for p,b in r['historical_git_blobs'].items()),'overwritten frozen source/evidence')
    require({p for p in base if actual.get(p)!=base[p]}==ALLOWED,'actual repair delta')
    for p,d in r['source_sha256'].items():
        require(p in actual and sha(git('show',source+':'+p))==d,'Git source '+p)
        require(sha((ROOT/p).read_bytes())==d,'working source '+p)
    evidence=tree(evidence_commit) if evidence_commit else None
    prefix=str(P.relative_to(ROOT))+'/'
    files={p for p in evidence if p.startswith(prefix) and p!=str(RECEIPT.relative_to(ROOT))} if evidence is not None else {str(p.relative_to(ROOT)) for p in P.rglob('*') if p.is_file() and '__pycache__' not in p.parts and p!=RECEIPT}
    require(files==set(r['file_sha256']),'complete evidence inventory')
    for p,d in r['file_sha256'].items():require(sha(git('show',evidence_commit+':'+p) if evidence else (ROOT/p).read_bytes())==d,'evidence '+p)
    require(sha((P/'trials/frozen_verifier.py').read_bytes())==sha(git('show',BASE+':tools/verify_column_interface.py')),'actual frozen verifier')
    frozen=(P/'trials/frozen-workflow.yml').read_text();current=(ROOT/'.github/workflows/rust-rheon.yml').read_text()
    require(frozen==git('show',BASE+':.github/workflows/rust-rheon.yml').decode(),'actual frozen workflow')
    require(current.split('jobs:\n',1)[1].replace('timeout-minutes: 30','timeout-minutes: 15')==frozen.split('jobs:\n',1)[1],'all original CI checks retained')
    for mode in ['normal','optimized']:
        tests=(P/f'qualification/{mode}-tests-stderr.log').read_text()
        require('Ran 5 tests' in tests and tests.rstrip().endswith('OK'),'actual five test methods '+mode)
        old=json.loads((P/f'qualification/{mode}-old-defects.json').read_text())
        require(old['status']=='REPRODUCED_OLD_ACCEPTANCE_DEFECT' and len(old['controls'])==3 and all(c['old_status']=='ACCEPTED' for c in old['controls']),'actual old acceptance witnesses')
        limits=json.loads((P/f'qualification/{mode}-scope-witnesses.json').read_text())
        require(len(limits['controls'])==2 and all(c['status']=='ACCEPTED_SCOPE_LIMIT' for c in limits['controls']) and not limits['closed_by_final_artifact_repair'] and not limits['closed_by_newer_coupled_replay'],'remaining limits preserved')
        reader=json.loads((P/f'qualification/{mode}-reader.json').read_text())
        require(len(reader['results'])==13 and sum(x.get('coupled_intervals',0) for x in reader['results'])==64,'actual numerical reader')
    for name in ['reader','old-defects','scope-witnesses']:
        require((P/f'qualification/normal-{name}.json').read_bytes()==(P/f'qualification/optimized-{name}.json').read_bytes(),'mode parity '+name)
    figure=json.loads((P/'verified-figure/receipt.json').read_text())
    require(figure['status']=='RENDERED_AFTER_FULL_FINAL_ARTIFACT_VERIFICATION' and not figure['historical_artifacts_overwritten'],'verified figure only')
    for p,d in figure['copied_native_file_sha256'].items():require(sha((ROOT/'evidence/column-interface'/p).read_bytes())==d,'render native input '+p)
    for p,d in figure['output_sha256'].items():require(sha((P/'verified-figure'/p).read_bytes())==d,'render output '+p)
    remaining=json.loads((P/'remaining-ci/receipt.json').read_text())
    require(remaining['status']=='PASS_LOCAL_REMAINING_STAGES' and len(remaining['commands'])==2 and all(c['exit']==0 for c in remaining['commands']) and remaining['warm_cache'] and remaining['hosted_CI_qualified'] is False,'actual local remaining stages')
    lifecycle=(P/'remaining-ci/lifecycle-stderr.log').read_text()
    require('Ran 37 tests' in lifecycle and lifecycle.rstrip().endswith('OK'),'actual complete lifecycle suite')
    require(r['hosted_CI_qualified'] is False and r['remaining_scope_limits']==2,'unqualified hosted run and older model limits')
    return dict(status='PASS_NARROW_REPAIR_LOCAL_QUALIFICATION',source=source,tree=r['qualified_source_tree'],preserved_baseline_paths=len(r['historical_git_blobs']),source_paths=len(r['source_sha256']),packet_files=len(files),corruption_controls_per_mode=80,ci_path_controls=52,remaining_scope_limits=2,local_lifecycle_methods=37,hosted_CI_qualified=False)
if __name__=='__main__':
    r=json.loads(RECEIPT.read_text());commit=sys.argv[sys.argv.index('--evidence-commit')+1] if '--evidence-commit' in sys.argv else None
    print(json.dumps(verify(r,commit),indent=2))
    if '--negative-self-test' in sys.argv:
        for field in ['source_sha256','file_sha256','historical_git_blobs']:
            bad=copy.deepcopy(r);bad[field][next(iter(bad[field]))]='0'*len(next(iter(bad[field].values())))
            try:verify(bad,commit)
            except ValueError as e:print('REJECTED '+field+': '+str(e))
            else:raise ValueError('corrupt binding accepted '+field)
