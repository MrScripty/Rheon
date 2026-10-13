"""Initial-residual negative controls; baseline/candidate pressure and particle binaries."""
import json
import random
import subprocess
import sys
from pathlib import Path

pressure_baseline, pressure_candidate, particle_baseline, particle_candidate, output = sys.argv[1:]
random.seed(603)
rows = []
cases = []
for workers in [1, 2, 4]:
    cases.append((f'emitter-32-{workers}', pressure_baseline, pressure_candidate,
                  ['32', str(workers), '8', 'adaptive', 'jacobi-pcg-v1', 'emitter'], 6))
    for particles in [8, 16, 32]:
        cases.append((f'particles-{particles}-{workers}', particle_baseline, particle_candidate,
                      [str(particles), str(workers), '20'], 8))
for case, baseline, candidate, args, trials in cases:
    orders = [['baseline', 'candidate'], ['candidate', 'baseline']] * (trials // 2)
    random.shuffle(orders)
    for trial, order in enumerate(orders):
        pair = {}
        for label in order:
            command = [baseline if label == 'baseline' else candidate, *args]
            result = subprocess.run(command, capture_output=True, text=True, check=True)
            fields = dict(item.split('=', 1) for item in result.stdout.splitlines()[0].split(' samples=')[0].split())
            if case.startswith('emitter'):
                fields.update(item.split('=', 1) for item in result.stdout.splitlines()[1].split(' samples=')[0].split())
            pair[label] = fields
            rows.append(dict(case=case, version=label, trial=trial, order=order,
                             command=command, fields=fields, stdout=result.stdout))
            Path(output).write_text(json.dumps(rows, indent=2) + '\n')
        keys = ['steps', 'accepted_time', 'managed_bytes', 'fingerprint']
        keys += ['particles'] if case.startswith('particles') else ['divergence', 'capture_bytes', 'iterations', 'residual', 'report_fingerprint']
        for key in keys:
            if pair['baseline'][key] != pair['candidate'][key]:
                raise RuntimeError((case, trial, key))
        print(case, trial, pair['baseline']['frame_seconds'], pair['candidate']['frame_seconds'], flush=True)
