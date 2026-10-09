"""Add the motion workspace to the unchanged, reviewed Pages edition. No deploy."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

from viewer_integration import inventory, publish, read, require

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from browser.viewer.build import external

# Same content identity as research-book-reviewed-pages.yml on main. This is
# historical artifact identity, never a qualification of the current source.
BOOK_DIGEST = 'e42ced0f51edf5695a4760cce55ea8889ee155b41c2cf5429413d1e613b83424'
ENTRY = '''<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Rheon motion workspace</title><style>
body{margin:0;font:16px system-ui;background:#f4f6f8;color:#17202a}
header{padding:12px 16px}a{color:#174d87}iframe{display:block;width:100%;height:1100px;border:0}
</style><header><a href="index.html">Research book</a> ·
<a href="viewer/index.html">Open workspace directly</a></header>
<iframe title="Rheon motion workspace" src="viewer/index.html"></iframe></html>
'''


def book_inventory(source):
    source = Path(source)
    require(source.is_dir() and not source.is_symlink(), 'Book must be a regular directory')
    files = {}; hashes = {}; total = 0
    for path in sorted(source.rglob('*')):
        require(not path.is_symlink(), 'Book symlinks are refused')
        if path.is_dir():
            continue
        name = path.relative_to(source).as_posix()
        require(len(files) < 1024, 'Book exceeds file limit')
        raw = read(source, name, 32 * 1024 * 1024)
        total += len(raw)
        require(total <= 256 * 1024 * 1024, 'Book exceeds byte limit')
        files[name] = raw; hashes[name] = hashlib.sha256(raw).hexdigest()
    digest = hashlib.sha256(json.dumps(hashes, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    require(digest == BOOK_DIGEST, 'Book differs from the exact reviewed Pages edition')
    require(not any(name in files for name in ('workspace.html', 'viewer-integration.json', 'gui-composition.json'))
            and not any(name.startswith('viewer/') for name in files), 'Book occupies viewer namespace')
    return files, hashes


def compose(book, viewer, output):
    output = external(Path(output))
    require(not output.exists(), 'Composition output must be fresh')
    for source in (book, viewer):
        source = Path(source).resolve()
        require(not output.is_relative_to(source) and not source.is_relative_to(output), 'Inputs and output must be separate')
    files, hashes = book_inventory(book)
    inventory(ROOT, viewer)  # Refuse invalid inputs before creating output.
    output.mkdir(parents=True)
    for name, raw in files.items():
        target = output / name; target.parent.mkdir(parents=True, exist_ok=True); target.write_bytes(raw)
    receipt = publish(ROOT, output, viewer)
    (output / 'workspace.html').write_text(ENTRY)
    for name, sha in hashes.items():
        require(hashlib.sha256((output / name).read_bytes()).hexdigest() == sha, 'Historical book bytes changed')
    result = {'schema': 'rheon-gui-composition-v1', 'book_files_sha256': hashes,
              'historical_book_content_sha256': BOOK_DIGEST,
              'viewer_source_head': receipt['rheon_head'], 'entry': 'workspace.html',
              'solver_runs': 0, 'lean_runs': 0, 'new_research_qualification': False}
    (output / 'gui-composition.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('book-dir', 'viewer-dir', 'output-dir'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    result = compose(args.book_dir, args.viewer_dir, args.output_dir)
    print(json.dumps({k: v for k, v in result.items() if k != 'book_files_sha256'}))
