"""Validate ordinary builds after declaring the known diagnostic cfg name."""
import hashlib
import json
import os
import subprocess
import time
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
P = Path(__file__).resolve().parent
BASE = 'f76841e9217e781d91c5bfeabadb6b4c6f42b20b'
PATHS = ['src/coupled_discrete.rs', 'examples/forcing_temporal_probe.rs', 'Cargo.toml']


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def stripped(data):
    output = []
    inside = False
    for line in data.splitlines(keepends=True):
        if b'BEGIN RHEON REFUSAL DIAGNOSTIC' in line:
            require(not inside, 'nested marker')
            inside = True
        elif b'END RHEON REFUSAL DIAGNOSTIC' in line:
            require(inside, 'unmatched marker')
            inside = False
        elif not inside:
            output.append(line)
    require(not inside, 'unterminated marker')
    return b''.join(output)


bindings = {}
for path in PATHS:
    original = subprocess.check_output(['git', 'show', BASE + ':' + path], cwd=ROOT)
    current = (ROOT / path).read_bytes()
    require(stripped(current) == original, 'unchanged original source/control/manifest ' + path)
    bindings[path] = dict(original_sha256=sha(original), current_sha256=sha(current),
                          stripped_original_byte_identical=True)
initial = json.loads((P / 'native-receipt.json').read_text())
require(all(initial['source_binding'][path]['instrumented_sha256'] == bindings[path]['current_sha256'] for path in PATHS[:2]), 'same previously traced Rust source')
records = []
env = dict(os.environ, CARGO_TARGET_DIR='/workspace/.rheon-tools/finer-refusal-trace-target')
env.pop('RUSTFLAGS', None)
env.pop('CARGO_ENCODED_RUSTFLAGS', None)
for mode, flags in [('default', []), ('no-default', ['--no-default-features'])]:
    for task, argv, suffix in [
        ('clippy', ['cargo', 'clippy', '--locked', '--all-targets', *flags, '--', '-D', 'warnings'], 'log'),
        ('native', ['cargo', 'run', '--locked', '--release', '--quiet', *flags, '--example', 'forcing_temporal_probe'], 'jsonl'),
    ]:
        out = P / ('ordinary-' + task + '-' + mode + '.' + suffix)
        err = P / ('ordinary-' + task + '-' + mode + '-stderr.log')
        require(not out.exists() and not err.exists(), 'refuse overwritten ordinary execution')
        start = time.monotonic()
        with out.open('wb') as stdout, err.open('wb') as stderr:
            result = subprocess.run(argv, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
        records.append(dict(argv=argv, exit=result.returncode, elapsed_seconds=time.monotonic() - start,
                            stdout=str(out.relative_to(ROOT)), stderr=str(err.relative_to(ROOT))))
        (P / 'ordinary-commands.json').write_text(json.dumps(dict(source_binding=bindings, commands=records), indent=2) + '\n')
        require(result.returncode == 0, 'ordinary build failed')
        if task == 'native':
            require(out.read_bytes() == (ROOT / 'evidence/forcing-temporal-diagnosis/first-native-probe.jsonl').read_bytes(), 'ordinary native outcomes unchanged')
            require(not err.read_bytes(), 'ordinary run has no cfg warnings')
        print('ordinary-' + task + '-' + mode + ' PASS', flush=True)
require(all(sha((ROOT / path).read_bytes()) == value['current_sha256'] for path, value in bindings.items()), 'stable configuration source')
(P / 'ordinary-receipt.json').write_text(json.dumps(dict(status='PASS_ORDINARY_BUILDS_WITH_DECLARED_DIAGNOSTIC_CFG',
    source_binding=bindings, commands=records, no_diagnostic_RUSTFLAGS_required=True,
    original_cargo_features_dependencies_and_unsafe_forbid_unchanged=True,
    native_outcomes_byte_identical=True), indent=2, sort_keys=True) + '\n')
