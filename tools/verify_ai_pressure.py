"""Independent checkpoint-to-native inference replay and CPU comparison summary.

Requires NumPy, not torch. Arguments: MODEL_PT EXAMPLE_OUTPUT_DIR.
Replays the pinned lab forward pass, checks native full operator residual, and
summarizes actual recorded provider invocations. No model training or mocks.
"""
import csv
import hashlib
import io
import json
from pathlib import Path
import platform
import statistics
import sys
import zipfile

import numpy as np
from export_ai_pressure import CHECKPOINT_SHA256


def verify(checkpoint, directory):
    raw = Path(checkpoint).read_bytes()
    if hashlib.sha256(raw).hexdigest() != CHECKPOINT_SHA256:
        raise ValueError('checkpoint identity')
    directory = Path(directory)
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        basis = np.frombuffer(archive.read('model/data/1'), '<f8').reshape(32, 32)
        gains = np.frombuffer(archive.read('model/data/0'), '<f8')
    rhs = np.fromfile(directory/'rhs.f64', '<f8')
    native = np.fromfile(directory/'native-pressure.f64', '<f8')
    if rhs.shape != (32768,) or native.shape != rhs.shape:
        raise ValueError('field shape')
    # Matrix multiplication along three axes, independently from native strides.
    value = rhs.reshape(32, 32, 32)
    for axis in (-1, -2, -3):
        value = np.moveaxis(np.moveaxis(value, axis, -1) @ basis.T, -1, axis)
    value = value.flatten() * np.concatenate(([0.0], gains))
    value = value.reshape(32, 32, 32)
    for axis in (-1, -2, -3):
        value = np.moveaxis(np.moveaxis(value, axis, -1) @ basis, -1, axis)
    expected = value.flatten() - value.flatten()[0]
    relative = float(np.linalg.norm(expected-native)/np.linalg.norm(expected))
    if not np.isfinite(relative) or relative > 1e-12:
        raise ValueError(f'native/checkpoint discrepancy: {relative}')
    field = native.reshape(32,32,32)
    product = np.zeros_like(field)
    for axis in range(3):
        lo = [slice(None)]*3
        hi = [slice(None)]*3
        lo[axis] = slice(0,-1)
        hi[axis] = slice(1,None)
        flux = (field[tuple(lo)] - field[tuple(hi)]) * (1/32)/1.25
        product[tuple(lo)] += flux
        product[tuple(hi)] -= flux
    residual = rhs-product.flatten()
    relative_residual = float(np.linalg.norm(residual)/np.linalg.norm(rhs))
    predicted = float(np.max(np.abs(residual)) * (1/60) / (1/32)**3)
    if not np.isfinite(relative_residual) or not np.isfinite(predicted) or relative_residual > 1e-8 or predicted > 1e-7:
        raise ValueError('independent physics gate')
    with (directory/'comparison.csv').open() as receipt:
        rows = list(csv.DictReader(receipt))
    expected_keys = {(str(seed), str(rep), method) for seed in (900,901,902,903) for rep in range(3) for method in ('jacobi','sgs','learned')}
    if len(rows) != 36 or {(r['seed'], r['repetition'], r['method']) for r in rows} != expected_keys:
        raise ValueError('comparison incomplete or duplicate')
    for r in rows:
        expected_id = {'jacobi':'jacobi-pcg-v1','sgs':'sgs-pcg-v1','learned':'lab-modal-pressure-v1'}[r['method']]
        if r['selected'] != expected_id:
            raise ValueError('unreported fallback')
        for key in ('total_ms','inference_ms','qualification_ms','fallback_ms','corrected_call_ms','residual_l2','relative_pressure_l2','actual_divergence_max'):
            if not np.isfinite(float(r[key])) or float(r[key]) < 0:
                raise ValueError('nonfinite or negative receipt')
        if float(r['actual_divergence_max']) > 1e-5 or float(r['relative_pressure_l2']) > 1e-5:
            raise ValueError('comparison quality')
        # The accepted divergence limit also bounds the full residual L2:
        # sqrt(cells) * divergence_limit * cell_volume / dt.
        if float(r['residual_l2']) > np.sqrt(32768)*1e-7/32768*60*(1+64*np.finfo(float).eps):
            raise ValueError('residual gate bound')
        if float(r['total_ms']) <= 0 or float(r['corrected_call_ms']) < float(r['total_ms']):
            raise ValueError('corrected call timing scope')
        active = ('inference_ms','qualification_ms') if r['method']=='learned' else ('fallback_ms',)
        if any(float(r[k]) <= 0 for k in active):
            raise ValueError('positive active stage timing')
        if float(r['total_ms']) < sum(float(r[k]) for k in ('inference_ms','qualification_ms','fallback_ms')):
            raise ValueError('timing scope')
    summary = {method: {'provider_median_ms': statistics.median(float(r['total_ms']) for r in rows if r['method']==method),
                        'corrected_call_median_ms': statistics.median(float(r['corrected_call_ms']) for r in rows if r['method']==method),
                        'max_pressure_relative_l2': max(float(r['relative_pressure_l2']) for r in rows if r['method']==method)}
               for method in ('jacobi','sgs','learned')}
    return {'schema':'rheon-ai-pressure-qualification-v1','checkpoint_sha256':CHECKPOINT_SHA256,
            'python':sys.version,'numpy':np.__version__,'platform':platform.platform(),
            'native_vs_checkpoint_relative_l2':relative,'independent_relative_residual':relative_residual,
            'independent_predicted_divergence':predicted,'cases':len(rows),'methods':summary,
            'scope':'single-thread native CPU, four deterministic MAC fields, three rotated repetitions; not full trajectories; construction separate'}


if __name__ == '__main__':
    print(json.dumps(verify(sys.argv[1], sys.argv[2]), indent=2))
