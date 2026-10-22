#!/usr/bin/env python3
"""Compare 16-step localized trajectories to an explicit tighter cold reference.
Usage: compare_reference.py candidate_binary output_directory
Timings from this diagnostic are not part of the performance qualification.
"""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
from measurement_checks import read_rows, require
import numpy as np

binary, directory = sys.argv[1:]
out = Path(directory)
out.mkdir(parents=True, exist_ok=False)

def read_fields(path):
    data = memoryview(path.read_bytes())
    counts = struct.unpack_from('<3Q', data)
    time, generation = struct.unpack_from('<dQ', data, 24)
    offset = 40
    nx, ny, nz = counts
    shapes = {'x':(nx+1)*ny*nz, 'y':nx*(ny+1)*nz, 'z':nx*ny*(nz+1),
              'tracer':nx*ny*nz, 'pressure':nx*ny*nz, 'fraction':nx*ny*nz}
    fields = {}
    for name, dtype in [('x','<f4'),('y','<f4'),('z','<f4'),('tracer','<f4'),('pressure','<f8'),('fraction','<f8')]:
        length, = struct.unpack_from('<Q', data, offset)
        offset += 8
        require(length == shapes[name], 'field length differs from geometry')
        fields[name] = np.frombuffer(data, dtype=dtype, count=length, offset=offset).astype(np.float64)
        offset += length * np.dtype(dtype).itemsize
    require(offset == len(data), 'malformed field snapshot')
    return counts, time, generation, fields

results = []
for counts in [(16,16,16),(32,32,32),(64,64,64),(48,24,12)]:
    label = '-'.join(map(str,counts))
    snapshots = {}
    commands = {}
    for variant, policy, accuracy in [('cold','cold','normal'),('warm','warm','normal'),('reference','cold','tight')]:
        path = out / f'{label}-{variant}.bin'
        args = [binary,*map(str,counts),'1','16',policy,'localized','jacobi',accuracy,str(path)]
        proc = subprocess.run(args,text=True,capture_output=True,check=False)
        (out/f'{label}-{variant}.csv').write_text(proc.stdout)
        (out/f'{label}-{variant}.stderr').write_text(proc.stderr)
        require(proc.returncode == 0, f'process exit {proc.returncode}: {args}')
        rows = read_rows(proc.stdout, 16, 'localized')
        require(len(rows)==16 and all(r['accepted']=='true' for r in rows), 'reference trajectory incomplete')
        snapshots[variant] = read_fields(path)
        require(snapshots[variant][:3] == (counts, 0.25, 16), 'snapshot geometry/horizon differs from request')
        commands[variant] = args
        print(label,variant,'complete',flush=True)
    reference = snapshots['reference']
    item = {'counts':counts,'commands':commands,'snapshot_sha256':{v:hashlib.sha256((out/f'{label}-{v}.bin').read_bytes()).hexdigest() for v in ['cold','warm','reference']},'metrics':{}}
    for variant in ['cold','warm']:
        actual = snapshots[variant]
        require(actual[:3] == reference[:3], 'reference geometry/horizon differs')
        metrics = {}
        for name in actual[3]:
            diff = actual[3][name]-reference[3][name]
            require(np.all(np.isfinite(diff)), 'nonfinite field discrepancy')
            metrics[name] = {'linf':float(np.max(np.abs(diff))), 'rms':float(np.sqrt(np.mean(diff*diff)))}
        item['metrics'][variant] = metrics
    results.append(item)
    (out/'reference-summary.json').write_text(json.dumps({'binary_sha256':hashlib.sha256(Path(binary).read_bytes()).hexdigest(),'accuracy_qualified':False,'interpretation':'terminal discrepancies are diagnostic; independent physical/oracle gates remain separate','cases':results},indent=2)+'\n')
