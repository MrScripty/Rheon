#!/usr/bin/env python3
"""Compare 16-step localized trajectories to an explicit tighter cold oracle.
Usage: compare_reference.py candidate_oracle_binary output_directory
Timings from this diagnostic are not part of the performance qualification.
"""
import csv
import hashlib
import io
import json
from pathlib import Path
import struct
import subprocess
import sys
import numpy as np

binary, directory = sys.argv[1:]
out = Path(directory)
out.mkdir(parents=True, exist_ok=True)

def read_fields(path):
    data = memoryview(path.read_bytes())
    counts = struct.unpack_from('<3Q', data)
    time, generation = struct.unpack_from('<dQ', data, 24)
    offset = 40
    fields = {}
    for name, dtype in [('x','<f4'),('y','<f4'),('z','<f4'),('tracer','<f4'),('pressure','<f8'),('fraction','<f8')]:
        length, = struct.unpack_from('<Q', data, offset)
        offset += 8
        fields[name] = np.frombuffer(data, dtype=dtype, count=length, offset=offset).astype(np.float64)
        offset += length * np.dtype(dtype).itemsize
    assert offset == len(data)
    return counts, time, generation, fields

results = []
for counts in [(16,16,16),(32,32,32),(64,64,64),(48,24,12)]:
    label = '-'.join(map(str,counts))
    snapshots = {}
    commands = {}
    for variant, policy, accuracy in [('cold','cold','normal'),('warm','warm','normal'),('reference','cold','tight')]:
        path = out / f'{label}-{variant}.bin'
        args = [binary,*map(str,counts),'1','16',policy,'localized','jacobi',accuracy,str(path)]
        proc = subprocess.run(args,text=True,capture_output=True,check=True)
        (out/f'{label}-{variant}.csv').write_text(proc.stdout)
        (out/f'{label}-{variant}.stderr').write_text(proc.stderr)
        rows = list(csv.DictReader(io.StringIO(proc.stdout)))
        assert len(rows)==16 and all(r['accepted']=='true' for r in rows)
        snapshots[variant] = read_fields(path)
        commands[variant] = args
        print(label,variant,'complete',flush=True)
    reference = snapshots['reference']
    item = {'counts':counts,'commands':commands,'metrics':{}}
    for variant in ['cold','warm']:
        actual = snapshots[variant]
        assert actual[:3] == reference[:3]
        metrics = {}
        for name in actual[3]:
            diff = actual[3][name]-reference[3][name]
            assert np.all(np.isfinite(diff))
            metrics[name] = {'linf':float(np.max(np.abs(diff))), 'rms':float(np.sqrt(np.mean(diff*diff)))}
        item['metrics'][variant] = metrics
    results.append(item)
    (out/'reference-summary.json').write_text(json.dumps({'binary_sha256':hashlib.sha256(Path(binary).read_bytes()).hexdigest(),'cases':results},indent=2)+'\n')
