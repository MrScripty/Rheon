#!/usr/bin/env python3
"""Record source identities and requalify preserved packets without timing runs."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path('tools').resolve()))
from rheon_compare import _qualify_png, _qualify_series, _qualify_settings

BASE = '416744700a45a8e74de27ca4e25cd00b0931926c'
ROOT = Path('evidence/publication-terminal-repair')
SOURCES = ('tools/rheon_compare.py', 'tools/test_result_qualification.py', 'docs/COMPARISON.md')
PROTECTED = ('src', 'tests', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'proofs',
             'docs/research-book', 'evidence/cloud-qualification', 'evidence/result-qualification',
             'evidence/demo-16', 'evidence/demo-plume', 'evidence/demo-64')


def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def check_protected_inventory():
    require(git('diff', BASE, '--', *PROTECTED) == '', 'Protected source/evidence changed')
    additions = subprocess.check_output(
        ['git', 'ls-files', '--others', '-z', '--', *PROTECTED])
    require(not additions, 'Untracked or ignored protected path additions: ' +
            ', '.join(p.decode(errors='replace') for p in additions.split(b'\0') if p))


def verify():
    source = git('log', '-1', '--format=%H', '--', *SOURCES)
    check_protected_inventory()
    hashes = {}
    for name in SOURCES:
        raw = Path(name).read_bytes()
        require(raw == subprocess.check_output(['git', 'show', f'{source}:{name}']), 'Uncommitted source: '+name)
        hashes[name] = hashlib.sha256(raw).hexdigest()
    binary_hash = hashlib.sha256(Path('target/release/rheon').read_bytes()).hexdigest()
    require(binary_hash == 'e10000f4fb61ba09765d7e9cbc76cac0463550cc7aecbcd72a66d458e0a1a584', 'Release binary differs')
    packets = 0
    for case in sorted(Path('evidence/result-qualification/benchmarks').glob('*/comparison.json')):
        result = json.loads(case.read_text())
        workload = result['workload']
        for run in result['runs']:
            path = case.parent / run['path']
            manifest = json.loads((path / 'run.json').read_text())
            _qualify_settings(manifest, run['implementation'], workload['size'], workload['steps'], workload['accuracy'])
            iterations, divergence = _qualify_series(manifest, path / 'steps.csv', workload['size'], workload['steps'], workload['accuracy'])
            require((iterations, divergence) == (run['pressure_iterations'], run['maximum_step_divergence']), 'Packet summary differs')
            _qualify_png(path / 'opacity.png', workload['size'])
            packets += 1
    require(packets == 48, 'Incomplete retained benchmark packet inventory')
    receipt = {'reviewed_base': BASE, 'repair_source_commit': source,
               'repair_source_tree': git('rev-parse', f'{source}^{{tree}}'), 'source_sha256': hashes,
               'binary_sha256': binary_hash, 'protected_paths_unchanged': list(PROTECTED),
               'retained_packets_requalified': packets, 'new_timing_measurements': False,
               'harness_tests': 20, 'baseline_combined_failure_errors': 2,
               'failure_contract': 'Terminal failed before cleanup/persistence; all errors retained; repeated polls remain failed',
               'limits': 'Cleanup failure may leave comparison.json; persistent status-write failure may leave stale status.json',
               'parent_independent_review': 'pending'}
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--receipt-output', type=Path, default=ROOT / 'successor-verification.json')
    args = parser.parse_args()
    receipt = verify()
    receipt['verifier_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    with args.receipt_output.open('x') as stream:
        stream.write(json.dumps(receipt, indent=2, allow_nan=False) + '\n')
    print('PASS exact historical sources, complete protected inventory, unchanged binary and 48 retained packets')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError, OverflowError) as exc:
        raise SystemExit(f'FAIL historical terminal-repair qualification: {exc}') from exc
