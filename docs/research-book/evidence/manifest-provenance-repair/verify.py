#!/usr/bin/env python3
"""Bind this isolated gate repair and exact preservation evidence."""
import hashlib
import json
from pathlib import Path
import subprocess

BASE = 'a97f1012f5570b4ed938a6ccead2f591313ecf67'
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


source = git('log', '-1', '--format=%H', '--', *SOURCES)
require(git('diff', BASE, '--', *PROTECTED) == '', 'Protected dependency/scientific source changed')
changes = git('diff', '--name-only', BASE).splitlines()
require(all(name in SOURCES or name.startswith(str(ROOT) + '/') for name in changes),
        'Unrelated work changed')
hashes = {}
for name in SOURCES:
    raw = Path(name).read_bytes()
    require(raw == subprocess.check_output(['git', 'show', f'{source}:{name}']), 'Uncommitted source: '+name)
    hashes[name] = hashlib.sha256(raw).hexdigest()
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
result = {'reviewed_base': BASE, 'repair_source_commit': source,
          'repair_source_tree': git('rev-parse', source+'^{tree}'), 'source_sha256': hashes,
          'protected_paths_unchanged': list(PROTECTED), 'mathlib_manifest_unchanged_identity': mathlib,
          'source_gate_tests_normal_and_optimized': 4, 'bootstrap_tests': 2,
          'comparator_tests_normal_and_optimized': 12, 'original_rejection_failures': 22,
          'kernel_or_dependency_changes': False, 'new_native_lean_qualification_claimed': False,
          'parent_independent_review': 'pending', 'pr1_advanced': False, 'bot_review_requested': False}
(ROOT / 'receipt.json').write_text(json.dumps(result, indent=2)+'\n')
print('PASS exact repaired sources, unchanged dependencies/proofs/fixtures/workflows, normal/optimized gates')
print('PENDING parent independent review before advancing frozen PR1 a97f1012')
