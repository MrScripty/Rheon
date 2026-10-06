"""Actual opt-in controller trace qualification, separate bounded build target."""
from pathlib import Path
import os,json,time,subprocess,hashlib
P=Path(__file__).resolve().parent;ROOT=P.parents[1];env=os.environ.copy();env['CARGO_TARGET_DIR']='/workspace/.rheon-tools/terminal-validation-trace-target';env['RUSTFLAGS']='--cfg rheon_newton_trace'
paths=subprocess.check_output(['git','ls-files','src','tests','examples','Cargo.toml','Cargo.lock','rust-toolchain.toml'],cwd=ROOT,text=True).splitlines();source={p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest()for p in paths}
record={'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'source_sha256':source,'RUSTFLAGS':env['RUSTFLAGS'],'commands':[]}
for name,argv in [('trace-clippy',['cargo','clippy','--locked','--no-default-features','--all-targets','--','-D','warnings']),('trace-boundary',['cargo','run','--locked','--release','--quiet','--example','solver_iteration_boundary']),('trace-finer',['cargo','run','--locked','--release','--quiet','--example','forcing_temporal_probe'])]:
 out=P/(name+('.jsonl'if name!='trace-clippy'else'.log'));err=P/(name+'-stderr.log')
 if out.exists()or err.exists():raise ValueError('refuse overwrite '+name)
 start=time.monotonic()
 with out.open('wb')as stdout,err.open('wb')as stderr:r=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
 record['commands'].append({'name':name,'argv':argv,'exit':r.returncode,'elapsed_seconds':time.monotonic()-start,'stdout':str(out.relative_to(ROOT)),'stderr':str(err.relative_to(ROOT))})
 record['status']='PASS'if r.returncode==0 else'FAIL';(P/'trace-receipt.json').write_text(json.dumps(record,indent=2)+'\n');print(name+' exit='+str(r.returncode),flush=True)
 if r.returncode:raise ValueError('unexpected actual trace result')
 for p,digest in source.items():
  if hashlib.sha256((ROOT/p).read_bytes()).hexdigest()!=digest:raise ValueError('source changed '+p)
