"""Compile frozen paired wrappers in an isolated worktree; never run equations."""
from pathlib import Path
import gzip
import hashlib
import json
import os
import subprocess
import time

P = Path(__file__).resolve().parent
ROOT = P.parents[1]


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    require(not (P / 'compile-binding.json').exists(), 'never overwrite a compile result')
    policy = json.loads((P / 'protocol.json').read_text())
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT).decode().strip()
    for path, digest in policy['source_sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'frozen source ' + path)
    scratch = Path(policy['build_worktree'])
    require(not scratch.exists(), 'separate new build worktree')
    subprocess.run(['git', 'worktree', 'add', '--detach', str(scratch), source], cwd=ROOT, check=True)
    env = dict(os.environ, CARGO_TARGET_DIR=policy['target'], CARGO_BUILD_JOBS='1',
               CARGO_HOME='/workspace/.rheon-tools/cargo', RUSTUP_HOME='/workspace/.rheon-tools/rustup',
               PATH='/workspace/.rheon-tools/cargo/bin:' + os.environ.get('PATH',''),
               RUSTFLAGS=policy['rustflags'], PYTHONDONTWRITEBYTECODE='1')
    for key in list(env):
        if key.startswith('RHEON_'):
            env.pop(key)
    commands = []

    def run(label, argv):
        start = time.monotonic()
        r = subprocess.run(argv, cwd=scratch, env=env, capture_output=True)
        (P / (label + '.log')).write_bytes(r.stdout)
        (P / (label + '-stderr.log')).write_bytes(r.stderr)
        commands.append(dict(argv=argv, exit=r.returncode, seconds=time.monotonic() - start,
                             stdout_sha256=sha(r.stdout), stderr_sha256=sha(r.stderr),
                             native_test_invocation=False))
        (P / 'compile-commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        require(r.returncode == 0, 'compile-only failure ' + label)
        return r.stdout

    native = scratch / 'evidence/case41-baseline-forecast-preparation-v1/native'
    run('format', ['rustfmt', '--edition', '2024', '--check', '--config', 'skip_children=true',
                   str(native.parent / 'paired_probe.rs'), str(native.parent / 'forecast_probe.rs'),
                   *[str(x) for x in sorted(native.glob('*.rs')) if 'overlay' not in x.name]])
    originals = {}
    overlays = {'src/fitted_height.rs': 'FittedHeightWorkspace-overlay.rs',
                'src/translated_viscous.rs': 'TranslatedViscousFlow-overlay.rs',
                'src/lib.rs': 'library-overlay.rs'}
    for path, name in overlays.items():
        original = (scratch / path).read_bytes()
        frozen = subprocess.check_output(['git', 'show', policy['production_basis'] + ':' + path], cwd=ROOT)
        require(original == frozen, 'exact production basis ' + path)
        originals[path] = original
        overlay = (native / name).read_bytes()
        (scratch / path).write_bytes(original + overlay if path == 'src/lib.rs' else overlay)
    try:
        data = run('build', ['cargo', 'test', '--release', '--no-default-features', '--lib',
                             '--locked', '--no-run', '--message-format=json'])
        rows = [json.loads(x) for x in data.splitlines() if x.startswith(b'{')]
        binaries = [r['executable'] for r in rows if r.get('reason') == 'compiler-artifact'
                    and r.get('executable') and r.get('profile', {}).get('test')
                    and r['target']['name'] == 'rheon']
        require(len(binaries) == 1, 'exact dedicated test ELF')
        binary = binaries[0]
        run('clippy', ['cargo', 'clippy', '--release', '--no-default-features', '--lib', '--tests',
                        '--locked', '--', '-D', 'warnings'])
        raw = subprocess.check_output(['objdump', '-d', '-C', binary])
        (P / 'native-disassembly.txt.gz').write_bytes(gzip.compress(raw, mtime=0))
        (P / 'native-disassembly.txt.sha256').write_text(sha(raw) + '\n')
        for name, argv in [('symbols', ['nm', '-S', '-nC', binary]),
                           ('relocations', ['readelf', '-rW', binary]),
                           ('headers', ['readelf', '-hW', binary])]:
            (P / ('native-' + name + '.txt')).write_bytes(subprocess.check_output(argv))
        result = dict(status='PASS_COMPILE_ONLY', source=source, source_tree=tree,
                      binary=binary, binary_sha256=sha(Path(binary).read_bytes()),
                      protocol_sha256=sha((P / 'protocol.json').read_bytes()),
                      native_test_invocations=0, native_equations=0, owner_advances=0,
                      observations_execution_allowed=False)
        (P / 'compile-binding.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result, indent=2))
    finally:
        for path, original in originals.items():
            (scratch / path).write_bytes(original)
        (P / 'restore.json').write_text(json.dumps(dict(production_bytes_restored=True)) + '\n')


if __name__ == '__main__':
    main()
