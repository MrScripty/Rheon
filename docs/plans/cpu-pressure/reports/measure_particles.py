"""Unchanged dense-particle CPU controls, with identical input/horizon per pair."""
import json
import subprocess
import sys
from pathlib import Path

baseline, candidate, output = sys.argv[1:]
rows = []
for samples in [8, 16, 32]:
    for workers in [1, 2, 4]:
        for trial in range(3):
            pair = {}
            versions = [('baseline', baseline), ('candidate', candidate)]
            for label, binary in versions if trial % 2 == 0 else reversed(versions):
                command = [binary, str(samples), str(workers), '5']
                result = subprocess.run(command, capture_output=True, text=True, check=True)
                fields = dict(item.split('=', 1) for item in result.stdout.splitlines()[0].split(' samples=')[0].split())
                pair[label] = fields
                rows.append(dict(version=label, samples_per_axis=samples, workers=workers, trial=trial, command=command, fields=fields, stdout=result.stdout))
                Path(output).write_text(json.dumps(rows, indent=2) + '\n')
            for key in ['particles', 'steps', 'accepted_time', 'managed_bytes', 'fingerprint']:
                if pair['baseline'][key] != pair['candidate'][key]:
                    raise RuntimeError((samples, workers, key))
            print(samples, workers, trial, pair['baseline']['frame_seconds'], pair['candidate']['frame_seconds'], flush=True)
