#!/usr/bin/env python3
"""Sequential reproducible benchmark runs using the real Rust executable."""
from __future__ import annotations
import argparse
import csv
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

ACCURACY = {
    'standard': (1e-9, 1e-7, 1e-5),
    'tight': (1e-11, 1e-9, 1e-5),
}
REQUESTED_DT = 0.02
ACTIVE_STATUSES = ('running', 'cancelling', 'timing_out')

def implementations(binary: Path) -> list[dict]:
    result = subprocess.run([str(binary), '--list-implementations'], check=True, capture_output=True, text=True, timeout=10)
    methods = json.loads(result.stdout)
    if not isinstance(methods, list) or not methods or any(not isinstance(m, dict) or not isinstance(m.get('id'), str) or not isinstance(m.get('label'), str) for m in methods):
        raise ValueError('Executable returned an invalid implementation registry')
    if len({m['id'] for m in methods}) != len(methods):
        raise ValueError('Executable returned duplicate implementation IDs')
    return methods

def hardware() -> dict:
    cpu = 'unavailable'
    try:
        cpu = next(line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines() if line.startswith('model name'))
    except (OSError, StopIteration):
        pass
    limits = {}
    for name in ('cpu.max', 'memory.max'):
        try:
            limits[name] = (Path('/sys/fs/cgroup') / name).read_text().strip()
        except OSError:
            limits[name] = 'unavailable'
    return {'cgroup_limits': limits, 'system': platform.system(), 'release': platform.release(), 'machine': platform.machine(),
            'cpu_model': cpu, 'logical_cpus': os.cpu_count(),
            'affinity_cpus': len(os.sched_getaffinity(0)) if hasattr(os, 'sched_getaffinity') else None,
            'hardware_scope': 'This host only; no other device qualification',
            'python': platform.python_version()}

class Comparison:
    """Owns one subprocess at a time; poll to completion and cancel before closing.

    Results use fresh directories. No run is overwritten or silently resumed.
    A failure/cancellation/timeout keeps diagnostics but never creates comparison.json.
    run_timeout is an optional wall-clock limit per measurement child, including
    warmups. Callers must keep polling through termination to reap the child.
    """
    def __init__(self, binary: Path, output: Path, methods: list[str], *, size=16, steps=12, repeats=3, accuracy='standard', run_timeout=None):
        if run_timeout is not None and (type(run_timeout) not in (int, float)
                                       or not math.isfinite(run_timeout) or run_timeout <= 0):
            raise ValueError('Run timeout must be finite and positive, or None')
        self.binary = binary.resolve(strict=True)
        known = {m['id'] for m in implementations(self.binary)}
        build = json.loads(subprocess.run([str(self.binary), '--build-info'], check=True, capture_output=True, text=True, timeout=10).stdout)
        if build.get('debug_assertions') is not False:
            raise ValueError('Comparison measurements require the release build')
        if not methods or len(set(methods)) != len(methods) or not set(methods) <= known:
            raise ValueError('Select distinct registered implementations')
        if any(type(n) is not int for n in (size, steps, repeats)) or not (1 <= size <= 128 and 1 <= steps <= 10000 and 1 <= repeats <= 20):
            raise ValueError('Size 1–128, steps 1–10000 and repeats 1–20 required')
        if accuracy not in ACCURACY:
            raise ValueError('Unknown accuracy setting')
        self.output = output.resolve()
        self.output.mkdir(parents=False, exist_ok=False)
        self.methods, self.size, self.steps, self.repeats, self.accuracy = methods, size, steps, repeats, accuracy
        # Rotate method order between repeats to reduce fixed-order bias. Warmup
        # per method is measured and retained but excluded from summary statistics.
        self.jobs = [(m, -1) for m in methods]
        for r in range(repeats):
            ordered = methods[r % len(methods):] + methods[:r % len(methods)]
            self.jobs.extend((m, r) for m in ordered)
        self.index = 0
        self.process = None
        self.log = None
        self.cancel_deadline = None
        self.run_timeout = run_timeout
        self.run_deadline = None
        self.rows = []
        self.status = 'running'
        self.error = None
        self.result = None
        self.environment = hardware() | {'executable_sha256': hashlib.sha256(self.binary.read_bytes()).hexdigest(),
                                         'timing': 'Rust Instant around accepted stepping and CSV writes, excludes process startup and PNG export',
                                         'build': build,
                                         'run_timeout_seconds': run_timeout,
                                         'build_requirement': 'cargo build --release --locked; debug assertions checked by executable, exact optimization flags require build receipt',
                                         'warmup_runs_per_method': 1, 'execution': 'sequential, rotating method order'}
        self._save_status()

    def _save_status(self):
        payload = {'status': self.status, 'completed_runs': len(self.rows), 'total_runs':len(self.jobs),
                   'error':self.error, 'run_timeout_seconds':self.run_timeout}
        temp = self.output / 'status.tmp'
        temp.write_text(json.dumps(payload, indent=2) + '\n')
        temp.replace(self.output / 'status.json')

    def cancel(self):
        self._stop('cancelling')

    def _stop(self, status):
        if self.status != 'running':
            return
        self.status = status
        if status == 'timing_out':
            self.error = f'Run {self.index} exceeded its {self.run_timeout:g}s process deadline; see its stderr.log'
        if self.process is not None:
            try:
                self.process.terminate()
            except ProcessLookupError:
                pass
            self.cancel_deadline = time.monotonic() + 2
        self._save_status()

    def poll(self):
        if self.status not in ACTIVE_STATUSES:
            return self.status
        if self.process is not None:
            if (self.status == 'running' and self.run_deadline is not None
                    and self.process.poll() is None and time.monotonic() >= self.run_deadline):
                self._stop('timing_out')
            if self.status in ('cancelling', 'timing_out') and self.process.poll() is None and time.monotonic() >= self.cancel_deadline:
                try:
                    self.process.kill()
                except ProcessLookupError:
                    pass
            rc = self.process.poll()
            if rc is None:
                return self.status
            self.process.wait()
            self.log.close()
            self.log = None
            self.process = None
            self.run_deadline = None
            self.cancel_deadline = None
            if self.status in ('cancelling', 'timing_out'):
                self.status = 'cancelled' if self.status == 'cancelling' else 'timed_out'
                self._save_status()
                return self.status
            if rc != 0:
                self.status = 'failed'
                self.error = f'Run {self.index} exited {rc}; see its stderr.log'
                self._save_status()
                return self.status
            try:
                method, repeat = self.jobs[self.index - 1]
                data = json.loads((self.run_dir / 'run.json').read_text())
                series = list(csv.DictReader((self.run_dir / 'steps.csv').read_text().splitlines()))
                if data['implementation'] != method or data['steps'] != self.steps or len(series) != self.steps:
                    raise ValueError('Run completion does not match requested workload')
                if data['requested_dt'] != REQUESTED_DT:
                    raise ValueError('Run requested_dt differs from requested comparison timestep')
                relative, pressure, actual = ACCURACY[self.accuracy]
                if (data['size'], data['pressure_relative_residual'], data['pressure_divergence_limit'], data['actual_divergence_limit']) != (self.size, relative, pressure, actual):
                    raise ValueError('Run settings differ from requested comparison settings')
                data.update(repeat=repeat, warmup=repeat < 0, path=self.run_dir.name,
                            pressure_iterations=sum(int(r['pressure_iterations']) for r in series),
                            maximum_step_divergence=max(float(r['actual_divergence_max']) for r in series))
                if not math.isfinite(data['measured_step_seconds']) or data['maximum_step_divergence'] > ACCURACY[self.accuracy][2]:
                    raise ValueError('Run failed comparison qualification')
                self.rows.append(data)
            except (OSError, ValueError, KeyError, TypeError) as exc:
                self.status, self.error = 'failed', str(exc)
                self._save_status()
                return self.status
        if self.status == 'cancelling':
            self.status = 'cancelled'
            self._save_status()
            return self.status
        if self.index == len(self.jobs):
            self._finish()
            return self.status
        method, repeat = self.jobs[self.index]
        self.run_dir = self.output / f'{self.index:03d}-{method}'
        relative, pressure, actual = ACCURACY[self.accuracy]
        command = [str(self.binary), '--implementation', method, '--size', str(self.size), '--steps', str(self.steps),
                   '--dt', str(REQUESTED_DT),
                   '--source-off-at', str(self.steps // 2), '--relative-residual', str(relative),
                   '--pressure-divergence-limit', str(pressure), '--actual-divergence-limit', str(actual),
                   '--output', str(self.run_dir)]
        self.log = (self.output / f'{self.index:03d}-stderr.log').open('w')
        try:
            started = time.monotonic()
            self.process = subprocess.Popen(command, stdout=self.log, stderr=self.log)
            self.run_deadline = None if self.run_timeout is None else started + self.run_timeout
        except OSError as exc:
            self.log.close(); self.log = None
            self.status, self.error = 'failed', str(exc)
        self.index += 1
        self._save_status()
        return self.status

    def _finish(self):
        summary = []
        for method in self.methods:
            samples = [r for r in self.rows if r['implementation'] == method and not r['warmup']]
            durations = [r['measured_step_seconds'] for r in samples]
            summary.append({'implementation':method, 'median_seconds':statistics.median(durations),
                            'min_seconds':min(durations), 'max_seconds':max(durations), 'samples':len(samples),
                            'managed_simulation_bytes':samples[0]['managed_simulation_bytes'],
                            'max_divergence':max(r['maximum_step_divergence'] for r in samples),
                            'pressure_iterations':samples[0]['pressure_iterations']})
        times = {r['accepted_time'] for r in self.rows}
        self.result = {'schema_version':1,'environment':self.environment,'workload':{'size':self.size,'steps':self.steps,
                       'repeats':self.repeats,'accuracy':self.accuracy,'requested_dt':REQUESTED_DT,'source_off_at':self.steps//2},
                       'equal_accepted_time':len(times)==1,'summary':summary,'runs':self.rows,
                       'accuracy_scope':'Algebraic residual and actual velocity divergence, not physical ground-truth error',
                       'memory_scope':'Retained simulation arrays, not process RSS', 'real_time_claim':False}
        temporary = self.output / 'comparison.tmp'
        temporary.write_text(json.dumps(self.result, indent=2, allow_nan=False)+'\n')
        temporary.replace(self.output / 'comparison.json')
        self.status = 'completed'
        self._save_status()

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,default=Path('target/release/rheon'))
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--size',type=int,default=16)
    parser.add_argument('--steps',type=int,default=12)
    parser.add_argument('--repeats',type=int,default=3)
    parser.add_argument('--accuracy',choices=ACCURACY,default='standard')
    parser.add_argument('--run-timeout',type=float,default=None,metavar='SECONDS',
                        help='optional positive per-process wall-clock deadline, including warmups; disabled by default')
    args=parser.parse_args()
    job=Comparison(args.binary,args.output,[m['id'] for m in implementations(args.binary.resolve())],size=args.size,steps=args.steps,repeats=args.repeats,accuracy=args.accuracy,run_timeout=args.run_timeout)
    try:
        while job.poll() in ACTIVE_STATUSES: time.sleep(0.05)
    except KeyboardInterrupt:
        job.cancel()
        while job.poll() in ACTIVE_STATUSES: time.sleep(0.05)
    print(json.dumps(job.result or {'status':job.status,'error':job.error},indent=2))
    return 0 if job.status=='completed' else 1

if __name__=='__main__':
    raise SystemExit(main())
