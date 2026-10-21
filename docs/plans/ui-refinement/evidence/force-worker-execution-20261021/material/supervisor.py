"""One admitted exact Worker/SQLite case. Preserve every remaining artifact."""
import hashlib, json, os, re, resource, signal, subprocess, sys, time
from pathlib import Path
REPO=Path('/workspace/Rheon-ui-authoring')
BINARY=Path('/workspace/.rheon-tools/target/debug/deps/rheon_desktop-51d01e4509203a9f')
SEAL='af2da1539521e10407ac73b11cd58494118aa93a3f99ed529b11697ed85727b5'
REV='3b3fccea3657d711897130643a478adb48b5edd0'
NAMES={
'native':'force_runtime_integration_tests::native_force_dispatch_records_and_cold_binds_v2',
'material':'force_runtime_integration_tests::material_force_dispatch_records_immutable_scene',
}
def digest(path):
    with path.open('rb') as file: return hashlib.file_digest(file,'sha256').hexdigest()
def inventory(root):
    result=[]
    for path in sorted(root.rglob('*')):
        meta=path.lstat()
        item={'path':str(path.relative_to(root)),'bytes':meta.st_size}
        if path.is_symlink(): item['kind']='symlink'
        elif path.is_dir(): item['kind']='directory'
        elif path.is_file(): item.update(kind='file',sha256=digest(path))
        else: item['kind']='other'
        result.append(item)
    return result
case=sys.argv[1]
assert case in NAMES
root=Path('/workspace/scratch/rheon-ui-force-worker-execution-20261021')
root.mkdir(exist_ok=True)
case_root=root/case
case_root.mkdir(exist_ok=False) # refuses duplicate execution, including prior failures
DATA=case_root/'DATA';CAPTURE=case_root/'capture'
DATA.mkdir();CAPTURE.mkdir()
assert DATA.resolve()==DATA and CAPTURE.resolve()==CAPTURE
log=CAPTURE/'combined.log';receipt=CAPTURE/'disposition.json'
command=[str(BINARY),NAMES[case],'--exact','--ignored','--test-threads=1','--nocapture']
limits={'wall_timeout_seconds':30,'cpu_seconds':20,'address_space_bytes':536870912,'per_file_and_log_bytes':1048576,'term_grace_seconds':1,'core_bytes':0}
state={'case':case,'command':command,'limits':limits,'source_commit':REV,'data':str(DATA),'capture':str(CAPTURE),'test_body_invocation_attempted':False,'actions':[]}
child=None
start=None
failure=None
try:
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()==REV
    assert not subprocess.check_output(['git','status','--porcelain'],cwd=REPO,text=True)
    assert digest(BINARY)==SEAL
    source_manifest=REPO/'docs/plans/ui-refinement/evidence/force-worker-preparation-20261021/source-files.json'
    for item in json.loads(source_manifest.read_text()):
        assert digest(REPO/item['file'])==item['sha256'],item['file']
    listed=(REPO/'docs/plans/ui-refinement/evidence/force-worker-preparation-20261021/rheon-ui-force-worker-list2.log').read_text().splitlines()
    assert listed.count(NAMES[case]+': test')==1
    state.update(binary_sha256=SEAL,binary_bytes=BINARY.stat().st_size,source_manifest_sha256=digest(source_manifest),supervisor_sha256=digest(Path(__file__)))
    (CAPTURE/'preflight-seals.json').write_text(json.dumps(state,indent=2)+'\n')
    (CAPTURE/'supervisor.py').write_bytes(Path(__file__).read_bytes())
    env=dict(os.environ,RHEON_UI_FORCE_TEST_ROOT=str(DATA),TMPDIR=str(DATA),SQLITE_TMPDIR=str(DATA))
    def bound():
        for key,value in [(resource.RLIMIT_CPU,20),(resource.RLIMIT_AS,536870912),(resource.RLIMIT_FSIZE,1048576),(resource.RLIMIT_CORE,0)]:
            resource.setrlimit(key,(value,value))
            assert resource.getrlimit(key)==(value,value)
    def group_signal(sig):
        # An unreaped Popen child reserves its PID; never signal after reaping.
        assert child.returncode is None
        try: os.killpg(child.pid,sig)
        except ProcessLookupError: pass
    usage_before=resource.getrusage(resource.RUSAGE_CHILDREN)
    with log.open('xb') as output:
        start=time.monotonic()
        state['test_body_invocation_attempted']=True
        child=subprocess.Popen(command,cwd=REPO,env=env,stdout=output,stderr=subprocess.STDOUT,start_new_session=True,preexec_fn=bound)
        state['pid']=child.pid
        try: child.wait(timeout=max(0.0,start+30-time.monotonic()))
        except subprocess.TimeoutExpired:
            state['timed_out']=True;state['actions'].append('TERM owned child group');group_signal(signal.SIGTERM)
            try: child.wait(timeout=1)
            except subprocess.TimeoutExpired:
                state['actions'].append('KILL owned child group');group_signal(signal.SIGKILL)
                try: child.wait(timeout=2)
                except subprocess.TimeoutExpired: state['unreaped']=True
    state['elapsed_seconds_including_teardown']=time.monotonic()-start
    state['exit_code']=child.poll()
    try:
        os.killpg(child.pid,0)
        state['group_present_after_wait']=True
        state['post_reap_group_probe']='unexpected group ID present; ownership unproven, not signaled'
    except ProcessLookupError: state['group_present_after_wait']=False
    usage_after=resource.getrusage(resource.RUSAGE_CHILDREN)
    state['observed_child_cpu_seconds']={'user':usage_after.ru_utime-usage_before.ru_utime,'system':usage_after.ru_stime-usage_before.ru_stime}
    state['individual_peak_memory']='not measured; address-space limit is not a measurement'
    data=log.read_bytes()
    state.update(captured_bytes=len(data),log_sha256=digest(log))
    assert not state.get('timed_out',False) and not state.get('unreaped',False)
    assert state['exit_code']==0 and not state['group_present_after_wait']
    assert b'running 1 test' in data and NAMES[case].encode() in data
    assert b'1 passed; 0 failed; 0 ignored;' in data
    items=inventory(DATA)
    assert all(item['kind']=='file' for item in items)
    assert len(items)<=8 and sum(item['bytes'] for item in items)<=8*1024*1024
    assert all(item['bytes']<=1048576 for item in items)
    db=[item for item in items if re.fullmatch(('jacobi-pcg-v1' if case=='native' else 'material')+r'-[0-9]+-[0-9]+-[0-9]+\.sqlite',item['path'])]
    assert len(db)==1
    expected={db[0]['path']} | ({'intent.json','member-0-started.json','member-0-finished.json','completed.json'} if case=='native' else set())
    assert {item['path'] for item in items}==expected
    state['result']='passed-one-exact-case'
except BaseException as error:
    failure=repr(error);state['result']='failed-or-uncertain';state['failure']=failure
finally:
    if child is not None and child.poll() is None:
        state['actions'].append('TERM still-running owned child group in final disposition')
        try: os.killpg(child.pid,signal.SIGTERM)
        except ProcessLookupError: pass
        try: child.wait(timeout=1)
        except subprocess.TimeoutExpired:
            state['actions'].append('KILL still-running owned child group after grace')
            try: os.killpg(child.pid,signal.SIGKILL)
            except ProcessLookupError: pass
            try: child.wait(timeout=2)
            except subprocess.TimeoutExpired: state['unreaped']=True
    state['child_pid_present']=child is not None and Path('/proc',str(child.pid)).exists()
    if state['child_pid_present'] or state.get('unreaped',False):
        state['result']='failed-or-uncertain'
        state['final_process_identity_note']='PID path remains or direct child unreaped; no post-reap signaling'
    for label,path in [('data_inventory',DATA),('capture_inventory_before_receipt',CAPTURE)]:
        try: state[label]=inventory(path)
        except BaseException as error: state[label+'_error']=repr(error);state['result']='failed-or-uncertain'
    state['temporary_file_peak']='unknown; final inventory is not aggregate or anonymous peak enforcement'
    receipt.write_text(json.dumps(state,indent=2)+'\n')
    print(json.dumps(state,indent=2))
assert state['result']=='passed-one-exact-case' and not state.get('child_pid_present',False)
