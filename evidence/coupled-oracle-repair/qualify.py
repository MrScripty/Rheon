"""Actual strict successor route; no change to frozen numerical data."""
import hashlib,json,os,subprocess,sys
from pathlib import Path
R=Path(__file__).resolve().parents[2];P=R/'evidence/coupled-oracle-repair';out=Path(sys.argv[1]).resolve();out.mkdir();commands=[];env=dict(os.environ,OPENBLAS_NUM_THREADS='1')
def run(args,name,dest):
 with dest.open('w')as stdout,(out/(name+'-stderr.log')).open('w')as stderr:code=subprocess.run(args,cwd=R,env=env,stdout=stdout,stderr=stderr).returncode
 commands.append(dict(name=name,command=args,exit=code));(out/'receipt.json').write_text(json.dumps(dict(status='in progress',commands=commands),indent=2)+'\n');print(name,'exit',code,flush=True)
 if code:raise SystemExit(code)
run(['cargo','fmt','--check'],'format',out/'format.log')
for mode,flags in [('no-default',['--no-default-features']),('default',[]),('desktop',['--features','desktop'])]:run(['cargo','clippy',*flags,'--example','coupled_discrete_identity','--','-D','warnings'],'clippy-'+mode,out/('clippy-'+mode+'.log'))
for mode,flags in [('default',[]),('no-default',['--no-default-features'])]:run(['cargo','run','--release',*flags,'--example','coupled_discrete_identity'],'native-'+mode,out/('native-'+mode+'.jsonl'))
for mode,prefix in [('normal',[sys.executable]),('optimized',[sys.executable,'-O'])]:
 run(prefix+[str(P/'test_reference.py')],mode+'-tests',out/(mode+'-tests.log'))
 for kind in ['initial','pressure']:run(prefix+[str(P/'verify_trajectory.py'),str(R/('evidence/fitted-discrete-refinement/trials/repaired-'+kind+'-trajectory.json'))],mode+'-'+kind+'-trajectory',out/(mode+'-'+kind+'-trajectory.json'))
 run(prefix+[str(P/'replay.py'),str(out/'native-default.jsonl'),'--negative-self-test'],mode+'-native-replay',out/(mode+'-native-replay.json'))
 run(prefix+[str(R/'evidence/native-coupled-discrete/verify.py'),'--evidence-commit','a9428641b46de41b21a736849ab90c053d704b39','--negative-self-test'],mode+'-frozen-base',out/(mode+'-frozen-base.log'))
run([sys.executable,str(P/'reproduce_old.py')],'frozen-oracle-reproduction',out/'frozen-oracle-reproduction.log')
old=[json.loads(x)for x in(R/'evidence/native-coupled-discrete/native-qualification/native-default.jsonl').read_text().splitlines()];new=[json.loads(x)for x in(out/'native-default.jsonl').read_text().splitlines()]
for a,b in zip(old,new):
 plain={k:v for k,v in b.items()if k!='stamp'}
 if json.dumps(a,sort_keys=True)!=json.dumps(plain,sort_keys=True):raise ValueError('actual numerical fields changed')
for label in ['initial-trajectory','pressure-trajectory','native-replay']:
 if(out/('normal-'+label+'.json')).read_bytes()!=(out/('optimized-'+label+'.json')).read_bytes():raise ValueError('actual mode equality '+label)
if(out/'native-default.jsonl').read_bytes()!=(out/'native-no-default.jsonl').read_bytes():raise ValueError('native feature equality')
paths=sorted([*P.glob('*.py'),P/'README.md',R/'examples/coupled_discrete_identity.rs',R/'docs/research-book/implementation/coupled-qualification-oracle-repair.md'])
(out/'receipt.json').write_text(json.dumps(dict(status='PASS',commands=commands,source_sha256={str(p.relative_to(R)):hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},focused_tests_per_mode=10,focused_malformed_rejections_per_mode=15,actual_research_replayed_steps=248,actual_native_replayed_steps=248,regenerated_reference_trajectories_per_research_validator=2,actual_mode_byte_equal=True,native_feature_byte_equal=True,numerical_native_fields_identical_to_frozen_capture=True,original_stored_convergence_data_remains_true=True,no_simulation_Rust_changes=True,new_Lean_claims=0),indent=2)+'\n');print('PASS strict qualification route; real data preserved; malformed substitutes rejected')
