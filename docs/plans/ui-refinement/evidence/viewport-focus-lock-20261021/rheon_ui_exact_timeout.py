import hashlib, json, os, resource, signal, subprocess, sys, time
from pathlib import Path
binary, name, prefix = sys.argv[1:]
log=Path(prefix+'.log');receipt=Path(prefix+'.json')
assert not log.exists() and not receipt.exists()
limits={'wall_seconds':10,'cpu_seconds':5,'address_space_bytes':536870912,'log_file_bytes':1048576,'term_grace_seconds':1}
def bound():
    for key,value in [(resource.RLIMIT_CPU,5),(resource.RLIMIT_AS,536870912),(resource.RLIMIT_FSIZE,1048576),(resource.RLIMIT_CORE,0)]:
        resource.setrlimit(key,(value,value))
def group_signal(pid, sig):
    try: os.killpg(pid,sig)
    except ProcessLookupError: pass
start=time.monotonic();timed_out=False;actions=[]
with log.open('xb') as output:
    child=subprocess.Popen([binary,name,'--exact','--test-threads=1','--nocapture'],stdout=output,stderr=subprocess.STDOUT,start_new_session=True,preexec_fn=bound)
    try: code=child.wait(timeout=10)
    except subprocess.TimeoutExpired:
        timed_out=True;actions.append('TERM owned process group');group_signal(child.pid,signal.SIGTERM)
        try: code=child.wait(timeout=1)
        except subprocess.TimeoutExpired:
            actions.append('KILL owned process group');group_signal(child.pid,signal.SIGKILL);code=child.wait(timeout=1)
data=log.read_bytes()
result={'command':[binary,name,'--exact','--test-threads=1','--nocapture'],'limits':limits,'pid':child.pid,'exit_code':code,'timed_out':timed_out,'actions':actions,'elapsed_seconds':time.monotonic()-start,'binary_sha256':hashlib.sha256(Path(binary).read_bytes()).hexdigest(),'captured_bytes':len(data),'log_sha256':hashlib.sha256(data).hexdigest()}
receipt.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
assert not timed_out and code==0 and b'1 passed; 0 failed; 0 ignored;' in data
