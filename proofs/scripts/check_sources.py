"""Pin and exact-source inventory gate. Kernel/axiom audit is separate."""
from pathlib import Path
import hashlib,json,re,tomllib
ROOT=Path(__file__).resolve().parents[1]
def require(condition,message):
    if not condition: raise ValueError(message)
def check():
    pins=json.loads((ROOT/'source-inventory.json').read_text())
    actual={str(p.relative_to(ROOT)) for p in ROOT.rglob('*.lean') if '.lake' not in p.parts}
    require(actual==set(pins), 'unreviewed or missing Lean file')
    for name,digest in pins.items():
        require(hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest, name)
    require((ROOT/'lean-toolchain').read_text().strip()=='leanprover/lean4:v4.19.0', 'Lean toolchain pin differs')
    cfg=tomllib.loads((ROOT/'lakefile.toml').read_text())
    rev='c44e0c8ee63ca166450922a373c7409c5d26b00b'
    require(cfg['require']==[{'name':'mathlib','git':'https://github.com/leanprover-community/mathlib4.git','rev':rev}], 'mathlib requirement differs')
    manifest=json.loads((ROOT/'lake-manifest.json').read_text())
    require(next(p for p in manifest['packages'] if p['name']=='mathlib')['rev']==rev, 'mathlib manifest revision differs')
    require(all(re.fullmatch('[0-9a-f]{40}',p['rev']) for p in manifest['packages']), 'dependency revision is not a full commit SHA')
    print('PASS exact reviewed Lean source inventory and dependency pins')
if __name__=='__main__':check()
