"""Retain actual new-source executable and lifecycle checks for critical-path data."""
from pathlib import Path
import subprocess,json,os,time,hashlib,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1];env=os.environ.copy();target=Path('/workspace/.rheon-tools/finer-refusal-trace-target');env['CARGO_TARGET_DIR']=str(target);env['OPENBLAS_NUM_THREADS']='1'
if (ROOT/'target').exists()or(ROOT/'target').is_symlink():raise ValueError('do not overwrite existing build target alias')
(ROOT/'target').symlink_to(target,target_is_directory=True)
source=json.loads((P/'captures-receipt.json').read_text());record={'source_commit':source['source_commit'],'source_sha256':source['source_sha256'],'local_ignored_target_alias':str(target),'commands':[]}
cmds=[('core-dependency-tree',['cargo','tree','--locked','--no-default-features']),('executable-smoke',['cargo','run','--locked','--release','--bin','rheon','--','--size','16','--steps','12','--source-off-at','8','--output',str(P/'smoke')]),('executable-smoke-assertions',[sys.executable,str(P/'verify_smoke.py')]),('lifecycle-release-build',['cargo','build','--locked','--release','--bin','rheon']),('lifecycle-tests',[sys.executable,'-m','unittest','discover','-s','tools','-p','test_*.py','-v'])]
for name,argv in cmds:
 out=P/(name+'.log');err=P/(name+'-stderr.log')
 if out.exists()or err.exists():raise ValueError('refuse overwrite '+name)
 start=time.monotonic()
 with out.open('wb')as stdout,err.open('wb')as stderr:r=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
 record['commands'].append({'name':name,'argv':argv,'exit':r.returncode,'elapsed_seconds':time.monotonic()-start,'stdout':str(out.relative_to(ROOT)),'stderr':str(err.relative_to(ROOT))})
 record['status']='PASS'if r.returncode==0 else'FAIL';(P/'common-receipt.json').write_text(json.dumps(record,indent=2)+'\n');print(name+' exit='+str(r.returncode),flush=True)
 if r.returncode:raise ValueError('unexpected actual common check failure')
 for p,digest in source['source_sha256'].items():
  if hashlib.sha256((ROOT/p).read_bytes()).hexdigest()!=digest:raise ValueError('Rust source changed '+p)
