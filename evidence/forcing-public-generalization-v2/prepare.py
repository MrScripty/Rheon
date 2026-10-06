"""Install/restore frozen test overlays in this isolated checkout only."""
from pathlib import Path
import hashlib,json,subprocess,sys
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
BASE='e3b6858d2d29fc8837dbf2acd84db0c01784a352'
FILES={'src/fitted_height.rs':'FittedHeightWorkspace-overlay.rs','src/translated_viscous.rs':'TranslatedViscousFlow-overlay.rs','src/lib.rs':'library-overlay.rs'}
def require(ok,message):
 if not ok:raise ValueError(message)
def original(path):return subprocess.check_output(['git','show',BASE+':'+path],cwd=ROOT)
def overlay(path,name):return original(path)+(P/name).read_bytes() if path=='src/lib.rs' else (P/name).read_bytes()
mode=sys.argv[1];require(mode in ['install','restore'],'install/restore only')
for path,name in FILES.items():
 current=(ROOT/path).read_bytes();expected=original(path) if mode=='install' else overlay(path,name)
 require(current==expected,'only exact frozen overlay target '+path)
for path,name in FILES.items():(ROOT/path).write_bytes(overlay(path,name) if mode=='install' else original(path))
print(json.dumps({'event':'test_overlay_'+mode,'production_implementation_changed':False}))
