"""Read-only exploratory-protocol validation; no native test or equation call."""
from pathlib import Path
from fractions import Fraction as Q
import hashlib
import importlib.util
import json
import math
import struct
import subprocess

P = Path(__file__).resolve().parent
ROOT = P.parents[1]
F = ROOT / 'evidence/case41-paired-observation-v1'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def bits(value):
    return struct.pack('>d', value).hex()


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def run():
    policy = json.loads((P / 'protocol.json').read_text())
    for path, digest in policy['sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'frozen protocol/source/input ' + path)
    compiled = json.loads((F / 'compile-binding.json').read_text())
    require(policy['binary_sha256'] == compiled['binary_sha256']
            and policy['compiled_source'] == compiled['source'], 'same unchanged numerical ELF/source')
    require(sha(Path(policy['binary']).read_bytes()) == policy['binary_sha256'], 'actual unchanged ELF')
    historical = json.loads((F / 'preflight-receipt.json').read_text())
    require(historical['complete_memory_bound'] is None and historical['FD_execution_allowed'] is False
            and historical['known_crate_and_storage_subtotal'] == 66368, 'historical memory block retained')
    require(policy['resource_prerequisite_changed'] and not policy['numerical_method_changed']
            and not policy['historical_memory_qualified'], 'honest distinct authorization')
    scalar = module('exploratory_scalar_oracle', F / 'check_scalar.py').run(F / 'scalar-native.log')
    affine = module('exploratory_affine_oracle', F / 'check_affine.py').run()
    require(scalar == historical['scalar'] and affine == historical['affine'], 'original scalar/affine checks retained')
    fixture = json.loads((P / 'fixture.json').read_text())
    original = next(x for x in json.loads((ROOT / 'evidence/forcing-e2-fixed-candidates-v1/inputs.json').read_text())['cases']
                    if x['index'] == 41)
    require(fixture['original_case'] == original, 'exact original case41 fixture')
    require(policy['roster'] == ['baseline', 0, 1, 2, 3, 4, 5] and policy['orders'] == [16], 'original seven-equation roster')
    require(policy['corrections'] == policy['owners'] == policy['reference_integrations'] == 0, 'fixed no-owner/no-correction scope')
    roster = []
    for j in range(6):
        alpha = original['final_authorized_unknowns'][j]
        summed = float(Q(abs(alpha)) + Q(0.01))
        delta = float(Q(1e-6) * Q(summed))
        perturbed = float(Q(alpha) + Q(delta))
        require(all(math.isfinite(v) and (v == 0 or abs(v) >= 2.0**-1022)
                    for v in [summed, delta, perturbed]), 'checked original perturbation range')
        row = dict(column=j, alpha_bits=bits(alpha), delta_bits=bits(delta),
                   perturbed_bits=bits(perturbed), nominal_denominator=delta,
                   actual_offset_exact=str(Q(perturbed) - Q(alpha)))
        require(row == fixture['perturbation_roster'][j], 'independent exact binary64 perturbation graph')
        roster.append(row)
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
    tree = subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=ROOT).decode().strip()
    return dict(status='PASS_DISTINCT_EXPLORATORY_PROTOCOL_ONLY', source=source, source_tree=tree,
                compiled_source=compiled['source'], binary_sha256=compiled['binary_sha256'],
                protocol_sha256=sha((P / 'protocol.json').read_bytes()),
                historical_memory_qualified=False, historical_complete_bound=None,
                historical_known_subtotal=66368, resource_prerequisite_changed=True,
                numerical_method_changed=False, original_scalar_groups=23,
                exact_scalar_probes=36, affine_probes=6, roster=roster,
                native_tests_executed=0, native_equations_executed=0,
                corrections=0, owners=0, limits=policy['limits'])


if __name__ == '__main__':
    require(not (P / 'preflight-receipt.json').exists(), 'never overwrite policy preflight')
    result = run()
    (P / 'preflight-receipt.json').write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print(json.dumps(result, indent=2, sort_keys=True))
