"""Actual precommit qualification under an explicit author-identity hold.

No Git mutations. Strict temporal refusal is required and remains a failure;
diagnostic completion is not a temporal qualification pass.
"""
import hashlib,json,os,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent;ROOT=P.parents[1];OUT=P/'local-qualification';OUT.mkdir(exist_ok=True)
phase=sys.argv[1];env=dict(os.environ,OPENBLAS_NUM_THREADS='1',MPLCONFIGDIR='/tmp/rheon-forced-render-cache',XDG_CACHE_HOME='/tmp/rheon-forced-font-cache')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
records=[]
def require(ok,message):
 if not ok:raise ValueError(message)
def manifest():
 inherited=json.loads((P.parent/'extruded-third-velocity/final-receipt.json').read_text())['source_sha256']
 names=set(inherited)|{'tests/forced_extruded_contract.rs','examples/forced_extruded.rs','docs/research-book/implementation/forced-extruded-liquid-proposal.md','docs/research-book/implementation/native-forced-extruded-liquid.md'}|{str(p.relative_to(ROOT))for p in P.glob('*.py')}|{str(P.relative_to(ROOT))+'/check_viewer.cjs'}
 return {n:sha(ROOT/n)for n in sorted(names)}
source=dict(compiled_native_source='5bb0b5d538ac0b1707142cb9691e61994fd68b8e',git_checkpoint=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),qualification_source_is_uncommitted=True,git_identity_hold=True,source_sha256=manifest())
def run(name,argv,suffix='log',expected_exit=0):
 output=OUT/(name+'.'+suffix);error=OUT/(name+'-stderr.log')
 require(not output.exists() and not error.exists(),'refusing to overwrite actual command '+name)
 with output.open('wb')as stdout,error.open('wb')as stderr:answer=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
 records.append(dict(name=name,command=argv,exit=answer.returncode,expected_exit=expected_exit,stdout=str(output.relative_to(ROOT)),stderr=str(error.relative_to(ROOT))))
 (OUT/(phase+'-commands.json')).write_text(json.dumps(dict(**source,commands=records,status='PASS_EXPECTED_COMMAND_RESULTS'if all(x['exit']==x['expected_exit']for x in records)else'FAIL'),indent=2)+'\n')
 require(answer.returncode==expected_exit,'actual unexpected command exit '+name);print(name+' actual exit '+str(answer.returncode),flush=True)
if phase=='captures':
 run('native-default',['cargo','run','--locked','--release','--quiet','--example','forced_extruded'],'jsonl')
 run('native-no-default',['cargo','run','--locked','--release','--quiet','--no-default-features','--example','forced_extruded'],'jsonl')
 run('legacy-planar',['cargo','run','--locked','--release','--quiet','--example','coupled_discrete_identity'],'jsonl')
 run('legacy-third',['cargo','run','--locked','--release','--quiet','--example','extruded_third'],'jsonl')
elif phase in ['normal','optimized']:
 python=[sys.executable]+(['-O']if phase=='optimized'else[])
 run(phase+'-schema-tests',python+[str(P/'test_replay.py')])
 run(phase+'-strict-temporal-refusal',python+[str(P/'replay.py'),str(P/'native-trials/first-native.jsonl')],expected_exit=1)
 require('independently expected original-band first order initial nonconstant reversed geometry_max_error'in (OUT/(phase+'-strict-temporal-refusal-stderr.log')).read_text(),'actual expected temporal reason')
 run(phase+'-diagnostic-replay',python+[str(P/'replay.py'),str(P/'native-trials/first-native.jsonl'),'--negative-self-test','--diagnostic-refinement'],'json')
elif phase=='render':
 run('render',[sys.executable,str(P/'render_native.py'),str(P/'native-trials/first-native.jsonl'),str(OUT/'renders')])
 run('viewer-vm',['node',str(P/'check_viewer.cjs'),str(OUT/'renders/published-forced.html'),str(P/'native-trials/first-native.jsonl')],'json')
 run('refinement-figure',[sys.executable,str(P/'plot_refinement.py'),str(OUT/'normal-diagnostic-replay.json'),str(OUT/'refinement.png')])
 run('browser',[sys.executable,str(P.parent/'extruded-third-velocity/check_browser.py'),str(OUT/'renders/published-forced.html')],'json')
elif phase=='checks':
 stages=[json.loads((OUT/(x+'-commands.json')).read_text())for x in ['captures','normal','optimized','render']]
 require(all(s['source_sha256']==source['source_sha256']and s['qualification_source_is_uncommitted']and s['git_identity_hold']and s['status']=='PASS_EXPECTED_COMMAND_RESULTS'for s in stages),'actual source stability during commands')
 require((OUT/'native-default.jsonl').read_bytes()==(OUT/'native-no-default.jsonl').read_bytes()==(P/'native-trials/first-native.jsonl').read_bytes(),'native publication parity')
 require((OUT/'legacy-planar.jsonl').read_bytes()==(P.parent/'coupled-oracle-repair/oracle-qualification/native-default.jsonl').read_bytes(),'old planar publication parity')
 require((OUT/'legacy-third.jsonl').read_bytes()==(P.parent/'extruded-third-velocity/final-qualification/native-default.jsonl').read_bytes(),'old third publication parity')
 require((OUT/'normal-diagnostic-replay.json').read_bytes()==(OUT/'optimized-diagnostic-replay.json').read_bytes(),'normal optimized numerical parity')
 result=json.loads((OUT/'normal-diagnostic-replay.json').read_text())
 require(result['status']=='FAIL_ORIGINAL_TEMPORAL_BAND'and result['physical_equation_replay']=='PASS'and len(result['failed_temporal_bands'])==4 and len(result['actual_corruption_rejections'])==19,'observed temporal limitation and physical gates')
 matrix=json.loads((P/'native-preflight/receipt.json').read_text());require(matrix['status']=='PASS'and len(matrix['commands'])==7 and all(c['exit']==0 for c in matrix['commands']),'actual native matrix')
 for n,d in matrix['source_sha256'].items():require(source['source_sha256'][n]==d,'reused compiled source '+n)
 viewer=json.loads((OUT/'viewer-vm.json').read_text());require(viewer['status']=='PASS'and viewer['publications']==536 and viewer['cases']==40,'actual viewer execution')
 browser=json.loads((OUT/'browser.json').read_text());require(browser['status']in ['PASS','BLOCKED_BROWSER_SANDBOX'],'explicit browser outcome')
 r=dict(**source,status='LOCAL_PHYSICAL_REPLAY_COMPLETE_TEMPORAL_GATE_FAILED_IDENTITY_HOLD',actual_commands=[c for s in stages for c in s['commands']],actual_publications=536,actual_constructors=40,actual_steps=496,physical_equation_replay='PASS',temporal_qualification=result['status'],failed_temporal_bands=result['failed_temporal_bands'],actual_corruption_rejections=19,normal_optimized_replays_byte_identical=True,legacy_planar_publications_byte_identical=True,legacy_third_publications_byte_identical=True,force_native_modes_identical=True,browser_status=browser['status'],browser_rasterization_qualified=browser['status']=='PASS',maxima=result['maxima'],file_sha256={str(p.relative_to(ROOT)):sha(p)for p in sorted(OUT.rglob('*'))if p.is_file()})
 (OUT/'receipt.json').write_text(json.dumps(r,indent=2)+'\n');print(r['status'])
else:raise ValueError('unknown phase')
