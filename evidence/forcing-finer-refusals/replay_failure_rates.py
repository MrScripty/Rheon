"""Independently evaluate captured candidates; never advance or publish them."""
import json
import sys
from pathlib import Path
import numpy as np

ROOT = Path(__file__).resolve().parents[2]
P = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'evidence/forced-extruded-liquid'))
import reference as r


def require(ok, message):
    if not ok:
        raise ValueError(message)


def run():
    events = [json.loads(line) for line in (P / 'native-trace-default-stderr.log').read_text().splitlines()]
    selected = []
    key = None
    checks = []
    retry = False
    failed = False
    for row in events:
        event = row['event']
        if event == 'case':
            key = (row['kind'], row['load'], row['h'])
            retry = False
            failed = False
        elif event == 'retry_begin':
            retry = True
        elif event == 'newton_check' and not retry:
            if row['iteration'] == 1:
                checks = []
            checks.append(row)
        elif event == 'refusal' and not retry:
            failed = True
            selected.extend((key, x, 16, 'original_check') for x in checks)
        elif event == 'post_window_observation' and failed and not retry:
            observed = dict(row, q=checks[0]['q'], eta=checks[0]['eta'])
            selected.append((key, observed, row['order'], 'unused_candidate_observation'))
    require(len(selected) == 54, '42 actual original checks plus twelve observations')
    results = []
    for key, row, order, role in selected:
        kind, load, h = key
        a = r.A * (1 if load == 'forward' else -1)
        value = r.evaluate(np.array(row['q']), np.array(row['eta']), np.array(row['unknown']), h, a, order)
        independent = value['residual'] / h
        direct = value['direct_residual'] / h
        gap = float(np.max(abs(independent - row['rate'])))
        direct_gap = float(np.max(abs(direct - row['direct_rate'])))
        require(np.all(np.isfinite(independent)) and max(gap, direct_gap) <= 1e-11, 'original full-momentum comparison scale')
        results.append(dict(kind=kind, load=load, h=h, order=order, role=role,
            original_iteration=row.get('iteration'), native_rate_norm=row['rate_norm'],
            independent_rate_norm=float(np.linalg.norm(independent)),
            max_native_independent_rate_component_gap=gap,
            max_native_independent_direct_component_gap=direct_gap,
            scope='Equation evaluation of captured candidates, not an independently accepted step.'))
    return dict(status='PASS_INDEPENDENT_CAPTURED_EQUATION_COMPARISON_AT_ORIGINAL_SCALE',
        actual_original_check_evaluations=42, actual_observational_evaluations=12,
        original_full_momentum_comparison_scale=1e-11,
        unchanged_native_newton_threshold=1e-13,
        max_native_independent_rate_component_gap=max(x['max_native_independent_rate_component_gap'] for x in results),
        max_native_independent_direct_component_gap=max(x['max_native_independent_direct_component_gap'] for x in results),
        original_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND',
        native_refusals_remain=6, independently_accepted_endpoints_added=0,
        arithmetic_floor_proved=False, rows=results)


if __name__ == '__main__':
    print(json.dumps(run(), indent=2))
