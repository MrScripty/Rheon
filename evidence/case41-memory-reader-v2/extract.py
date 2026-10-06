"""Readable archived-binary evidence only; no rebuild or test execution."""
from pathlib import Path
import gzip, hashlib, json, re, subprocess
P=Path(__file__).resolve().parent; F=Path('/workspace/Rheon-fd-pure-e2');D=F/'evidence/case41-scaled-fd-preparation-v1'
b=json.loads((F/'evidence/case41-six-fd-columns-v1/compiled-input.json').read_text())
def require(x,msg):
 if not x:raise ValueError(msg)
def sha(x):return hashlib.sha256(x).hexdigest()
require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'actual frozen ELF')
raw=subprocess.check_output(['objdump','-d','-C',b['binary']]);require(raw==gzip.decompress((D/'native-disassembly.txt.gz').read_bytes()),'exact archived instructions')
require(not(P/'extraction.json').exists(),'run extraction once')
parts=re.split(r'(?m)(?=^[a-f0-9]+ <)',raw.decode());names={}
for part in parts:
 m=re.match(r'^([a-f0-9]+) <(.+)>:',part)
 if m:names[int(m[1],16)]=(m[2],part)
selected={'outer-test':0x128ce0,'candidate-capture':0x12a050,'candidate-observer':0x140080,'reference-observer':0x17db00,'candidate-point':0x1566a0,'candidate-partition':0x166c30,'candidate-array-helper-1':0x1b3710,'candidate-array-helper-2':0x1b3390,'candidate-sort-helper':0x1ceb40}
# Discover the outer symbol rather than assuming a compiler address.
selected['outer-test']=next(a for a,(n,s) in names.items() if n.endswith('::case41_six_columns_only'))
files={}
for key,a in selected.items():
 name,part=names[a];path=P/(key+'.asm.txt');path.write_text(part);files[path.name]=dict(address=hex(a),symbol=name,sha256=sha(path.read_bytes()))
(P/'extraction.json').write_text(json.dumps(dict(binary=b['binary'],binary_sha256=b['binary_sha256'],compiled_source=b['source'],compiled_source_tree=b['source_tree'],objdump_argv=['objdump','-d','-C',b['binary']],matches_original_archive=True,files=files,native_equations=0,owner_advances=0,build_invocations=0),indent=2,sort_keys=True)+'\n')
print('PASS_READABLE_ARCHIVED_BINARY_EXTRACTION')
