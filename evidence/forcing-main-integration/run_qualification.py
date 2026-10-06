"""Qualify the ordinary main merge while retaining frozen forcing negatives."""
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
P = Path(__file__).resolve().parent
MERGE = '8a19cc209436e052422e8f72fbb982a92dddf484'
MAIN = '773bd2725e35590cfe9cbca5625f5239b98f03c2'
PREMERGE = '8ad29bba1e33d0e3bfb8d79f33dc64be2cf2bbcd'
EVIDENCE = '0c2c53737f0785cffbbfbf56bc8c6b0dc94f73f9'
SAFE_TARGET = '/workspace/.rheon-tools/forced-extruded-capture-target'
env = dict(os.environ, OPENBLAS_NUM_THREADS='1', CARGO_TARGET_DIR=SAFE_TARGET)
records = []


def require(ok, message):
    if not ok:
        raise ValueError(message)


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT).decode().strip()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rust_manifest():
    names = git('ls-files', '--', '*.rs', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml').splitlines()
    return {name: sha(ROOT / name) for name in names}


def run(name, argv, suffix='log'):
    require(git('rev-parse', 'HEAD') == MERGE and rust_manifest() == rust_sources, 'unchanged actual integration source')
    out = P / (name + '.' + suffix)
    err = P / (name + '-stderr.log')
    require(not out.exists() and not err.exists(), 'refuse overwritten execution ' + name)
    start = time.monotonic()
    with out.open('wb') as stdout, err.open('wb') as stderr:
        result = subprocess.run(argv, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
    records.append(dict(name=name, argv=argv, exit=result.returncode,
                        elapsed_seconds=time.monotonic() - start,
                        stdout=str(out.relative_to(ROOT)), stderr=str(err.relative_to(ROOT))))
    (P / 'commands.json').write_text(json.dumps(dict(source_commit=MERGE,
        source_tree=git('rev-parse', MERGE + '^{tree}'), rust_source_sha256=rust_sources,
        environment=dict(CARGO_TARGET_DIR=SAFE_TARGET, OPENBLAS_NUM_THREADS='1', CARGO_BUILD_JOBS=env.get('CARGO_BUILD_JOBS')),
        commands=records, status='PASS_COMPLETED_COMMANDS' if all(row['exit'] == 0 for row in records) else 'FAIL'), indent=2) + '\n')
    require(result.returncode == 0, 'actual integration failure ' + name)
    print(name + ' PASS', flush=True)


require(git('rev-parse', 'HEAD') == MERGE, 'exact integration checkpoint')
require(git('show', '-s', '--format=%P', MERGE).split() == [PREMERGE, MAIN], 'ordinary ordered merge parents')
require(git('rev-parse', MAIN + '^{tree}') == '24222bab14a8ca2118578544b9982ca50e5beacf', 'reviewed main tree')
rust_sources = rust_manifest()
for name in rust_sources:
    require(git('rev-parse', MERGE + ':' + name) == git('rev-parse', PREMERGE + ':' + name), 'main merge changed Rust source ' + name)
run('format', ['cargo', 'fmt', '--all', '--check'])
for mode, flags in [('default', []), ('no-default', ['--no-default-features']), ('desktop', ['--features', 'desktop'])]:
    run('tests-' + mode, ['cargo', 'test', '--locked', '--all-targets', *flags])
    run('clippy-' + mode, ['cargo', 'clippy', '--locked', '--all-targets', *flags, '--', '-D', 'warnings'])
counts = {}
for mode, expected in [('default', 246), ('no-default', 240), ('desktop', 252)]:
    pairs = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed;', (P / ('tests-' + mode + '.log')).read_text())
    counts[mode] = sum(int(passed) for passed, failed in pairs)
    require(counts[mode] == expected and all(int(failed) == 0 for passed, failed in pairs), 'actual Rust test count ' + mode)
for mode, flags in [('normal', []), ('optimized', ['-O'])]:
    run('frozen-' + mode, [sys.executable, *flags, 'evidence/forcing-temporal-diagnosis/verify_frozen.py', '--evidence-commit', EVIDENCE, '--negative-self-test'])
    run('column-tests-' + mode, [sys.executable, *flags, '-m', 'unittest', 'discover', '-s', 'tools', '-p', 'test_column_interface_verifier.py'])
    run('column-ledger-' + mode, [sys.executable, *flags, 'evidence/column-interface-path-ledger/run_ledger.py'], 'json')
for mode, flags in [('default', []), ('no-default', ['--no-default-features'])]:
    run('native-probe-' + mode, ['cargo', 'run', '--locked', '--release', '--quiet', *flags, '--example', 'forcing_temporal_probe'], 'jsonl')
require((P / 'native-probe-default.jsonl').read_bytes() == (P / 'native-probe-no-default.jsonl').read_bytes()
        == (ROOT / 'evidence/forcing-temporal-diagnosis/first-native-probe.jsonl').read_bytes(), 'unchanged actual 143-publication capture')
for stem, extension in [('frozen', 'log'), ('column-ledger', 'json')]:
    require((P / (stem + '-normal.' + extension)).read_bytes() == (P / (stem + '-optimized.' + extension)).read_bytes(), 'actual integration mode parity ' + stem)
ledger = json.loads((P / 'column-ledger-normal.json').read_text())
require(ledger['tracked_example_count'] == 24 and ledger['actual_control_count'] == 54
        and all(row['matched_patterns'] for row in ledger['controls']), 'new tracked examples covered by unchanged reviewed patterns')
require(len(records) == 15 and rust_manifest() == rust_sources, 'complete stable integration checks')
receipt = dict(status='PASS_LOCAL_MAIN_INTEGRATION_ORIGINAL_FORCING_GATE_FAILURE_RETAINED',
    source_commit=MERGE, source_tree=git('rev-parse', MERGE + '^{tree}'), ordered_source_parents=[PREMERGE, MAIN],
    reviewed_main=MAIN, reviewed_main_tree=git('rev-parse', MAIN + '^{tree}'),
    frozen_diagnosis_source='f609c0244b41bcf765705032777aff1e588cd3ae', frozen_diagnosis_evidence=EVIDENCE,
    frozen_diagnosis_receipt_sha256=sha(ROOT / 'evidence/forcing-temporal-diagnosis/final-receipt.json'),
    commands=records, Rust_tests_passed=counts, rust_source_sha256=rust_sources,
    production_Rust_unchanged_by_merge=True, normal_optimized_binding_and_path_ledger_byte_identical=True,
    native_probe_byte_identical_to_frozen=True, actual_probe_publications=143,
    actual_tracked_examples=24, actual_current_path_controls=54,
    historical_column_50_controls_preserved=True,
    original_forcing_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND', fine_native_refusals=6,
    hosted_CI_qualified=False,
    file_sha256={str(path.relative_to(ROOT)): sha(path) for path in sorted(P.glob('*')) if path.is_file() and path.name != 'receipt.json'})
(P / 'receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
print(receipt['status'], counts, flush=True)
