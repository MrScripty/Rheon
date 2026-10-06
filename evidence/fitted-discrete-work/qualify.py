"""Budget-bounded verification of actual normal experiments, modes and replay."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import numpy,scipy,sympy,mpmath
ROOT=Path(__file__).resolve().parents[2];P=ROOT/'evidence/fitted-discrete-work';out=Path(sys.argv[1]).resolve();out.mkdir();commands=[]

def run(args,name,dest):
 with dest.open('w')as stdout,(out/(name+'-stderr.log')).open('w')as stderr:
  code=subprocess.run(args,cwd=ROOT,stdout=stdout,stderr=stderr).returncode
 commands.append(dict(name=name,command=args,exit=code));print(name,'exit',code,flush=True)
 (out/'receipt.json').write_text(json.dumps(dict(status='in progress',commands=commands),indent=2)+'\n')
 if code:raise SystemExit(code)

normal=json.loads((P/'trials/final-cells.json').read_text());trajectory=json.loads((P/'trials/partial-trajectory.json').read_text())
if 'Ran 17 tests'not in (P/'trials/final-tests-stderr.log').read_text()or not (P/'trials/final-tests-stderr.log').read_text().rstrip().endswith('OK'):raise ValueError('actual normal tests')
(out/'normal-cells.json').write_bytes((P/'trials/final-cells.json').read_bytes());(out/'normal-trajectory.json').write_bytes((P/'trials/partial-trajectory.json').read_bytes())
for row in normal['rows']:
 for c in row['cells']:(out/(row['kind']+'-'+str(c['interval'])+'-cell.json')).write_text(json.dumps(c,indent=2)+'\n')
for optimized in [False,True]:
 mode='optimized'if optimized else 'normal';prefix=[sys.executable]+(['-O']if optimized else [])
 for kind in ['initial','pressure_state']:
  for h in [.05,.025,.0125]:
   label=mode+'-'+kind+'-'+str(h)+'-replay';run(prefix+[str(P/'verify_cell.py'),str(out/(kind+'-'+str(h)+'-cell.json')),'--negative-self-test'],label,out/(label+'.log'))
 run(prefix+[str(ROOT/'evidence/fitted-coupled-temporal/verify.py'),'--evidence-commit','c6c88365c1f52ed9dc0644e2ed48e64b3e24d567','--negative-self-test'],mode+'-frozen-predecessor',out/(mode+'-frozen-predecessor.log'))
run([sys.executable,'-O',str(P/'test_reference.py')],'optimized-tests',out/'optimized-tests.log')
for kind in ['initial','pressure_state']:
 run([sys.executable,str(P/'independent_work.py'),str(out/(kind+'-0.05-cell.json')),'16'],kind+'-independent',out/(kind+'-independent.json'))
rollback=json.loads((P/'trials/final-rollback.json').read_text())
if len(rollback['checks'])!=8 or not all(c['accepted_state_unchanged']for c in rollback['checks']):raise ValueError('actual rollback trials')
(out/'normal-rollback.json').write_bytes((P/'trials/final-rollback.json').read_bytes())
(out/'normal-reference.json').write_bytes((P/'trials/final-reference.json').read_bytes())
if 'Ran 17 tests'not in (out/'optimized-tests-stderr.log').read_text():raise ValueError('actual optimized tests')
paths=sorted([*P.glob('*.py'),P/'README.md',ROOT/'docs/research-book/implementation/fitted-discrete-work-temporal.md'])
record=dict(status='PASS_CHECKPOINT_WITH_REJECTED_REFINEMENT',commands=commands,actual_prior_normal_commands=[dict(command='python evidence/fitted-discrete-work/cells.py',stdout='trials/final-cells.json',stderr='trials/final-cells-stderr.log',exit=0),dict(command='python evidence/fitted-discrete-work/test_reference.py',stdout='trials/final-tests.log',stderr='trials/final-tests-stderr.log',exit=0),dict(command='python evidence/fitted-discrete-work/trajectory.py',stdout='trials/bounded-trajectory.json',stderr='trials/bounded-trajectory-stderr.log',exit=1,expected_outcome='retained failed fourth refinement; not a passing qualification'),dict(command='python rollback_checks()',stdout='trials/final-rollback.json',stderr='trials/final-rollback-stderr.log',exit=0),dict(command='python reference()',stdout='trials/final-reference.json',stderr='trials/final-reference-stderr.log',exit=0)],source_sha256={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},versions=dict(python=sys.version,numpy=numpy.__version__,scipy=scipy.__version__,sympy=sympy.__version__,mpmath=mpmath.__version__),tests_per_mode=17,actual_cell_replays=12,cell_corruption_rejections=144,trajectory_corruption_rejections=0,actual_completed_case_steps=trajectory['actual_completed_case_steps'],repeated_accuracy_qualification=False,failed_refinement_interval=.00625,actual_cancellation_trials=4,actual_injected_failure_trials=4,invalid_interval_failures=4,independent_80_digit_runs=2,new_Rust_temporal_builds=0,new_Lean_claims=0,new_public_step_enabled=False,limits=['Normal actual nonlinear/rollback experiment, three completed refinement cases and retained fourth failure; normal and optimized replay and tests. No optimized re-solve/refinement run or byte-equality claim.','Path root discovery, quadrature and repeated geometry/rank checks are numerical; no new whole-interval sign/rank certificate.','Measured host time accuracy on the same fitted spatial model; no continuum or native publication claim.'])
(out/'receipt.json').write_text(json.dumps(record,indent=2)+'\n');print('PASS checkpoint verification: single-cell work and rollback; finer repeated refinement REJECTED; public step pending')
