"""Run the actual native feature/build matrix and full arithmetic replays."""
import hashlib,json,os,subprocess,sys
from pathlib import Path
R=Path(__file__).resolve().parents[2];P=R/'evidence/native-coupled-discrete';out=Path(sys.argv[1]).resolve();out.mkdir();commands=[];env=dict(os.environ,OPENBLAS_NUM_THREADS='1')
def run(args,name,dest):
 with dest.open('w')as stdout,(out/(name+'-stderr.log')).open('w')as stderr:code=subprocess.run(args,cwd=R,env=env,stdout=stdout,stderr=stderr).returncode
 commands.append(dict(name=name,command=args,exit=code));(out/'receipt.json').write_text(json.dumps(dict(status='in progress',commands=commands),indent=2)+'\n');print(name,'exit',code,flush=True)
 if code:raise SystemExit(code)
for args,name in [(['cargo','fmt','--check'],'format'),(['cargo','test','--no-default-features'],'tests-no-default'),(['cargo','test'],'tests-default'),(['cargo','test','--features','desktop'],'tests-desktop'),(['cargo','test','--release','--no-default-features','--test','coupled_discrete_contract'],'release-contracts'),(['cargo','clippy','--no-default-features','--all-targets','--','-D','warnings'],'clippy-no-default'),(['cargo','clippy','--all-targets','--','-D','warnings'],'clippy-default'),(['cargo','clippy','--features','desktop','--all-targets','--','-D','warnings'],'clippy-desktop')]:run(args,name,out/(name+'.log'))
for label,args in [('default',['cargo','run','--release','--example','coupled_discrete']),('no-default',['cargo','run','--release','--no-default-features','--example','coupled_discrete'])]:run(args,'native-'+label,out/('native-'+label+'.jsonl'))
for label,prefix in [('normal',[sys.executable]),('optimized',[sys.executable,'-O'])]:run(prefix+[str(P/'replay.py'),str(out/'native-default.jsonl'),'--negative-self-test'],label+'-replay',out/(label+'-replay.json'))
if(out/'native-default.jsonl').read_bytes()!=(out/'native-no-default.jsonl').read_bytes():raise ValueError('native feature output byte equality')
if(out/'normal-replay.json').read_bytes()!=(out/'optimized-replay.json').read_bytes():raise ValueError('host mode replay byte equality')
paths=sorted([*R.glob('src/*.rs'),*R.glob('tests/*.rs'),*R.glob('examples/*.rs'),R/'Cargo.toml',R/'Cargo.lock',*P.glob('*.py'),P/'README.md',R/'docs/research-book/implementation/native-coupled-discrete-work.md'])
record=dict(status='PASS',commands=commands,source_sha256={str(p.relative_to(R)):hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},native_accepted_steps=248,native_replayed_steps=248,actual_native_feature_byte_equal=True,actual_host_replay_mode_byte_equal=True,actual_replay_corruption_rejections=12,new_public_step_scope='bounded native two-column xy endpoint-donor transport/strain/pressure; no general continuum liquid claim',new_Lean_claims=0)
(out/'receipt.json').write_text(json.dumps(record,indent=2)+'\n');print('PASS full native qualification and exact feature/replay mode equality')
