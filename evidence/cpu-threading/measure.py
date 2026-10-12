#!/usr/bin/env python3
"""Record equal-input timings; all generated observations stay in a temp directory."""
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

parser = argparse.ArgumentParser(description='Compare original and threaded release executables on equal inputs.')
parser.add_argument('--baseline', type=Path, required=True)
parser.add_argument('--candidate', type=Path, required=True)
parser.add_argument('--implementation', default='jacobi-pcg-v1')
parser.add_argument('--baseline-threads', type=int, nargs='+')
parser.add_argument('--threads', type=int, nargs='+', default=[1, 4, 8])
parser.add_argument('--sizes', type=int, nargs='+', default=[32, 64])
parser.add_argument('--repeats', type=int, default=3)
args = parser.parse_args()
if args.repeats < 1:
    parser.error('--repeats must be positive')
root = Path(tempfile.mkdtemp(prefix='rheon-thread-benchmark-'))
if args.baseline_threads is None:
    variants = [('original', str(args.baseline.resolve()), None)]
else:
    variants = [(f'baseline-{n}', str(args.baseline.resolve()), n) for n in args.baseline_threads]
variants += [(f'threads-{n}', str(args.candidate.resolve()), n) for n in args.threads]
results = {
    'machine': platform.uname()._asdict(),
    'cpu_affinity': sorted(os.sched_getaffinity(0)),
    'build': 'cargo build --locked --release --features desktop --bins',
    'binaries': {name: hashlib.sha256(Path(binary).read_bytes()).hexdigest() for name, binary, _ in variants},
    'workloads': [],
}
for size in args.sizes:
    workload = {'size': size, 'steps': 2, 'source_off_at': 1, 'implementation': args.implementation,
                'repeats': args.repeats, 'samples': []}
    expected = None
    for repeat in range(args.repeats):
        shift = repeat % len(variants)
        order = variants[shift:] + variants[:shift]
        for name, binary, threads in order:
            out = root / f'{size}-{repeat}-{name}'
            cmd = [binary, '--implementation', args.implementation, '--size', str(size),
                   '--steps', '2', '--source-off-at', '1', '--output', str(out)]
            if threads is not None:
                cmd += ['--threads', str(threads)]
            before = resource.getrusage(resource.RUSAGE_CHILDREN)
            start = time.monotonic()
            subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL)
            wall = time.monotonic() - start
            after = resource.getrusage(resource.RUSAGE_CHILDREN)
            cpu = after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime
            data = json.loads((out / 'run.json').read_text())
            fields = [(out / f).read_bytes() for f in ('steps.csv', 'opacity.png')]
            if expected is None:
                expected = fields
            assert fields == expected, (size, repeat, name, 'numerical results differ')
            workload['samples'].append({
                'variant': name, 'repeat': repeat, 'seconds': data['measured_step_seconds'],
                'process_average_cpu_cores': cpu / wall,
                'managed_simulation_bytes': data['managed_simulation_bytes'],
            })
            print(size, repeat, name, data['measured_step_seconds'], f'{cpu / wall:.2f} average CPU cores', flush=True)
    workload['summary'] = {}
    for name, _, _ in variants:
        samples = [s for s in workload['samples'] if s['variant'] == name]
        times = [s['seconds'] for s in samples]
        workload['summary'][name] = {
            'median_seconds': statistics.median(times), 'min_seconds': min(times), 'max_seconds': max(times),
            'median_process_average_cpu_cores': statistics.median(s['process_average_cpu_cores'] for s in samples),
        }
    workload['csv_and_png_bit_identical'] = True
    results['workloads'].append(workload)
    (root / 'summary.json').write_text(json.dumps(results, indent=2) + '\n')
    print('SUMMARY', size, json.dumps(workload['summary']), flush=True)
print('RESULT', root, flush=True)
