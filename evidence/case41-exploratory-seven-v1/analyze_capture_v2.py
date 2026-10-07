"""Read-only checks on one frozen seven-equation journal; never call an ELF."""
from pathlib import Path
from fractions import Fraction as Q
import hashlib
import importlib.util
import json
import math
import struct
import sys

P = Path(__file__).resolve().parent
ROOT = P.parents[1]


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def bits(value):
    return struct.pack('>d', value).hex()


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def finite(value):
    if isinstance(value, dict):
        return all(finite(x) for x in value.values())
    if isinstance(value, list):
        return all(finite(x) for x in value)
    return not isinstance(value, float) or math.isfinite(value)


def same_bits(left, right):
    if isinstance(left, list):
        return isinstance(right, list) and len(left) == len(right) and all(same_bits(a, b) for a, b in zip(left, right))
    if isinstance(left, float) or isinstance(right, float):
        return bits(left) == bits(right)
    return left == right


def rank_exact(matrix):
    work = [list(row) for row in matrix]
    rows, columns = len(work), len(work[0])
    pivot_row = 0
    for column in range(columns):
        pivot = next((i for i in range(pivot_row, rows) if work[i][column]), None)
        if pivot is None:
            continue
        work[pivot_row], work[pivot] = work[pivot], work[pivot_row]
        scale = work[pivot_row][column]
        work[pivot_row] = [x / scale for x in work[pivot_row]]
        for i in range(pivot_row + 1, rows):
            scale = work[i][column]
            work[i] = [x - scale * y for x, y in zip(work[i], work[pivot_row])]
        pivot_row += 1
        if pivot_row == rows:
            break
    return pivot_row


def run():
    policy = json.loads((P / 'protocol.json').read_text())
    fixture = json.loads((P / 'fixture.json').read_text())
    before = json.loads((P / 'capture-before.json').read_text())
    completed = json.loads((P / 'capture-completed.json').read_text())
    for path, digest in policy['sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'original immutable input ' + path)
    require(before['protocol_sha256'] == completed['protocol_sha256'] == sha((P / 'protocol.json').read_bytes()), 'execution protocol identity')
    require(completed['status'] == 'PROCESS_COMPLETED_READER_PENDING' and completed['exit'] == 0
            and not completed['faults'] and completed['native_process_launches'] == 1
            and completed['retries'] == 0, 'one clean native invocation')
    require(completed['binary_sha256'] == policy['binary_sha256']
            and not completed['historical_memory_qualified'], 'no substituted ELF or memory pass')
    require(before['argv'] == [policy['binary'], '--exact', 'research_public_call::case41_paired_candidate_roster',
                               '--nocapture', '--test-threads=1'], 'exact candidate-only argv')
    require(set(before['environment']) == {'LANG', 'LC_ALL', 'TZ', 'RUST_BACKTRACE',
                                         'RUST_MIN_STACK', 'RHEON_CASE41_PAIRED_ALLOW'}, 'closed numerical environment')
    limits = policy['limits']
    require(completed['seconds'] <= limits['wall_seconds']
            and completed['cpu_user_seconds'] + completed['cpu_system_seconds'] <= limits['cpu_seconds'], 'time budgets')
    require(0 < completed['measured_peak_rss_bytes'] <= limits['address_space_bytes'], 'measured native peak')
    raw = (P / 'native.log').read_bytes()
    stderr = (P / 'native-stderr.log').read_bytes()
    require(sha(raw) == completed['stdout_sha256'] and sha(stderr) == completed['stderr_sha256'], 'raw journal hashes')
    require(len(raw) == completed['stdout_bytes'] <= limits['stdout_bytes']
            and len(stderr) == completed['stderr_bytes'] <= limits['stderr_bytes'] and not stderr, 'no truncation or hidden native stderr')
    require(b'1 passed; 0 failed' in raw, 'actual native assertion/bit-guard success')
    prefix = b'test research_public_call::case41_paired_candidate_roster ... '
    lines = raw.splitlines()
    require(sum(line.startswith(prefix) for line in lines) == 1, 'one exact frozen libtest prefix')
    numerical_lines = [line[len(prefix):] if line.startswith(prefix) else line for line in lines]
    rows = [json.loads(line) for line in numerical_lines if line.startswith(b'{')]
    require(rows[0]['event'] == 'fd_equation' and rows[0]['column'] == -1,
            'prefix correction retains the actual baseline record')
    require(all(finite(row) for row in rows), 'every observation finite')
    require(not any(row['event'] in ['fd_equation_refusal', 'fd_column_unavailable'] for row in rows), 'no numerical refusal')
    expected_events = [('fd_equation', -1)]
    for j in range(6):
        expected_events += [('fd_equation', j), ('fd_column', j)]
    expected_events += [('fd_complete', None)]
    require([(row['event'], row.get('column')) for row in rows] == expected_events, 'exact ordered seven-equation/column roster')
    require(all(row['case'] == 41 and row['owners'] == row['corrections'] == 0 for row in rows), 'zero owner/correction scope')
    terminal = rows[-1]
    require(terminal['equations_attempted'] == 7 and not terminal['baseline_failed'] and not terminal['published'], 'honest native completion')
    equations = [row for row in rows if row['event'] == 'fd_equation']
    columns = [row for row in rows if row['event'] == 'fd_column']
    baseline = equations[0]
    expected_rate_bits = [struct.pack('>Q', value).hex() for value in fixture['expected_baseline']['rate_bits']]
    require([bits(x) for x in baseline['rate']] == expected_rate_bits, 'all 22 exact frozen E2 baseline bits')
    archive = ROOT / 'evidence/forcing-e2-fixed-candidates-v1/candidate-fixed.log'
    original = next(row for row in [json.loads(line) for line in archive.read_text().splitlines() if line.startswith('{')]
                    if row['event'] == 'fixed_equation' and row['index'] == 41 and row['order'] == 16)
    matching = ['r', 'rate', 'direct_rate', 'start_z', 'start_mass', 'end_z', 'end_velocity', 'end_mass',
                'end_force', 'end_q', 'end_eta', 'end_d', 'end_b', 'pairs', 'plus', 'minus',
                'maximum_constraints', 'sign_roots', 'physical_convection', 'endpoint_convection']
    require(all(same_bits(baseline[key], original[key]) for key in matching), 'all retained baseline stored fields reproduce archive')
    c = dict(fixture['original_case'], acceleration=[0.0625, -0.125, 0.03125])
    oracle = module('exploratory_stored_equation', ROOT / 'evidence/forcing-e1-fixed-candidates-v1/analyze.py')
    outcomes = []
    exact_equations = []
    for position, equation in enumerate(equations):
        column = position - 1
        unknown = c['final_authorized_unknowns'][:]
        if column >= 0:
            roster = fixture['perturbation_roster'][column]
            unknown[column] = struct.unpack('>d', bytes.fromhex(roster['perturbed_bits']))[0]
            require(bits(equation['delta']) == roster['delta_bits'], 'original nominal perturbation denominator')
        else:
            require(bits(equation['delta']) == bits(0.0), 'unchanged baseline delta')
        require(same_bits(equation['unknown'], unknown), 'frozen 22-coordinate roster bits')
        require(equation['order'] == 16 and not equation['published'], 'original order and unpublished state')
        require(same_bits(equation['r'], baseline['r'])
                and same_bits(equation['start_z'], baseline['start_z'])
                and same_bits(equation['start_mass'], c['accepted_publication']['mass']), 'fixed embedding/start state')
        require(len(equation['rate']) == len(equation['direct_rate']) == len(equation['end_z']) == 22
                and len(equation['end_mass']) == len(equation['end_velocity']) == 16
                and len(equation['end_d']) == 24 and all(len(row) == 22 for row in equation['end_d'])
                and len(equation['end_b']) == 16 and all(len(row) == 22 for row in equation['end_b']), 'complete stored layouts')
        require(same_bits(equation['pairs'], baseline['pairs'])
                and len(equation['pairs']) == len(equation['plus']) == len(equation['minus']) <= 72
                and 0 <= equation['sign_roots'] <= 64, 'fixed topology and bounded sign partition')
        require(all(x > 0 for x in equation['end_mass'])
                and all(x >= 0 for x in equation['plus'] + equation['minus']), 'positive mass and donor integrals')
        require(equation['maximum_constraints'] <= 1e-11, 'original constraint gate unchanged')
        require(same_bits(equation['start_q'], c['q']) and same_bits(equation['start_eta'], c['eta']), 'original starting coordinates')
        for i in range(16):
            require(same_bits(equation['start_velocity'][i][:2], c['accepted_publication']['velocity'][i][:2])
                    and equation['start_velocity'][i][2] == equation['end_velocity'][i][2] == 0.0, 'original planar stored velocity authority')
        expected_eta = [float(Q(c['eta'][i]) + Q(float(Q(c['h']) * Q(unknown[i])))) for i in range(6)]
        expected_q = []
        for i in range(3):
            first = float(Q(c['q'][i]) + Q(float(Q(c['h']) * Q(c['eta'][i]))))
            half_h = float(Q(0.5) * Q(c['h']))
            half_h_h = float(Q(half_h) * Q(c['h']))
            quadratic = float(Q(half_h_h) * Q(unknown[i]))
            expected_q.append(float(Q(first) + Q(quadratic)))
        require(same_bits(equation['end_eta'], expected_eta) and same_bits(equation['end_q'], expected_q), 'original checked endpoint arithmetic')
        require(max(abs(a - b) for a, b in zip(expected_q, c['q'])) <= 0.125, 'original geometry path limit')
        for k, index in enumerate(oracle.KNOWN):
            expected = -expected_eta[oracle.COORD[k]] if k == 6 else expected_eta[oracle.COORD[k]]
            require(bits(equation['end_z'][index]) == bits(expected), 'original native known endpoint bits')
        native_rate, _ = oracle.native(c, equation)
        direct_rate, _ = oracle.native(c, equation, True)
        require(same_bits(native_rate, equation['rate']) and same_bits(direct_rate, equation['direct_rate']), 'bit-exact whole native stored-field rate replay')
        exact_rate, _ = oracle.exact(c, equation)
        exact_direct, _ = oracle.exact(c, equation, direct=True)
        old_embedded = oracle.embedding(equation, list(map(Q, equation['start_z'])))
        new_embedded = oracle.embedding(equation, list(map(Q, equation['end_z'])))
        defect = [Q(0)] * 22
        for i in range(16):
            for d in range(2):
                value = Q(equation['end_mass'][i]) * ((new_embedded[i][d] - Q(equation['end_velocity'][i][d]))
                    - (old_embedded[i][d] - Q(c['accepted_publication']['velocity'][i][d]))) / Q(c['h'])
                for j in range(22):
                    defect[j] += Q(equation['r'][i][d][j]) * value
        require([a - b for a, b in zip(exact_rate, exact_direct)] == defect, 'exact stable/direct embedding identity')
        exact_equations.append(exact_rate)
        rate_norm = math.sqrt(oracle.sequential(x * x for x in native_rate))
        direct_norm = math.sqrt(oracle.sequential(x * x for x in direct_rate))
        outcomes.append(dict(column=column, native_rate_norm=rate_norm, native_direct_norm=direct_norm,
                             original_Newton_eligible=rate_norm <= 1e-13,
                             original_physical_rate_eligible=rate_norm <= 1e-11 and direct_norm <= 1e-11,
                             maximum_constraints=equation['maximum_constraints'], sign_roots=equation['sign_roots'],
                             stored_native_replay=True, stable_direct_identity=True))
        if column >= 0:
            emitted = columns[column]
            require(bits(emitted['delta']) == bits(equation['delta']), 'native column denominator identity')
            independent = []
            for a, b in zip(equation['rate'], baseline['rate']):
                subtraction = float(Q(a) - Q(b))
                independent.append(float(Q(subtraction) / Q(equation['delta'])))
            require(same_bits(independent, emitted['native_column']), 'exactly rounded checked native FD subtraction/division')
    # Original production Newton model uses these six FD columns and the
    # baseline endpoint B^T as its sixteen pressure columns. No solve follows.
    A = [[Q(columns[j]['native_column'][i]) for j in range(6)] for i in range(22)]
    B = [[Q(baseline['end_b'][j][i]) for j in range(16)] for i in range(22)]
    J = [a + b for a, b in zip(A, B)]
    require(rank_exact(B) == 16, 'complete frozen pressure-column provenance/rank')
    exact_differences = [[str((exact_equations[j + 1][i] - exact_equations[0][i]) / Q(equations[j + 1]['delta']))
                          for j in range(6)] for i in range(22)]
    return dict(status='PASS_COMPLETE_EXPLORATORY_SEVEN_EQUATION_CAPTURE',
                execution_source=completed['source'], execution_tree=completed['source_tree'],
                compiled_source=policy['compiled_source'], binary_sha256=policy['binary_sha256'],
                raw_stdout_sha256=sha(raw), raw_stderr_sha256=sha(stderr),
                baseline_all_22_bits_exact=True, baseline_all_retained_fields_exact=True,
                numerical_equations=7, numerical_invocations=1, retries=0, corrections=0, owners=0,
                historical_memory_qualified=False, historical_known_subtotal=66368,
                resource_prerequisite_changed=True, numerical_method_changed=False,
                original_Newton_gate=1e-13, original_physical_rate_gate=1e-11,
                observed_equations=outcomes, measured_peak_rss_bytes=completed['measured_peak_rss_bytes'],
                observed_wall_seconds=completed['seconds'],
                FD_rank_exact=rank_exact(A), pressure_rank_exact=16,
                complete_linear_model_rank_exact=rank_exact(J),
                native_linear_model=[[float(x) for x in row] for row in J],
                exact_stored_field_FD_differences=exact_differences,
                pressure_provenance='baseline.end_b transposed; identical original E2 stored operator and unchanged Newton assembly source',
                complete_step_physical_qualification=False, physical_gate_changed=False,
                missing_qualifiers=['no order32 comparison', 'no public step/energy-ledger qualifier',
                                    'no third-component solve', 'no FD truncation/smooth-neighborhood bound'],
                scope='Stored-field diagnostic only; exact ranks describe captured rounded linear model, not exact smooth derivative, arithmetic floor or accepted step.')


if __name__ == '__main__':
    try:
        result = run()
    except Exception as error:
        result = dict(status='INCOMPLETE_TERMINAL_NO_RETRY', error=type(error).__name__ + ': ' + str(error),
                      historical_memory_qualified=False, retry_allowed=False)
        print(json.dumps(result, indent=2, sort_keys=True))
        sys.exit(1)
    print(json.dumps(result, indent=2, sort_keys=True))
