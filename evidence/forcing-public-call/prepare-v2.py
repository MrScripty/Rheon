"""Install/restore only the frozen test overlays in this isolated checkout."""
from pathlib import Path
import hashlib,json,sys
P=Path(__file__).resolve().parent
ROOT=P.parents[1]
BASE='1e90ed562f200327925c3f0b0255067aa413e0f1'
def sha(data):return hashlib.sha256(data).hexdigest()
binding=json.loads((P/'clone-binding.json').read_text())
lib=(ROOT/'src/lib.rs')
original_lib=(ROOT/'evidence/forcing-increment-comparison/baseline-binding.json')
lib_sha=json.loads(original_lib.read_text())['production_sha256']['src/lib.rs']
if sys.argv[1]=='install':
 if sha(lib.read_bytes())!=lib_sha:raise ValueError('unmodified original lib required')
 for item in binding:
  if sha((ROOT/item['path']).read_bytes())!=item['original_sha256']:raise ValueError('unmodified clone overlay target')
 for item in binding:(ROOT/item['path']).write_bytes((ROOT/item['overlay']).read_bytes())
 lib.write_bytes(lib.read_bytes()+(P/'library-overlay-v2.rs').read_bytes())
elif sys.argv[1]=='restore':
 import subprocess
 for path in ['src/lib.rs',*[item['path'] for item in binding]]:
  original=subprocess.check_output(['git','show',BASE+':'+path],cwd=ROOT)
  if path=='src/lib.rs':expected=original+(P/'library-overlay-v2.rs').read_bytes()
  else:expected=(ROOT/next(i['overlay'] for i in binding if i['path']==path)).read_bytes()
  current=(ROOT/path).read_bytes()
  if current not in (original,expected):raise ValueError('refuse restoring an unrelated edit')
  (ROOT/path).write_bytes(original)
else:raise ValueError('install or restore only')
print(json.dumps({'event':'test_overlays_'+sys.argv[1],'production_implementation_changed':False}))
