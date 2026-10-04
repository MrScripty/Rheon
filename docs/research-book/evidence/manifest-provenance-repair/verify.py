#!/usr/bin/env python3
"""Verify the fixed historical repair record and protected checkout paths.

This does not run tests or qualify the current source revision. It never updates
the archived receipt: later qualification belongs in separate evidence.
"""
import hashlib
import json
from pathlib import Path
import subprocess

BASE = 'a97f1012f5570b4ed938a6ccead2f591313ecf67'
SOURCE = 'd4147b311ebfd1b05919251dab703a628eb32316'
ARCHIVE = 'd72db7c1bd3fc10b27978c6dcc8ec10709195ace'
ROOT = Path('docs/research-book/evidence/manifest-provenance-repair')
SOURCES = ('proofs/scripts/check_sources.py', 'proofs/scripts/test_check_sources.py')
PROTECTED = ('proofs/Rheon', 'proofs/Rheon.lean', 'proofs/AxiomAudit.lean',
             'proofs/source-inventory.json', 'proofs/lean-toolchain', 'proofs/lakefile.toml',
             'proofs/lake-manifest.json', '.github', 'docs/research-book/companion',
             'docs/research-book/chapters', 'docs/research-book/appendices',
             'docs/research-book/artwork', 'docs/research-book/figures')


def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()


def require(condition, message):
    if not condition:
        raise ValueError(message)


require(git('diff', BASE, '--', *PROTECTED) == '', 'Protected dependency/scientific source changed')
require(git('ls-files', '--others', '--', *PROTECTED) == '',
        'Untracked or ignored protected path added')
changes = git('diff', '--name-only', BASE).splitlines()
require(all(name in SOURCES or name.startswith(str(ROOT) + '/') for name in changes),
        'Unrelated work changed')
archive_files = ('receipt.json', 'tests.log', 'tests-optimized.log',
                 'comparator-tests.log', 'comparator-tests-optimized.log',
                 'gate.log', 'gate-optimized.log', 'bootstrap-tests.log', 'before.log')
for name in archive_files:
    path = ROOT / name
    require(path.read_bytes() == subprocess.check_output(['git', 'show', f'{ARCHIVE}:{path}']),
            'Historical evidence changed: '+name)
receipt = json.loads((ROOT / 'receipt.json').read_text())
require(receipt['reviewed_base'] == BASE and receipt['repair_source_commit'] == SOURCE
        and receipt['repair_source_tree'] == git('rev-parse', SOURCE+'^{tree}'),
        'Historical source identity differs')
hashes = {name: hashlib.sha256(subprocess.check_output(
    ['git', 'show', f'{SOURCE}:{name}'])).hexdigest() for name in SOURCES}
require(receipt['source_sha256'] == hashes, 'Historical source hashes differ')
for name in ('tests.log', 'tests-optimized.log'):
    log = (ROOT / name).read_text()
    require('Ran 4 tests' in log and log.rstrip().endswith('OK'), 'Gate suite failed: '+name)
for name in ('comparator-tests.log', 'comparator-tests-optimized.log'):
    log = (ROOT / name).read_text()
    require('Ran 12 tests' in log and log.rstrip().endswith('OK'), 'Comparator suite failed: '+name)
for name in ('gate.log', 'gate-optimized.log'):
    require('PASS exact reviewed' in (ROOT / name).read_text(), 'Reviewed manifest rejected: '+name)
bootstrap = (ROOT / 'bootstrap-tests.log').read_text()
require('Ran 2 tests' in bootstrap and bootstrap.rstrip().endswith('OK'), 'Bootstrap checks failed')
require('FAILED (failures=22)' in (ROOT / 'before.log').read_text(), 'Original gate reproduction missing')
manifest = json.loads(Path('proofs/lake-manifest.json').read_text())
mathlib = next(p for p in manifest['packages'] if p['name'] == 'mathlib')
require(receipt['protected_paths_unchanged'] == list(PROTECTED)
        and receipt['mathlib_manifest_unchanged_identity'] == mathlib,
        'Historical preservation identity differs')
print('PASS archived source '+SOURCE+' and evidence '+ARCHIVE)
print('PASS protected tracked paths unchanged; no untracked or ignored additions')
print('Historical log/source association only; no current-source test qualification')
