#!/usr/bin/env python3
"""Independent 32-cubed production-provider fixture transitions.

Usage: verify_production_transition.py CAPTURE CANDIDATE_TOOLS
Uses candidate NumPy interpolation helpers, not its production Rust operators.
Numerical allowances cover f32 storage rounding; they are not physical accuracy.
"""
import csv
import json
import sys
from pathlib import Path
import numpy as np
sys.path.insert(0, sys.argv[2])
import ai_pressure_trajectory_oracle as o
root = Path(sys.argv[1])
rows = list(csv.DictReader((root / 'frames.csv').open()))
if not len(rows) == 24:
    raise ValueError('production transition check failed')
old = [np.zeros(s) for s in o.SHAPES]
tracer = np.zeros((32,) * 3)
fraction = np.broadcast_to(0.25 + 0.5 * np.arange(32) / 31.0, (32, 32, 32)).copy()
maxima = dict(velocity=0.0, tracer=0.0, fraction=0.0, divergence=0.0, volume_error=0.0)
time = 0.0

def inside(position):
    return np.logical_and.reduce([(p >= 0.25) & (p < 0.75) for p in position])
for frame, row in enumerate(rows, 1):
    dt = float(row['dt'])
    o.DT = dt
    time += dt
    if not (int(row['frame']) == frame and float(row['time']) == time):
        raise ValueError('production transition check failed')
    if not dt == (1 / 120 if frame == 9 else 1 / 60):
        raise ValueError('production transition check failed')
    a = np.fromfile(root / f'{frame:03}.bin', dtype='<f8')
    if not (len(a) == 32 ** 3 * 3 + o.FACES * 3 and np.isfinite(a).all()):
        raise ValueError('production transition check failed')
    pressure = a[:32 ** 3].reshape((32,) * 3)
    if pressure.flat[0] != 0.0:
        raise ValueError('pressure gauge')
    expected_provider = 'jacobi-pcg-v1' if frame == 9 else 'lab-modal-pressure-v1'
    if row['selected'] != expected_provider:
        raise ValueError('selected provider')
    off = 32 ** 3
    velocity = []
    for shape in o.SHAPES:
        velocity.append(a[off:off + o.FACES].reshape(shape))
        off += o.FACES
    observed_tracer = a[off:off + 32 ** 3].reshape((32,) * 3)
    off += 32 ** 3
    observed_fraction = a[off:].reshape((32,) * 3)
    divergence = np.zeros((32,) * 3)
    expected_fraction = fraction.copy()
    for d, (prior, shape, offset) in enumerate(zip(old, o.SHAPES, o.OFFSETS)):
        axis = 2 - d
        position = o.coordinates(shape, offset, 0.0)
        transported = o.rounded(o.sample(prior, offset, o.backtrace(old, position, 0.0), 0.0))
        impulse = inside(position) * dt * (0.001, 0.002, 0.0)[d]
        candidate = o.rounded(transported + impulse)
        interior = [slice(None)] * 3
        interior[axis] = slice(1, -1)
        interior = tuple(interior)
        expected = np.zeros(shape)
        delta = dt * 32 / 1.25 * np.diff(pressure, axis=axis)
        expected[interior] = o.rounded(candidate[interior] - delta)
        bound = 8 * o.EPS32 * (np.abs(transported) + np.abs(impulse)) + 8 * o.TINY32
        bound[interior] += 8 * o.EPS32 * (np.abs(candidate[interior]) + np.abs(delta)) + 8 * o.TINY32
        error = np.abs(expected - velocity[d])
        if not np.all(error <= bound):
            raise ValueError((frame, d, 'velocity', error.max(), bound.max()))
        maxima['velocity'] = max(maxima['velocity'], float(error.max()))
        divergence += 32 * np.diff(velocity[d], axis=axis)
        low = np.concatenate([np.zeros_like(np.take(fraction, [0], axis=axis)), fraction], axis=axis)
        high = np.concatenate([fraction, np.zeros_like(np.take(fraction, [0], axis=axis))], axis=axis)
        flux = dt * 32 * velocity[d] * np.where(velocity[d] > 0, low, high)
        expected_fraction -= np.diff(flux, axis=axis)
    position = o.coordinates((32,) * 3, (0.5,) * 3, 0.0)
    transported = o.rounded(o.sample(tracer, (0.5,) * 3, o.backtrace(velocity, position, 0.0), 0.0))
    emission = inside(position) * dt * 0.2
    expected = o.rounded(np.minimum(transported + emission, 1.0))
    error = np.abs(expected - observed_tracer)
    bound = 16 * o.EPS32 * (np.abs(transported) + emission) + 128 * o.EPS64 * max(1.0, float(tracer.max()))
    if not np.all(error <= bound):
        raise ValueError((frame, 'tracer', error.max()))
    maxima['tracer'] = max(maxima['tracer'], float(error.max()))
    error = np.max(np.abs(expected_fraction - observed_fraction))
    if not error <= 64 * o.EPS64:
        raise ValueError((frame, 'fraction', error))
    maxima['fraction'] = max(maxima['fraction'], float(error))
    div = float(np.abs(divergence).max())
    if not (div <= 1e-05 and div == float(row['divergence'])):
        raise ValueError('production transition check failed')
    maxima['divergence'] = max(maxima['divergence'], div)
    volume_error = abs(float(observed_fraction.mean()) - 0.5)
    if not volume_error <= 64 * o.EPS64:
        raise ValueError('production transition check failed')
    maxima['volume_error'] = max(maxima['volume_error'], volume_error)
    old, tracer, fraction = (velocity, observed_tracer, observed_fraction)
print(json.dumps({'frames': len(rows), 'accepted_seconds': time, 'maxima': maxima, 'scope': 'independent declared discrete production transitions; engineering roundoff allowances; no continuum qualification'}, indent=2))
