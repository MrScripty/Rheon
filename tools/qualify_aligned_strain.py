#!/usr/bin/env python3
"""Run a fresh isolated native/rational qualification into a new external directory.

This receipt records nearest-rounded experiments; it is never a timestep
authorization or an IEEE enclosure. Lean checking, when requested, is separate
exact-real evidence. No previous receipt/pass count is consumed.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
BASE = 'b61ae293224ff8262e99984a1b8fa8b2135b8c83'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command, output, name, cwd=ROOT):
    with (output / (name + '.log')).open('xb') as log:
        result = subprocess.run(command, cwd=cwd, stdout=log,
                                stderr=subprocess.STDOUT, timeout=1800)
    if result.returncode:
        raise RuntimeError(f'{name} failed ({result.returncode}); see external log')


def qualify(args):
    output = args.output.resolve()
    # Reject any Git-owned destination, including a nested independent repository.
    existing = output
    while not existing.exists():
        existing = existing.parent
    probe = subprocess.run(['git', '-C', str(existing), 'rev-parse',
                            '--is-inside-work-tree'], capture_output=True)
    if probe.returncode == 0:
        raise ValueError('generated evidence must be outside Git')
    dirty = bool(git('status', '--porcelain'))
    if dirty and not args.allow_dirty:
        raise ValueError('qualification requires a clean committed source tree')
    subprocess.run(['git', 'merge-base', '--is-ancestor', BASE, 'HEAD'],
                   cwd=ROOT, check=True)
    output.mkdir(parents=True, exist_ok=False)
    head, tree = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD^{tree}')
    sources = {p: digest(ROOT / p) for p in git('ls-files').splitlines()
               if (ROOT / p).is_file()}
    manifest = {'schema': 'rheon-reconstructed-aligned-strain-qualification-v1',
                'base': BASE, 'source_head': head, 'source_tree': tree,
                'clean': not dirty, 'source_sha256': sources,
                'new_reconstruction': True, 'stepping_authorized': False,
                'floating_point_estimate_enclosed': False,
                'commands': [], 'status': 'running'}
    receipt = output / 'qualification.json'
    receipt.write_text(json.dumps(manifest, indent=2) + '\n')
    commands = [
        ('format', ['cargo', 'fmt', '--check']),
        ('rust-contracts', ['cargo', 'test', '--locked', '--no-default-features',
                           '--test', 'aligned_strain_contract', '--test',
                           'obstacle_flow_contract', '--test', 'static_obstacle_contract']),
        ('rust-build', ['cargo', 'build', '--locked', '--no-default-features',
                        '--example', 'aligned_strain']),
        ('oracle-tests', [sys.executable, '-m', 'unittest', 'discover', '-s',
                          'tools', '-p', 'test_aligned_strain_oracle.py', '-v']),
        ('oracle-tests-optimized', [sys.executable, '-O', '-m', 'unittest',
                                    'discover', '-s', 'tools', '-p',
                                    'test_aligned_strain_oracle.py', '-v']),
    ]
    if args.lean:
        commands.extend([
            ('lean-source-policy', [sys.executable, 'scripts/check_sources.py']),
            ('lean-build', ['lake', 'build']),
            ('lean-axiom-audit', ['lake', 'env', 'lean', 'AxiomAudit.lean']),
            ('lean-rejection-tests', [sys.executable, 'scripts/test_audit.py']),
            ('lean-rejection-tests-optimized', [sys.executable, '-O', 'scripts/test_audit.py']),
        ])
    try:
        for name, command in commands:
            run(command, output, name, ROOT / 'proofs' if name.startswith('lean-') else ROOT)
            manifest['commands'].append({'name': name, 'argv': command, 'exit': 0})
            receipt.write_text(json.dumps(manifest, indent=2) + '\n')
        target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
        if not target.is_absolute():
            target = ROOT / target
        executable = target / 'debug/examples/aligned_strain'
        records = output / 'native.tsv'
        run([str(executable), str(records)], output, 'native-run')
        run([sys.executable, 'tools/aligned_strain_oracle.py', str(records),
             '--output', str(output / 'oracle-summary.json')], output, 'native-oracle-comparison')
        if git('rev-parse', 'HEAD') != head or any(digest(ROOT / p) != h for p, h in sources.items()):
            raise RuntimeError('source changed during qualification')
        manifest.update(status='passed', lean_checked=args.lean,
                        executable_sha256=digest(executable),
                        records_sha256=digest(records),
                        artifacts_sha256={p.name: digest(p) for p in output.iterdir()
                                          if p.is_file() and p != receipt})
    except Exception as error:
        manifest.update(status='failed', error=str(error))
        raise
    finally:
        receipt.write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps({k: manifest[k] for k in ['status', 'source_head', 'source_tree',
                                              'clean', 'lean_checked', 'records_sha256']}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--lean', action='store_true')
    parser.add_argument('--allow-dirty', action='store_true',
                        help='intermediate evidence only; receipt records clean=false')
    qualify(parser.parse_args())
