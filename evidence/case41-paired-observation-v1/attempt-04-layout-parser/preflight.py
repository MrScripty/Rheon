"""One scalar/layout-only preflight. There is deliberately no numerical runner."""
from pathlib import Path
import gzip
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import time

P = Path(__file__).resolve().parent
ROOT = P.parents[1]


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    require(not (P / 'preflight-receipt.json').exists(), 'never overwrite a preflight')
    policy = json.loads((P / 'preflight-protocol.json').read_text())
    for path, digest in policy['sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'frozen preflight source ' + path)
    compiled = json.loads((P / 'compile-binding.json').read_text())
    require(sha(Path(compiled['binary']).read_bytes()) == compiled['binary_sha256'], 'exact ELF')
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT).decode().strip()
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
    for key in list(env):
        if key.startswith('RHEON_'):
            env.pop(key)
    commands = []

    def run(label, argv, compress=False):
        start = time.monotonic()
        r = subprocess.run(argv, cwd=ROOT, env=env, capture_output=True)
        output = label + ('.json.gz' if compress else '.log')
        (P / output).write_bytes(gzip.compress(r.stdout, mtime=0) if compress else r.stdout)
        (P / (label + '-stderr.log')).write_bytes(r.stderr)
        commands.append(dict(argv=argv, exit=r.returncode, seconds=time.monotonic() - start,
                             stdout=output, stdout_sha256=sha(r.stdout), stderr_sha256=sha(r.stderr),
                             numerical_allow_variables_present=False))
        (P / 'preflight-commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        require(r.returncode == 0, 'guard-only preflight failure ' + label)
        return r.stdout

    for label, test in [('scalar-native', 'research_public_scalar_preflight'),
                        ('type-layout', 'research_public_type_layout'),
                        ('affine-native', 'scalar_affine_preflight'),
                        ('paired-layout', 'case41_paired_layout_preflight')]:
        run(label, [compiled['binary'], '--exact', 'research_public_call::' + test, '--nocapture'])
    scalar = load('paired_scalar_oracle', P / 'check_scalar.py').run(P / 'scalar-native.log')
    affine = load('paired_affine_oracle', P / 'check_affine.py').run()
    layouts = [json.loads(x) for x in (P / 'paired-layout.log').read_text().splitlines()
               if x.startswith('{')]
    require(len(layouts) == 1 and layouts[0]['event'] == 'paired_layout', 'one pure layout record')
    layout = layouts[0]
    require(layout['candidate'][1:] == layout['reference'][1:], 'paired identical return/point layouts')
    require(layout['candidate'][-1] == 176, 'three fixed 176-byte buffers')
    memory_bytes = run('memory-normal', [sys.executable, str(P / 'audit_memory.py')], True)
    optimized = run('memory-optimized', [sys.executable, '-O', str(P / 'audit_memory.py')], True)
    require(memory_bytes == optimized, 'normal/optimized fail-closed reader parity')
    memory = json.loads(memory_bytes)
    require(memory['status'] == 'BLOCKED_EXTERNAL_TRANSFER_ACCOUNTING', 'no execution permission')
    allocations = []
    names = ['nodes', 'macro_triangles', 'edges', 'triangles', 'embedding', 'pieces', 'faces',
             'pressure_terms', 'pressure_columns', 'rank_scratch', 'mass', 'motion',
             'diagnostics', 'triangle_scratch']
    for name, (count, size) in zip(names, layout['geometry'], strict=True):
        allocations.append(dict(name=name, requested_capacity=count, element_bytes=size,
                                requested_payload_bytes=count * size,
                                lifetime='one shared geometry, constructor through final observation/drop'))
    result = dict(status='PASS_SCALAR_LAYOUT_AFFINE__COMPLETE_MEMORY_BLOCKED',
                  source=source, source_tree=tree, compiled_source=compiled['source'],
                  compiled_source_tree=compiled['source_tree'], binary_sha256=compiled['binary_sha256'],
                  protocol_sha256=sha((P / 'preflight-protocol.json').read_bytes()),
                  scalar=scalar, affine=affine, paired_layout=layout, geometry_allocations=allocations,
                  common_geometry_requested_payload_bytes=sum(x['requested_payload_bytes'] for x in allocations),
                  geometry_allocator_internal_cost=None,
                  known_crate_and_storage_subtotal=memory['known_crate_and_storage_subtotal'],
                  complete_memory_bound=None, cap=67584, native_equations=0, owner_advances=0,
                  new_corrections=0, native_test_invocations=4, FD_test_invocations=0,
                  FD_execution_allowed=False, independent_memory_acceptance=False,
                  memory_ledger_sha256=sha(memory_bytes))
    (P / 'preflight-receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: result[k] for k in ['status', 'source', 'source_tree', 'compiled_source',
                                          'binary_sha256', 'known_crate_and_storage_subtotal',
                                          'complete_memory_bound', 'cap', 'native_equations']} , indent=2))


if __name__ == '__main__':
    main()
