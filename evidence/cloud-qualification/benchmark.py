#!/usr/bin/env python3
"""Reproduce serial cloud qualification using the existing comparison harness.

Run from the repository root after a locked release build, with no concurrent
builds: python3 evidence/cloud-qualification/benchmark.py BINARY FRESH_OUTPUT
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

sys.path.insert(0, str(Path('tools').resolve()))
from rheon_compare import Comparison, implementations


def snapshot():
    result = {'unix_seconds': time.time(), 'load_average': os.getloadavg()}
    for name in ('/proc/stat', '/proc/pressure/cpu', '/sys/fs/cgroup/cpu.stat'):
        path = Path(name)
        result[name] = path.read_text() if path.exists() else None
    result['processes'] = subprocess.check_output(
        ['ps', '-eo', 'pid,comm,pcpu'], text=True)
    return result


def main():
    binary = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2])
    output.mkdir(exist_ok=False)
    methods = [m['id'] for m in implementations(binary)]
    receipt = {
        'source_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
        'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
        'contention_scope': 'Serial runs after builds; shared VM, no exclusive host claim. Before/after snapshots do not rule out transient contention.',
        'cases': [],
    }
    for size, steps in ((16, 12), (32, 6), (64, 3)):
        for accuracy in ('standard', 'tight'):
            case = {'size': size, 'steps': steps, 'accuracy': accuracy, 'before': snapshot()}
            job = Comparison(binary, output / f'{size}-{accuracy}', methods,
                             size=size, steps=steps, repeats=3, accuracy=accuracy)
            try:
                while job.poll() in ('running', 'cancelling'):
                    time.sleep(0.05)
            except KeyboardInterrupt:
                job.cancel()
                while job.poll() == 'cancelling':
                    time.sleep(0.05)
                raise
            case.update(after=snapshot(), status=job.status, error=job.error)
            if job.result:
                case['summary'] = job.result['summary']
                assert job.result['equal_accepted_time']
            receipt['cases'].append(case)
            (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
            print(size, accuracy, job.status, flush=True)
            if job.status != 'completed':
                return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
