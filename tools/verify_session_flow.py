#!/usr/bin/env python3
"""Independent oracle for API headless_session's documented 4x2x2 flow.

Checks exported complete flow, not generic dense3d admission or continuum accuracy.
The fixed fixture is an open unit box, u=(1/4,0,0), dt=1/16, rho=1000,
represented rho=800, initial fraction 1/2 in x-cell 1, and zero inlet fraction.
"""
import argparse
import json
import math
from pathlib import Path

LIMIT = 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def pairs(items):
    out = {}
    for key, value in items:
        require(key not in out, 'duplicate JSON key: ' + key)
        out[key] = value
    return out


def near(actual, expected, tolerance, name):
    require(type(actual) in (int, float) and math.isfinite(actual)
            and abs(actual - expected) <= tolerance, name)


def fraction(step, x):
    distance = x - 1
    if not 0 <= distance <= step:
        return 0.0
    return 0.5 * math.comb(step, distance) * (1 / 16) ** distance * (15 / 16) ** (step - distance)


def verify(path):
    path = Path(path)
    with path.open('rb') as stream:
        content = stream.read(LIMIT + 1)
    require(len(content) <= LIMIT, 'oversized frame stream')
    records = [json.loads(line, object_pairs_hook=pairs) for line in content.splitlines()]
    require(len(records) == 9, 'expected constructor plus eight accepted frames')
    previous_volume = 0.125
    max_fraction_error = 0.0
    max_divergence = 0.0
    for step, record in enumerate(records):
        require(set(record) == {'frame', 'time_s', 'dt_s', 'carrier_stamp', 'liquid_stamp',
                                'fields', 'diagnostics'}, 'frame schema')
        require(type(record['frame']) is int and record['frame'] == step, 'accepted frame sequence')
        near(record['time_s'], step / 16, 0, 'accepted time')
        near(record['dt_s'], 0 if step == 0 else 1 / 16, 0, 'accepted interval')
        for key, identity in (('carrier_stamp', 2**64 - 2), ('liquid_stamp', 2**64 - 1)):
            require(record[key] == {'id': str(identity), 'version': str(step)}, 'exact decimal stamp')
        fields = record['fields']
        lengths = {'velocity_x': 20, 'velocity_y': 24, 'velocity_z': 24,
                   'tracer': 16, 'fraction': 16, 'pressure': 16}
        require(set(fields) == set(lengths), 'field inventory')
        for key, length in lengths.items():
            require(type(fields[key]) is list and len(fields[key]) == length, 'field shape: ' + key)
            require(all(type(x) in (int, float) and math.isfinite(x) for x in fields[key]),
                    'nonfinite field: ' + key)
        for key in ('velocity_x', 'velocity_y', 'velocity_z', 'tracer'):
            expected = 0.25 if step and key == 'velocity_x' else 0.0
            for value in fields[key]:
                near(value, expected, 1e-8, 'hand uniform carrier/tracer: ' + key)
        # First projection accelerates the interior from rest: dp/dx=-rho*u/dt.
        # Pressure is interval projection pressure; subsequent uniform steps need zero.
        for i, value in enumerate(fields['pressure']):
            near(value, -1000 * (i % 4) if step == 1 else 0, 1e-8, 'hand projection pressure')
        for i, value in enumerate(fields['fraction']):
            error = abs(value - fraction(step, i % 4))
            max_fraction_error = max(max_fraction_error, error)
            require(0 <= value <= 1 and error <= 1e-12, 'binomial transported fraction')
        # Recompute divergence from exported staggered faces, not reported diagnostics.
        u, v, w = (fields[name] for name in ('velocity_x', 'velocity_y', 'velocity_z'))
        frame_divergence = 0.0
        for k in range(2):
            for j in range(2):
                for i in range(4):
                    div = ((u[i + 1 + 5 * (j + 2 * k)] - u[i + 5 * (j + 2 * k)]) * 4
                           + (v[i + 4 * (j + 1 + 3 * k)] - v[i + 4 * (j + 3 * k)]) * 2
                           + (w[i + 4 * (j + 2 * (k + 1))] - w[i + 4 * (j + 2 * k)]) * 2)
                    frame_divergence = max(frame_divergence, abs(div))
                    max_divergence = max(max_divergence, abs(div))
                    require(abs(div) <= 1e-8, 'exported face divergence')
        volume = math.fsum(fields['fraction']) / 16
        if step == 0:
            require(record['diagnostics'] is None, 'constructor is not an accepted step')
        else:
            diagnostics = record['diagnostics']
            require(type(diagnostics) is dict, 'missing accepted diagnostics')
            require(all(type(v) in (int, float) and math.isfinite(v)
                        for v in diagnostics.values()), 'nonfinite diagnostics')
            for key in ('carrier_divergence_s_inv', 'liquid_divergence_s_inv'):
                require(key in diagnostics, 'missing divergence diagnostic')
                near(diagnostics[key], frame_divergence, 1e-12, 'reported divergence fidelity')
            require('pressure_residual_m3_s2' in diagnostics
                    and 0 <= diagnostics['pressure_residual_m3_s2'] <= 1e-9, 'pressure residual diagnostic')
            require(type(diagnostics.get('pressure_iterations')) is int
                    and diagnostics['pressure_iterations'] >= 0, 'pressure iterations')
            require('rounding_budget_m3' in diagnostics
                    and 0 <= diagnostics['rounding_budget_m3'] <= 1e-12, 'rounding budget')
            outflow = fraction(step - 1, 3) / 64
            expected = {'volume_before_m3': previous_volume, 'volume_after_m3': volume,
                        'mass_after_kg': volume * 800, 'inward_m3': 0, 'outward_m3': outflow,
                        'source_m3': 0, 'balance_m3': 0}
            for key, value in expected.items():
                require(key in diagnostics, 'missing volume ledger: ' + key)
                near(diagnostics[key], value, 1e-12, 'independent volume ledger: ' + key)
            near(volume, previous_volume - outflow, 1e-12, 'open-boundary volume conservation')
        previous_volume = volume
    return {'schema': 'rheon-session-flow-oracle-v1', 'frames': len(records),
            'accepted_time_s': 0.5, 'final_volume_m3': previous_volume,
            'max_fraction_error': max_fraction_error, 'max_recomputed_divergence': max_divergence,
            'scope': 'fixed 4x2x2 session fixture; independent discrete reference, not continuum, AI or performance qualification'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('frames', type=Path)
    print(json.dumps(verify(parser.parse_args().frames), indent=2, allow_nan=False))
