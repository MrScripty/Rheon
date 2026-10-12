#!/usr/bin/env python3
"""Compare frozen release scheduling_lab binaries; store raw data in /tmp."""
import argparse
import hashlib
import json
import os
import platform
import resource
import statistics
import subprocess
import tempfile
import time
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--baseline', required=True, type=Path)
parser.add_argument('--candidate', required=True, type=Path)
parser.add_argument('--repeats', type=int, default=3)
args = parser.parse_args()
root = Path(tempfile.mkdtemp(prefix='rheon-scheduling-measure-'))
variants = [('baseline', args.baseline.resolve()), ('candidate', args.candidate.resolve())]
result = {'machine': platform.uname()._asdict(), 'affinity': sorted(os.sched_getaffinity(0)),
          'binary_sha256': {name: hashlib.sha256(path.read_bytes()).hexdigest()
                            for name, path in variants}, 'samples': []}
cases = [(32, 8, 5, 'step-sgs'), (32, 24, 5, 'step-sgs'),
         (64, 24, 2, 'step-sgs'), (128, 24, 2, 'sparse'), (128, 24, 2, 'dense')]
expected = {}
for size, threads, iterations, mode in cases:
    for repeat in range(args.repeats):
        for name, binary in variants[::1 if repeat % 2 == 0 else -1]:
            before = resource.getrusage(resource.RUSAGE_CHILDREN)
            start = time.monotonic()
            output = subprocess.check_output([str(binary), str(size), str(threads),
                                              str(iterations), mode], text=True)
            wall = time.monotonic() - start
            after = resource.getrusage(resource.RUSAGE_CHILDREN)
            cpu = after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime
            for line in output.splitlines():
                fields = dict(field.split('=', 1) for field in line.split())
                if 'fingerprint' not in fields:
                    continue  # Progress and kernel profiles are not result samples.
                stage = fields.get('stage', 'step')
                key = (size, threads, mode, stage)
                expected.setdefault(key, fields['fingerprint'])
                assert fields['fingerprint'] == expected[key], (name, key, 'numerical mismatch')
                sample = {'variant': name, 'size': size, 'threads': threads, 'iterations': iterations,
                          'mode': mode, 'stage': stage, 'repeat': repeat, 'seconds': float(fields['seconds']),
                          'process_average_cpu_cores': cpu / wall, 'fingerprint': fields['fingerprint']}
                if 'workers' in fields:
                    sample['observed_workers'] = int(fields['workers'])
                result['samples'].append(sample)
                print(json.dumps(sample), flush=True)
            (root / 'measurements.json').write_text(json.dumps(result, indent=2) + '\n')
    for stage in sorted({s['stage'] for s in result['samples'] if s['mode'] == mode and s['size'] == size}):
        for name, _ in variants:
            selected = [s for s in result['samples'] if (s['size'], s['threads'], s['mode'], s['stage'], s['variant'])
                        == (size, threads, mode, stage, name)]
            if selected:
                print('MEDIAN', size, threads, mode, stage, name,
                      statistics.median(s['seconds'] for s in selected), flush=True)
print('RESULT', root, flush=True)
