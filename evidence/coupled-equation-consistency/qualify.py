"""Actual mode-qualified exact derivation, physical limits and finer refusals."""
import hashlib,json,os,subprocess,sys
from pathlib import Path
R=Path(__file__).resolve().parents[2];P=R/'evidence/coupled-equation-consistency';out=Path(sys.argv[1]).resolve();out.mkdir();commands=[];env=dict(os.environ,OPENBLAS_NUM_THREADS='1')
def run(args,name,dest):
 with dest.open('w')as stdout,(out/(name+'-stderr.log')).open('w')as stderr:code=subprocess.run(args,cwd=R,env=env,stdout=stdout,stderr=stderr).returncode
 commands.append(dict(name=name,command=args,exit=code));(out/'receipt.json').write_text(json.dumps(dict(status='in progress',commands=commands),indent=2)+'\n');print(name,'exit',code,flush=True)
 if code:raise SystemExit(code)
run(['cargo','fmt','--check'],'unchanged-rust-format',out/'unchanged-rust-format.log')
for mode,prefix in [('normal',[sys.executable]),('optimized',[sys.executable,'-O'])]:
 for name in ['taylor','test_reference','study']:run(prefix+[str(P/(name+'.py'))],mode+'-'+name,out/(mode+'-'+name+('.json'if name!='test_reference'else'.log')))
 run(prefix+[str(P/'finer.py'),str(out/(mode+'-finer.json'))],mode+'-finer',out/(mode+'-finer.log'))
 run(prefix+[str(R/'evidence/native-coupled-discrete/verify.py'),'--evidence-commit','a9428641b46de41b21a736849ab90c053d704b39','--negative-self-test'],mode+'-frozen-base',out/(mode+'-frozen-base.log'))
for name in ['taylor','study','finer']:
 if(out/('normal-'+name+'.json')).read_bytes()!=(out/('optimized-'+name+'.json')).read_bytes():raise ValueError('actual mode byte equality '+name)
finer=json.loads((out/'normal-finer.json').read_text());expected=[('initial',.0015625,'PASS'),('initial',.00078125,'REJECTED'),('pressure_state',.0015625,'PASS'),('pressure_state',.00078125,'PASS')]
if[(r['kind'],r['interval'],r['status'])for r in finer['records']]!=expected:raise ValueError('retained actual finer qualification statuses')
paths=sorted([*P.glob('*.py'),P/'README.md',R/'docs/research-book/implementation/coupled-equation-consistency.md'])
(out/'receipt.json').write_text(json.dumps(dict(status='PASS_WITH_RETAINED_ARITHMETIC_AND_INTERPRETATION_COUNTEREXAMPLES',commands=commands,source_sha256={str(p.relative_to(R)):hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},tests_per_mode=7,independent_exact_tangent_and_derivative_fields=2,actual_observed_saved_cells_per_mode=10,actual_finer_attempts_per_mode=4,actual_accepted_finer_cells_per_mode=3,actual_refused_finer_cells_per_mode=1,actual_mode_byte_equal=True,physical_or_numerical_allowances_relaxed=0,new_Rust_changes=0,new_Lean_claims=0,pressure_first_difference_derivative_diagnostic_pass=False,original_coarse_pressure_flux_diagnostic_pass=False,next_bounded_physics_only_proposed=True),indent=2)+'\n');print('PASS conditional equation limit and expected coefficients; retained finer/refused/evolved-pressure counterexamples')
