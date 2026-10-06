"""Qualify cancellation/budget/publication contracts under actual release optimization."""
from pathlib import Path
import subprocess,time,os,json
P=Path(__file__).resolve().parent;ROOT=P.parents[1];env=os.environ.copy();env['CARGO_TARGET_DIR']='/workspace/.rheon-tools/finer-refusal-trace-target'
argv=['cargo','test','--locked','--release','--test','solver_iteration_boundary_contract'];out=P/'release-focused.log';err=P/'release-focused-stderr.log'
if out.exists()or err.exists():raise ValueError('do not overwrite actual release contract run')
start=time.monotonic()
with out.open('wb')as stdout,err.open('wb')as stderr:r=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
source=json.loads((P/'captures-receipt.json').read_text());record={'source_commit':source['source_commit'],'source_sha256':source['source_sha256'],'command':{'argv':argv,'exit':r.returncode,'elapsed_seconds':time.monotonic()-start,'stdout':str(out.relative_to(ROOT)),'stderr':str(err.relative_to(ROOT))},'status':'PASS'if r.returncode==0 else'FAIL'}
(P/'release-focused-receipt.json').write_text(json.dumps(record,indent=2)+'\n');print(record['status'],flush=True)
if r.returncode:raise ValueError('actual release contract failed')
