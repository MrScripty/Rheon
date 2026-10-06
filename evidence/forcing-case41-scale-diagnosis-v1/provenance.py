"""Pin searched local threshold provenance; absence is bounded to this corpus."""
from pathlib import Path
import hashlib,json,re,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
paths=sorted(set([ROOT/'src/coupled_discrete.rs']+list((ROOT/'docs/research-book').rglob('*.md'))+list(ROOT.glob('**/*.lean'))))
paths=[p for p in paths if '.git/'not in str(p)and 'target/'not in str(p)]
pattern=re.compile(r'NEWTON|1e-13|1e−13|10\^\{-13\}|Newton.*target|Newton.*threshold|10⁻¹³',re.I)
rows=[]
for p in paths:
 data=p.read_bytes();matches=[{'line':i,'text':line}for i,line in enumerate(data.decode().splitlines(),1)if pattern.search(line)];rows.append({'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(data).hexdigest(),'matches':matches})
print(json.dumps({'status':'PINNED_LOCAL_PROVENANCE_SEARCH','base':'2f23503d9bf74842e39f9fe0760edf4bfeb41b4c','corpus':rows,'introduction_commit':subprocess.check_output(['git','log','--format=%H','-S','const NEWTON','--','src/coupled_discrete.rs'],cwd=ROOT).decode().splitlines(),'finding':'The pinned owner and implementation documents specify an operational 1e-13 rate target stricter than 1e-11 physical gates. This search finds no state-specific physical/error-bound derivation for this value. This is an absence finding limited to listed source/book/Lean files, not a claim about all possible research. Lean real algebra and floating-point upper error discussions do not establish an arithmetic lower floor.'},indent=2,sort_keys=True))
