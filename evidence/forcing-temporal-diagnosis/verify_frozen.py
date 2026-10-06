"""Verify this diagnosis without promoting the original failed qualification."""
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
P = Path(__file__).resolve().parent
BASE = 'f0e2d3d722fa9cdbc7467c996a1b942a2bddca77'
STATUS = 'DIAGNOSIS_SUPPORTED_ORIGINAL_GATE_FAILURE_RETAINED'
CLASSIFICATION = 'PRE_ASYMPTOTIC_DISCRETIZATION_ERROR_CANCELLATION_AND_MAX_NORM_COMPONENT_SWITCH'
OLD_ROOT = 'evidence/forced-extruded-liquid/final-receipt.json'
OLD_ROOT_SHA = '80faef4c4e6a690c3b4f0a1e3d53333aacf1ae49a08b736c4ac20eb3bc290bae'
IDENTITY = ['MrScripty', 'TheEnvironmentGuy@protonmail.com'] * 2


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


def packet_files():
    return {str(path.relative_to(ROOT)) for path in P.rglob('*')
            if path.is_file() and '__pycache__' not in path.parts
            and path.name != 'final-receipt.json'}


def verify(receipt, evidence_commit=None):
    require(receipt['status'] == STATUS and receipt['classification'] == CLASSIFICATION,
            'supported narrow diagnosis')
    require(receipt['original_temporal_gate'] == 'FAIL_ORIGINAL_TEMPORAL_BAND'
            and receipt['original_geometry_band'] == [1.7, 2.3], 'unchanged failed temporal gate')
    source = receipt['qualified_source_commit']
    source_tree = tree(source)
    require(git('rev-parse', source + '^{tree}').decode().strip() == receipt['qualified_source_tree'], 'source tree')
    require(git('show', '-s', '--format=%P', source).decode().split() == [BASE]
            == receipt['ordered_source_parents'], 'ordered source parent')
    require(receipt['historical_base'] == BASE and receipt['legitimate_changed_baseline_paths'] == [], 'additive diagnosis scope')
    require(receipt['historical_git_blobs'] == tree(BASE), 'complete original baseline inventory')
    require(all(source_tree.get(path) == blob for path, blob in receipt['historical_git_blobs'].items()),
            'original source/evidence rewritten')
    require(git('show', '-s', '--format=%an%n%ae%n%cn%n%ce', source).decode().splitlines() == IDENTITY,
            'authorized new source identity')
    for path, digest in receipt['source_sha256'].items():
        require(path in source_tree and sha(git('show', source + ':' + path)) == digest, 'Git source ' + path)
    for path, digest in receipt['executed_source_sha256'].items():
        require(receipt['source_sha256'].get(path) == digest and sha((ROOT / path).read_bytes()) == digest,
                'unchanged executed source ' + path)
    if evidence_commit:
        inventory = tree(evidence_commit)
        prefix = str(P.relative_to(ROOT)) + '/'
        files = {path for path in inventory if path.startswith(prefix) and path != prefix + 'final-receipt.json'}
    else:
        files = packet_files()
    require(files == set(receipt['file_sha256']), 'complete new packet inventory')
    for path, digest in receipt['file_sha256'].items():
        require(sha(git('show', evidence_commit + ':' + path) if evidence_commit else (ROOT / path).read_bytes()) == digest,
                'frozen diagnosis evidence ' + path)
    require(sha((ROOT / OLD_ROOT).read_bytes()) == OLD_ROOT_SHA == receipt['prior_frozen_receipt_sha256'], 'preserved original negative root')
    prior = json.loads((ROOT / OLD_ROOT).read_text())
    for path, digest in prior['file_sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'preserved earlier packet ' + path)
    pairs = [('first-reference-diagnosis.json', 'optimized-reference-diagnosis.json'),
             ('first-finite-diagnosis.json', 'optimized-finite-diagnosis.json'),
             ('first-probe-replay.json', 'optimized-probe-replay.json'),
             ('diagnosis.json', 'optimized-diagnosis.json'),
             ('first-native-probe.jsonl', 'native-no-default-probe.jsonl'),
             ('probe-controls-normal.json', 'probe-controls-optimized.json')]
    require(all((P / a).read_bytes() == (P / b).read_bytes() for a, b in pairs), 'actual mode parity')
    diagnosis = json.loads((P / 'diagnosis.json').read_text())
    require(diagnosis['status'] == STATUS and diagnosis['classification'] == CLASSIFICATION
            and diagnosis['original_five_interval_geometry_qualification'] == receipt['original_temporal_gate']
            and diagnosis['coordinate_order'] == ['cap x0', 'cap H0', 'cap x1', 'cap H1'], 'honest diagnosis and coordinate correction')
    finite = json.loads((P / 'first-finite-diagnosis.json').read_text())
    require(len(finite['rows']) == 20 and all(row['status'] == 'COMPLETE' and row['actual_independent_steps'] == row['expected_steps'] for row in finite['rows'])
            and sum(row['actual_independent_steps'] for row in finite['rows']) == 248, 'actual independent finite trajectories')
    require(max(row['max_native_geometry_gap'] for row in finite['rows']) == diagnosis['native_vs_independent_finite_max_geometry_gap'] <= 1e-14
            and max(row['max_native_eta_gap'] for row in finite['rows']) == diagnosis['native_vs_independent_finite_max_eta_gap'] <= 1e-11,
            'native finite comparison at original scales')
    refs = json.loads((P / 'first-reference-diagnosis.json').read_text())
    require(len(refs['rows']) == 4 and refs['actual_paired_planar_publications'] == 268
            and all(len(row['independent_RK4']) == 3 for row in refs['rows']), 'actual 16 geometry references and field independence')
    for row in refs['rows']:
        require(row['original_geometry_band'] == [1.7, 2.3]
                and row['original_band_pass'] == (row['load'] == 'forward')
                and max(row['existing_geometry_gaps_to_tight'][-1], row['RK4_geometry_gap_to_tight'], row['RK4_finest_refinement_gap']) <= row['geometry_resolution_allowance'],
                'retained reference resolution and original failures')
    probe = json.loads((P / 'first-probe-replay.json').read_text())
    require((probe['actual_publications'], probe['actual_steps'], probe['actual_constructors'], probe['completed_cases'], probe['refused_cases']) == (143, 135, 8, 2, 6), 'all actual fine outcomes')
    require(all(row['physical_equation_replay'] == 'PASS_UNCHANGED_FROZEN_GATES' for row in probe['rows'])
            and all(row['state_preserved'] is True and row['error'] == 'IterationLimit' for row in probe['rows'] if row['probe_status'] == 'REFUSED'),
            'unchanged physical replay and native preserved-state refusals')
    for mode in ['normal', 'optimized']:
        controls = json.loads((P / ('probe-controls-' + mode + '.json')).read_text())
        require(len(controls['actual_rejections']) == 8 and all(row['status'] == 'REJECTED' for row in controls['actual_rejections']), 'actual eight corruption rejections ' + mode)
    execution = json.loads((P / 'execution-receipt.json').read_text())
    require(execution['git_checkpoint'] == BASE and execution['source_was_uncommitted']
            and execution['collection'] == 'Recorded after actual completed tool executions; no wall times reconstructed.'
            and len(execution['commands']) == 17 and all(command['exit'] == 0 for command in execution['commands']), 'actual completed command records')
    for path, digest in execution['source_sha256'].items():
        require(receipt['executed_source_sha256'].get(path) == digest, 'execution source binding ' + path)
    require(not receipt['hosted_CI_qualified'] and not receipt['production_kernel_changed'], 'retained scope')
    return dict(status=STATUS, source=source, tree=receipt['qualified_source_tree'],
                source_paths=len(receipt['source_sha256']), packet_files=len(files),
                preserved_baseline_paths=len(receipt['historical_git_blobs']),
                independent_finite_cases=20, independent_finite_steps=248,
                independent_geometry_references=16, actual_fine_publications=143,
                fine_complete_cases=2, fine_refusals=6, actual_corruption_rejections=8,
                original_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND', hosted_CI_qualified=False)


if __name__ == '__main__':
    receipt = json.loads((P / 'final-receipt.json').read_text())
    commit = sys.argv[sys.argv.index('--evidence-commit') + 1] if '--evidence-commit' in sys.argv else None
    print(json.dumps(verify(receipt, commit), indent=2))
    if '--negative-self-test' in sys.argv:
        for field in ['source_sha256', 'file_sha256', 'historical_git_blobs', 'original_temporal_gate']:
            bad = copy.deepcopy(receipt)
            if field == 'original_temporal_gate':
                bad[field] = 'PASS'
            else:
                key = next(iter(bad[field]))
                bad[field][key] = '0' * len(bad[field][key])
            try:
                verify(bad, commit)
            except ValueError as error:
                print('REJECTED ' + field + ': ' + str(error))
            else:
                raise ValueError('corrupt diagnosis binding accepted ' + field)
