#!/usr/bin/env python3
"""Fresh source-bound qualification of the opt-in aligned Stokes transaction.

No prior receipt is consumed. Native interval evidence is conditional on the
specified arithmetic environment; Lean establishes implications from explicit
containment premises, not compiler or IEEE correspondence.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
BASE = '0e13b02294b247d9f59326f98e6233fb9280967e'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_snapshot():
    paths = git('ls-files', '--cached', '--others', '--exclude-standard').splitlines()
    return {'head': git('rev-parse', 'HEAD'), 'tree': git('rev-parse', 'HEAD^{tree}'),
            'status': git('status', '--porcelain'), 'paths': paths,
            'sha256': {p: digest(ROOT / p) for p in paths if (ROOT / p).is_file()}}


def external_new_directory(output):
    output = output.resolve()
    existing = output
    while not existing.exists():
        existing = existing.parent
    probe = subprocess.run(['git', '-C', str(existing), 'rev-parse',
                            '--is-inside-work-tree'], capture_output=True)
    if probe.returncode == 0:
        raise ValueError('generated evidence must be outside Git')
    if output.exists():
        raise ValueError('qualification destination must be new')
    return output


def run(command, output, name, cwd=ROOT, env=None):
    with (output / (name + '.log')).open('xb') as log:
        result = subprocess.run(command, cwd=cwd, stdout=log, env=env,
                                stderr=subprocess.STDOUT, timeout=1800)
    if result.returncode:
        raise RuntimeError(f'{name} failed ({result.returncode}); see external log')


def qualify(args):
    output = external_new_directory(args.output)
    rust_version = subprocess.check_output(['rustc', '--version'], text=True).strip()
    if not rust_version.startswith('rustc 1.92.0 '):
        raise ValueError('qualification requires pinned Rust 1.92.0')
    source = source_snapshot()
    if source['status'] and not args.allow_dirty:
        raise ValueError('qualification requires clean committed source')
    subprocess.run(['git', 'merge-base', '--is-ancestor', BASE, 'HEAD'], cwd=ROOT, check=True)
    required = ['experiments/aligned_stokes.rs', 'experiments/outward.rs',
                'experiments/proofs/AlignedStepAcceptance.lean', 'experiments/proofs/check_acceptance.py', 'experiments/proofs/AlignedStepAcceptanceAudit.lean',
                'examples/aligned_stokes.rs', 'examples/outward_probe.rs',
                'tests/aligned_stokes_contract.rs', 'tools/aligned_stokes_oracle.py',
                'tools/test_aligned_stokes_oracle.py', 'tools/test_qualify_aligned_stokes.py']
    if any(not (ROOT / p).is_file() for p in required):
        raise ValueError('missing required qualification source')
    output.mkdir(parents=True, exist_ok=False)
    receipt = output / 'qualification.json'
    manifest = {'schema': 'rheon-experimental-aligned-stokes-qualification-v1',
                'base': BASE, 'source_head': source['head'], 'source_tree': source['tree'],
                'clean': not bool(source['status']), 'source_sha256': source['sha256'],
                'status': 'running', 'commands': [], 'lean_checked': False,
                'ieee_correspondence_proved': False,
                'native_arithmetic_premise': 'Linux x86_64 default Rust scalar binary64 semantics; no altered FP controls',
                'production_activation': False, 'rust_version': rust_version,
                'platform': {'system': platform.system(), 'machine': platform.machine()}}

    def execute(name, command, cwd=ROOT, env=None):
        run(command, output, name, cwd, env)
        manifest['commands'].append({'name': name, 'argv': command, 'exit': 0})
        receipt.write_text(json.dumps(manifest, indent=2) + '\n')

    try:
        receipt.write_text(json.dumps(manifest, indent=2) + '\n')
        execute('format', ['cargo', 'fmt', '--check'])
        execute('rust-contracts', ['cargo', 'test', '--locked', '--no-default-features',
                                  '--test', 'aligned_stokes_contract', '--test', 'aligned_strain_contract',
                                  '--test', 'obstacle_flow_contract', '--test', 'static_obstacle_contract',
                                  '--example', 'aligned_stokes', '--example', 'outward_probe'])
        execute('rust-build', ['cargo', 'build', '--locked', '--no-default-features',
                              '--example', 'aligned_stokes', '--example', 'outward_probe'])
        execute('rust-clippy', ['cargo', 'clippy', '--locked', '--no-default-features', '--lib',
                               '--example', 'aligned_stokes', '--example', 'outward_probe',
                               '--test', 'aligned_stokes_contract', '--', '-D', 'warnings'])
        target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
        if not target.is_absolute():
            target = ROOT / target
        native, probe = [target / 'debug/examples' / name for name in ['aligned_stokes', 'outward_probe']]
        env = dict(os.environ, RHEON_ALIGNED_STOKES_NATIVE=str(native), RHEON_OUTWARD_PROBE=str(probe))
        execute('independent-rational-native', [sys.executable, 'tools/aligned_stokes_oracle.py',
                                               '--native', str(native), '--probe', str(probe),
                                               '--output', str(output / 'rational')], env=env)
        for name, flags in [('tests', []), ('tests-optimized', ['-O'])]:
            for group in ['aligned_stokes_oracle', 'qualify_aligned_stokes']:
                execute(group + '-' + name, [sys.executable, *flags, '-m', 'unittest', 'discover',
                                             '-s', 'tools', '-p', 'test_' + group + '.py', '-v'], env=env)
        if args.lean:
            execute('lean-existing-build', ['lake', 'build'], ROOT / 'proofs')
            execute('lean-step-kernel-and-audit', ['lake', 'env', 'lean',
                                                 '../experiments/proofs/AlignedStepAcceptance.lean'], ROOT / 'proofs')
            execute('lean-step-audit-refusals', [sys.executable, 'experiments/proofs/check_acceptance.py'])
            execute('lean-step-audit-refusals-optimized', [sys.executable, '-O', 'experiments/proofs/check_acceptance.py'])
            manifest['lean_checked'] = True
        if source_snapshot() != source:
            raise RuntimeError('source changed during qualification')
        manifest.update(status='passed', executable_sha256={p.name: digest(p) for p in [native, probe]},
                        artifacts_sha256={str(p.relative_to(output)): digest(p) for p in output.rglob('*')
                                          if p.is_file() and p != receipt})
    except Exception as error:
        manifest.update(status='failed', error=str(error))
        raise
    finally:
        receipt.write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps({k: manifest[k] for k in ['status', 'source_head', 'source_tree', 'clean', 'lean_checked']}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--lean', action='store_true')
    parser.add_argument('--allow-dirty', action='store_true', help='intermediate evidence only; records clean=false')
    qualify(parser.parse_args())
