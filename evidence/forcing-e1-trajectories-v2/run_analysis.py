"""Record actual replay commands and byte-bound outcomes without native reruns."""
from pathlib import Path
import hashlib,json,os,subprocess,sys,time
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def sha(data):return hashlib.sha256(data).hexdigest()
def require(ok,message):
 if not ok:raise ValueError(message)
require(not(P/'analysis-commands.json').exists(),'never overwrite analysis attempts')
jobs=[(label,script,opt)for label,script in [('native','analyze_native.py'),('convergence','convergence.py'),('physical','replay_physics.py')]for opt in [False,True]]
commands=[]
for label,script,opt in jobs:
 name=label+('-optimized'if opt else'-outcome');argv=[sys.executable]+(['-O']if opt else[])+[str(P/script)];out=P/(name+'.json');err=P/(name+'-stderr.log');require(not out.exists()and not err.exists(),'fresh analysis output')
 command={'argv':argv,'cwd':str(ROOT),'reader_sha256':sha((P/script).read_bytes()),'stdout':out.name,'stderr':err.name,'native_execution':False};commands.append(command);(P/'analysis-commands.json').write_text(json.dumps(commands,indent=2,sort_keys=True)+'\n');start=time.monotonic()
 env=os.environ.copy();env.update(OPENBLAS_NUM_THREADS='1',OMP_NUM_THREADS='1');command['resource_env']={'OPENBLAS_NUM_THREADS':'1','OMP_NUM_THREADS':'1'}
 with out.open('wb')as stdout,err.open('wb')as stderr:answer=subprocess.run(argv,cwd=ROOT,env=env,stdout=stdout,stderr=stderr)
 command.update(exit=answer.returncode,seconds=time.monotonic()-start,stdout_sha256=sha(out.read_bytes()),stderr_sha256=sha(err.read_bytes()));(P/'analysis-commands.json').write_text(json.dumps(commands,indent=2,sort_keys=True)+'\n')
 print(json.dumps({'event':'analysis_finished','label':label,'optimized':opt,'exit':answer.returncode}),flush=True);require(answer.returncode==0,'analysis failure: preserve output and stop')
 if opt:require((P/(label+'-outcome.json')).read_bytes()==out.read_bytes(),'normal/optimized exact parity')
print(json.dumps({'status':'ALL_SIX_ANALYSIS_COMMANDS_PASSED','native_execution':False}))
