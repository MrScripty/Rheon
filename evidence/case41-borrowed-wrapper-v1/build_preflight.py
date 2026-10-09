"""Compile and run ONLY scalar/affine/layout guards; no numerical roster entry."""
from pathlib import Path
import gzip, hashlib, json, os, subprocess, sys, time
P = Path(__file__).resolve().parent
ROOT = P.parents[1]
OUT = Path(sys.argv[1]).resolve()
def require(ok, msg):
    if not ok:
        raise ValueError(msg)
def sha(data):
    return hashlib.sha256(data).hexdigest()
require(not (OUT / 'compile-binding.json').exists(), 'never overwrite previous compile')
OUT.mkdir(parents=True, exist_ok=True)
source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT).decode().strip()
require(not subprocess.check_output(['git','status','--porcelain'],cwd=ROOT), 'freeze clean source before compile')
env = dict(os.environ, CARGO_TARGET_DIR=str(OUT / 'target'), CARGO_BUILD_JOBS='1',
           RUSTFLAGS='--cfg rheon_newton_trace -C llvm-args=-stack-size-section -C link-arg=-Wl,--no-gc-sections', PYTHONDONTWRITEBYTECODE='1')
for key in list(env):
    if key.startswith('RHEON_'):
        env.pop(key)
commands = []
def run(label, argv):
    start = time.monotonic()
    r = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True)
    (OUT / (label + '.log')).write_bytes(r.stdout)
    (OUT / (label + '-stderr.log')).write_bytes(r.stderr)
    commands.append(dict(label=label, argv=argv, exit=r.returncode, seconds=time.monotonic()-start,
                         stdout_sha256=sha(r.stdout), stderr_sha256=sha(r.stderr),
                         numerical_enabling_variables_present=False))
    (OUT / 'commands.json').write_text(json.dumps(commands, indent=2, sort_keys=True)+'\n')
    require(r.returncode == 0, 'preserved failure: ' + label)
    return r.stdout
run('install', [sys.executable, str(P / 'prepare.py'), 'install'])
try:
    data = run('build', ['cargo','test','--release','--no-default-features','--lib','--locked','--no-run','--message-format=json'])
    rows = [json.loads(x) for x in data.splitlines() if x.startswith(b'{')]
    bins = [r['executable'] for r in rows if r.get('reason') == 'compiler-artifact' and r.get('executable') and r.get('profile',{}).get('test') and r['target']['name'] == 'rheon']
    require(len(bins) == 1, 'one dedicated test ELF')
    binary = bins[0]
    raw = subprocess.check_output(['objdump','-d','-C',binary])
    (OUT / 'native-disassembly.txt.gz').write_bytes(gzip.compress(raw,mtime=0))
    (OUT / 'native-disassembly.txt.sha256').write_text(sha(raw)+'\n')
    for label, argv in [('symbols',['nm','-S','-nC',binary]), ('relocations',['readelf','-rW',binary]), ('headers',['readelf','-hW',binary])]:
        (OUT / ('native-'+label+'.txt')).write_bytes(subprocess.check_output(argv))
    binding = dict(source=source, source_tree=tree, binary=binary, binary_sha256=sha(Path(binary).read_bytes()),
                   rustflags=env['RUSTFLAGS'], native_equations=0, source_inventory={str(f.relative_to(ROOT)):sha(f.read_bytes()) for f in sorted(P.rglob('*')) if f.is_file()})
    (OUT / 'compile-binding.json').write_text(json.dumps(binding,indent=2,sort_keys=True)+'\n')
    for label,test in [('scalar','research_public_scalar_preflight'), ('layout','research_public_type_layout'), ('affine','scalar_affine_preflight'), ('borrowed-layout','borrowed_wrapper_layout')]:
        run(label,[binary,'--exact','research_public_call::'+test,'--nocapture'])
finally:
    run('restore',[sys.executable,str(P/'prepare.py'),'restore'])
print(json.dumps(binding,indent=2,sort_keys=True))
