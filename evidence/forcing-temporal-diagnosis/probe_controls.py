"""Execute bounded corrupt-input probes against the unchanged physical gates."""
import copy
import hashlib
import json
import tempfile
from pathlib import Path
import replay_probe as replay


def require(ok, message):
    if not ok:
        raise ValueError(message)


def run():
    source = replay.P / 'first-native-probe.jsonl'
    original = [json.loads(line) for line in source.read_text().splitlines()]
    terminal = next(i for i, row in enumerate(original) if 'probe_status' in row)
    accepted = next(i for i, row in enumerate(original) if row.get('step') == 1)
    results = []
    controls = ['budget', 'identity_type', 'nonfinite_geometry',
                'physical_pressure', 'accepted_force_work',
                'refusal_preservation', 'refusal_error', 'fabricated_completion']
    with tempfile.TemporaryDirectory(prefix='rheon-fine-controls-') as directory:
        path = Path(directory) / 'one-corrupted-capture.jsonl'
        for control in controls:
            rows = copy.deepcopy(original)
            if control == 'budget':
                rows[0]['allocated_bytes'] -= 1
            elif control == 'identity_type':
                rows[0]['stamp']['id'] = float(rows[0]['stamp']['id'])
            elif control == 'nonfinite_geometry':
                rows[0]['positions'][0][0] = float('nan')
            elif control == 'physical_pressure':
                rows[0]['physical_pressure'][0] += 1.
            elif control == 'accepted_force_work':
                rows[accepted]['forcing']['total_work'] += .001
            elif control == 'refusal_preservation':
                rows[terminal]['state_preserved'] = False
            elif control == 'refusal_error':
                rows[terminal]['error'] = 'WorkFailure'
            else:
                rows[terminal]['probe_status'] = 'COMPLETE'
            data = '\n'.join(json.dumps(row) for row in rows) + '\n'
            path.write_text(data)
            try:
                replay.run(path)
            except ValueError as error:
                results.append(dict(control=control, status='REJECTED',
                                    error=str(error),
                                    corrupted_sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
            else:
                raise ValueError('corrupt actual probe accepted: ' + control)
    require(len(results) == 8, 'eight actual diagnostic corruptions')
    return dict(status='PASS_EIGHT_ACTUAL_CORRUPTION_REJECTIONS',
                original_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                actual_rejections=results)


if __name__ == '__main__':
    print(json.dumps(run(), indent=2))
