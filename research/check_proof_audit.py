"""Shared fresh source-bound Lean audit of the bounded research leaves.

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


def require_external_output(output, root=None, message='output must remain outside Git'):
    """Resolve symlinks and probe an existing ancestor before any mutation."""
    output = Path(output).resolve()
    if output.exists():
        raise ValueError('new external output required')
    if root is not None and output.is_relative_to(Path(root).resolve()):
        raise ValueError(message)
    ancestor = output.parent
    while not ancestor.exists():
        ancestor = ancestor.parent
    # Discover from the canonical output ancestor, independently of an inherited
    # Git invocation's repository, ceiling, or command/config overrides.
    probe_env = {key: value for key, value in os.environ.items()
                 if not key.startswith('GIT_')}
    probe_env['LC_ALL'] = 'C'
    probe_env['GIT_DISCOVERY_ACROSS_FILESYSTEM'] = '1'
    probe = subprocess.run(['git', '-C', str(ancestor), 'rev-parse', '--is-inside-work-tree'],
                           capture_output=True, text=True, env=probe_env, timeout=30)
    if probe.returncode == 0:
        raise ValueError(message)
    if probe.returncode != 128 or not probe.stderr.startswith('fatal: not a git repository'):
        raise ValueError('cannot establish external output location')
    return output


def compiler_errors(text):
    return re.findall(r'^.*:\d+:\d+: error: (.*)$', text, re.M)


def cli(root, source, prefix, scope, checker):
    parser = argparse.ArgumentParser(description='Fresh source-bound research Lean audit')
    parser.add_argument('--lean', type=Path, required=True)
    parser.add_argument('--dependencies', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    audit(root, source, prefix, scope, checker, args.lean.resolve(),
          args.dependencies.resolve(), args.output.resolve())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit(root, source_path, prefix, scope, checker, lean, dependencies, output):
    ROOT, SOURCE, PREFIX = root.resolve(), source_path.resolve(), prefix
    output = require_external_output(output, ROOT, 'proof output must remain outside Git')
    pins = json.loads((ROOT/'proofs/lake-manifest.json').read_text())['packages']
    checked = {}
    for package in pins:
        actual = subprocess.check_output(['git', '-C', str(dependencies/package['name']), 'rev-parse', 'HEAD'], text=True, timeout=30).strip()
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
    if (Lean.privateToUserName name).toString.startsWith "AUDIT_PREFIX" then
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
'''.replace('EXPECTED', expected).replace('AUDIT_PREFIX', PREFIX)
    output.mkdir(parents=True)
    paths = [str(p/'.lake/build/lib/lean') for p in sorted(dependencies.iterdir()) if (p/'.lake/build/lib/lean').is_dir()]
    env = dict(os.environ, LEAN_PATH=':'.join(paths))
    commands = []

    def run(path, name, expected_error=None):
        command = [str(lean), str(path)]
        r = subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True, timeout=120)
        log = output/(name+'.log')
        log.write_text(r.stdout+r.stderr)
        errors = compiler_errors(r.stdout+r.stderr)
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
        ('custom', 'axiom injected : False', 'Disallowed axiom '+PREFIX+'injected in '+PREFIX+'injected'),
        ('sorry', 'theorem injected : False := by sorry', 'Disallowed axiom sorryAx in '+PREFIX+'injected')]:
        path = output/('Negative-'+label+'.lean')
        path.write_text('import Lean.Util.CollectAxioms\n'+source+
                        '\nnamespace '+PREFIX.rstrip('.')+'\n'+declaration+
                        '\nend '+PREFIX.rstrip('.')+'\n'+audit)
        run(path, 'negative-'+label, error)
    result = {'qualified': True, 'scope': scope,
              'source_sha256': sha(SOURCE), 'checker_sha256': sha(checker), 'shared_driver_sha256': sha(Path(__file__)),
              'source_declarations': [PREFIX+name for name in names], 'allowed_axioms': ['propext', 'Classical.choice', 'Quot.sound'],
              'dependency_pins': checked, 'commands': commands,
              'IEEE_refinement': False, 'physical_load_accuracy': False, 'production_change': False}
    (output/'receipt.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'qualified': True, 'receipt': str(output/'receipt.json'), 'sha256': sha(output/'receipt.json')}))
