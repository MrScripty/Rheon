#!/usr/bin/env python3
"""Verify accepted integration sources, local logs, preserved packets and replay.

Run from repository root after combined gates. Never reruns benchmark timings or
rewrites any historical receipt; current release and old measured binary remain
separate identities.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

sys.path.insert(0, str(Path('tools').resolve()))
from rheon_compare import _qualify_png, _qualify_series, _qualify_settings

BASE = '0de6a857adb17a6ee0312e889aff50e2cd7aa6dd'
VALIDATION = '416744700a45a8e74de27ca4e25cd00b0931926c'
ACCEPTED = '8bc0f54604d98d838b3a1ab2f6fcdfff9af2ac51'
ROOT = Path('evidence/result-integration-qualification')
PROTECTED = ('src', 'tests', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'proofs',
             'docs/research-book', 'evidence/cloud-qualification', 'evidence/result-qualification',
             'evidence/publication-terminal-repair', 'evidence/demo-16', 'evidence/demo-plume', 'evidence/demo-64')


def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()


def require(condition, message):
    if not condition:
        raise ValueError(message)


for ancestor in (BASE, VALIDATION):
    subprocess.run(['git', 'merge-base', '--is-ancestor', ancestor, ACCEPTED], check=True)
# Accepted validation evidence legitimately differs from BASE; numerical/proof
# source and all evidence must match the accepted snapshot at integration.
require(git('diff', ACCEPTED, '--', *PROTECTED) == '', 'Accepted protected source/evidence changed')
require(git('diff', BASE, '--', *PROTECTED[:7], *PROTECTED[-3:]) == '', 'Numerical/proof source changed from prior PR3')
paths = git('ls-files', 'src', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
            '.github/workflows/rust-rheon.yml', 'docs/COMPARISON.md').splitlines()
hashes = {}
for name in paths:
    raw = Path(name).read_bytes()
    require(raw == subprocess.check_output(['git', 'show', f'{ACCEPTED}:{name}']), 'Accepted source differs: '+name)
    hashes[name] = hashlib.sha256(raw).hexdigest()
counts = {}
for feature, expected in (('default', 49), ('core', 44), ('desktop', 55)):
    log = (ROOT / f'test-{feature}.log').read_text()
    counts[feature] = sum(map(int, re.findall(r'test result: ok\. (\d+) passed;', log)))
    require(counts[feature] == expected and '; 0 failed;' in log and 'test result: FAILED' not in log,
            'Rust feature tests differ: '+feature)
    clippy = (ROOT / f'clippy-{feature}.log').read_text()
    require('Finished' in clippy and 'error:' not in clippy, 'Strict Clippy failed: '+feature)
require((ROOT / 'fmt.log').read_text().strip() == '', 'Formatting output not clean')
harness = (ROOT / 'harness-tests.log').read_text()
require('Ran 20 tests' in harness and harness.rstrip().endswith('OK'), 'Harness tests failed')
packets = 0
for case in sorted(Path('evidence/result-qualification/benchmarks').glob('*/comparison.json')):
    comparison = json.loads(case.read_text())
    workload = comparison['workload']
    require(comparison['environment']['executable_sha256'] == 'e10000f4fb61ba09765d7e9cbc76cac0463550cc7aecbcd72a66d458e0a1a584',
            'Historical measured identity changed')
    for run in comparison['runs']:
        path = case.parent / run['path']
        data = json.loads((path / 'run.json').read_text())
        _qualify_settings(data, run['implementation'], workload['size'], workload['steps'], workload['accuracy'])
        require(_qualify_series(data, path / 'steps.csv', workload['size'], workload['steps'], workload['accuracy'])
                == (run['pressure_iterations'], run['maximum_step_divergence']), 'Retained packet summary differs')
        _qualify_png(path / 'opacity.png', workload['size'])
        packets += 1
require(packets == 48, 'Retained packet inventory incomplete')
replays = json.loads((ROOT / 'replay/receipt.json').read_text())
require(len(replays) == 3, 'Original demo replay incomplete')
for replay in replays:
    fixture = Path(replay['fixture'])
    actual = ROOT / 'replay' / fixture.name
    for name in ('steps.csv', 'opacity.png'):
        require((actual / name).read_bytes() == (fixture / name).read_bytes(), 'Original fixture differs: '+str(fixture / name))
binary = Path('target/release/rheon').resolve(strict=True)
build = json.loads(subprocess.check_output([str(binary), '--build-info'], text=True))
require(build['debug_assertions'] is False, 'Release executable required')
environment = {name: os.environ.get(name) for name in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS',
               'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER', 'CARGO_BUILD_TARGET', 'CARGO_BUILD_JOBS', 'CARGO_TARGET_DIR')}
receipt = {'prior_pr3_head': BASE, 'accepted_validation_head': VALIDATION,
           'accepted_terminal_repair_head': ACCEPTED, 'accepted_source_tree': git('rev-parse', ACCEPTED+'^{tree}'),
           'accepted_source_sha256': hashes, 'local_rust_test_counts': counts, 'python_harness_tests': 20,
           'strict_clippy_features': ['default', 'core-only', 'desktop'], 'formatting': 'passed',
           'current_release_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
           'current_release_build_info': build, 'current_build_environment': environment,
           'profile': 'default Cargo optimized release; compiler.txt and release-build.log retain toolchain/build facts',
           'historical_measured_release_sha256': 'e10000f4fb61ba09765d7e9cbc76cac0463550cc7aecbcd72a66d458e0a1a584',
           'original_demo_png_csv_pairs_byte_identical': 3, 'retained_packets_requalified': packets,
           'new_timing_measurements': False, 'default_process_deadline': None,
           'limits': ['historical fixed-binary verifier rejects the rebuilt executable; original receipt unchanged',
                      'shared-host historical timings; no controlled speedup or cross-platform scientific guarantee',
                      'headless desktop tests; no fresh native-window interaction qualification']}
(ROOT / 'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
print('PASS accepted ancestry/source hashes, 49/44/55 Rust tests, strict feature lints, 20 harness tests')
print('PASS 3 byte-identical original demo PNG/CSV pairs and 48 retained packets; distinct rebuilt binary identity')
