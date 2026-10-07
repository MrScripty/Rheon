#!/usr/bin/env python3
"""Validate retained diagnosis without executing or replacing original fixtures."""
import copy
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent
COMPANION = ROOT.parents[1] / 'companion'
sys.path.insert(0, str(COMPANION))
from verify_results import compare, profile_baselines, CG_PROFILE
from historical_baselines import historical_baselines

ARCHIVE = historical_baselines()
profile = json.loads((ARCHIVE / 'reproduction' / f'{CG_PROFILE}.json').read_text(encoding='utf-8'))
qualified = profile_baselines(ARCHIVE, profile)
baseline = {name: json.loads((ARCHIVE / name).read_text(encoding='utf-8'))
            for name in ('results.json', 'depth-results.json')}
reference = json.loads((ROOT / 'runs/Haswell-1/oracle.json').read_text(encoding='utf-8'))
for kernel in ('SkylakeX', 'Haswell', 'Nehalem'):
    for name in baseline:
        first = (ROOT / 'runs' / f'{kernel}-1' / name).read_bytes()
        repeated = (ROOT / 'runs' / f'{kernel}-4' / name).read_bytes()
        assert first == repeated
        actual = json.loads(first)
        if kernel == 'SkylakeX':
            compare(baseline[name], actual)
        if kernel == 'Haswell':
            compare(qualified[name], actual)
        expected = copy.deepcopy(baseline[name])
        if name == 'results.json':
            actual['projection']['histories']['cg'] = []
            expected['projection']['histories']['cg'] = []
        compare(expected, actual)
    for threads in (1, 4):
        oracle = json.loads((ROOT / 'runs' / f'{kernel}-{threads}' / 'oracle.json').read_text(encoding='utf-8'))
        assert oracle['input_sha256'] == reference['input_sha256']
        assert oracle['reduced_relative_residual'] < 1e-11
        assert oracle['corrected_divergence_l2'] < 1e-7
        assert oracle['pressure_relative_l2_difference_from_direct'] < 1e-11
    print(f'PASS {kernel}: exact input identity, thread-repeat bytes, original non-history tolerances, direct-solve and residual diagnostics')
print('PASS historical fixture/source hashes, original SkylakeX replay and explicit Haswell profile')
