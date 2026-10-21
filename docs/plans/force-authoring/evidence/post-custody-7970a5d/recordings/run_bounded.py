import hashlib, json, os, pathlib, resource, signal, subprocess, sys, time
root = pathlib.Path('/workspace/force-validation-post-custody-7970a5d/recordings')
binary = root.parent / 'recording_tests'
expected_hash = '4ee715678545204a996b70121d26dcb930578e7e5f94cafc27d111372af69e5b'
names = ['carrier_force_archive_round_trip_and_failed_attempt_invalidation', 'particle_force_archive_round_trip_and_rejected_control_custody']

def bounded():
    resource.setrlimit(resource.RLIMIT_CPU, (20,20))
    resource.setrlimit(resource.RLIMIT_AS, (512*1024*1024,512*1024*1024))
    resource.setrlimit(resource.RLIMIT_FSIZE, (1024*1024,1024*1024))
    resource.setrlimit(resource.RLIMIT_CORE, (0,0))
    # Check effective child limits before exec; failure prevents execution.
    for limit, expected in [(resource.RLIMIT_CPU,20),(resource.RLIMIT_AS,512*1024*1024),(resource.RLIMIT_FSIZE,1024*1024)]:
        assert resource.getrlimit(limit) == (expected,expected)

def run(label, args):
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == expected_hash
    path = root / (label+'.log')
    command = [str(binary), *args]
    began = time.monotonic()
    signals = []
    with path.open('xb') as log:
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
            cwd='/workspace/Rheon-force', env={**os.environ,'RUST_BACKTRACE':'0'},
            preexec_fn=bounded, start_new_session=True)
        try:
            code = child.wait(timeout=30)
        except subprocess.TimeoutExpired:
            child.send_signal(signal.SIGTERM); signals.append('TERM after 30 s')
            try: code = child.wait(timeout=1)
            except subprocess.TimeoutExpired:
                child.kill(); signals.append('KILL owned child after 1 s grace'); code=child.wait()
    data = path.read_bytes()
    record = {'command':command,'binary_sha256':expected_hash,'returncode':code,
        'wall_seconds':time.monotonic()-began,'signals':signals,'log_bytes':len(data),
        'log_sha256':hashlib.sha256(data).hexdigest(),
        'limits':{'wall_seconds':30,'term_to_owned_child_kill_seconds':1,'cpu_seconds':20,
            'address_space_bytes':512*1024*1024,'file_and_log_bytes':1024*1024,'test_threads':1}}
    (root/(label+'.json')).write_text(json.dumps(record,indent=2)+'\n')
    print(json.dumps(record),flush=True)
    if code != 0 or signals or len(data) >= 1024*1024:
        raise SystemExit('STOP: failure, timeout or log bound; no retries')
    return data.decode('utf-8')

if sys.argv[1:] == ['enumerate']:
    out = run('enumeration',['--list','--format','terse'])
    actual = [line[:-6] for line in out.splitlines() if line.endswith(': test')]
    assert len(actual)==2 and set(actual)==set(names), 'STOP: unexpected enumeration'
    print(out)
elif sys.argv[1:] == ['execute']:
    listed = (root/'enumeration.log').read_text()
    assert {line[:-6] for line in listed.splitlines() if line.endswith(': test')} == set(names)
    for name in names:
        out = run(name,[name,'--exact','--ignored','--test-threads=1','--show-output'])
        if out.count('running 1 test') != 1 or f'test {name} ... ok' not in out or 'test result: ok. 1 passed; 0 failed; 0 ignored;' not in out:
            raise SystemExit('STOP: unexpected test selection or outcome; no retries')
else: raise SystemExit('enumerate or execute required')
