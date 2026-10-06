"""Sequential exact-source local qualification; preserves every actual log."""
from pathlib import Path
import subprocess,os,json,time,hashlib,sys
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
env=os.environ.copy();env['CARGO_TARGET_DIR']='/workspace/.rheon-tools/finer-refusal-trace-target';env['OPENBLAS_NUM_THREADS']='1'
phase=sys.argv[1]; cmds=[]
if phase=='matrix':
 cmds=[('format',['cargo','fmt','--all','--check'])]
 for label,flags in [('default',[]),('core-only',['--no-default-features']),('desktop',['--features','desktop'])]:
  cmds.extend([(f'clippy-{label}',['cargo','clippy','--locked',*flags,'--all-targets','--','-D','warnings']),(f'tests-{label}',['cargo','test','--locked',*flags,'--all-targets']),(f'doc-tests-{label}',['cargo','test','--locked',*flags,'--doc'])])
elif phase=='captures':
 for label,flags in [('default',[]),('core-only',['--no-default-features'])]:
  for example in ['solver_iteration_boundary','forcing_temporal_probe','forced_extruded']:
   cmds.append((example+'-'+label,['cargo','run','--locked','--release','--quiet',*flags,'--example',example]))
else:raise ValueError('unknown phase')
source={p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in subprocess.check_output(['git','ls-files','src','tests','examples','Cargo.toml','Cargo.lock','rust-toolchain.toml'],cwd=ROOT,text=True).splitlines()}
record={'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'source_sha256':source,'commands':[],'status':'RUNNING'}
for name,argv in cmds:
 out=P/(name+'.'+('jsonl'if phase=='captures'else'log'));err=P/(name+'-stderr.log')
 if out.exists()or err.exists():raise ValueError('refuse overwrite '+name)
 start=time.monotonic()
 with out.open('wb')as stdout,err.open('wb')as stderr: result=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
 record['commands'].append({'name':name,'argv':argv,'exit':result.returncode,'elapsed_seconds':time.monotonic()-start,'stdout':str(out.relative_to(ROOT)),'stderr':str(err.relative_to(ROOT))})
 record['status']='PASS'if result.returncode==0 else'FAIL'
 for path,digest in source.items():
  if hashlib.sha256((ROOT/path).read_bytes()).hexdigest()!=digest:raise ValueError('source changed '+path)
 (P/(phase+'-receipt.json')).write_text(json.dumps(record,indent=2)+'\n')
 print(name+' exit='+str(result.returncode),flush=True)
 if result.returncode:sys.exit(result.returncode)
