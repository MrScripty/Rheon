#!/usr/bin/env python3
"""Bind repaired sources and preservation checks; never rerun or invent gates."""
import hashlib
import json
from pathlib import Path
import subprocess

BASE = '50ff0b2218cd62e3023fa57eecd2a2411486e039'
ROOT = Path('docs/research-book/evidence/pr1-review-repair')
SOURCES = ('.github/workflows/lean-rheon.yml', '.github/workflows/research-numerics.yml',
           'docs/research-book/appendices/B-reproduction.md',
           'docs/research-book/chapters/10-forces-and-viscosity.md',
           'proofs/AxiomAudit.lean', 'proofs/scripts/check_sources.py',
           'proofs/scripts/test_audit.py', 'proofs/scripts/test_bootstrap.py',
           'proofs/scripts/test_check_sources.py', 'proofs/source-inventory.json')
PROTECTED = ('docs/research-book/companion', 'docs/research-book/artwork',
             'docs/research-book/figures', 'docs/research-book/evidence/cg-reproduction',
             'docs/research-book/evidence/proof-qualification.json',
             'proofs/Rheon', 'proofs/Rheon.lean', 'proofs/lean-toolchain',
             'proofs/lakefile.toml', 'proofs/lake-manifest.json')


def git(*args):
    return subprocess.check_output(['git', *args], text=True).strip()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


source = git('log', '-1', '--format=%H', '--', *SOURCES)
require(git('diff', BASE, '--', *PROTECTED) == '', 'Protected scientific/proof source changed')
hashes = {}
for name in SOURCES:
    require(Path(name).read_bytes() == subprocess.check_output(['git', 'show', f'{source}:{name}']),
            f'Uncommitted repaired source: {name}')
    hashes[name] = sha256(Path(name))
old_inventory = json.loads(subprocess.check_output(['git', 'show', f'{BASE}:proofs/source-inventory.json']))
inventory = json.loads(Path('proofs/source-inventory.json').read_text())
require(old_inventory.keys() == inventory.keys(), 'Proof inventory membership changed')
require(all(old_inventory[k] == inventory[k] for k in inventory if k != 'AxiomAudit.lean'),
        'Theorem source inventory changed')
require(inventory['AxiomAudit.lean'] == hashes['proofs/AxiomAudit.lean'], 'Auditor inventory digest differs')
provenance = json.loads((ROOT / 'elan-provenance.json').read_text())
require(provenance['sha256'] in Path('.github/workflows/lean-rheon.yml').read_text(), 'Elan pin differs')
result = {'reviewed_head': BASE, 'repair_source_commit': source,
          'repair_source_tree': git('rev-parse', f'{source}^{{tree}}'),
          'source_sha256': hashes, 'protected_paths_unchanged': list(PROTECTED),
          'only_changed_lean_inventory_entry': 'AxiomAudit.lean',
          'audit_inventory_before': old_inventory['AxiomAudit.lean'],
          'audit_inventory_after': inventory['AxiomAudit.lean'],
          'elan_provenance': provenance,
          'full_actual_lean_project_gate': 'blocked: pinned mathlib cache HTTP 403',
          'auditor_gate_scope': 'Exact auditor compiles against core-only fixture; real negative driver passes normal/optimized Python; no actual theorem qualification claimed',
          'parent_action': 'Arrange independent dot review and full pinned Lean project qualification before integration',
          'external_review_requested': False, 'merge_performed': False}
(ROOT / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
print('PASS repaired source identities, exact scientific/theorem preservation and auditor-only inventory update')
print('PENDING full actual Lean project qualification: cache HTTP 403; parent independent review')
