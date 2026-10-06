"""Compile/Clippy only. No test ELF invocation and no native equation execution."""
from pathlib import Path
import gzip,hashlib,json,os,subprocess,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1];N=P/'native'
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
policy=json.loads((P/'protocol.json').read_text());source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip();tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=ROOT).decode().strip()
for path,digest in policy['source_sha256'].items():require(sha((ROOT/path).read_bytes())==digest,'frozen preparation source '+path)
require(not(P/'compile-binding.json').exists(),'never overwrite compiled preparation');env=dict(os.environ,CARGO_TARGET_DIR=policy['target'],CARGO_BUILD_JOBS='1',RUSTFLAGS=policy['rustflags']);env.pop('RHEON_CASE41_FD_ALLOW',None);commands=[]
def run(label,argv):
 start=time.monotonic();r=subprocess.run(argv,cwd=ROOT,env=env,capture_output=True);(P/(label+'.log')).write_bytes(r.stdout);(P/(label+'-stderr.log')).write_bytes(r.stderr);commands.append({'argv':argv,'exit':r.returncode,'seconds':time.monotonic()-start,'stdout':label+'.log','stderr':label+'-stderr.log','stdout_sha256':sha(r.stdout),'stderr_sha256':sha(r.stderr),'native_test_invocation':False});(P/'compile-commands.json').write_text(json.dumps(commands,indent=2,sort_keys=True)+'\n');require(r.returncode==0,'actual compile-only failure '+label);return r.stdout
run('format',['rustfmt','--edition','2024','--check','--config','skip_children=true',str(P/'fd_probe.rs'),*[str(x)for x in sorted(N.glob('*.rs'))if 'overlay'not in x.name]])
run('install',['python',str(N/'prepare.py'),'install'])
try:
 data=run('build',['cargo','test','--release','--no-default-features','--lib','--locked','--no-run','--message-format=json']);rows=[json.loads(x)for x in data.splitlines()if x.startswith(b'{')];bins=[r['executable']for r in rows if r.get('reason')=='compiler-artifact'and r.get('executable')and r.get('profile',{}).get('test')and r['target']['name']=='rheon'];require(len(bins)==1,'one actual compiled test ELF');binary=bins[0];run('clippy',['cargo','clippy','--release','--no-default-features','--lib','--tests','--locked','--','-D','warnings']);raw=subprocess.check_output(['objdump','-d','-C',binary]);(P/'native-disassembly.txt.gz').write_bytes(gzip.compress(raw,mtime=0));(P/'native-disassembly.txt.sha256').write_text(sha(raw)+'\n')
 for name,argv in [('symbols',['nm','-S','-nC',binary]),('relocations',['readelf','-rW',binary]),('headers',['readelf','-hW',binary])]: (P/('native-'+name+'.txt')).write_bytes(subprocess.check_output(argv))
 binding={'status':'PASS_FORMAT_RELEASE_COMPILE_CLIPPY_ONLY','source':source,'source_tree':tree,'binary':binary,'binary_sha256':sha(Path(binary).read_bytes()),'protocol_sha256':sha((P/'protocol.json').read_bytes()),'target':policy['target'],'rustflags':policy['rustflags'],'native_test_invocations':0,'native_equations':0,'owner_advances':0,'scalar_runtime_preflight_pending':True,'FD_memory_preflight_pending':True,'FD_execution_authorization_pending':True};(P/'compile-binding.json').write_text(json.dumps(binding,indent=2,sort_keys=True)+'\n')
finally:run('restore',['python',str(N/'prepare.py'),'restore'])
print(json.dumps(binding,indent=2,sort_keys=True))
