#!/usr/bin/env python3
"""Black-box receipt rejection probes against real candidate trajectory output.

No solver is changed. A successful native output is required first. Exit 1 means
at least one corrupted receipt was accepted; it does not invalidate original data.
"""
import argparse
import csv
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile


def mutate_csv(path, operation):
    with path.open() as stream:
        reader = csv.DictReader(stream)
        fields, rows = reader.fieldnames, list(reader)
    operation(rows)
    path.unlink()  # Copies are hard links: never mutate the original artifact.
    with path.open('w') as stream:
        writer = csv.DictWriter(stream, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)


def replace(path, content):
    path.unlink()
    path.write_bytes(content)


def zero_times(root):
    def frames(rows):
        for row in rows:
            for key in ('step_ms', 'provider_ms', 'inference_ms', 'qualification_ms', 'fallback_ms'):
                row[key] = '0'
    def runs(rows):
        for row in rows:
            for key in ('initialization_ms', 'steps_ms', 'rollout_wall_ms'):
                row[key] = '0'
    mutate_csv(root / 'frames.csv', frames)
    mutate_csv(root / 'runs.csv', runs)
    zero_process(root)


def zero_process(root):
    path = root / 'complete.json'
    data = json.loads(path.read_text())
    data.update(process_ms=0, model_load_ms=0)
    replace(path, json.dumps(data).encode())


def bad_timed_residual(root):
    def mutate(rows):
        for row in rows:
            if row['phase'] == 'timed':
                row['residual_l2'] = '1e100'
    mutate_csv(root / 'frames.csv', mutate)


def duplicate_evolution(root):
    # Keep expected frame/time/provider identities, but substitute all modes'
    # earlier physical states and matching numerical diagnostics at frame two.
    # A per-frame projection/reference test alone cannot establish a transition.
    for seed in (900, 901):
        for mode in ('jacobi', 'sgs', 'learned', 'injected-fallback',
                     'shifted-jacobi', 'unsupported-fallback'):
            directory = root / f'{seed}-{mode}'
            replace(directory / '002.bin', (directory / '001.bin').read_bytes())
    def mutate(rows):
        lookup = {(r['seed'], r['mode']): r for r in rows
                  if r['phase'] == 'capture' and r['frame'] == '1'}
        for row in rows:
            if row['phase'] == 'capture' and row['frame'] == '2':
                source = lookup[row['seed'], row['mode']]
                for key in ('residual_l2', 'predicted_divergence', 'actual_divergence',
                            'courant', 'tracer_integral'):
                    row[key] = source[key]
    mutate_csv(root / 'frames.csv', mutate)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('verifier', type=Path, help='candidate verify_ai_pressure_trajectory.py')
    parser.add_argument('native_output', type=Path, help='actual complete native trajectory directory')
    args = parser.parse_args()
    sys.path.insert(0, str(args.verifier.resolve().parent))
    spec = importlib.util.spec_from_file_location('candidate_trajectory_verifier', args.verifier)
    verifier = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(verifier)
    verifier.verify(args.native_output)  # A broken baseline cannot certify refusal probes.
    results = []
    for mutation in (zero_times, zero_process, bad_timed_residual, duplicate_evolution):
        with tempfile.TemporaryDirectory(dir=args.native_output.resolve().parent) as temporary:
            root = Path(temporary) / 'mutated'
            shutil.copytree(args.native_output, root, copy_function=os.link)
            mutation(root)
            try:
                verifier.verify(root)
            except ValueError as error:
                results.append({'probe': mutation.__name__, 'rejected': True, 'reason': str(error)})
            else:
                results.append({'probe': mutation.__name__, 'rejected': False})
    passed = all(r['rejected'] for r in results)
    print(json.dumps({'status': 'passed' if passed else 'rejection-gaps', 'probes': results}, indent=2))
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main())
