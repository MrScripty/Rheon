"""Clean original checkout + exact archived cache overlay; read-only native-free replay."""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
P=Path(__file__).resolve().parent
ROOT=P.parents[1]
def require(ok,msg):
    if not ok:
        raise ValueError(msg)
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def command(argv,cwd,env):
    done=subprocess.run(argv,cwd=cwd,env=env,capture_output=True)
    return done.returncode,done.stdout,done.stderr

def run(output_directory):
    provenance=json.loads((P/'provenance.json').read_text())
    for row in provenance['archives']:
        require(sha(ROOT/row['archive_path'])==row['sha256'], 'exact retained archive '+row['archive_path'])
    out=Path(output_directory).resolve()
    require(not out.is_relative_to(ROOT.resolve()), 'output must be outside this repository')
    require(not out.exists(),'exclusive external output; do not overwrite')
    out.mkdir(parents=True, exist_ok=False)
    temp=Path(tempfile.mkdtemp(prefix='case41-clean-cache-replay-',dir=out.parent))
    tree=temp/'original'
    subprocess.run(['git','worktree','add','--detach',str(tree),provenance['original_publication']],cwd=ROOT,check=True,capture_output=True)
    env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
    # No RUN/driver imports. Original preflight.run() returns a read-only result;
    # the CLI that would overwrite its old receipt is deliberately not used.
    original=tree/'evidence/case41-exploratory-seven-v1'
    policy=json.loads((original/'protocol.json').read_text())
    require(sha(original/'protocol.json')==provenance['original_protocol_sha256'],'exact historical protocol')
    missing=[f for f in policy['sha256'] if not (tree/f).exists()]
    require(set(missing)=={r['historical_path'] for r in provenance['archives']},'exact original committed omission')
    require(not list(tree.rglob('__pycache__')), 'clean checkout initially has no caches')
    def script(name):
        return [sys.executable,'-B',str(original/name)]
    preflight=[sys.executable,'-B','-c',"import importlib.util,json; from pathlib import Path; p=Path('evidence/case41-exploratory-seven-v1/preflight.py'); s=importlib.util.spec_from_file_location('original_preflight_readonly',p); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); print(json.dumps(m.run(),indent=2,sort_keys=True))"]
    code,stdout,stderr=command(script('analyze_capture_v2.py'),tree,env)
    (out/'original-clean-failure-stdout.log').write_bytes(stdout)
    (out/'original-clean-failure-stderr.log').write_bytes(stderr)
    require(code!=0 and b'FileNotFoundError' in stdout,'original clean checkout reproduces blocker')
    for row in provenance['archives']:
        dest=tree/row['historical_path'];dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/row['archive_path'],dest)
        require(sha(dest)==row['sha256'],'exact archive overlay')
    completed=[]
    for name,argv in [('capture-normal',script('analyze_capture_v2.py')),('capture-optimized',[sys.executable,'-B','-O',str(original/'analyze_capture_v2.py')]),('preflight-readonly',preflight),('verify-normal',script('verify.py')),('verify-optimized',[sys.executable,'-B','-O',str(original/'verify.py')])]:
        code,stdout,stderr=command(argv,tree,env)
        (out/(name+'.json')).write_bytes(stdout);(out/(name+'-stderr.log')).write_bytes(stderr)
        require(code==0 and not stderr,'clean replay '+name)
        completed.append({'name':name,'argv':argv,'exit':code,'stdout_sha256':hashlib.sha256(stdout).hexdigest()})
    require((out/'capture-normal.json').read_bytes()==(original/'analysis-v2-normal.json').read_bytes(),'exact published captured analysis reproduced')
    require((out/'capture-normal.json').read_bytes()==(out/'capture-optimized.json').read_bytes(),'reader normal/-O parity')
    require((out/'verify-normal.json').read_bytes()==(original/'verify-normal.json').read_bytes(),'exact original published integrity result reproduced')
    require((out/'verify-normal.json').read_bytes()==(out/'verify-optimized.json').read_bytes(),'verifier normal/-O parity')
    # Negative controls exercise original unconditional hashes; no amended bypass.
    controls=[(r['historical_path'],r['sha256']) for r in provenance['archives']]
    controls += [(f,policy['sha256'][f]) for f in ['evidence/case41-paired-observation-v1/check_scalar.py']]
    controls += [('evidence/case41-exploratory-seven-v1/native.log',sha(original/'native.log'))]
    negatives=[]
    for relative,digest in controls:
        target=tree/relative;raw=target.read_bytes();target.write_bytes(raw[:-1]+bytes([raw[-1]^1]))
        try:
            code,stdout,stderr=command(script('analyze_capture_v2.py'),tree,env)
            require(code!=0 and b'ValueError' in stdout,'negative hash rejected '+relative)
            if relative.endswith('native.log'):
                require(b'raw journal hashes' in stdout,'raw corruption hits raw hash')
            else:
                require(('original immutable input '+relative).encode() in stdout,'input corruption hits exact hash')
            index=len(negatives)
            (out/f'negative-{index}-stdout.log').write_bytes(stdout);(out/f'negative-{index}-stderr.log').write_bytes(stderr)
            negatives.append({'path':relative,'original_sha256':digest,'rejected':True,'exit':code})
        finally:
            target.write_bytes(raw)
        require(sha(target)==digest,'negative restored exact bytes')
    # Cache bytes still match after all source imports: writes disabled throughout.
    for row in provenance['archives']:
        require(sha(tree/row['historical_path'])==row['sha256'],'retained bytes not regenerated')
    require(not subprocess.check_output(['git','diff','--name-only'],cwd=tree),'original tracked tree unchanged')
    result=dict(status='PASS_CLEAN_ORIGINAL_REPLAY_WITH_EXACT_ARCHIVED_CACHES',original_publication=provenance['original_publication'],original_manifest_committed_complete=False,omitted_paths=missing,exact_archives=provenance['archives'],clean_worktree=str(tree),bytecode_writes_disabled=True,reader_published_output_identical=True,original_verifier_published_output_identical=True,checks=completed,negative_hash_controls=negatives,native_equations=0,native_reruns=0,corrections=0,owner_advances=0,historical_memory_qualified=False,ELF_scope='Original verifier reads retained same-host absolute ELF hash only; it never executes it. Cross-host users need that exact ELF for original verifier/preflight.',original_packet_modified=False)
    (out/'RESULTS.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    return result
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, help='New external output directory; must not exist or be inside the repository.')
    args=parser.parse_args()
    print(json.dumps(run(args.output),indent=2,sort_keys=True))
