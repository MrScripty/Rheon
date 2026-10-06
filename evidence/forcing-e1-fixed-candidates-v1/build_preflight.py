"""Compile frozen private harness and run preflight only; never enable E1."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,os,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,message):
 if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
policy=json.loads((P/'protocol.json').read_text());source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip();tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=ROOT).decode().strip()
for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen source '+path)
require(not (P/'binary-binding.json').exists(),'never overwrite a compiled preflight')
env=os.environ.copy();env.pop('RHEON_PUBLIC_CALL_ALLOW',None);env.pop('RHEON_CASE_INDEX',None);env['CARGO_TARGET_DIR']=policy['target'];env['RUSTFLAGS']=policy['rustflags'];env['CARGO_BUILD_JOBS']='1'
commands=[]
def run(label,argv):
 start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 (P/(label+'.log')).write_bytes(result.stdout);(P/(label+'-stderr.log')).write_bytes(result.stderr)
 commands.append({'argv':argv,'exit':result.returncode,'seconds':time.monotonic()-start,'stdout':label+'.log','stderr':label+'-stderr.log','stdout_sha256':sha(result.stdout),'stderr_sha256':sha(result.stderr)})
 (P/'preflight-commands.json').write_text(json.dumps({'source':source,'source_tree':tree,'commands':commands,'generalization_executed':False},indent=2,sort_keys=True)+'\n')
 require(result.returncode==0,'genuine preflight failure '+label);return result.stdout
run('format',['rustfmt','--edition','2024','--check','--config','skip_children=true',*[str(P/n) for n in ['harness.rs','inputs.rs','trajectory_inputs.rs','fixed_inputs.rs','candidate_probe.rs','reference_probe.rs','candidate.rs','reference.rs','scalar.rs','candidate_helpers.rs','reference_helpers.rs']]])
run('install',[sys.executable,str(P/'prepare.py'),'install'])
try:
 data=run('build',['cargo','test','--release','--no-default-features','--lib','--locked','--no-run','--message-format=json'])
 rows=[json.loads(l) for l in data.splitlines() if l.startswith(b'{')];binaries=[r['executable'] for r in rows if r.get('reason')=='compiler-artifact' and r.get('executable') and r.get('profile',{}).get('test') and r['target']['name']=='rheon'];require(len(binaries)==1,'one actual core test binary');binary=binaries[0]
 run('clippy',['cargo','clippy','--release','--no-default-features','--lib','--tests','--locked','--','-D','warnings'])
 for label,test in [('scalar-native','research_public_scalar_preflight'),('type-layout','research_public_type_layout'),('reference-fixed-native','fixed_capture_reference_preflight')]:
  run(label,[binary,'--exact','research_public_call::'+test,'--nocapture'])
 spec=importlib.util.spec_from_file_location('new_exact_scalar',P/'check_scalar.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);oracle=m.run(P/'scalar-native.log');require(oracle['native_assertion_groups']==23,'all native scalar groups');(P/'scalar-oracle.json').write_text(json.dumps(oracle,indent=2,sort_keys=True)+'\n')
 raw=subprocess.check_output(['objdump','-d','-C',binary]);(P/'native-disassembly.txt.gz').write_bytes(gzip.compress(raw,mtime=0));(P/'native-disassembly.txt.sha256').write_text(sha(raw)+'\n')
 for name,argv in [('symbols',['nm','-S','-nC',binary]),('relocations',['readelf','-rW',binary]),('headers',['readelf','-hW',binary])]: (P/('native-'+name+'.txt')).write_bytes(subprocess.check_output(argv))
 binding={'prototype':source,'prototype_tree':tree,'binary':binary,'binary_sha256':sha(Path(binary).read_bytes()),'protocol_sha256':sha((P/'protocol.json').read_bytes()),'rustflags':policy['rustflags'],'target':policy['target'],'generalization_executed':False};(P/'binary-binding.json').write_text(json.dumps(binding,indent=2,sort_keys=True)+'\n')
finally:run('restore',[sys.executable,str(P/'prepare.py'),'restore'])
print(json.dumps({'status':'PASS_NEW_BINARY_SCALAR_LAYOUT_BASELINE_CLIPPY','source':source,'binary_sha256':binding['binary_sha256'],'memory_audit_still_required':True,'generalization_executed':False}))
