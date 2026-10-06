"""Bind actual scalar/affine/selector and fresh full-public memory before owners."""
from pathlib import Path
import hashlib,importlib.util,json,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
 if not ok:raise ValueError(msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
require(not(P/'preflight-receipt-v2.json').exists(),'never overwrite qualification')
b=json.loads((P/'binary-binding.json').read_text());require(sha(Path(b['binary']).read_bytes())==b['binary_sha256'],'same actual ELF');policy=json.loads((P/'protocol.json').read_text())
for name,digest in policy['source_sha256'].items():require(sha((ROOT/name).read_bytes())==digest,'frozen pure-E2 source')
commands=json.loads((P/'preflight-commands.json').read_text())['commands'];require(len(commands)==9 and all(c['exit']==0 for c in commands),'format/install/build/Clippy/four scalar tests/restore')
for c in commands:
 for channel in ['stdout','stderr']:require(sha((P/c[channel]).read_bytes())==c[channel+'_sha256'],'actual preflight logs')
scalar=load('public_scalar',P/'check_scalar.py').run(P/'scalar-native.log');affine=load('public_affine',P/'check_affine.py').run();memory=json.loads((P/'memory-v2-normal.json').read_text());require(memory['status']=='PASS_PURE_E2_MEMORY_PREFLIGHT' and memory['bound']<=67584,'fresh memory pass');require((P/'memory-v2-normal.json').read_bytes()==(P/'memory-v2-optimized.json').read_bytes(),'memory mode parity')
names=[c[channel] for c in commands for channel in ['stdout','stderr']]+['binary-binding.json','preflight-commands.json','scalar-oracle.json','memory-v2-normal.json','memory-v2-normal-stderr.log','memory-v2-optimized.json','memory-v2-optimized-stderr.log','memory-v2-commands.json','native-disassembly.txt.gz','native-disassembly.txt.sha256','native-symbols.txt','native-headers.txt','native-relocations.txt']
r={'status':'PASS_PURE_E2_SCALAR_AND_FULL_PUBLIC_MEMORY','source':b['prototype'],'source_tree':b['prototype_tree'],'binary_sha256':b['binary_sha256'],'protocol_sha256':sha((P/'protocol.json').read_bytes()),'scalar':scalar,'affine':affine,'memory_bound':memory['bound'],'cap':67584,'native_owner_advances':0,'new_accepted_snapshots':0,'artifact_sha256':{n:sha((P/n).read_bytes()) for n in sorted(set(names))}}
(P/'preflight-receipt-v2.json').write_text(json.dumps(r,indent=2,sort_keys=True)+'\n');print(json.dumps({'status':r['status'],'bound':memory['bound'],'remaining':67584-memory['bound'],'binary_sha256':b['binary_sha256'],'source':b['prototype']},indent=2,sort_keys=True))
