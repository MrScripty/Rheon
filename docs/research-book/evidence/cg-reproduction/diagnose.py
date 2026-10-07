#!/usr/bin/env python3
"""Run unchanged fixtures in fresh directories with explicit OpenBLAS kernels.

Usage: python3 diagnose.py FRESH_OUTPUT
Requires the pinned companion dependencies and threadpoolctl 3.6.0.
The host must support all tested kernels, including SkylakeX AVX-512; this
cross-kernel diagnosis is not the normal Haswell-only CI reproduction command.
"""
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys

SOURCE = Path(__file__).resolve().parents[2] / 'companion'
sys.path.insert(0, str(SOURCE))
from historical_baselines import historical_baselines
ORACLE = '''
import hashlib, json, sys
sys.path.insert(0, sys.argv[1])
import experiments as e
import numpy as np
from scipy.sparse.linalg import cg, spsolve
from threadpoolctl import threadpool_info
B=e.box_incidence(8); A=(B@B.T).tocsr()
u=np.random.default_rng(20261003).normal(size=B.shape[1])
b=B@u/.025; reduced=A[1:,1:]; rhs=b[1:]
x,info=cg(reduced,rhs,rtol=1e-11,atol=0,maxiter=2000)
direct=spsolve(reduced,rhs)
p=np.r_[0.,x]; corrected=u-.025*(B.T@p)
assert info==0
print(json.dumps(dict(
    input_sha256={name:hashlib.sha256(a.tobytes()).hexdigest() for name,a in
                  [('u',u),('rhs',rhs),('A_data',reduced.data),('A_indices',reduced.indices),('A_indptr',reduced.indptr)]},
    pressure_linf_difference_from_direct=float(np.max(abs(x-direct))),
    pressure_relative_l2_difference_from_direct=float(np.linalg.norm(x-direct)/np.linalg.norm(direct)),
    full_residual_l2=float(np.linalg.norm(b-A@p)),
    corrected_divergence_l2=float(np.linalg.norm(B@corrected)),
    reduced_relative_residual=float(np.linalg.norm(rhs-reduced@x)/np.linalg.norm(rhs)),
    pressure=x.tolist(), runtime=threadpool_info()),indent=2))
'''


def main():
    output = Path(sys.argv[1]).resolve()
    output.mkdir(exist_ok=False)
    baseline = json.loads((historical_baselines() / 'results.json').read_text(encoding='utf-8'))
    summary = []
    for core in ('SkylakeX', 'Haswell', 'Nehalem'):
        for threads in (1, 4):
            case = output / f'{core}-{threads}'
            companion = case / 'companion'
            companion.mkdir(parents=True)
            for name in ('experiments.py', 'depth_experiments.py'):
                shutil.copyfile(SOURCE / name, companion / name)
            env = os.environ | {'OPENBLAS_CORETYPE': core, 'OPENBLAS_NUM_THREADS': str(threads)}
            for script in ('experiments', 'depth_experiments'):
                with (case / f'{script}.log').open('w', encoding='utf-8') as log:
                    subprocess.run([sys.executable, str(companion / f'{script}.py')],
                                   env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
            oracle = json.loads(subprocess.check_output(
                [sys.executable, '-c', ORACLE, str(companion)], env=env, text=True))
            (case / 'oracle.json').write_text(json.dumps(oracle, indent=2)+'\n', encoding='utf-8')
            result = json.loads((companion / 'results.json').read_text(encoding='utf-8'))
            history = result['projection']['histories']['cg']
            expected = baseline['projection']['histories']['cg']
            differences = [dict(index=i, expected=x, actual=y, absolute=abs(x-y))
                           for i, (x, y) in enumerate(zip(expected, history))
                           if not math.isclose(x, y, rel_tol=1e-8, abs_tol=1e-12)]
            summary.append(dict(core=core, threads=threads, count=len(history),
                                index53=history[53], differences=differences))
            print(core, threads, history[53], len(differences), flush=True)
    (output / 'summary.json').write_text(json.dumps(summary, indent=2)+'\n', encoding='utf-8')


if __name__ == '__main__':
    main()
