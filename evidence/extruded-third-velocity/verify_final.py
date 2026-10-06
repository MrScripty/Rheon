"""Verify the complete successor binding, immutable history and actual receipts."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

P = Path(__file__).resolve().parent
ROOT = P.parents[1]
RECEIPT = P / 'final-receipt.json'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def tree(commit):
    return {line.split(b'\t', 1)[1].decode():line.split(b'\t', 1)[0].split()[2].decode() for line in git('ls-tree', '-r', commit).splitlines()}


def verify(receipt, evidence_commit=None):
    require(receipt['status'] == 'PASS', 'completed milestone status')
    source = receipt['qualified_source_commit']
    require(git('rev-parse', source + '^{tree}').decode().strip() == receipt['qualified_source_tree'], 'qualified source tree')
    require(git('show', '-s', '--format=%P', source).decode().split() == receipt['ordered_source_parents'], 'ordered source parents')
    actual = tree(source)
    for path, digest in receipt['source_sha256'].items():
        require(path in actual and sha(git('show', source + ':' + path)) == digest, 'Git qualified source ' + path)
        require(sha((ROOT / path).read_bytes()) == digest, 'working source ' + path)
    evidence = tree(evidence_commit) if evidence_commit else None
    for path, digest in receipt['file_sha256'].items():
        data = git('show', evidence_commit + ':' + path) if evidence is not None else (ROOT / path).read_bytes()
        require(sha(data) == digest, 'frozen evidence ' + path)
    prefix = str(P.relative_to(ROOT)) + '/'
    files = {str(p.relative_to(ROOT)) for p in P.rglob('*') if p.is_file() and '__pycache__' not in p.parts and p != RECEIPT}
    if evidence is not None:
        files = {p for p in evidence if p.startswith(prefix) and p != str(RECEIPT.relative_to(ROOT))}
    require(files == set(receipt['file_sha256']), 'complete successor inventory')
    base = tree(receipt['historical_base'])
    allowed = {'src/coupled_discrete.rs', 'src/lib.rs', 'src/translated_viscous.rs'}
    require(set(receipt['legitimate_changed_baseline_paths']) == allowed, 'explicit reviewed production delta')
    require(receipt['historical_git_blobs'] == {p:b for p,b in base.items() if p not in allowed}, 'complete frozen original history')
    require(all(actual.get(p) == b for p,b in receipt['historical_git_blobs'].items()), 'original source/evidence overwritten')
    derivation = json.loads((P / 'derivation-receipt.json').read_text())
    for path, digest in derivation['source_sha256'].items():
        require(receipt['source_sha256'].get(path) == digest, 'frozen third derivation source ' + path)
    binding = json.loads((P / 'composition-binding.json').read_text())
    require(binding['ordered_source_parents'] == ['d1617802d184d17933fc56c101f89733708802b6', 'ec26e48df958f0ebc60af414253ff98c0007dc13'], 'independent sibling composition')
    for name in ['full-feature-tests/source-binding.json', 'native-preflight/source-binding.json']:
        prior = json.loads((P / name).read_text())
        for path,digest in prior['source_sha256'].items():
            if path.endswith('.rs'):
                require(receipt['source_sha256'][path] == digest, 'reused compiled source ' + path)
        raw = json.loads((P / name).with_name('receipt.json').read_text())
        require(raw['status'] == 'PASS' and all(x['exit'] == 0 for x in raw['commands']), 'actual compiled command ' + name)
    final = json.loads((P / 'final-qualification/receipt.json').read_text())
    require(final['qualified_source_commit'] == source and final['status'] == 'PASS' and all(x['exit'] == 0 for x in final['actual_commands']), 'actual successor commands')
    require(final['actual_corruption_rejections'] == 13 and final['actual_steps'] == 248, 'qualified physical cells')
    require(receipt['browser_status'] == final['browser_status'], 'explicit browser limitation')
    return dict(status='PASS', source=source, tree=receipt['qualified_source_tree'], source_paths=len(receipt['source_sha256']), successor_files=len(files), preserved_baseline_paths=len(receipt['historical_git_blobs']), actual_steps=248, actual_corruptions_rejected=13, browser_status=receipt['browser_status'])


if __name__ == '__main__':
    receipt = json.loads(RECEIPT.read_text())
    commit = sys.argv[sys.argv.index('--evidence-commit')+1] if '--evidence-commit' in sys.argv else None
    print(json.dumps(verify(receipt, commit), indent=2))
    if '--negative-self-test' in sys.argv:
        for field in ['source_sha256', 'file_sha256', 'historical_git_blobs']:
            bad = copy.deepcopy(receipt)
            bad[field][next(iter(bad[field]))] = '0' * len(next(iter(bad[field].values())))
            try:
                verify(bad, commit)
            except ValueError as error:
                print('REJECTED ' + field + ': ' + str(error))
            else:
                raise ValueError('corrupt binding accepted: ' + field)
