"""Frozen result integrity only. Never launch a native test or equation."""
from pathlib import Path
import gzip
import hashlib
import json
import subprocess

P = Path(__file__).resolve().parent
ROOT = P.parents[1]
R = ROOT / 'evidence/case41-boundary-reader-repair-v1'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def run():
    inventory = json.loads((P / 'result-inventory.json').read_text())
    for path, digest in inventory['sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'frozen result artifact ' + path)
    protocol = json.loads((P / 'protocol.json').read_text())
    for path, digest in protocol['sha256'].items():
        require(sha((ROOT / path).read_bytes()) == digest, 'frozen exploratory input ' + path)
    before = json.loads((P / 'capture-before.json').read_text())
    after = json.loads((P / 'capture-completed.json').read_text())
    preflight = json.loads((P / 'preflight-receipt.json').read_text())
    analysis = json.loads((P / 'analysis-v2-normal.json').read_text())
    require(analysis['status'] == 'PASS_COMPLETE_EXPLORATORY_SEVEN_EQUATION_CAPTURE'
            and analysis['numerical_equations'] == 7 and analysis['retries'] == 0
            and analysis['baseline_all_22_bits_exact'], 'validated original seven-equation capture')
    require(sha(Path(protocol['binary']).read_bytes()) == protocol['binary_sha256']
            == after['binary_sha256'], 'actual unchanged numerical ELF')
    require(before['protocol_sha256'] == after['protocol_sha256'] == preflight['protocol_sha256']
            == sha((P / 'protocol.json').read_bytes()), 'frozen resource policy')
    require(sha((P / 'preflight-receipt.json').read_bytes()) == before['preflight_receipt_sha256'], 'actual exploratory preflight')
    require(sha((P / 'native.log').read_bytes()) == after['stdout_sha256'] == analysis['raw_stdout_sha256']
            and sha((P / 'native-stderr.log').read_bytes()) == after['stderr_sha256'], 'raw journal provenance')
    for identity in [before, preflight]:
        actual = subprocess.check_output(['git', 'rev-parse', identity['source'] + '^{tree}'], cwd=ROOT).decode().strip()
        require(actual == identity['source_tree'], 'exact source/tree ' + identity['source'])
    require((P / 'analysis-v2-normal.json').read_bytes() == (P / 'analysis-v2-optimized.json').read_bytes(), 'capture reader parity')
    models = []
    for precision in [80, 120]:
        normal = (P / f'model-{precision}-normal.json').read_bytes()
        require(normal == (P / f'model-{precision}-optimized.json').read_bytes(), 'model normal/optimized parity')
        result = json.loads(normal)
        require(result['new_native_equations'] == result['new_Newton_corrections'] == 0
                and result['historical_memory_qualified'] is False, 'model-only scope')
        models.append({key: value for key, value in result.items() if key != 'precision_digits'})
    require(models[0] == models[1], '80/120-digit rounded display consistency')
    repair = json.loads((R / 'RESULTS.json').read_text())
    require(set(repair['both_actual_boundaries']) == {'candidate', 'reference'}
            and len(repair['negative_controls']) == 4 and not repair['historical_memory_qualified'], 'both-side forward coverage repair')
    require((R / 'regression-normal.json').read_bytes() == (R / 'regression-optimized.json').read_bytes(), 'both-side regression parity')
    require(gzip.decompress((R / 'memory-normal.json.gz').read_bytes())
            == gzip.decompress((R / 'memory-optimized.json.gz').read_bytes()), 'repaired static reader parity')
    old = json.loads((ROOT / 'evidence/case41-paired-observation-v1/preflight-receipt.json').read_text())
    require(old['complete_memory_bound'] is None and old['FD_execution_allowed'] is False
            and old['known_crate_and_storage_subtotal'] == 66368, 'old unqualified receipt unchanged')
    changed = subprocess.check_output(['git', 'diff', '--name-only', '72367cc6436bfd1386d291b21110c0a5961deeb6'], cwd=ROOT).decode().splitlines()
    require(all(path.startswith('evidence/case41-exploratory-seven-v1/')
                or path.startswith('evidence/case41-boundary-reader-repair-v1/')
                or path == 'docs/research-book/implementation/case41-exploratory-seven-equations.md'
                for path in changed), 'old evidence and production preserved')
    require(subprocess.check_output(['git', 'rev-parse', 'origin/main'], cwd=ROOT).decode().strip()
            == '9cd4587a54befa61bdfddc8e35014bd3c34f02fb', 'main unchanged')
    return dict(status='PASS_FROZEN_EXPLORATORY_RESULT_INTEGRITY',
                protocol_source=preflight['source'], execution_source=before['source'],
                compiled_source=protocol['compiled_source'], binary_sha256=protocol['binary_sha256'],
                exact_equations=7, native_reruns=0, corrections=0, owners=0,
                measured_peak_rss_bytes=after['measured_peak_rss_bytes'],
                historical_memory_qualified=False, production_unchanged=True, old_evidence_unchanged=True)


if __name__ == '__main__':
    print(json.dumps(run(), indent=2, sort_keys=True))
