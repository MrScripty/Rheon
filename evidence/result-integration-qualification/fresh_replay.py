#!/usr/bin/env python3
"""Produce or check a separate fresh replay with local executable provenance.

No retrospective producing-binary identity is inferred for old replay files.
The receipt is local provenance, not signed attestation or a binary/source proof.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

DEMOS = ('demo-16', 'demo-plume', 'demo-64')
ARTIFACTS = ('run.json', 'steps.csv', 'opacity.png')


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_outputs(fixture, actual):
    original = json.loads((fixture / 'run.json').read_text())
    replay = json.loads((actual / 'run.json').read_text())
    fields = [k for k in original if k != 'measured_step_seconds']
    for field in fields:
        require(type(replay[field]) is type(original[field]) and replay[field] == original[field],
                f'{fixture.name}: stable replay manifest differs: {field}')
    for name in ('steps.csv', 'opacity.png'):
        require((actual / name).read_bytes() == (fixture / name).read_bytes(),
                f'{fixture.name}: replay bytes differ: {name}')
    return fields


def verify_replay(output, binary, fixture_root=Path('.'), receipt=None):
    if receipt is None:
        receipt = json.loads((output / 'receipt.json').read_text())
    require(type(receipt) is dict and receipt.get('schema_version') == 1,
            'Fresh producing-binary provenance required; historical replay receipts have none')
    require(receipt['scope'] == 'fresh-original-demo-replay', 'Fresh replay scope differs')
    require(receipt['producing_binary_sha256'] == sha256(binary), 'Replay producing binary differs')
    build = json.loads(subprocess.check_output([str(binary.resolve(strict=True)), '--build-info'], text=True))
    require(build['debug_assertions'] is False, 'Release executable required')
    require(build == receipt['producing_binary_build_info'], 'Producing binary build metadata differs')
    require([r['fixture'] for r in receipt['runs']] == [f'evidence/{name}' for name in DEMOS],
            'Fresh original demo inventory differs')
    for entry, name in zip(receipt['runs'], DEMOS):
        fixture, actual = fixture_root / 'evidence' / name, output / name
        fields = check_outputs(fixture, actual)
        require(entry['equal_manifest_fields'] == fields, 'Fresh manifest field inventory differs')
        require(set(entry['produced_sha256']) == set(ARTIFACTS), 'Produced artifact inventory differs')
        require(set(entry['fixture_sha256']) == set(ARTIFACTS), 'Fixture artifact inventory differs')
        for file in ARTIFACTS:
            require(sha256(actual / file) == entry['produced_sha256'][file], 'Produced artifact hash differs: '+file)
            require(sha256(fixture / file) == entry['fixture_sha256'][file], 'Original fixture hash differs: '+file)
    require(sha256(binary) == receipt['producing_binary_sha256'], 'Executable changed during qualification')
    return receipt


def produce_replay(binary, output):
    binary = binary.resolve(strict=True)
    identity = sha256(binary)
    build = json.loads(subprocess.check_output([str(binary), '--build-info'], text=True))
    require(build['debug_assertions'] is False, 'Release executable required')
    require(sha256(binary) == identity, 'Executable changed during build-info query')
    output.mkdir(exist_ok=False)
    runs = []
    for name in DEMOS:
        fixture, actual = Path('evidence') / name, output / name
        original = json.loads((fixture / 'run.json').read_text())
        command = [str(binary), '--size', str(original['size']), '--steps', str(original['steps']),
                   '--source-off-at', str(original['source_off_at']), '--dt', str(original['requested_dt']),
                   '--output', str(actual)]
        require(sha256(binary) == identity, 'Executable changed before replay')
        subprocess.run(command, check=True)
        require(sha256(binary) == identity, 'Executable changed during replay')
        fields = check_outputs(fixture, actual)
        runs.append({'fixture': str(fixture), 'command': command, 'equal_manifest_fields': fields,
                     'produced_sha256': {file: sha256(actual / file) for file in ARTIFACTS},
                     'fixture_sha256': {file: sha256(fixture / file) for file in ARTIFACTS}})
    receipt = {'schema_version': 1, 'scope': 'fresh-original-demo-replay',
               'producing_binary_sha256': identity, 'producing_binary_path': str(binary),
               'producing_binary_build_info': build, 'producer_driver_sha256': sha256(Path(__file__)),
               'runs': runs, 'new_performance_claim': False,
               'identity_limit': 'Executable hash checked before and after runs; no signed attestation or binary/source reproducibility claim.'}
    verify_replay(output, binary, receipt=receipt)
    with (output / 'receipt.json').open('x') as stream:
        stream.write(json.dumps(receipt, indent=2, allow_nan=False)+'\n')
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=('produce', 'check'))
    parser.add_argument('binary', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--receipt-output', type=Path,
                        help='fresh qualification receipt; never overwrite a historical receipt')
    args = parser.parse_args()
    receipt = produce_replay(args.binary, args.output) if args.action == 'produce' else verify_replay(args.output, args.binary)
    if args.receipt_output:
        with args.receipt_output.open('x') as stream:
            stream.write(json.dumps(receipt, indent=2, allow_nan=False)+'\n')
    print('PASS fresh producing-binary identity, 3 original manifests and PNG/CSV pairs')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError, OverflowError, subprocess.CalledProcessError) as exc:
        raise SystemExit(f'FAIL fresh replay qualification: {exc}') from exc
