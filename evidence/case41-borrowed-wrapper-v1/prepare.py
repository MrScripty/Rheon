"""Local test overlay only; verify every production byte before either operation."""
from pathlib import Path
import subprocess, sys
P = Path(__file__).resolve().parent
ROOT = P.parents[1]
BASE = '998de18438a339d44896d24ecf9634ad469cdcdf'
FILES = {'src/fitted_height.rs': 'FittedHeightWorkspace-overlay.rs',
         'src/translated_viscous.rs': 'TranslatedViscousFlow-overlay.rs',
         'src/lib.rs': 'library-overlay.rs'}
def require(ok, msg):
    if not ok:
        raise ValueError(msg)
def original(path):
    return subprocess.check_output(['git', 'show', BASE + ':' + path], cwd=ROOT)
def overlay(path, name):
    data = (P / 'native' / name).read_bytes()
    return original(path) + data if path == 'src/lib.rs' else data
mode = sys.argv[1]
require(mode in ('install', 'restore'), 'install/restore only')
for path, name in FILES.items():
    require((ROOT / path).read_bytes() == (original(path) if mode == 'install' else overlay(path, name)), 'exact overlay target ' + path)
for path, name in FILES.items():
    (ROOT / path).write_bytes(overlay(path, name) if mode == 'install' else original(path))
