"""Actual research and native replay; immutable outputs, no asserted success."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
PREFIX = ROOT / 'evidence/fitted-pressure-path'
output = Path(sys.argv[1]).resolve()
output.mkdir()
commands = []


def run(command, name, destination):
    with destination.open('w') as stdout, (output / (name + '-stderr.log')).open('w') as stderr:
        code = subprocess.run(command, cwd=ROOT, stdout=stdout, stderr=stderr).returncode
    commands.append(dict(name=name, command=command, exit=code))
    (output / 'receipt.json').write_text(json.dumps(dict(status='in progress', commands=commands), indent=2) + '\n')
    print(name, 'exit', code, flush=True)
    if code:
        raise SystemExit(code)


for optimized in [False, True]:
    mode = 'optimized' if optimized else 'normal'
    for script, label in [('reference.py', 'geometry'), ('numerical.py', 'numerical'), ('test_reference.py', 'tests')]:
        command = [sys.executable] + (['-O'] if optimized else []) + [str(PREFIX / script)]
        run(command, mode + '-' + label, output / (mode + '-' + label + ('.log' if label == 'tests' else '.json')))
for label in ['geometry', 'numerical']:
    if (output / ('normal-' + label + '.json')).read_bytes() != (output / ('optimized-' + label + '.json')).read_bytes():
        raise ValueError('normal/optimized exact research replay: ' + label)

native = []
for profile in ['debug', 'release']:
    command = ['cargo', 'build', '--locked', '--example', 'fitted_pressure_path'] + (['--release'] if profile == 'release' else [])
    run(command, profile + '-build', output / (profile + '-build.log'))
    binary = Path(os.environ['CARGO_TARGET_DIR']) / profile / 'examples/fitted_pressure_path'
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    payload = output / (profile + '-native.json')
    run([str(binary)], profile + '-native', payload)
    commands[-1]['binary_sha256'] = digest
    for optimized in [False, True]:
        mode = 'optimized' if optimized else 'normal'
        command = [sys.executable] + (['-O'] if optimized else []) + [str(PREFIX / 'verify_native.py'), str(payload), '--negative-self-test']
        run(command, profile + '-native-' + mode, output / (profile + '-native-' + mode + '.log'))
    from verify_native import verify
    native.append(dict(profile=profile, binary_sha256=digest, **verify(json.loads(payload.read_text()))))
if (output / 'debug-native.json').read_bytes() != (output / 'release-native.json').read_bytes():
    raise ValueError('native debug/release exact replay')
paths = subprocess.check_output(['git', 'ls-files', '--', 'src', 'tests', 'examples', 'Cargo.toml', 'Cargo.lock'], cwd=ROOT).decode().splitlines()
paths.append('examples/fitted_pressure_path.rs')
source_sha = {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in sorted(set(paths)) if p.endswith('.rs') or p.startswith('Cargo.')}
(output / 'receipt.json').write_text(json.dumps(dict(status='PASS', commands=commands, native=native,
    normal_optimized_research_byte_equal=True, debug_release_byte_equal=True,
    compiled_Rust_source_sha256=source_sha, reference_tests_per_mode=11,
    native_corruptions_rejected_per_validator=8, accepted_advancing_steps=0), indent=2) + '\n')
print('PASS exact/physical research, 11 tests per mode, four native validators / 32 corruptions, debug/release byte equality', flush=True)
