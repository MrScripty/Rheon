"""Read-only integrity/provenance checks. No native test or numerical invocation."""
from pathlib import Path
import ctypes
import gzip
import hashlib
import importlib.util
import json
import re
import subprocess

P = Path(__file__).resolve().parent
ROOT = P.parents[1]


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git_bytes(source, path):
    return subprocess.check_output(['git', 'show', source + ':' + path], cwd=ROOT)


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    inventory = P / 'packet-inventory.json'
    if inventory.exists():
        for path, digest in json.loads(inventory.read_text())['sha256'].items():
            require(sha((ROOT / path).read_bytes()) == digest, 'frozen result artifact ' + path)
    compiled = json.loads((P / 'compile-binding.json').read_text())
    receipt = json.loads((P / 'preflight-receipt.json').read_text())
    require(receipt['compiled_source'] == compiled['source'], 'preflight/ELF source binding')
    require(receipt['native_equations'] == 0 and receipt['FD_execution_allowed'] is False
            and receipt['complete_memory_bound'] is None, 'no numerical permission')
    require(receipt['native_test_invocations'] == 4 and receipt['resumed_without_native_rerun'], 'four exact guards once')
    require(subprocess.check_output(['git', 'rev-parse', compiled['source'] + '^{tree}'], cwd=ROOT).decode().strip()
            == compiled['source_tree'], 'compiled source tree')
    require(subprocess.check_output(['git', 'rev-parse', receipt['source'] + '^{tree}'], cwd=ROOT).decode().strip()
            == receipt['source_tree'], 'preflight source tree')
    policy = json.loads((P / 'protocol.json').read_text())
    for path, digest in policy['source_sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'compiled source remains frozen ' + path)
        require(sha(git_bytes(compiled['source'], path)) == digest, 'compiled Git source ' + path)
    prepolicy = json.loads((P / 'preflight-protocol.json').read_text())
    for path, digest in prepolicy['sha256'].items():
        require(sha(git_bytes(receipt['source'], path)) == digest, 'exact preflight Git input ' + path)
        if path != str(P.relative_to(ROOT) / 'preflight-commands.json'):
            require(sha((ROOT / path).read_bytes()) == digest, 'frozen preflight bytes ' + path)
    # The partial four-command manifest is a frozen input. Its completed output
    # appends only the two static readers; it does not replace any guard row.
    partial = json.loads(git_bytes(receipt['source'], 'evidence/case41-paired-observation-v1/preflight-commands.json'))
    commands = json.loads((P / 'preflight-commands.json').read_text())
    require(len(commands) == 6 and commands[:4] == partial, 'append-only preflight manifest')
    whitelist = ['research_public_scalar_preflight', 'research_public_type_layout',
                 'scalar_affine_preflight', 'case41_paired_layout_preflight']
    for command, test in zip(commands[:4], whitelist, strict=True):
        require(command['argv'] == [compiled['binary'], '--exact', 'research_public_call::' + test, '--nocapture'], 'closed scalar/layout roster')
    for command in commands:
        data = (P / command['stdout']).read_bytes()
        if command['stdout'].endswith('.gz'):
            data = gzip.decompress(data)
        require(sha(data) == command['stdout_sha256'] and command['exit'] == 0
                and not command['numerical_allow_variables_present'], 'actual argv/exit/output binding')
    for kind in ['candidate', 'reference']:
        old = ROOT / 'evidence/case41-scaled-fd-preparation-v1/native' / (kind + '.rs')
        new = P / 'native' / (kind + '.rs')
        marker = 'include!("' + kind + '_helpers.rs");'
        require(old.read_text().split(marker)[0] == new.read_text().split(marker)[0], 'actual equation/point/partition untouched ' + kind)
        for name in [kind + '_helpers.rs', kind + '_probe.rs']:
            require((P / 'native' / name).read_bytes() == (old.parent / name).read_bytes(), 'original helper bytes ' + name)
    for name in ['scalar.rs', 'fixed_inputs.rs', 'inputs.rs', 'trajectory_inputs.rs']:
        require((P / 'native' / name).read_bytes() == (ROOT / 'evidence/case41-scaled-fd-preparation-v1/native' / name).read_bytes(), 'original frozen scalar/input ' + name)
    old_fields = set(re.findall(r'\\"([a-z_]+)\\":', (ROOT / 'evidence/case41-scaled-fd-preparation-v1/fd_probe.rs').read_text()))
    new_fields = set(re.findall(r'\\"([a-z_]+)\\":', (P / 'paired_probe.rs').read_text()))
    require(old_fields <= new_fields | {'error'}, 'no required old observation field dropped')
    scalar = load('paired_verify_scalar', P / 'check_scalar.py').run(P / 'scalar-native.log')
    affine = load('paired_verify_affine', P / 'check_affine.py').run()
    require(scalar == receipt['scalar'] and affine == receipt['affine'], 'exact rational oracles')
    raw = gzip.decompress((P / 'native-disassembly.txt.gz').read_bytes())
    require(sha(raw) == (P / 'native-disassembly.txt.sha256').read_text().strip(), 'lossless ELF instructions')
    binary = compiled['binary']
    require(sha(Path(binary).read_bytes()) == compiled['binary_sha256'], 'actual frozen ELF')
    require(subprocess.check_output(['objdump', '-d', '-C', binary]) == raw, 'actual ELF disassembly')
    for name, argv in [('symbols', ['nm', '-S', '-nC', binary]),
                       ('relocations', ['readelf', '-rW', binary]), ('headers', ['readelf', '-hW', binary])]:
        require(subprocess.check_output(argv) == (P / ('native-' + name + '.txt')).read_bytes(), 'actual ELF metadata ' + name)
    memory_bytes = gzip.decompress((P / 'memory-normal.json.gz').read_bytes())
    require(memory_bytes == gzip.decompress((P / 'memory-optimized.json.gz').read_bytes())
            and sha(memory_bytes) == receipt['memory_ledger_sha256'], 'reader parity and receipt')
    memory = json.loads(memory_bytes)
    require(memory['status'] == 'BLOCKED_EXTERNAL_TRANSFER_ACCOUNTING'
            and memory['geometry_recursion_max_frames'] == 2
            and memory['serialization_outside_numerical_boundary'], 'fail-closed boundary and recursion claims')
    # Resolve the default libc IFUNC on this executor; no selector/CPU override.
    class DlInfo(ctypes.Structure):
        _fields_ = [('name', ctypes.c_char_p), ('base', ctypes.c_void_p),
                    ('symbol', ctypes.c_char_p), ('address', ctypes.c_void_p)]
    libc = ctypes.CDLL('libc.so.6')
    info = DlInfo()
    ptr = ctypes.cast(libc.memset, ctypes.c_void_p).value
    require(libc.dladdr(ctypes.c_void_p(ptr), ctypes.byref(info)) != 0, 'actual default libc binding')
    actual_offset = ptr - info.base
    archived = json.loads((ROOT / 'evidence/forcing-increment-comparison/memset-binding.json').read_text())
    require(actual_offset == archived['resolved_file_address'], 'same actual closed memset leaf selected')
    changed = subprocess.check_output(['git', 'diff', '--name-only', policy['parent']], cwd=ROOT).decode().splitlines()
    require(all(path.startswith('evidence/case41-paired-observation-v1/')
                or path == 'docs/research-book/implementation/case41-paired-observation-preflight.md' for path in changed), 'all old evidence and production retained')
    require(subprocess.check_output(['git', 'rev-parse', 'origin/main'], cwd=ROOT).decode().strip()
            == '9cd4587a54befa61bdfddc8e35014bd3c34f02fb', 'main unchanged')
    print(json.dumps(dict(status='PASS_READ_ONLY_INTEGRITY__NUMERICAL_STILL_BLOCKED',
                          compiled_source=compiled['source'], preflight_source=receipt['source'],
                          binary_sha256=compiled['binary_sha256'], native_equations=0,
                          native_test_reruns=0, actual_libc=info.name.decode(),
                          actual_memset_file_offset=actual_offset,
                          old_evidence_unchanged=True, production_unchanged=True), indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
