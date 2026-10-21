"""Alternate prebuilt equal-source-harness binaries; no concurrent CPU workloads."""
import json
import os
import tempfile
from types import SimpleNamespace
import subprocess
import sys
from pathlib import Path

baseline, candidate, output = sys.argv[1:4]
mode = sys.argv[4:]
controls = "--controls" in mode
sustained = "--sustained" in mode
two_workers = "--two-worker-control" in mode
mode = [arg for arg in mode if arg not in ["--controls", "--sustained", "--two-worker-control"]]
rows = []
cases = [(grid, workers, 'jacobi-pcg-v1') for grid in ['16', '32', '64', '16x32x8'] for workers in [1, 2, 4]]
cases += [('32', workers, 'sgs-pcg-v1') for workers in [1, 4]]
if controls:
    cases = [(grid, 1, 'jacobi-pcg-v1') for grid in ['16', '32', '64', '16x32x8']] + [('32', 4, 'jacobi-pcg-v1')]
if sustained:
    cases = [('16', 1, 'jacobi-pcg-v1'), ('32', 1, 'jacobi-pcg-v1')]
if two_workers:
    cases = [('32', 2, 'jacobi-pcg-v1')]
for grid, workers, solver in cases:
    steps = (1024 if grid == '16' else 64) if sustained else (3 if grid == '64' else 8)
    if two_workers:
        steps = 32
    for trial in range(6 if two_workers else (1 if sustained and grid == '16' else 3)):
        pair = {}
        for label, binary in ([('baseline', baseline), ('candidate', candidate)] if trial % 2 == 0 else [('candidate', candidate), ('baseline', baseline)]):
            command = [binary, grid, str(workers), str(steps), 'adaptive', solver] + mode
            with tempfile.TemporaryFile(mode="w+") as log:
                process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
                _, status, usage = os.wait4(process.pid, 0)
                process.returncode = os.waitstatus_to_exitcode(status)
                log.seek(0)
                result = SimpleNamespace(stdout=log.read(), stderr=f"peak_rss_kib={usage.ru_maxrss}")
                if process.returncode:
                    raise RuntimeError((command, process.returncode, result.stdout))
            fields = dict(item.split('=', 1) for item in result.stdout.splitlines()[0].split())
            extra = dict(item.split('=', 1) for item in result.stdout.splitlines()[1].split(' samples=')[0].split())
            fields.update(extra)
            pair[label] = fields
            rows.append(dict(version=label, grid=grid, workers=workers, solver=solver, trial=trial, command=command, stdout=result.stdout, resources=result.stderr, mode=mode, fields=fields))
            Path(output).write_text(json.dumps(rows, indent=2) + '\n')
        for key in ['fingerprint', 'report_fingerprint', 'iterations', 'accepted_time', 'divergence', 'residual', 'managed_bytes', 'capture_bytes']:
            if pair['baseline'][key] != pair['candidate'][key]:
                raise RuntimeError((grid, workers, key, pair))
        print(grid, workers, solver, trial, pair['baseline']['frame_seconds'], pair['candidate']['frame_seconds'], flush=True)
