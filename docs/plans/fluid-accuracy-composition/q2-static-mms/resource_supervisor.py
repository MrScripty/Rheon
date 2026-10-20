"""Prospective CV3-A child custody; importing this module launches nothing.

Not an admission grant. Future fixed campaign commands must bind this source,
the child executable/oracle hashes and resource settings before numerical use.
"""
import hashlib
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import time


def require(condition, message):
    if not condition:
        raise RuntimeError('CV3 resource guard: ' + message)


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def output_size(root):
    total = 0
    for path in root.rglob('*'):
        require(not path.is_symlink(), 'unexpected output symlink')
        if path.is_file():
            total += path.stat().st_size
    return total


def rss_bytes(pid):
    try:
        lines = Path(f'/proc/{pid}/status').read_text().splitlines()
    except FileNotFoundError:
        return 0
    for line in lines:
        if line.startswith('VmRSS:'):
            return int(line.split()[1])*1024
    return 0


def run_child(argv, output_root, receipt, *, kind, frozen_files, total_deadline):
    """Called only by a separately admitted, source-frozen serialized driver.

    Caller creates a fresh corpus root and owns all-nine case completeness.
    Child owns one case, no subprocess descendants; each child gets a new group.
    No retry, fallback, precision change or output deletion occurs here.
    """
    require(kind in ('native', 'reference'), 'declared workload kind')
    require(argv and all(isinstance(a, str) and a for a in argv), 'complete exact argv')
    output_root = Path(output_root)
    receipt = Path(receipt)
    require(output_root.is_dir() and not output_root.is_symlink(), 'fresh owned corpus directory')
    require(receipt.resolve().is_relative_to(output_root.resolve()), 'receipt/log within accounted output root')
    require(not receipt.exists(), 'fresh receipt path')
    require(frozen_files, 'nonempty source/executable freeze')
    for path, expected in frozen_files.items():
        require(digest(path) == expected, 'frozen executable/oracle/source identity changed')
    cpu_seconds, address_bytes, output_bytes = (
        (120, 4 << 30, 2 << 30) if kind == 'native' else (1800, 8 << 30, 8 << 30))
    wall_seconds = cpu_seconds
    started = time.monotonic()
    require(total_deadline > started, 'total wall resource exhausted before case')
    def child_limits():
        resource.setrlimit(resource.RLIMIT_AS, (address_bytes, address_bytes))
        resource.setrlimit(resource.RLIMIT_CPU, (cpu_seconds, cpu_seconds))
        resource.setrlimit(resource.RLIMIT_FSIZE, (output_bytes, output_bytes))
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    env = os.environ.copy()
    env.update({key: '1' for key in ('OPENBLAS_NUM_THREADS', 'OMP_NUM_THREADS',
        'MKL_NUM_THREADS', 'BLIS_NUM_THREADS', 'NUMEXPR_NUM_THREADS')})
    log = receipt.with_suffix('.log')
    require(not log.exists(), 'fresh child log path')
    record = {'argv': argv, 'kind': kind, 'status': 'starting',
        'frozen_files': frozen_files, 'cpu_seconds': cpu_seconds,
        'address_bytes': address_bytes, 'output_bytes': output_bytes,
        'wall_seconds': wall_seconds, 'clock': 'monotonic', 'peak_observed_rss': 0}
    def retain():
        receipt.write_text(json.dumps(record, indent=2, sort_keys=True)+'\n')
    retain()
    child = None
    with log.open('xb') as stream:
        try:
            child = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=stream,
                stderr=subprocess.STDOUT, env=env, start_new_session=True,
                preexec_fn=child_limits)
            record.update(status='running', pid=child.pid)
            retain()
            reason = None
            while child.poll() is None:
                now = time.monotonic()
                resident = rss_bytes(child.pid)
                record['peak_observed_rss'] = max(record['peak_observed_rss'], resident)
                size = output_size(output_root)
                if now >= total_deadline:
                    reason = 'total wall resource refusal'
                elif now-started >= wall_seconds:
                    reason = 'per-case wall resource refusal'
                elif resident > address_bytes:
                    reason = 'RSS resource refusal'
                elif size > output_bytes:
                    reason = 'aggregate output resource refusal'
                if reason:
                    os.killpg(child.pid, signal.SIGTERM)
                    try:
                        child.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        os.killpg(child.pid, signal.SIGKILL)
                        child.wait()
                    break
                time.sleep(0.1)
            child.wait()
            record.update(exit_code=child.returncode, elapsed=time.monotonic()-started,
                output_bytes_observed=output_size(output_root), status='completed')
            if reason:
                record.update(status='refused', reason=reason)
            elif time.monotonic() >= total_deadline:
                record.update(status='refused', reason='final total wall resource refusal')
            elif record['elapsed'] >= wall_seconds:
                record.update(status='refused', reason='final per-case wall resource refusal')
            elif child.returncode != 0:
                record.update(status='refused', reason='child failure/resource signal')
            elif record['output_bytes_observed'] > output_bytes:
                record.update(status='refused', reason='final aggregate output resource refusal')
            retain()
            require(record['status'] == 'completed', record.get('reason', 'child refusal'))
        except BaseException as error:
            if child is not None and child.poll() is None:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
            record.update(status='refused', reason=str(error), elapsed=time.monotonic()-started)
            if child is not None:
                record['exit_code'] = child.returncode
            retain()
            raise
    return record
