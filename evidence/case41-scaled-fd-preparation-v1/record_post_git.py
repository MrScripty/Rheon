"""Readonly binding of this preparation, preserved evidence and frozen worktrees."""
from pathlib import Path
import hashlib, json, subprocess, sys

P = Path(__file__).resolve().parent
ROOT = P.parents[1]
BASE = '998de18438a339d44896d24ecf9634ad469cdcdf'
BASE_TREE = '351f165a4bcc4aa6338e2be96d1e31698aec0b6e'
IDENTITY = 'MrScripty <TheEnvironmentGuy@protonmail.com>'
FROZEN = {
    'Rheon': 'f76841e9217e781d91c5bfeabadb6b4c6f42b20b',
    'Rheon-public-generalization': '676c6feaf906a965920f37792e07e2d4d623b3d2',
    'Rheon-public-call-66k': 'e3b6858d2d29fc8837dbf2acd84db0c01784a352',
    'Rheon-public-call': '4d60e9637431a8d9792944bcd9ff00979ead237e',
    'Rheon-increment-comparison': '1e90ed562f200327925c3f0b0255067aa413e0f1',
    'Rheon-increment-contract': '704842c368953e47df279728935e20cfda89f0c5',
    'Rheon-forcing-arithmetic': 'f0b81b4a5f76cb706e645fad4f38faf028c5727e',
    'Rheon-finer-refusals': 'dad53b4054034fe0c2ff6240464df7441ab4e6a9',
    'Rheon-column-final-repair': 'be99b8962ba9c20d9e0a99218a1d2f62f120da45',
    'Rheon-oracle-repair': 'ec26e48df958f0ebc60af414253ff98c0007dc13',
    'Rheon-terminal-main': '49c47b9a344ec277e2eaa0a5315b27a51ed2f17c',
    'Rheon-e1-trajectories': '008fdd02ca41e88821bfa8999283347bab3455eb',
    'Rheon-e1-diagnostics': 'b46d26c8fe06f9c331c5cee4e06af6e34fa9190c',
    'Rheon-affine-rounding-contract': 'c210d9ff630504748fad129337aaa84e013522a6',
    'Rheon-e2-fixed-candidates': '2f23503d9bf74842e39f9fe0760edf4bfeb41b4c',
    'Rheon-e2-followup': BASE,
}

def require(ok, message):
    if not ok:
        raise ValueError(message)

def git(*args, cwd=ROOT):
    return subprocess.check_output(['git', *args], cwd=cwd).decode().strip()

def sha(data):
    return hashlib.sha256(data).hexdigest()

def inventory(commit):
    count = 0
    digest = hashlib.sha256()
    for entry in subprocess.check_output(['git', 'ls-tree', '-rz', commit], cwd=ROOT).split(b'\0'):
        if not entry:
            continue
        meta, name = entry.split(b'\t', 1)
        mode, kind, oid = meta.split()
        require(mode in [b'100644', b'100755'] and kind == b'blob', 'regular source/evidence file')
        data = (ROOT / name.decode()).read_bytes()
        require(hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest().encode() == oid,
                'exact tree byte preservation: ' + name.decode())
        digest.update(name + b'\0' + hashlib.sha256(data).digest())
        count += 1
    return {'files': count, 'path_content_sha256': digest.hexdigest()}

def run(result):
    require(git('rev-parse', BASE + '^{tree}') == BASE_TREE, 'base tree identity')
    old = inventory(BASE)
    require(old['files'] == 8058, 'all inherited files retained')
    receipt = json.loads((P / 'receipt.json').read_text())
    for path, digest in receipt['artifact_sha256'].items():
        require(sha((P / path).read_bytes()) == digest, 'preparation receipt: ' + path)
    normal = (P / 'verification-normal.json').read_bytes()
    require(normal == (P / 'verification-optimized.json').read_bytes(), 'verification parity')
    checks = json.loads(normal)
    require(checks['preserved_base'] == old and checks['native_equations_executed'] ==
            checks['native_test_ELF_invocations'] == checks['new_corrections'] == checks['owner_advances'] == 0,
            'zero execution and inherited bytes')
    binding = json.loads((P / 'compile-binding.json').read_text())
    require(sha(Path(binding['binary']).read_bytes()) == binding['binary_sha256'], 'same unexecuted compiled ELF')
    identities = {}
    for commit in [binding['source'], receipt['reader_amendment_source'], result]:
        ident = git('show', '-s', '--format=%an <%ae>%n%cn <%ce>', commit).splitlines()
        require(ident == [IDENTITY, IDENTITY], 'author and committer identity: ' + commit)
        identities[commit] = ident
    frozen = {}
    for name, expected in FROZEN.items():
        cwd = Path('/workspace') / name
        head = git('rev-parse', 'HEAD', cwd=cwd)
        status = git('status', '--porcelain', cwd=cwd)
        require(head == expected and not status, 'frozen worktree unchanged: ' + name)
        frozen[name] = {'head': head, 'status': status}
    return {
        'status': 'PASS_PUBLISHED_COMPILE_ONLY_PREPARATION',
        'base': BASE, 'base_tree': BASE_TREE, 'preserved_base': old,
        'result_commit': result, 'result_tree': git('rev-parse', result + '^{tree}'),
        'result_inventory': inventory(result), 'receipt_sha256': sha((P / 'receipt.json').read_bytes()),
        'protocol_sha256': sha((P / 'protocol.json').read_bytes()),
        'binary_sha256': binding['binary_sha256'], 'identities': identities,
        'frozen_worktrees': frozen, 'native_ELF_invocations': 0,
        'native_equations': 0, 'new_corrections': 0, 'owner_advances': 0,
        'FD_capture_authorized': False, 'pure_E2_jobs_authorized': False,
        'scalar_runtime_preflight_pending': True, 'FD_memory_preflight_pending': True,
        'all_original_failures_retained': True,
    }

if __name__ == '__main__':
    print(json.dumps(run(sys.argv[1]), indent=2, sort_keys=True))
