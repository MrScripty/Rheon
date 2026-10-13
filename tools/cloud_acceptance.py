#!/usr/bin/env python3
"""Linux CPU qualification: explicit correctness gates and paired scheduling trials."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = (
    'pressure_contract', 'implementation_contract', 'parallel_contract',
    'liquid_volume_contract', 'viscosity_contract', 'dense3d_sequence_contract',
    'dense3d_controls_contract',
)


def require(ok, message):
    if not ok:
        raise ValueError(message)


def parse_sample(output, steps):
    lines = output.strip().splitlines()
    require(len(lines) == 1, 'expected exactly one benchmark record')
    pairs = [item.split('=', 1) for item in lines[0].split()]
    require(all(len(pair) == 2 for pair in pairs), 'malformed benchmark field')
    fields = dict(pairs)
    require(len(pairs) == len(fields), 'duplicate benchmark fields')
    expected = {'steps', 'accepted_time', 'divergence', 'step_seconds', 'capture_seconds',
                'frame_seconds', 'managed_bytes', 'capture_bytes', 'fingerprint'}
    require(set(fields) == expected, 'unexpected benchmark schema')
    sample = {k: (v if k == 'fingerprint' else float(v)) for k, v in fields.items()}
    require(all(math.isfinite(v) and v >= 0 for k, v in sample.items() if k != 'fingerprint'),
            'nonfinite or negative measurement')
    require(sample['steps'] == steps, 'incomplete trajectory')
    require(all(sample[k] > 0 and sample[k].is_integer() for k in ('managed_bytes', 'capture_bytes')),
            'invalid retained payload')
    require(sample['managed_bytes'] <= 192 * 1024 * 1024, 'simulation memory budget exceeded')
    require(0 < sample['accepted_time'] <= steps * 0.02 + 1e-12, 'invalid accepted time')
    require(sample['divergence'] <= 1e-5, 'divergence acceptance failed')
    require(sample['frame_seconds'] > 0 and sample['step_seconds'] > 0, 'empty timing')
    require(sample['step_seconds'] + sample['capture_seconds'] <= sample['frame_seconds'] + 3e-9,
            'frame timer excludes measured work')
    require(len(fields['fingerprint']) == 16 and all(c in '0123456789abcdef' for c in fields['fingerprint']),
            'invalid fingerprint')
    return sample


def measure(command, log, timeout=900):
    """wait4 reports this child RSS (Linux KiB), never cumulative child high-water RSS."""
    start = time.monotonic()
    with log.open('w') as stream:
        child = subprocess.Popen(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT)
        while True:
            pid, status, usage = os.wait4(child.pid, os.WNOHANG)
            if pid:
                child.returncode = os.waitstatus_to_exitcode(status)
                break
            if time.monotonic() - start > timeout:
                child.kill()
                _, status, usage = os.wait4(child.pid, 0)
                child.returncode = os.waitstatus_to_exitcode(status)
                raise TimeoutError(f'timed out: {command}; log: {log}')
            time.sleep(0.01)
    require(child.returncode == 0, f'command failed ({child.returncode}): {command}; log: {log}')
    return {'process_seconds': time.monotonic() - start,
            'cpu_seconds': usage.ru_utime + usage.ru_stime, 'peak_rss_kib': usage.ru_maxrss}


def read_optional(path):
    path = Path(path)
    return path.read_text() if path.exists() else None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path, help='fresh external output directory')
    parser.add_argument('--sizes', type=int, nargs='+', default=[16, 32])
    parser.add_argument('--threads', type=int, nargs='+', default=[1, 2, 4])
    parser.add_argument('--repeats', type=int, default=3)
    parser.add_argument('--steps', type=int, default=12)
    parser.add_argument('--extra-contract', action='append', default=[])
    args = parser.parse_args()
    require(platform.system() == 'Linux', 'RSS units and wait4 qualification require Linux')
    require(args.repeats >= 3, 'at least three independent process repetitions required')
    require(1 <= args.steps <= 4096 and all(8 <= n <= 64 for n in args.sizes)
            and all(1 <= n <= 256 for n in args.threads), 'invalid matrix')
    require(not subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT),
            'qualification requires a clean committed checkout')
    output = args.output.resolve()
    require(not output.is_relative_to(ROOT), 'generated evidence must be outside this checkout')
    output.mkdir(parents=True, exist_ok=False)
    receipt = {'schema': 'rheon-cloud-acceptance-v1', 'status': 'running',
               'source_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
               'source_dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT)),
               'machine': platform.uname()._asdict(), 'affinity': sorted(os.sched_getaffinity(0)),
               'cpuinfo': read_optional('/proc/cpuinfo'), 'cpu_max': read_optional('/sys/fs/cgroup/cpu.max'),
               'memory_max': read_optional('/sys/fs/cgroup/memory.max'),
               'load_before': os.getloadavg(), 'commands': [], 'samples': [], 'summary': [],
               'scope': 'Non-exclusive Linux CPU step plus owned capture/checksum; no renderer, GPU, game-engine, AI-provider or device frame qualification.'}
    def save():
        (output / 'receipt.json').write_text(json.dumps(receipt, indent=2, allow_nan=False) + '\n')
    save()
    try:
        commands = [['rustc', '--version', '--verbose'], ['cargo', '--version'],
                    ['cargo', 'test', '--locked', '--release', '--features', 'parallel'] +
                    [part for name in (*CONTRACTS, *args.extra_contract) for part in ('--test', name)],
                    ['python3', 'proofs/scripts/check_sources.py'],
                    ['cargo', 'build', '--locked', '--release', '--features', 'parallel', '--example', 'cloud_frame_lab']]
        for i, command in enumerate(commands):
            log = output / f'gate-{i}.log'
            receipt['commands'].append({'command': command, 'log': log.name, **measure(command, log)})
            save()
        target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
        if not target.is_absolute():
            target = ROOT / target
        binary = target / 'release/examples/cloud_frame_lab'
        receipt['binary_sha256'] = hashlib.sha256(binary.read_bytes()).hexdigest()
        expected = {}
        for n in args.sizes:
            for threads in args.threads:
                for solver in ('jacobi-pcg-v1', 'sgs-pcg-v1'):
                    for repeat in range(args.repeats):
                        for policy in (('fixed', 'adaptive') if repeat % 2 == 0 else ('adaptive', 'fixed')):
                            command = [str(binary), str(n), str(threads), str(args.steps), policy, solver]
                            log = output / f'sample-{len(receipt["samples"])}.log'
                            metrics = measure(command, log)
                            sample = parse_sample(log.read_text(), args.steps)
                            require(sample['capture_bytes'] >= 4 * (3 * n * n * (n + 1) + n ** 3),
                                    'capture cannot hold all fields')
                            key = (n, solver)
                            value = (sample['fingerprint'], sample['accepted_time'])
                            require(expected.setdefault(key, value) == value,
                                    f'thread/policy/repetition replay mismatch: {key}')
                            receipt['samples'].append(dict(size=n, threads=threads, solver=solver,
                                repeat=repeat, policy=policy, log=log.name, **sample, **metrics))
                            save()
                    pair = {}
                    for policy in ('fixed', 'adaptive'):
                        samples = [s for s in receipt['samples'] if (s['size'], s['threads'], s['solver'], s['policy']) == (n, threads, solver, policy)]
                        times = [s['frame_seconds'] / args.steps for s in samples]
                        pair[policy] = {'median_cpu_frame_seconds': statistics.median(times),
                                        'min_cpu_frame_seconds': min(times), 'max_cpu_frame_seconds': max(times),
                                        'max_peak_rss_kib': max(s['peak_rss_kib'] for s in samples)}
                    receipt['summary'].append(dict(size=n, threads=threads, solver=solver, policies=pair,
                        adaptive_speedup=pair['fixed']['median_cpu_frame_seconds'] / pair['adaptive']['median_cpu_frame_seconds']))
                    print(n, threads, solver, receipt['summary'][-1]['adaptive_speedup'], flush=True)
        receipt.update(status='passed', load_after=os.getloadavg())
    except Exception as error:
        receipt.update(status='failed', error=str(error))
        raise
    finally:
        save()


if __name__ == '__main__':
    main()
