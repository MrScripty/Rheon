"""Negative kernel-assumption audit probe; uses a temporary workspace file."""
from pathlib import Path
import subprocess,tempfile
ROOT=Path(__file__).resolve().parents[1]
audit=(ROOT/'AxiomAudit.lean').read_text()
for declaration in ['axiom injected : False', 'theorem injected : False := by sorry']:
    modified=audit.replace('open Lean Elab Command', 'namespace Rheon\n'+declaration+'\nend Rheon\nopen Lean Elab Command')
    with tempfile.NamedTemporaryFile(mode='w',suffix='.lean',dir=ROOT,delete=False) as f:
        f.write(modified); path=Path(f.name)
    try:
        r=subprocess.run(['lake','env','lean',str(path)],cwd=ROOT,text=True,capture_output=True)
        assert r.returncode != 0 and 'Disallowed axiom' in r.stdout+r.stderr,r.stdout+r.stderr
    finally:path.unlink()
print('PASS audit rejects custom and admitted assumptions')
