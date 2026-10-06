"""Audit actual Newton checks, bounded refusals and observational candidates."""
import copy
import json
import math
import struct
import sys
from pathlib import Path

P = Path(__file__).resolve().parent
THRESHOLD = 1e-13


def require(ok, message):
    if not ok:
        raise ValueError(message)


def bits(values):
    if isinstance(values, list):
        return [bits(value) for value in values]
    return struct.pack('>d', float(values)).hex()


def finite(value):
    if isinstance(value, list):
        return all(finite(x) for x in value)
    return type(value) in [int, float] and math.isfinite(value)


def vector(row, name, shape):
    value = row[name]
    require(isinstance(value, list) and len(value) == shape[0], 'fixed shape ' + name)
    if len(shape) > 1:
        require(all(isinstance(x, list) and len(x) == shape[1] for x in value), 'fixed nested shape ' + name)
    require(finite(value), 'finite ' + name)


def actual_norm(values):
    squared = 0.
    for value in values:
        squared = squared + value * value
    return math.sqrt(squared)


def audit(events):
    require(0 < len(events) <= 20000, 'bounded eight-case diagnostic stream')
    public = [json.loads(line) for line in (P / 'native-trace-default.jsonl').read_text().splitlines()]
    native = {}
    terminal = {}
    for row in public:
        key = (row['kind'], row['load'], row['h'])
        if 'model' in row:
            native.setdefault(key, []).append(row)
        else:
            terminal[key] = row
    require(len(native) == len(terminal) == 8 and sum(map(len, native.values())) == 143, 'actual eight native cases')
    groups = {}
    current = None
    key = None
    retry = False
    for row in events:
        event = row['event']
        if event == 'case':
            key = (row['kind'], row['load'], row['h'])
            require(key in native and key not in groups, 'unique actual case')
            groups[key] = []
            current = None
            retry = False
        elif event == 'newton_check':
            require(key is not None and row['newton_threshold'] == THRESHOLD and row['iteration_limit'] == 7, 'unchanged native Newton limits')
            require(type(row['iteration']) is int and type(row['calls']) is int and type(row['accepted_version']) is int, 'native integer counters')
            if row['iteration'] == 1:
                if current is not None:
                    require(current['checks'][-1]['rate_norm'] <= THRESHOLD and not current['refusals'], 'previous primary step genuinely passed its Newton check')
                current = dict(retry=retry, checks=[], corrections=[], refusals=[], observations=[])
                groups[key].append(current)
            require(current is not None and row['iteration'] == len(current['checks']) + 1
                    and row['calls'] == 1 + 7 * (row['iteration'] - 1), 'actual original check/call sequence')
            for name, shape in [('q', (3,)), ('eta', (6,)), ('unknown', (22,)), ('rate', (22,)),
                                ('direct_rate', (22,)), ('candidate_q', (3,)), ('candidate_planar_velocity', (16, 3)), ('candidate_mass', (16,))]:
                vector(row, name, shape)
            require(finite(row['rate_norm']) and bits(row['rate_norm']) == bits(actual_norm(row['rate'])), 'exact native residual norm from actual components')
            before = native[key][row['accepted_version']]
            require(bits(row['q']) == bits(before['end_q']) and bits(row['eta']) == bits(before['end_eta'])
                    and bits(row['accepted_time']) == bits(before['time']), 'actual unchanged accepted input to the candidate')
            if current['checks']:
                require(current['checks'][-1]['rate_norm'] > THRESHOLD, 'solver must stop on an original passing check')
            current['checks'].append(row)
        elif event == 'correction':
            require(current is not None and row['iteration'] == len(current['checks'])
                    and row['calls'] == 7 * row['iteration'] and len(current['corrections']) == row['iteration'] - 1,
                    'six derivative calls and one actual correction per failed check')
            check = current['checks'][-1]
            require(check['rate_norm'] > THRESHOLD, 'no correction after a passing check')
            vector(row, 'correction', (22,))
            vector(row, 'unknown_after', (22,))
            require(bits(row['unknown_after']) == bits([a + b for a, b in zip(check['unknown'], row['correction'])]), 'actual correction arithmetic')
            current['corrections'].append(row)
        elif event == 'refusal':
            require(current is not None and row['reason'] == 'seven_check_newton_window_exhausted'
                    and row['checks'] == 7 and row['calls'] == 49 and row['newton_threshold'] == THRESHOLD,
                    'exact refusal site; equation-call budget not exhausted')
            require(len(current['checks']) == len(current['corrections']) == 7
                    and all(check['rate_norm'] > THRESHOLD for check in current['checks']), 'all seven original checks actually fail')
            require(row['accepted_version'] == terminal[key]['accepted_steps']
                    and terminal[key]['probe_status'] == 'REFUSED' and terminal[key]['state_preserved'] is True
                    and terminal[key]['error'] == 'IterationLimit', 'original accepted state/refusal result')
            current['refusals'].append(row)
        elif event == 'post_window_observation':
            require(current is not None and len(current['refusals']) == 1 and row['did_not_participate_in_acceptance'] is True
                    and row['order'] == [16, 32][len(current['observations'])], 'observations cannot replace original checks or publish')
            require('error' not in row, 'actual observation succeeded at both quadrature orders')
            for name, shape in [('unknown', (22,)), ('rate', (22,)), ('direct_rate', (22,)),
                                ('candidate_q', (3,)), ('candidate_planar_velocity', (16, 3)), ('candidate_mass', (16,))]:
                vector(row, name, shape)
            require(bits(row['unknown']) == bits(current['corrections'][-1]['unknown_after'])
                    and bits(row['rate_norm']) == bits(actual_norm(row['rate'])), 'observe exactly the unused seventh correction')
            current['observations'].append(row)
        elif event == 'retry_begin':
            require(current is not None and not retry and len(current['refusals']) == 1
                    and len(current['observations']) == 2, 'bounded same-owner retry after actual refusal')
            retry = True
            current = None
        elif event == 'retry_end':
            require(current is not None and retry and len(current['refusals']) == 1 and len(current['observations']) == 2
                    and row['error'] == 'IterationLimit' and row['state_preserved'] is True, 'same-owner retry must preserve accepted state')
            current['retry_end'] = row
        else:
            raise ValueError('undeclared diagnostic event ' + str(event))
    require(set(groups) == set(native), 'all eight cases traced')
    results = []
    accepted_attempts = 0
    primary_refusals = 0
    for key, attempts in groups.items():
        original = [x for x in attempts if not x['retry']]
        retries = [x for x in attempts if x['retry']]
        marker = terminal[key]
        refused = marker['probe_status'] == 'REFUSED'
        require(len(original) == marker['accepted_steps'] + int(refused) and len(retries) == int(refused), 'all actual primary steps and bounded retry')
        for index, attempt in enumerate(original):
            require(attempt['checks'][0]['accepted_version'] == index, 'consecutive actual accepted versions')
            if index < marker['accepted_steps']:
                require(not attempt['refusals'] and attempt['checks'][-1]['rate_norm'] <= THRESHOLD
                        and len(attempt['corrections']) == len(attempt['checks']) - 1, 'actual accepted step passes original Newton check')
                accepted_attempts += 1
        if not refused:
            continue
        last = original[-1]
        again = retries[0]
        require('retry_end' in again and len(last['refusals']) == 1 and len(last['observations']) == 2, 'all original and repeat failure evidence')
        norms = [x['rate_norm'] for x in last['checks']]
        observation16, observation32 = last['observations']
        gap = max(abs(x - y) for x, y in zip(observation16['rate'], observation32['rate']))
        transitions = []
        for a, b in zip(last['checks'][2:], last['checks'][3:]):
            transitions.append(dict(from_iteration=a['iteration'], to_iteration=b['iteration'],
                candidate_q_bits_unchanged=bits(a['candidate_q']) == bits(b['candidate_q']),
                candidate_mass_bits_unchanged=bits(a['candidate_mass']) == bits(b['candidate_mass']),
                candidate_velocity_bits_unchanged=bits(a['candidate_planar_velocity']) == bits(b['candidate_planar_velocity'])))
        results.append(dict(kind=key[0], load=key[1], h=key[2], attempted_step=marker['attempted_step'],
            accepted_steps_preserved=marker['accepted_steps'], original_error='IterationLimit',
            exact_reason='seven_check_newton_window_exhausted', original_checks=7, original_equation_calls=49,
            equation_call_budget=200, original_newton_threshold=THRESHOLD,
            actual_check_rate_norms=norms, minimum_checked_rate=min(norms), last_checked_rate=norms[-1],
            actual_unused_seventh_correction_rate_16=observation16['rate_norm'],
            actual_unused_seventh_correction_rate_32=observation32['rate_norm'],
            unused_candidate_meets_newton_threshold_only=observation16['rate_norm'] <= THRESHOLD,
            post_window_rate_vector_16_32_max_gap=gap,
            same_owner_retry_checks_bit_identical=last['checks'] == again['checks'],
            same_owner_retry_corrections_bit_identical=last['corrections'] == again['corrections'],
            same_owner_retry_observations_bit_identical=last['observations'] == again['observations'],
            accepted_state_preserved_on_original_and_retry=True, late_candidate_bit_transitions=transitions,
            scope='Unused candidate is observational: no full qualification, third-component step, publication or final-time endpoint.'))
        primary_refusals += 1
    require(accepted_attempts == 135 and primary_refusals == len(results) == 6, '135 original accepted steps and six exact refusals')
    require(sum(row['unused_candidate_meets_newton_threshold_only'] for row in results) == 2, 'actual two unused candidates, never substituted for refused steps')
    return dict(status='PASS_EXACT_REFUSAL_DIAGNOSIS_ORIGINAL_GATE_FAILURE_RETAINED',
        actual_trace_events=len(events), actual_accepted_steps=accepted_attempts,
        actual_original_refusals=6, actual_preserved_state_retries=6,
        original_newton_threshold=THRESHOLD, original_iteration_window=7, equation_call_budget=200,
        unused_candidates_below_threshold_only=2,
        original_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND',
        classification='ALL_SIX_EXHAUST_ORIGINAL_CHECK_WINDOW; TWO_UNUSED_FINAL_CORRECTIONS_MEET_NEWTON_THRESHOLD_ONLY',
        arithmetic_floor_proved=False, general_convergence_rate_proved=False, production_repair_implemented=False,
        rows=results)


def controls(events):
    first_check = next(i for i, row in enumerate(events) if row['event'] == 'newton_check')
    refusal = next(i for i, row in enumerate(events) if row['event'] == 'refusal')
    observation = next(i for i, row in enumerate(events) if row['event'] == 'post_window_observation')
    retry_end = next(i for i, row in enumerate(events) if row['event'] == 'retry_end')
    results = []
    for name in ['forged_norm', 'relaxed_threshold', 'wrong_call_budget', 'missing_seventh_check',
                 'false_retry_preservation', 'observation_promoted', 'forged_accepted_q', 'nonfinite_rate']:
        bad = copy.deepcopy(events)
        if name == 'forged_norm':
            bad[first_check]['rate_norm'] = 0.
        elif name == 'relaxed_threshold':
            bad[first_check]['newton_threshold'] = 2e-13
        elif name == 'wrong_call_budget':
            bad[refusal]['calls'] = 201
        elif name == 'missing_seventh_check':
            index = max(i for i in range(refusal) if bad[i]['event'] == 'newton_check')
            del bad[index]
        elif name == 'false_retry_preservation':
            bad[retry_end]['state_preserved'] = False
        elif name == 'observation_promoted':
            bad[observation]['did_not_participate_in_acceptance'] = False
        elif name == 'forged_accepted_q':
            bad[first_check]['q'][0] += .01
        else:
            bad[first_check]['rate'][0] = float('nan')
        try:
            audit(bad)
        except ValueError as error:
            results.append(dict(control=name, status='REJECTED', error=str(error)))
        else:
            raise ValueError('corrupt diagnostic trace accepted ' + name)
    return results


if __name__ == '__main__':
    events = [json.loads(line) for line in (P / 'native-trace-default-stderr.log').read_text().splitlines()]
    result = audit(events)
    if '--negative-self-test' in sys.argv:
        result['actual_corruption_rejections'] = controls(events)
    print(json.dumps(result, indent=2))
