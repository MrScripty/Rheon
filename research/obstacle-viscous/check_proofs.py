"""Fresh source-bound Lean audit of the research-only conditional viscous potential/stress/force statements.

Uses existing pinned third-party dependencies; never downloads or builds more.
Generated audit/probe files stay in a new external output directory.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]
SOURCE = Path(__file__).with_name('ViscousStress.lean')
PREFIX = 'Rheon.ObstacleViscous.'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(lean, dependencies, output):
    if output.exists():
        raise ValueError('new external output required')
    if subprocess.run(['git', '-C', str(output.parent), 'rev-parse', '--is-inside-work-tree'], capture_output=True).returncode == 0:
        raise ValueError('proof output must remain outside Git')
    pins = json.loads((ROOT/'proofs/lake-manifest.json').read_text())['packages']
    checked = {}
    for package in pins:
        actual = subprocess.check_output(['git', '-C', str(dependencies/package['name']), 'rev-parse', 'HEAD'], text=True).strip()
        if actual != package['rev']:
            raise ValueError('dependency pin differs: '+package['name'])
        checked[package['name']] = actual
    source = SOURCE.read_text()
    names = re.findall(r'^theorem\s+(\w+)', source, re.M)
    if len(names) != len(set(names)) or not names:
        raise ValueError('unambiguous source theorem inventory required')
    expected = ','.join('`'+PREFIX+name for name in names)
    audit = r'''
open Lean Elab Command
run_cmd do
  let env ← getEnv
  let expected : Array Name := #[EXPECTED]
  let allowed : Array Name := #[`propext, `Classical.choice, `Quot.sound]
  let mut audited : Array Name := #[]
  for (name, info) in env.constants.toList do
    if (Lean.privateToUserName name).toString.startsWith "Rheon.ObstacleViscous." then
      let logical ← match info with
        | .thmInfo _ => pure true
        | .axiomInfo _ => pure true
        | .opaqueInfo _ => pure true
        | .defnInfo value =>
            if value.safety == .safe then pure true
            else if value.type.getUsedConstants.all (env.contains ·) then
              liftTermElabM (Lean.Meta.isProp value.type)
            else pure false
        | _ => pure false
      if logical || expected.contains name then
        let axioms ← Lean.collectAxioms name
        for axiomName in axioms do
          unless allowed.contains axiomName do
            throwError "Disallowed axiom {axiomName} in {name}"
        logInfo m!"AUDITED {name}: {axioms}"
        audited := audited.push name
  for name in expected do
    unless audited.contains name do
      throwError "Expected declaration not audited: {name}"
  logInfo "Research axiom audit complete"
'''.replace('EXPECTED', expected)
    output.mkdir(parents=True)
    paths = [str(p/'.lake/build/lib/lean') for p in sorted(dependencies.iterdir()) if (p/'.lake/build/lib/lean').is_dir()]
    env = dict(os.environ, LEAN_PATH=':'.join(paths))
    commands = []

    def run(path, name, expected_error=None):
        command = [str(lean), str(path)]
        r = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True, timeout=120)
        log = output/(name+'.log')
        log.write_text(r.stdout+r.stderr)
        errors = re.findall(r'^.*:\d+:\d+: error: (.*)$', r.stdout+r.stderr, re.M)
        if expected_error is None:
            if r.returncode or 'Research axiom audit complete' not in r.stdout:
                raise ValueError('fresh research audit failed: '+name)
        elif not r.returncode or errors != [expected_error]:
            raise ValueError('wrong kernel probe rejection: '+name)
        commands.append({'command': command, 'returncode': r.returncode,
                         'expected_error': expected_error, 'log': log.name, 'log_sha256': sha(log)})

    combined = 'import Lean.Util.CollectAxioms\n'+source+'\n'+audit
    positive = output/'AuditedSource.lean'
    positive.write_text(combined)
    run(positive, 'positive')
    for label, declaration, error in [
        ('custom', 'axiom injected : False', 'Disallowed axiom Rheon.ObstacleViscous.injected in Rheon.ObstacleViscous.injected'),
        ('sorry', 'theorem injected : False := by sorry', 'Disallowed axiom sorryAx in Rheon.ObstacleViscous.injected')]:
        path = output/('Negative-'+label+'.lean')
        path.write_text('import Lean.Util.CollectAxioms\n'+source+
                        '\nnamespace Rheon.ObstacleViscous\n'+declaration+
                        '\nend Rheon.ObstacleViscous\n'+audit)
        run(path, 'negative-'+label, error)
    result = {'qualified': True, 'scope': 'conditional exact-real finite potential variation, symmetric stress, matched force work and positive dissipation only',
              'source_sha256': sha(SOURCE), 'checker_sha256': sha(Path(__file__)),
              'source_declarations': [PREFIX+name for name in names], 'allowed_axioms': ['propext', 'Classical.choice', 'Quot.sound'],
              'dependency_pins': checked, 'commands': commands,
              'IEEE_refinement': False, 'physical_load_accuracy': False, 'production_change': False}
    (output/'receipt.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'qualified': True, 'receipt': str(output/'receipt.json'), 'sha256': sha(output/'receipt.json')}))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lean', type=Path, required=True)
    parser.add_argument('--dependencies', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    main(args.lean.resolve(), args.dependencies.resolve(), args.output.resolve())
