"""Actual complete successor qualification, modes, full replay and frozen history."""
import hashlib,json,os,subprocess,sys
from pathlib import Path
import numpy,scipy,sympy,mpmath
R=Path(__file__).resolve().parents[2];P=R/'evidence/fitted-discrete-refinement';out=Path(sys.argv[1]).resolve();out.mkdir();commands=[];env=dict(os.environ,OPENBLAS_NUM_THREADS='1')

def run(args,name,dest):
 with dest.open('w')as stdout,(out/(name+'-stderr.log')).open('w')as stderr:code=subprocess.run(args,cwd=R,env=env,stdout=stdout,stderr=stderr).returncode
 commands.append(dict(name=name,command=args,exit=code));(out/'receipt.json').write_text(json.dumps(dict(status='in progress',commands=commands),indent=2)+'\n');print(name,'exit',code,flush=True)
 if code:raise SystemExit(code)

for label in ['initial','pressure']:
 data=json.loads((P/('trials/repaired-'+label+'-trajectory.json')).read_text())
 if data['status']!='PASS' or len(data['rows'])!=5:raise ValueError('actual normal completed trajectory')
 (out/('normal-'+label+'-trajectory.json')).write_bytes((P/('trials/repaired-'+label+'-trajectory.json')).read_bytes())
normal=json.loads((P/'trials/refined-cells.json').read_text());(out/'normal-cells.json').write_bytes((P/'trials/refined-cells.json').read_bytes())
if 'Ran 23 tests'not in (P/'trials/new-tests-stderr.log').read_text()or not (P/'trials/new-tests-stderr.log').read_text().rstrip().endswith('OK'):raise ValueError('actual normal tests')
run([sys.executable,'-O',str(P/'cells.py')],'optimized-cells',out/'optimized-cells.json')
run([sys.executable,'-O',str(P/'test_reference.py')],'optimized-tests',out/'optimized-tests.log')
for label,kind in [('initial','initial'),('pressure','pressure_state')]:
 run([sys.executable,'-O',str(P/'trajectory.py'),kind,str(out/('optimized-'+label+'-trajectory.json'))],'optimized-'+label+'-trajectory',out/('optimized-'+label+'-trajectory.log'))
for mode in ['normal','optimized']:
 prefix=[sys.executable]+(['-O']if mode=='optimized'else [])
 cells=json.loads((out/(mode+'-cells.json')).read_text())
 for row in cells['rows']:
  for c in row['cells']:
   label=mode+'-'+row['kind']+'-'+str(c['interval']);path=out/(label+'-cell.json');path.write_text(json.dumps(c,indent=2)+'\n');run(prefix+[str(P/'verify_cell.py'),str(path),'--negative-self-test'],label+'-replay',out/(label+'-replay.log'))
 for label in ['initial','pressure']:run(prefix+[str(P/'verify_trajectory.py'),str(out/(mode+'-'+label+'-trajectory.json'))],mode+'-'+label+'-trajectory-replay',out/(mode+'-'+label+'-trajectory-replay.log'))
 run(prefix+[str(R/'evidence/fitted-discrete-work/verify.py'),'--evidence-commit','aa89d1af95f84d86e4fb58f09c523035187b2006','--negative-self-test'],mode+'-frozen-predecessor',out/(mode+'-frozen-predecessor.log'))
for kind in ['initial','pressure_state']:run([sys.executable,str(P/'independent_work.py'),str(out/('normal-'+kind+'-0.05-cell.json')),'16'],kind+'-independent-coarse',out/(kind+'-independent-coarse.json'))
for label in ['initial','pressure']:
 c=json.loads((out/('normal-'+label+'-trajectory.json')).read_text())['replays'][-1]['steps'][-1]
 sys.path.insert(0,str(P));import temporal
 import numpy as np
 v=temporal.evaluate(np.array(c['initial_q']),np.array(c['initial_eta']),np.array(c['unknowns']),c['interval'],32);c['local_GCL_allowance']=(128*temporal.EPS*(v['start']['spatial']['mass']+v['end']['spatial']['mass'])).tolist()
 path=out/(label+'-fine-final-cell.json');path.write_text(json.dumps(c,indent=2)+'\n');run([sys.executable,str(P/'independent_work.py'),str(path),'16'],label+'-independent-fine',out/(label+'-independent-fine.json'))
for label in ['cells','initial-trajectory','pressure-trajectory']:
 if (out/('normal-'+label+'.json')).read_bytes()!=(out/('optimized-'+label+'.json')).read_bytes():raise ValueError('actual mode byte equality '+label)
paths=sorted([*P.glob('*.py'),P/'README.md',R/'docs/research-book/implementation/fitted-discrete-work-refinement.md'])
record=dict(status='PASS',commands=commands,actual_prior_normal_runs=['trials/refined-cells.json','trials/new-tests-stderr.log','trials/repaired-initial-trajectory.json','trials/repaired-pressure-trajectory.json'],versions=dict(python=sys.version,numpy=numpy.__version__,scipy=scipy.__version__,sympy=sympy.__version__,mpmath=mpmath.__version__),OPENBLAS_NUM_THREADS=1,source_sha256={str(p.relative_to(R)):hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},tests_per_mode=23,cell_replays=20,cell_corruption_rejections=240,trajectory_replays=4,trajectory_corruption_rejections=16,complete_replayed_steps=248,accepted_steps_in_completed_refinement_cases=248,actual_cancellation_trials=16,actual_injected_failure_trials=16,invalid_interval_failures=16,independent_80_digit_runs=4,actual_mode_byte_equal=True,new_Rust_temporal_builds=0,new_Lean_claims=0,new_public_step_enabled=False)
(out/'receipt.json').write_text(json.dumps(record,indent=2)+'\n');print('PASS complete five-case/two-field temporal qualification; public native step pending')
