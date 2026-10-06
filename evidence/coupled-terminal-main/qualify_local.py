"""Exact-source local matrix, preserving actual outputs; shared debug dependencies only."""
from pathlib import Path
import subprocess,json,hashlib,os,time,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1];env=os.environ.copy();env['CARGO_TARGET_DIR']='/workspace/.rheon-tools/finer-refusal-trace-target'
# The frozen diagnosis source/evidence is untouched; only mutable debug build
# artifacts/dependencies are reused. Release baseline has its own target.
paths=subprocess.check_output(['git','ls-files','src','tests','examples','Cargo.toml','Cargo.lock','rust-toolchain.toml','.github/workflows/rust-rheon.yml'],cwd=ROOT,text=True).splitlines();source={p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest()for p in paths}
record={'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'source_sha256':source,'debug_target':env['CARGO_TARGET_DIR'],'commands':[],'status':'RUNNING'}
cmds=[('format',['cargo','fmt','--all','--check'])]
for mode,flags in [('default',[]),('core-only',['--no-default-features']),('desktop',['--features','desktop'])]:
 cmds += [('clippy-'+mode,['cargo','clippy','--locked',*flags,'--all-targets','--','-D','warnings']),('tests-'+mode,['cargo','test','--locked',*flags,'--all-targets']),('doc-tests-'+mode,['cargo','test','--locked',*flags,'--doc'])]
for name,argv in cmds:
 out=P/(name+'.log');err=P/(name+'-stderr.log')
 if out.exists()or err.exists():raise ValueError('refuse overwrite '+name)
 start=time.monotonic()
 with out.open('wb')as stdout,err.open('wb')as stderr:r=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
 record['commands'].append({'name':name,'argv':argv,'exit':r.returncode,'elapsed_seconds':time.monotonic()-start,'stdout':str(out.relative_to(ROOT)),'stderr':str(err.relative_to(ROOT))});record['status']='PASS'if r.returncode==0 else'FAIL'
 for p,d in source.items():
  if hashlib.sha256((ROOT/p).read_bytes()).hexdigest()!=d:raise ValueError('source changed '+p)
 (P/'local-matrix.json').write_text(json.dumps(record,indent=2)+'\n');print(name+' exit='+str(r.returncode),flush=True)
 if r.returncode:sys.exit(r.returncode)
