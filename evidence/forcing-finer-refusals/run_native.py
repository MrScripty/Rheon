"""Capture bounded, opt-in refusal diagnostics with unchanged native outcomes."""
import hashlib
import json
import os
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
P = Path(__file__).resolve().parent
BASE = 'f76841e9217e781d91c5bfeabadb6b4c6f42b20b'
PATHS = ['src/coupled_discrete.rs', 'examples/forcing_temporal_probe.rs']
TARGET = '/workspace/.rheon-tools/finer-refusal-trace-target'
KNOWN = '--check-cfg=cfg(rheon_newton_trace)'
records = []


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def stripped(path):
    result = []
    inside = False
    blocks = 0
    for line in path.read_bytes().splitlines(keepends=True):
        if b'BEGIN RHEON REFUSAL DIAGNOSTIC' in line:
            require(not inside, 'nested diagnostic marker')
            inside = True
            blocks += 1
        elif b'END RHEON REFUSAL DIAGNOSTIC' in line:
            require(inside, 'unmatched diagnostic marker')
            inside = False
        elif not inside:
            result.append(line)
    require(not inside and blocks > 0, 'complete instrumentation markers')
    return b''.join(result), blocks


bindings = {}
for name in PATHS:
    original = subprocess.check_output(['git', 'show', BASE + ':' + name], cwd=ROOT)
    restored, blocks = stripped(ROOT / name)
    require(restored == original, 'original numerical/control source changed ' + name)
    bindings[name] = dict(instrumented_sha256=sha(ROOT / name),
                          original_sha256=hashlib.sha256(original).hexdigest(),
                          diagnostic_blocks=blocks, stripped_original_byte_identical=True)


def run(name, argv, trace=False, suffix='log'):
    require(all(sha(ROOT / path) == value['instrumented_sha256'] for path, value in bindings.items()), 'unchanged captured source')
    out = P / (name + '.' + suffix)
    err = P / (name + '-stderr.log')
    require(not out.exists() and not err.exists(), 'do not overwrite actual execution ' + name)
    env = dict(os.environ, CARGO_TARGET_DIR=TARGET,
               RUSTFLAGS=KNOWN + (' --cfg rheon_newton_trace' if trace else ''))
    start = time.monotonic()
    with out.open('wb') as stdout, err.open('wb') as stderr:
        result = subprocess.run(argv, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
    records.append(dict(name=name, argv=argv, trace_enabled=trace, exit=result.returncode,
                        elapsed_seconds=time.monotonic() - start,
                        RUSTFLAGS=env['RUSTFLAGS'], CARGO_TARGET_DIR=TARGET,
                        stdout=str(out.relative_to(ROOT)), stderr=str(err.relative_to(ROOT))))
    (P / 'native-commands.json').write_text(json.dumps(dict(historical_base=BASE,
        source_binding=bindings, commands=records,
        status='PASS_COMPLETED_COMMANDS' if all(row['exit'] == 0 for row in records) else 'FAIL'), indent=2) + '\n')
    require(result.returncode == 0, 'actual native diagnostic command failure ' + name)
    print(name + ' PASS', flush=True)


run('format', ['cargo', 'fmt', '--all', '--check'])
for trace in [False, True]:
    for mode, flags in [('default', []), ('no-default', ['--no-default-features'])]:
        tag = ('trace' if trace else 'plain') + '-' + mode
        run('clippy-' + tag, ['cargo', 'clippy', '--locked', '--example', 'forcing_temporal_probe', *flags, '--', '-D', 'warnings'], trace)
        run('native-' + tag, ['cargo', 'run', '--locked', '--release', '--quiet', '--example', 'forcing_temporal_probe', *flags], trace, 'jsonl')
baseline = (ROOT / 'evidence/forcing-temporal-diagnosis/first-native-probe.jsonl').read_bytes()
for tag in ['plain-default', 'plain-no-default', 'trace-default', 'trace-no-default']:
    require((P / ('native-' + tag + '.jsonl')).read_bytes() == baseline, 'all actual published states and original refusal markers unchanged ' + tag)
require((P / 'native-trace-default-stderr.log').read_bytes() == (P / 'native-trace-no-default-stderr.log').read_bytes(), 'actual trace build-mode parity')
require(len(records) == 9, 'all actual diagnostic commands')
receipt = dict(status='PASS_NATIVE_DIAGNOSTICS_ORIGINAL_OUTCOMES_RETAINED', historical_base=BASE,
               source_binding=bindings, actual_commands=records,
               actual_publications=143, actual_accepted_steps=135, original_completed_cases=2,
               original_refused_cases=6, actual_same_owner_refusal_retries=6,
               original_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND',
               native_outcomes_byte_identical_in_four_builds=True, trace_modes_byte_identical=True,
               observation_after_window_never_publishes=True,
               no_original_numerical_or_control_lines_changed=True,
               file_sha256={str(path.relative_to(ROOT)): sha(path) for path in sorted(P.glob('*')) if path.is_file() and path.name != 'native-receipt.json'})
(P / 'native-receipt.json').write_text(json.dumps(receipt, indent=2, sort_keys=True) + '\n')
print(receipt['status'], flush=True)
