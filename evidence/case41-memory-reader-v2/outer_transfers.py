"""Expose every outer-test call/tail; unresolved costs keep certificate blocked."""
from pathlib import Path
import hashlib,importlib.util,json,re,subprocess
P=Path(__file__).resolve().parent;F=Path('/workspace/Rheon-fd-pure-e2');D=F/'evidence/case41-scaled-fd-preparation-v1'
spec=importlib.util.spec_from_file_location('frames_outer',F/'evidence/forcing-increment-memory-audit/audit.py');h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
b=json.loads((F/'evidence/case41-six-fd-columns-v1/compiled-input.json').read_text())
h.require(hashlib.sha256(Path(b['binary']).read_bytes()).hexdigest()==b['binary_sha256'],'same frozen ELF')
e=json.loads((P/'extraction.json').read_text());part=(P/'outer-test.asm.txt').read_text();h.require(hashlib.sha256(part.encode()).hexdigest()==e['files']['outer-test.asm.txt']['sha256'],'published full outer excerpt')
raw=subprocess.check_output(['objdump','-d','-C',b['binary']]).decode();h.require(part in raw,'actual linked outer instructions')
symbols=(D/'native-symbols.txt').read_text();f=next(iter(h.functions(part,symbols).values()));rows=[r for r in h.transfers(f) if r['kind']!='internal_branch']
print(json.dumps(dict(status='READONLY_OUTER_TRANSFER_SUPPLEMENT__NO_COMPLETE_BOUND',binary_sha256=b['binary_sha256'],outer_symbol=f['name'],outer_frame=h.fixed_frame(f),transfers=rows,native_equations=0,owner_advances=0,complete_memory_bound=None,scope='The main transfer ledger starts at capture roots. These additional outer initialization/test/panic transfers are retained separately and require boundary/cost justification too.'),indent=2,sort_keys=True))
