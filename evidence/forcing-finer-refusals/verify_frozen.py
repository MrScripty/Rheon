"""Verify exact refusal evidence without promoting observational candidates."""
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
P = Path(__file__).resolve().parent
BASE = 'f76841e9217e781d91c5bfeabadb6b4c6f42b20b'
ALLOWED = ['Cargo.toml', 'examples/forcing_temporal_probe.rs', 'src/coupled_discrete.rs']
STATUS = 'FROZEN_EXACT_REFUSAL_DIAGNOSIS_ORIGINAL_FAILURE_RETAINED'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def tree(commit):
    return {line.split(b'\t', 1)[1].decode(): line.split(b'\t', 1)[0].split()[2].decode()
            for line in git('ls-tree', '-r', commit).splitlines()}


def restored(data):
    output = []
    inside = False
    count = 0
    for line in data.splitlines(keepends=True):
        if b'BEGIN RHEON REFUSAL DIAGNOSTIC' in line:
            require(not inside, 'nested diagnostic block')
            inside = True
            count += 1
        elif b'END RHEON REFUSAL DIAGNOSTIC' in line:
            require(inside, 'unmatched diagnostic block')
            inside = False
        elif not inside:
            output.append(line)
    require(not inside and count > 0, 'complete diagnostic block extraction')
    return b''.join(output)


def inventory():
    return {str(path.relative_to(ROOT)) for path in P.rglob('*') if path.is_file()
            and '__pycache__' not in path.parts and path.name != 'final-receipt.json'}


def verify(r, evidence_commit=None):
    require(r['status'] == STATUS and r['original_temporal_gate'] == 'FAIL_ORIGINAL_TEMPORAL_BAND'
            and r['observed_candidates_publish'] is False and not r['arithmetic_floor_proved']
            and not r['general_convergence_rate_proved'] and not r['production_repair_implemented'], 'honest diagnostic scope')
    require(r['original_geometry_band'] == [1.7, 2.3] and r['original_newton_threshold'] == 1e-13
            and r['original_iteration_window'] == 7 and r['original_equation_call_budget'] == 200
            and r['actual_original_refusals'] == r['actual_preserved_state_retries'] == 6, 'exact receipt limits and counts')
    source = r['qualified_source_commit']
    source_tree = tree(source)
    require(git('rev-parse', source + '^{tree}').decode().strip() == r['qualified_source_tree']
            and git('show', '-s', '--format=%P', source).decode().split() == r['ordered_source_parents'], 'exact additive source parent/tree')
    chain = git('rev-list', '--reverse', BASE + '..' + source).decode().splitlines()
    require(chain == r['ordered_source_chain'] and chain[-1] == source, 'complete additive source chain')
    parent = BASE
    for commit in chain:
        require(git('show', '-s', '--format=%P', commit).decode().split() == [parent], 'ordinary source chain without rewriting')
        parent = commit
    require(r['historical_base'] == BASE and r['legitimate_changed_baseline_paths'] == ALLOWED, 'only marked diagnostic source/config additions')
    require(r['historical_git_blobs'] == {p: b for p, b in tree(BASE).items() if p not in ALLOWED}, 'complete preserved baseline inventory')
    require(all(source_tree.get(p) == b for p, b in r['historical_git_blobs'].items()), 'frozen source/evidence rewritten')
    for path in ALLOWED:
        require(restored(git('show', source + ':' + path)) == git('show', BASE + ':' + path), 'original numerical/control/config bytes ' + path)
    require(git('show', '-s', '--format=%an%n%ae%n%cn%n%ce', source).decode().splitlines()
            == ['MrScripty', 'TheEnvironmentGuy@protonmail.com'] * 2, 'authorized new identity')
    for path, digest in r['source_sha256'].items():
        require(sha(git('show', source + ':' + path)) == digest and sha((ROOT / path).read_bytes()) == digest, 'qualified source ' + path)
    if evidence_commit:
        prefix = str(P.relative_to(ROOT)) + '/'
        files = {p for p in tree(evidence_commit) if p.startswith(prefix) and p != prefix + 'final-receipt.json'}
    else:
        files = inventory()
    require(files == set(r['file_sha256']), 'complete exact-refusal packet inventory')
    for path, digest in r['file_sha256'].items():
        require(sha(git('show', evidence_commit + ':' + path) if evidence_commit else (ROOT / path).read_bytes()) == digest, 'frozen refusal evidence ' + path)
    native = json.loads((P / 'native-receipt.json').read_text())
    require(len(native['actual_commands']) == 9 and all(row['exit'] == 0 for row in native['actual_commands'])
            and native['actual_same_owner_refusal_retries'] == 6, 'actual four native builds and six bounded retries')
    ordinary = json.loads((P / 'ordinary-receipt.json').read_text())
    require(len(ordinary['commands']) == 4 and all(row['exit'] == 0 for row in ordinary['commands'])
            and ordinary['no_diagnostic_RUSTFLAGS_required'], 'ordinary build/config qualification')
    for path, binding in ordinary['source_binding'].items():
        require(r['source_sha256'][path] == binding['current_sha256']
                and sha(git('show', BASE + ':' + path)) == binding['original_sha256'], 'executed ordinary source ' + path)
    for path, binding in native['source_binding'].items():
        require(r['source_sha256'][path] == binding['instrumented_sha256'], 'unchanged first traced Rust source ' + path)
    final_config = json.loads((P / 'final-configuration-trace-receipt.json').read_text())
    require(len(final_config['actual_commands']) == 2 and all(row['exit'] == 0 for row in final_config['actual_commands']), 'actual final-configuration trace captures')
    for path, digest in final_config['source_sha256'].items():
        require(r['source_sha256'][path] == digest, 'exact final traced source/config ' + path)
    baseline = (ROOT / 'evidence/forcing-temporal-diagnosis/first-native-probe.jsonl').read_bytes()
    for name in ['native-plain-default', 'native-plain-no-default', 'native-trace-default', 'native-trace-no-default',
                 'ordinary-native-default', 'ordinary-native-no-default']:
        require((P / (name + '.jsonl')).read_bytes() == baseline, 'all actual native publications and refusal markers unchanged ' + name)
    require((P / 'native-trace-default-stderr.log').read_bytes() == (P / 'native-trace-no-default-stderr.log').read_bytes(), 'actual native trace parity')
    for mode in ['default', 'no-default']:
        require((P / ('final-trace-' + mode + '.jsonl')).read_bytes() == baseline
                and (P / ('final-trace-' + mode + '-stderr.log')).read_bytes()
                == (P / ('native-trace-' + mode + '-stderr.log')).read_bytes(), 'final configuration trace/outcome parity ' + mode)
    for stem in ['analysis', 'independent']:
        require((P / (stem + '-normal.json')).read_bytes() == (P / (stem + '-optimized.json')).read_bytes(), 'actual Python mode parity ' + stem)
    analysis = json.loads((P / 'analysis-normal.json').read_text())
    require(analysis['actual_trace_events'] == 1227 and analysis['actual_accepted_steps'] == 135
            and analysis['actual_original_refusals'] == analysis['actual_preserved_state_retries'] == 6
            and len(analysis['actual_corruption_rejections']) == 8
            and all(row['status'] == 'REJECTED' for row in analysis['actual_corruption_rejections']), 'actual trace audit and eight corruption controls')
    require(all(row['original_checks'] == 7 and row['original_equation_calls'] == 49
                and all(value > 1e-13 for value in row['actual_check_rate_norms'])
                and row['same_owner_retry_checks_bit_identical'] and row['same_owner_retry_corrections_bit_identical']
                and row['same_owner_retry_observations_bit_identical'] for row in analysis['rows']), 'exact deterministic original refusal window')
    require(sum(row['unused_candidate_meets_newton_threshold_only'] for row in analysis['rows']) == 2
            and analysis['original_temporal_gate'] == r['original_temporal_gate'], 'two unused candidates never promote refused steps')
    independent = json.loads((P / 'independent-normal.json').read_text())
    require(independent['actual_original_check_evaluations'] == 42 and independent['actual_observational_evaluations'] == 12
            and independent['independently_accepted_endpoints_added'] == 0
            and independent['max_native_independent_rate_component_gap'] <= 1e-11
            and independent['max_native_independent_direct_component_gap'] <= 1e-11, 'independent captured-equation comparison at unchanged original scale')
    require(not r['hosted_CI_qualified'], 'hosted CI remains unqualified')
    return dict(status=STATUS, source=source, tree=r['qualified_source_tree'], source_paths=len(r['source_sha256']),
                packet_files=len(files), preserved_baseline_paths=len(r['historical_git_blobs']),
                exact_original_refusals=6, preserved_state_retries=6, original_checks_per_refusal=7,
                original_calls_per_refusal=49, unused_candidates_meet_threshold_only=2,
                actual_corruption_rejections=8, independent_candidate_evaluations=54,
                original_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND', observed_candidates_publish=False,
                arithmetic_floor_proved=False, general_convergence_rate_proved=False, hosted_CI_qualified=False)


if __name__ == '__main__':
    r = json.loads((P / 'final-receipt.json').read_text())
    commit = sys.argv[sys.argv.index('--evidence-commit') + 1] if '--evidence-commit' in sys.argv else None
    print(json.dumps(verify(r, commit), indent=2))
    if '--negative-self-test' in sys.argv:
        for field in ['source_sha256', 'file_sha256', 'historical_git_blobs', 'observed_candidates_publish', 'original_temporal_gate', 'original_newton_threshold']:
            bad = copy.deepcopy(r)
            if field == 'observed_candidates_publish':
                bad[field] = True
            elif field == 'original_temporal_gate':
                bad[field] = 'PASS'
            elif field == 'original_newton_threshold':
                bad[field] = 2e-13
            else:
                key = next(iter(bad[field]))
                bad[field][key] = '0' * len(bad[field][key])
            try:
                verify(bad, commit)
            except ValueError as error:
                print('REJECTED ' + field + ': ' + str(error))
            else:
                raise ValueError('corrupt diagnostic binding accepted ' + field)
