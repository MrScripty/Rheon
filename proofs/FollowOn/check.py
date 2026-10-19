"""Reproduce the isolated kernel checks and exercise audit refusal paths.

Outputs are generated evidence. The caller supplies a fresh writable directory.
Uses the pinned project Lake environment; does not update root proof inventory.
"""
from pathlib import Path
import argparse, os, subprocess

parser = argparse.ArgumentParser()
parser.add_argument('output', type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=True)
(out / 'FollowOn').mkdir(exist_ok=True)
env = os.environ.copy()
# lake env prepends its configured paths, retaining this additional import root.
env['LEAN_PATH'] = str(out) + os.pathsep + env.get('LEAN_PATH', '')

def run(name, command, expected_error=None):
    result = subprocess.run(command, cwd=root, env=env, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (out / (name + '.log')).write_text(result.stdout)
    if expected_error is None:
        if result.returncode:
            raise SystemExit(f'{name}: failed ({result.returncode}); see {out}')
    elif result.returncode == 0 or expected_error not in result.stdout:
        raise SystemExit(f'{name}: rejection not established; see {out}')
    print(f'PASS {name}')

for module in ['Publication', 'StokesWork']:
    run(module, ['lake', 'env', 'lean', '-o', str(out/'FollowOn'/f'{module}.olean'),
                 f'FollowOn/{module}.lean'])
run('axiom-audit', ['lake', 'env', 'lean', 'FollowOn/AxiomAudit.lean'])
audit = (root/'FollowOn/AxiomAudit.lean').read_text()
probes = [
    ('custom-axiom', audit.replace('open Lean Elab Command',
      'namespace Rheon.FollowOn\naxiom injected : False\nend Rheon.FollowOn\nopen Lean Elab Command'),
      'Disallowed axiom'),
    ('admitted-proof', audit.replace('open Lean Elab Command',
      'namespace Rheon.FollowOn\ntheorem injected : False := by sorry\nend Rheon.FollowOn\nopen Lean Elab Command'),
      'Disallowed axiom'),
    ('skipped-expected', audit.replace('let expected : Array Name := #[',
      'let expected : Array Name := #[`Nat.add_zero,'), 'Expected declaration not audited'),
]
for name, source, expected in probes:
    path = out / f'{name}.lean'
    path.write_text(source)
    run(name, ['lake', 'env', 'lean', str(path)], expected)
print('PASS isolated follow-on kernel/axiom qualification (not Rust/physics validation)')
