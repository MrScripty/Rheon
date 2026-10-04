"""Negative kernel-assumption audit probe; uses a temporary workspace file."""
from pathlib import Path
import subprocess,tempfile
ROOT=Path(__file__).resolve().parents[1]
audit=(ROOT/'AxiomAudit.lean').read_text()
probes=[
    (audit.replace('open Lean Elab Command', 'namespace Rheon\n'+declaration+'\nend Rheon\nopen Lean Elab Command'), 'Disallowed axiom')
    for declaration in ['axiom injected : False', 'theorem injected : False := by sorry']
]
# Existing non-Rheon declaration passes getConstInfo but must fail membership:
# generated Rheon proofs cannot compensate for an expected name being skipped.
probes.append((audit.replace('let expected : Array Name := #[',
                            'let expected : Array Name := #[`Nat.add_zero,'),
               'Expected declaration not audited: Nat.add_zero'))
for modified,expected_error in probes:
    with tempfile.NamedTemporaryFile(mode='w',suffix='.lean',dir=ROOT,delete=False) as f:
        f.write(modified); path=Path(f.name)
    try:
        r=subprocess.run(['lake','env','lean',str(path)],cwd=ROOT,text=True,capture_output=True)
        if r.returncode == 0 or expected_error not in r.stdout+r.stderr:
            raise RuntimeError('Audit negative probe failed: '+r.stdout+r.stderr)
    finally:path.unlink()
print('PASS audit rejects custom/admitted assumptions and skipped expected names')
