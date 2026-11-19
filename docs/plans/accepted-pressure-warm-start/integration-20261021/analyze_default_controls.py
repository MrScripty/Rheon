#!/usr/bin/env python3
"""Summarize the frozen n=12 per-case control; never run or extend a cohort."""
import hashlib
import json
import math
from pathlib import Path
import random
import statistics
import sys


def require(ok, message):
    if not ok:
        raise ValueError(message)


def analyze(summary):
    cases = summary['cases']
    require(len(cases) == 2, 'exactly two cases required')
    require([(c['particles'], c['workers']) for c in cases] == [(1000, 1), (4096, 1)],
            'frozen particle and worker cases required')
    expected_orders = [['base', 'candidate'], ['candidate', 'base']] * 6
    random.Random(20261013).shuffle(expected_orders)
    results = []
    for case in cases:
        pairs = case['pairs']
        require(len(pairs) == 12, 'exactly twelve pairs; no extension or deletion')
        require([p['order'] for p in pairs] == expected_orders, 'frozen balanced orders')
        deltas = []
        for pair in pairs:
            base, candidate = pair['base'], pair['candidate']
            for row in (base, candidate):
                require(math.isfinite(row['frame_s']) and row['frame_s'] > 0,
                        'finite positive total frame time')
                require(len(row['samples_s']) == 64 and
                        all(math.isfinite(x) and x >= 0 for x in row['samples_s']),
                        'complete finite per-step samples')
                require(0.063999999999 < float(row['time']) < 0.064000000001 and
                        int(row['particles']) == case['particles'], 'accepted horizon/count')
            for key in ('time', 'fingerprint', 'bytes', 'particles'):
                require(base[key] == candidate[key], 'paired accepted output: ' + key)
            delta = 100 * (candidate['frame_s'] / base['frame_s'] - 1)
            require(math.isfinite(delta) and delta == pair['delta_pct'],
                    'finite full-precision paired delta')
            deltas.append(delta)
        ordered = sorted(deltas)
        median = statistics.median(deltas)
        require(median == case['median_delta_pct'], 'runner median mismatch')
        lower, upper = ordered[2], ordered[9]
        decision = ('adverse' if median > 0 else 'inconclusive' if upper > 0
                    else 'meets-frozen-non-adverse-rule')
        results.append({
            'particles': case['particles'], 'workers': 1, 'pairs': 12,
            'paired_deltas_pct': deltas, 'median_delta_pct': median,
            'minimum_delta_pct': ordered[0], 'maximum_delta_pct': ordered[-1],
            'negative': sum(x < 0 for x in deltas),
            'zero': sum(x == 0 for x in deltas),
            'positive': sum(x > 0 for x in deltas),
            'median_interval_pct': [lower, upper], 'decision': decision,
        })
    return {
        'schema': 'rheon-pr59-frozen-default-control-analysis-v1',
        'baseline_sha256': summary['baseline_sha256'],
        'candidate_sha256': summary['candidate_sha256'],
        'median_interval_coverage_pct': 96.142578125,
        'coverage_scope': 'pointwise per case, conditional on independent representative pairs',
        'host_limit': 'balanced order does not prove independence or stationarity under host drift',
        'cases': results,
        'performance_admission': 'meets-frozen-rule' if all(
            c['decision'] == 'meets-frozen-non-adverse-rule' for c in results) else 'blocked',
        'construction_evidence_substituted': False,
        'cohort_extended': False,
    }


def main():
    require(len(sys.argv) == 3, 'usage: analyze_default_controls.py summary.json new-output.json')
    source, output = map(Path, sys.argv[1:])
    raw = source.read_bytes()
    result = analyze(json.loads(raw))
    result['input_summary_sha256'] = hashlib.sha256(raw).hexdigest()
    result['analysis_script_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    encoded = json.dumps(result, indent=2, allow_nan=False) + '\n'
    with output.open('x') as stream:
        stream.write(encoded)
    print(result['performance_admission'])
    return 0 if result['performance_admission'] == 'meets-frozen-rule' else 2


if __name__ == '__main__':
    sys.exit(main())
