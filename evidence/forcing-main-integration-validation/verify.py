"""Validate the ordinary integration's real command logs and unchanged scope."""
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
P = ROOT / 'evidence/forcing-main-integration'
M = '8a19cc209436e052422e8f72fbb982a92dddf484'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT).decode().strip()


def verify(r):
    require(r['status'] == 'PASS_LOCAL_MAIN_INTEGRATION_ORIGINAL_FORCING_GATE_FAILURE_RETAINED'
            and r['original_forcing_temporal_gate'] == 'FAIL_ORIGINAL_TEMPORAL_BAND'
            and r['fine_native_refusals'] == 6 and not r['hosted_CI_qualified'], 'retained failed gate and scope')
    require(r['source_commit'] == M and r['source_tree'] == git('rev-parse', M + '^{tree}')
            and r['ordered_source_parents'] == git('show', '-s', '--format=%P', M).split(), 'exact integration source')
    require(r['ordered_source_parents'] == ['8ad29bba1e33d0e3bfb8d79f33dc64be2cf2bbcd', '773bd2725e35590cfe9cbca5625f5239b98f03c2'], 'ordinary ordered parents')
    require(git('show', '-s', '--format=%an%n%ae%n%cn%n%ce', M).splitlines()
            == ['MrScripty', 'TheEnvironmentGuy@protonmail.com'] * 2, 'authorized identity')
    for path, digest in r['rust_source_sha256'].items():
        require(sha(subprocess.check_output(['git', 'show', M + ':' + path], cwd=ROOT)) == digest
                and sha((ROOT / path).read_bytes()) == digest, 'unchanged Rust source ' + path)
        require(git('rev-parse', M + ':' + path) == git('rev-parse', M + '^1:' + path), 'merge changed Rust source ' + path)
    files = {str(path.relative_to(ROOT)) for path in P.glob('*') if path.is_file() and path.name != 'receipt.json'}
    require(files == set(r['file_sha256']), 'complete integration packet')
    for path, digest in r['file_sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'integration evidence ' + path)
    require(len(r['commands']) == 15 and all(row['exit'] == 0 for row in r['commands']), 'actual 15 completed commands')
    commands = json.loads((P / 'commands.json').read_text())
    require(commands['commands'] == r['commands'] and commands['source_commit'] == M
            and commands['rust_source_sha256'] == r['rust_source_sha256'], 'actual command checkpoint')
    counts = {}
    for mode in ['default', 'no-default', 'desktop']:
        pairs = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed;', (P / ('tests-' + mode + '.log')).read_text())
        counts[mode] = sum(int(a) for a, b in pairs)
        require(all(int(b) == 0 for a, b in pairs), 'native test failure ' + mode)
    require(counts == r['Rust_tests_passed'] == dict(default=246, **{'no-default': 240}, desktop=252), 'actual native counts')
    require((P / 'native-probe-default.jsonl').read_bytes() == (P / 'native-probe-no-default.jsonl').read_bytes()
            == (ROOT / 'evidence/forcing-temporal-diagnosis/first-native-probe.jsonl').read_bytes(), 'actual capture parity')
    require((P / 'column-ledger-normal.json').read_bytes() == (P / 'column-ledger-optimized.json').read_bytes()
            and (P / 'frozen-normal.log').read_bytes() == (P / 'frozen-optimized.log').read_bytes(), 'actual Python mode parity')
    ledger = json.loads((P / 'column-ledger-normal.json').read_text())
    require(ledger['source_commit'] == M and ledger['tracked_example_count'] == 24
            and len(ledger['controls']) == ledger['actual_control_count'] == 54
            and all(row['matched_patterns'] for row in ledger['controls']), 'actual live path coverage')
    require(sha((ROOT / 'evidence/forcing-temporal-diagnosis/final-receipt.json').read_bytes())
            == r['frozen_diagnosis_receipt_sha256'], 'preserved diagnosis root')
    return dict(status='PASS_LOCAL_INTEGRATION_BINDING', source=M, tree=r['source_tree'],
                Rust_tests_passed=counts, actual_commands=15, current_path_controls=54,
                original_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND', fine_native_refusals=6,
                hosted_CI_qualified=False)


if __name__ == '__main__':
    r = json.loads((P / 'receipt.json').read_text())
    print(json.dumps(verify(r), indent=2))
    if '--negative-self-test' in sys.argv:
        for field in ['rust_source_sha256', 'file_sha256', 'Rust_tests_passed', 'original_forcing_temporal_gate']:
            bad = copy.deepcopy(r)
            if field == 'original_forcing_temporal_gate':
                bad[field] = 'PASS'
            elif field == 'Rust_tests_passed':
                bad[field]['default'] += 1
            else:
                bad[field][next(iter(bad[field]))] = '0' * 64
            try:
                verify(bad)
            except ValueError as error:
                print('REJECTED ' + field + ': ' + str(error))
            else:
                raise ValueError('corrupt integration binding accepted ' + field)
