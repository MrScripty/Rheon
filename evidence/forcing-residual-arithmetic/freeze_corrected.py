"""Add corrected receipt; retain the complete first failed attempt unchanged."""
from pathlib import Path
import hashlib,json
P=Path(__file__).resolve().parent
if(P/'receipt-corrected.json').exists():raise ValueError('refuse overwrite corrected receipt')
r=json.loads((P/'receipt.json').read_text());bad=[name for name,d in r['file_sha256'].items()if hashlib.sha256((P/name).read_bytes()).hexdigest()!=d]
# README now contains the transparent additive correction note; the original
# receipt, native/analysis sources and all numerical captures remain unchanged.
if set(bad)!={'freeze.log','README.md'}:raise ValueError('unexpected first receipt change '+repr(bad))
r['first_failed_receipt']={'file':'receipt.json','sha256':hashlib.sha256((P/'receipt.json').read_bytes()).hexdigest(),'cause':'producer log was hashed while still in progress; numerical evidence unchanged'}
r['file_sha256']={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(P.rglob('*'))if p.is_file()and '__pycache__'not in p.parts and p.name!='receipt-corrected.json'}
(P/'receipt-corrected.json').write_text(json.dumps(r,indent=2)+'\n')
print('Corrected frozen receipt SHA256 '+hashlib.sha256((P/'receipt-corrected.json').read_bytes()).hexdigest())
