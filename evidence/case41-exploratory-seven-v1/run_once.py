"""One isolated unchanged-ELF invocation under a distinct resource policy."""
from pathlib import Path
import hashlib
import json
import os
import resource
import selectors
import signal
import subprocess
import time

P = Path(__file__).resolve().parent
ROOT = P.parents[1]


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    policy = json.loads((P / 'protocol.json').read_text())
    preflight = json.loads((P / 'preflight-receipt.json').read_text())
    require(preflight['status'] == 'PASS_DISTINCT_EXPLORATORY_PROTOCOL_ONLY', 'new policy preflight')
    require(preflight['historical_memory_qualified'] is False, 'never forge old memory qualification')
    require(preflight['protocol_sha256'] == sha((P / 'protocol.json').read_bytes()), 'same exploratory protocol')
    for path, digest in policy['sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'frozen input ' + path)
    binary = policy['binary']
    require(sha(Path(binary).read_bytes()) == policy['binary_sha256'], 'exact frozen ELF')
    require(not (P / 'native.log').exists() and not (P / 'native-stderr.log').exists(), 'no previous numerical output')
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT).decode().strip()
    argv = [binary, '--exact', 'research_public_call::case41_paired_candidate_roster',
            '--nocapture', '--test-threads=1']
    env = {'LANG': 'C', 'LC_ALL': 'C', 'TZ': 'UTC', 'RUST_BACKTRACE': '0',
           'RUST_MIN_STACK': str(policy['limits']['stack_bytes']),
           'RHEON_CASE41_PAIRED_ALLOW': 'case41-paired-observation-independent-preflight-required-v1'}
    before = dict(source=source, source_tree=tree, argv=argv, environment=env,
                  compiled_source=policy['compiled_source'], binary_sha256=policy['binary_sha256'],
                  protocol_sha256=sha((P / 'protocol.json').read_bytes()),
                  preflight_receipt_sha256=sha((P / 'preflight-receipt.json').read_bytes()),
                  resource_prerequisite_changed=True, numerical_method_changed=False,
                  historical_memory_qualified=False, historical_known_subtotal=66368,
                  corrections=0, owners=0, equations_max=7, retries_allowed=0,
                  limits=policy['limits'])
    # This marker is exclusive and survives failures/disconnects. Never retry.
    with (P / 'capture-before.json').open('x') as handle:
        json.dump(before, handle, indent=2, sort_keys=True)
        handle.write('\n')
        handle.flush()
        os.fsync(handle.fileno())

    limits = policy['limits']

    def child_limits():
        resource.setrlimit(resource.RLIMIT_AS, (limits['address_space_bytes'],) * 2)
        resource.setrlimit(resource.RLIMIT_STACK, (limits['stack_bytes'],) * 2)
        resource.setrlimit(resource.RLIMIT_CPU, (limits['cpu_seconds'],) * 2)
        resource.setrlimit(resource.RLIMIT_FSIZE, (limits['stdout_bytes'],) * 2)
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        resource.setrlimit(resource.RLIMIT_NOFILE, (32, 32))

    start = time.monotonic()
    faults = []
    sizes = {'stdout': 0, 'stderr': 0}
    sampled_peaks = {}
    samples = 0
    process = None
    usage = None
    status = None
    try:
        with (P / 'native.log').open('xb') as out, (P / 'native-stderr.log').open('xb') as err:
            process = subprocess.Popen(argv, cwd=ROOT, env=env, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, preexec_fn=child_limits,
                                       start_new_session=True)
            selector = selectors.DefaultSelector()
            for name, pipe, handle in [('stdout', process.stdout, out), ('stderr', process.stderr, err)]:
                os.set_blocking(pipe.fileno(), False)
                selector.register(pipe, selectors.EVENT_READ, (name, handle))
            (P / 'process-started.json').write_text(json.dumps(dict(pid=process.pid,
                source=source, limits=limits, native_process_launches=1), indent=2) + '\n')

            def terminate(reason):
                if reason not in faults:
                    faults.append(reason)
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass

            while status is None or selector.get_map():
                if time.monotonic() - start > limits['wall_seconds']:
                    terminate('WALL_TIME_LIMIT')
                try:
                    fields = Path(f'/proc/{process.pid}/status').read_text().splitlines()
                    sample = {}
                    for line in fields:
                        key, _, value = line.partition(':')
                        if key in ['VmRSS', 'VmHWM', 'VmPeak', 'VmSize']:
                            sample[key] = int(value.split()[0]) * 1024
                    for key, value in sample.items():
                        sampled_peaks[key] = max(sampled_peaks.get(key, 0), value)
                    samples += bool(sample)
                except FileNotFoundError:
                    pass
                for key, _ in selector.select(timeout=0.02):
                    name, handle = key.data
                    chunk = os.read(key.fd, 65536)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        key.fileobj.close()
                        continue
                    allowance = limits[name + '_bytes'] - sizes[name]
                    kept = chunk[:max(0, allowance)]
                    handle.write(kept)
                    sizes[name] += len(kept)
                    if len(chunk) > allowance:
                        terminate(name.upper() + '_TRUNCATED_AT_LIMIT')
                if status is None:
                    pid, result, child_usage = os.wait4(process.pid, os.WNOHANG)
                    if pid:
                        status, usage = result, child_usage
                        process.returncode = os.waitstatus_to_exitcode(status)
            selector.close()
            out.flush()
            err.flush()
            os.fsync(out.fileno())
            os.fsync(err.fileno())
    except BaseException as error:
        faults.append(type(error).__name__ + ': ' + str(error))
        if process is not None and status is None:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            _, status, usage = os.wait4(process.pid, 0)
            process.returncode = os.waitstatus_to_exitcode(status)
    after = {**before, 'seconds': time.monotonic() - start, 'faults': faults,
             'exit': None if status is None else os.waitstatus_to_exitcode(status),
             'native_process_launches': int(process is not None), 'retries': 0,
             'stdout_bytes': sizes['stdout'], 'stderr_bytes': sizes['stderr'],
             'proc_status_samples': samples, 'sampled_proc_peaks_bytes': sampled_peaks,
             'measured_peak_rss_bytes': None if usage is None else int(usage.ru_maxrss) * 1024,
             'peak_rss_measurement': 'Linux wait4 ru_maxrss in KiB, native child only',
             'cpu_user_seconds': None if usage is None else usage.ru_utime,
             'cpu_system_seconds': None if usage is None else usage.ru_stime,
             'historical_additional_memory_bound_proved': False,
             'status': 'PROCESS_COMPLETED_READER_PENDING' if not faults and status == 0
                       else 'INCOMPLETE_TERMINAL_NO_RETRY'}
    for label, path in [('stdout', P / 'native.log'), ('stderr', P / 'native-stderr.log')]:
        after[label + '_sha256'] = sha(path.read_bytes()) if path.exists() else None
    (P / 'capture-completed.json').write_text(json.dumps(after, indent=2, sort_keys=True) + '\n')
    print(json.dumps(after, indent=2, sort_keys=True))
    require(after['status'] == 'PROCESS_COMPLETED_READER_PENDING', 'terminal failure; retain evidence, never retry')


if __name__ == '__main__':
    main()
