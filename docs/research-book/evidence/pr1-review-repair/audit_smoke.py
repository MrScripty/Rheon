#!/usr/bin/env python3
"""Compile the exact auditor with Lean core fixture declarations, not Mathlib.

Usage from repository root: python3 PATH_TO_THIS_SCRIPT LEAN_BINARY
This tests auditor syntax/behavior and its negative Python driver. It cannot
qualify the actual Rheon theorems, which still need the full pinned project gate.
"""
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

lean = str(Path(sys.argv[1]).resolve(strict=True))
audit = Path('proofs/AxiomAudit.lean').read_text()
names = re.findall(r'`(Rheon\.[A-Za-z_.]+)', audit.split('for name in expected')[0])


def run(command, root, environment, *, accepted, error=None):
    result = subprocess.run(command, cwd=root, env=environment, text=True,
                            capture_output=True, timeout=30)
    print(result.stdout + result.stderr, end='')
    if (result.returncode == 0) != accepted or (error is not None and error not in result.stdout + result.stderr):
        raise RuntimeError('Unexpected fixture auditor outcome: ' + repr(command))


with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    definitions = ['import Lean']
    for name in names:
        namespace, identifier = name.rsplit('.', 1)
        definitions.append(f'namespace {namespace}\ndef {identifier} : Nat := 0\nend {namespace}')
    # More fixture proofs ensure count-only completeness can miss an expected
    # name without failing the old threshold. These are not production proofs.
    definitions.extend(f'theorem Rheon.fixture_{i} : True := by trivial' for i in range(8))
    (root / 'Rheon.lean').write_text('\n'.join(definitions) + '\n')
    environment = os.environ | {'LEAN_PATH': str(root)}
    run([lean, '-o', str(root / 'Rheon.olean'), str(root / 'Rheon.lean')], root, environment, accepted=True)
    (root / 'AxiomAudit.lean').write_text(audit)
    run([lean, 'AxiomAudit.lean'], root, environment, accepted=True)
    old = subprocess.check_output(['git', 'show', '50ff0b2218cd62e3023fa57eecd2a2411486e039:proofs/AxiomAudit.lean'], text=True)
    (root / 'Before.lean').write_text(old.replace('let expected : Array Name := #[',
                                                'let expected : Array Name := #[`Nat.add_zero,'))
    run([lean, 'Before.lean'], root, environment, accepted=True)
    print('REPRODUCED old count-only auditor accepts a skipped expected name')
    scripts = root / 'scripts'
    scripts.mkdir()
    (scripts / 'test_audit.py').write_bytes(Path('proofs/scripts/test_audit.py').read_bytes())
    # Only command routing is stubbed: every audit/probe uses the real compiler.
    router = root / 'lake'
    router.write_text('#!/bin/bash\nexec "$AUDIT_LEAN_BINARY" "$3"\n')
    router.chmod(0o700)
    environment |= {'PATH': str(root) + os.pathsep + os.environ['PATH'], 'AUDIT_LEAN_BINARY': lean}
    for optimized in (False, True):
        run([sys.executable, *(['-O'] if optimized else []), str(scripts / 'test_audit.py')],
            root, environment, accepted=True)
    print('PASS exact auditor compiles; negative driver rejects axiom, sorry and missing membership in normal/optimized Python')
    print('LIMIT: Lean core fixture audit only; no actual Rheon/Mathlib theorem qualification')
