"""Run the existing invariant suite with a disposable invoking repository."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
INTEGRATION = '1e966357c45a92ee2c318fc47921b3e2f8e539a7'
LOCAL_NAMES = subprocess.check_output(['git', 'rev-parse', '--local-env-vars'], text=True).splitlines()


def clean_environment():
    return {name: value for name, value in os.environ.items() if name not in LOCAL_NAMES}


def git(root, *args):
    environment = clean_environment()
    environment['GIT_OPTIONAL_LOCKS'] = '0'
    return subprocess.check_output(['git', '-C', str(root), *args], env=environment)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def snapshot(root):
    tracked = {}
    for name in git(root, 'ls-files', '-z').decode().split('\0'):
        if name:
            path = root / name
            tracked[name] = digest(path.read_bytes()) if path.is_file() else None
    return {
        'head': git(root, 'rev-parse', 'HEAD').decode().strip(),
        'refs_sha256': digest(git(root, 'for-each-ref', '--format=%(refname) %(objectname)')),
        'tracked_bytes_sha256': digest(json.dumps(tracked, sort_keys=True).encode()),
        'index_sha256': digest((root / '.git/index').read_bytes()),
        'operator_index_sha256': digest((root / '.git/operator-index').read_bytes()),
        'operator_marker_sha256': digest((root / 'operator-marker.txt').read_bytes()),
        'status': git(root, 'status', '--porcelain=v1', '--untracked-files=all').decode(),
    }


def child(before):
    original_environment = dict(os.environ)
    path = ROOT / 'evidence/pr7-verifier-repair/test_invariants.py'
    spec = importlib.util.spec_from_file_location('isolated_invariants', path)
    module = importlib.util.module_from_spec(spec)
    if before:
        source = git(ROOT, 'show', INTEGRATION + ':evidence/pr7-verifier-repair/test_invariants.py')
        exec(compile(source, str(path), 'exec'), module.__dict__)
    else:
        spec.loader.exec_module(module)
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromModule(module))
    restored_after_suite = dict(os.environ) == original_environment
    setup_failure_restored = None
    if result.wasSuccessful():
        class SetupFailure(module.Invariants):
            pass
        try:
            with mock.patch.object(module.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, ['git', 'clone'])):
                try:
                    SetupFailure.setUpClass()
                except subprocess.CalledProcessError:
                    pass
                else:
                    raise RuntimeError('Injected seed-clone failure did not occur')
        finally:
            SetupFailure.doClassCleanups()
            if hasattr(SetupFailure, 'pool'):
                SetupFailure.pool.cleanup()
        setup_failure_restored = dict(os.environ) == original_environment
    print(json.dumps({'tests_run': result.testsRun, 'suite_success': result.wasSuccessful(),
                      'environment_restored_after_suite': restored_after_suite,
                      'environment_restored_after_setup_failure': setup_failure_restored}))
    return 0 if result.wasSuccessful() and restored_after_suite and setup_failure_restored else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--child', action='store_true')
    parser.add_argument('--before', action='store_true')
    parser.add_argument('--expect-red', action='store_true')
    parser.add_argument('--variables', choices=('all', 'index'), default='all')
    args = parser.parse_args()
    if args.child:
        return child(args.before)
    with tempfile.TemporaryDirectory() as tmp:
        invoking = Path(tmp) / 'invoking'
        subprocess.run(['git', 'clone', '--quiet', '--shared', '--no-checkout', str(ROOT), str(invoking)],
                       env=clean_environment(), check=True)
        subprocess.run(['git', '-C', str(invoking), 'checkout', '--quiet', '--detach', INTEGRATION],
                       env=clean_environment(), check=True)
        (invoking / 'operator-marker.txt').write_text('Preserve this invoking-repository file.\n')
        shutil.copyfile(invoking / '.git/index', invoking / '.git/operator-index')
        before = snapshot(invoking)
        environment = clean_environment()
        inherited = {'GIT_INDEX_FILE': str(invoking / '.git/operator-index')}
        if args.variables == 'all':
            inherited.update(GIT_DIR=str(invoking / '.git'), GIT_WORK_TREE=str(invoking))
        environment.update(inherited)
        command = [sys.executable]
        if sys.flags.optimize:
            command.append('-O')
        command.extend([str(Path(__file__).resolve()), '--child'])
        if args.expect_red:
            command.append('--before')
        run = subprocess.run(command, cwd=ROOT, env=environment, text=True, capture_output=True)
        after = snapshot(invoking)
        child_result = json.loads(run.stdout)
        report = {'python': sys.version, 'optimization': sys.flags.optimize, 'integration': INTEGRATION,
                  'inherited_variables': sorted(inherited),
                  'child_exit_code': run.returncode, 'child': child_result,
                  'before': before, 'after': after, 'invoking_repository_preserved': before == after,
                  'changed_snapshot_fields': [name for name in before if before[name] != after[name]]}
        print(json.dumps(report, indent=2, sort_keys=True))
        print(run.stderr, file=sys.stderr, end='')
        if args.expect_red:
            return 0 if run.returncode != 0 and (args.variables == 'all' or before != after) else 1
        return 0 if run.returncode == 0 and before == after and child_result['tests_run'] == 12 else 1


if __name__ == '__main__':
    sys.exit(main())
