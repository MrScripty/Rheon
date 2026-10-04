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
import struct
import subprocess
import time
import zlib

ACCURACY = {
    'standard': (1e-9, 1e-7, 1e-5),
    'tight': (1e-11, 1e-9, 1e-5),
}
REQUESTED_DT = 0.02
PRESSURE_ABSOLUTE_RESIDUAL = 1e-12
PRESSURE_MAX_ITERATIONS = 2000
MEMORY_MIB = 64
RUN_SCHEMA_VERSION = 2
SMOKE_MODEL = 'fixed-box smoke tracer, prescribed localized Y acceleration'
STEP_FIELDS = ('step', 'time', 'dt', 'pressure_iterations', 'full_residual_max',
               'actual_divergence_max', 'courant', 'tracer_integral', 'kinetic_energy')
ACTIVE_STATUSES = ('running', 'cancelling', 'timing_out')

def _settings(size, steps, accuracy):
    relative, pressure, actual = ACCURACY[accuracy]
    return {'schema_version': RUN_SCHEMA_VERSION, 'model': SMOKE_MODEL,
            'size': size, 'steps': steps, 'source_off_at': steps // 2,
            'requested_dt': REQUESTED_DT, 'pressure_relative_residual': relative,
            'pressure_absolute_residual': PRESSURE_ABSOLUTE_RESIDUAL,
            'pressure_divergence_limit': pressure, 'actual_divergence_limit': actual,
            'pressure_iteration_limit': PRESSURE_MAX_ITERATIONS,
            'managed_budget_bytes': MEMORY_MIB * 1024 * 1024}

def _qualify_settings(data, method, size, steps, accuracy):
    if not isinstance(data, dict):
        raise ValueError('Run manifest must be an object')
    expected = _settings(size, steps, accuracy) | {'implementation': method}
    for key, value in expected.items():
        if type(data[key]) is not type(value) or data[key] != value:
            raise ValueError(f'Run {key} differs from requested comparison settings')

def _finite_nonnegative(value, field):
    if type(value) not in (int, float) or not math.isfinite(value) or value < 0:
        raise ValueError(f'Run {field} must be finite and nonnegative')
    return value

def _qualify_series(data, path, size, steps, accuracy):
    with path.open(newline='') as stream:
        reader = csv.DictReader(stream, strict=True)
        if reader.fieldnames != list(STEP_FIELDS):
            raise ValueError('Run step columns differ from schema 2')
        series = list(reader)
    if len(series) != steps:
        raise ValueError('Run completion does not match requested workload')
    elapsed, iterations, maximum_divergence = 0.0, 0, 0.0
    h = 1.0 / size
    cell_volume = h * h * h
    for index, row in enumerate(series, 1):
        if set(row) != set(STEP_FIELDS):
            raise ValueError('Run step has extra columns')
        if int(row['step']) != index:
            raise ValueError('Run steps must be sequential')
        count = int(row['pressure_iterations'])
        if not 0 <= count <= PRESSURE_MAX_ITERATIONS:
            raise ValueError('Run pressure iterations exceed the admitted range')
        values = {field: _finite_nonnegative(float(row[field]), field)
                  for field in STEP_FIELDS[1:] if field != 'pressure_iterations'}
        dt = values['dt']
        if not 0 < dt <= REQUESTED_DT:
            raise ValueError('Run accepted dt exceeds the requested range')
        previous_time, elapsed = elapsed, elapsed + dt
        if elapsed <= previous_time or values['time'] != elapsed:
            raise ValueError('Run time differs from cumulative accepted dt')
        # CLI RHS/residual use integrated flux units, as in pressure.rs.
        predicted = values['full_residual_max'] * dt / cell_volume
        if (predicted > ACCURACY[accuracy][1]
                or values['actual_divergence_max'] > ACCURACY[accuracy][2]
                or values['courant'] > 1.0):
            raise ValueError('Run step exceeds pressure, actual divergence or Courant admission')
        iterations += count
        maximum_divergence = max(maximum_divergence, values['actual_divergence_max'])
    for field in ('accepted_time', 'last_divergence_max', 'measured_step_seconds'):
        _finite_nonnegative(data[field], field)
    if data['accepted_time'] != elapsed or data['last_divergence_max'] != values['actual_divergence_max']:
        raise ValueError('Run manifest differs from final accepted diagnostics')
    # Two f32 field sets plus six f64 cell arrays; capacities may exceed lengths.
    minimum_payload = 8 * (3 * size * size * (size + 1)) + 56 * size ** 3
    for field, lower, upper in (('managed_simulation_bytes', minimum_payload, data['managed_budget_bytes']),
                                ('raw_export_pixel_bytes', size * size, 16 * 1024 * 1024)):
        if type(data[field]) is not int or not lower <= data[field] <= upper:
            raise ValueError(f'Run {field} exceeds its retained payload range')
    for field in ('whole_process_memory_cap_claimed', 'real_time_performance_claimed'):
        if data[field] is not False:
            raise ValueError(f'Run {field} must remain false')
    return iterations, maximum_divergence

def _qualify_png(path, size):
    # Check the exported container, dimensions and chunk integrity, not a pixel
    # decode or agreement with the simulation; Rust renderer tests own those.
    with path.open('rb') as stream:
        payload = stream.read(16 * 1024 * 1024 + 1)
    if len(payload) > 16 * 1024 * 1024 or payload[:8] != b'\x89PNG\r\n\x1a\n':
        raise ValueError('Run opacity export is not a bounded PNG')
    position, chunks = 8, []
    while position + 12 <= len(payload):
        length = int.from_bytes(payload[position:position + 4], 'big')
        end = position + 12 + length
        if end > len(payload):
            raise ValueError('Run opacity PNG is truncated')
        kind = payload[position + 4:position + 8]
        body = payload[position + 8:end - 4]
        crc = int.from_bytes(payload[end - 4:end], 'big')
        if zlib.crc32(kind + body) != crc:
            raise ValueError('Run opacity PNG chunk checksum differs')
        if not chunks:
            if kind != b'IHDR' or body != struct.pack('>IIBBBBB', size, size, 8, 0, 0, 0, 0):
                raise ValueError('Run opacity PNG differs from the requested grayscale dimensions')
        chunks.append(kind)
        position = end
        if kind == b'IEND':
            if length != 0 or position != len(payload) or b'IDAT' not in chunks:
                raise ValueError('Run opacity PNG has invalid completion')
            return
    raise ValueError('Run opacity PNG has no complete end chunk')

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
                _qualify_settings(data, method, self.size, self.steps, self.accuracy)
                iterations, divergence = _qualify_series(data, self.run_dir / 'steps.csv',
                                                        self.size, self.steps, self.accuracy)
                _qualify_png(self.run_dir / 'opacity.png', self.size)
                data.update(repeat=repeat, warmup=repeat < 0, path=self.run_dir.name,
                            pressure_iterations=iterations, maximum_step_divergence=divergence)
                self.rows.append(data)
            except (OSError, ValueError, KeyError, TypeError, OverflowError, csv.Error) as exc:
                self.status, self.error = 'failed', str(exc)
                self._save_status()
                return self.status
        if self.status == 'cancelling':
            self.status = 'cancelled'
            self._save_status()
            return self.status
        if self.index == len(self.jobs):
            try:
                self._finish()
            except (OSError, ValueError, TypeError) as exc:
                self.result = None
                (self.output / 'comparison.json').unlink(missing_ok=True)
                self.status, self.error = 'failed', str(exc)
                self._save_status()
            return self.status
        method, repeat = self.jobs[self.index]
        self.run_dir = self.output / f'{self.index:03d}-{method}'
        relative, pressure, actual = ACCURACY[self.accuracy]
        command = [str(self.binary), '--implementation', method, '--size', str(self.size), '--steps', str(self.steps),
                   '--dt', str(REQUESTED_DT),
                   '--source-off-at', str(self.steps // 2), '--relative-residual', str(relative),
                   '--pressure-divergence-limit', str(pressure), '--actual-divergence-limit', str(actual),
                   '--max-iterations', str(PRESSURE_MAX_ITERATIONS), '--memory-mib', str(MEMORY_MIB),
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
        workload = _settings(self.size, self.steps, self.accuracy)
        workload['run_schema_version'] = workload.pop('schema_version')
        workload.update(repeats=self.repeats, accuracy=self.accuracy)
        result = {'schema_version':1,'environment':self.environment,'workload':workload,
                  'equal_accepted_time':len(times)==1,'summary':summary,'runs':self.rows,
                  'accuracy_scope':'Algebraic residual and actual velocity divergence, not physical ground-truth error',
                  'memory_scope':'Retained simulation arrays, not process RSS', 'real_time_claim':False}
        temporary = self.output / 'comparison.tmp'
        temporary.write_text(json.dumps(result, indent=2, allow_nan=False)+'\n')
        temporary.replace(self.output / 'comparison.json')
        self.status = 'completed'
        self._save_status()
        self.result = result

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
