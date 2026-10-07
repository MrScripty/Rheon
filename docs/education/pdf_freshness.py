"""Bind the generated reading PDF to the inputs used by the book renderer."""
from pathlib import Path
import hashlib
import json


def input_hashes(repo):
    repo = Path(repo)
    book = repo / 'docs/research-book'
    paths = set((book / 'chapters').glob('*.md'))
    paths.update((book / 'appendices').glob('*.md'))
    paths.update((book / 'figures').glob('*.svg'))
    paths.update((book / 'expansion/figures').glob('*.svg'))
    paths.update((book / 'expansion/figures').glob('*.jpg'))
    paths.update(repo / 'docs/education' / name
                 for name in ['build.py', 'verify_browser.py', 'style.css', 'package-lock.json'])
    return {str(p.relative_to(repo)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(paths)}


def write_receipt(repo, inputs):
    """Called only after a successful PDF render and browser qualification."""
    repo = Path(repo)
    if input_hashes(repo) != inputs:
        raise ValueError('PDF inputs changed during rendering; rebuild and rerender.')
    here = repo / 'docs/education'
    build = json.loads((here / '_site/build-receipt.json').read_text())
    if build.get('pdf_inputs') != inputs:
        raise ValueError('Rendered HTML has stale PDF inputs; rebuild first.')
    receipt = {'schema': 'rheon-pdf-inputs-v1', 'inputs': inputs,
               'pdf_sha256': hashlib.sha256(
                   (here / 'downloads/Rheon-expanded-book.pdf').read_bytes()).hexdigest()}
    (here / 'pdf-inputs.json').write_text(json.dumps(receipt, indent=2) + '\n')


def verify_pdf(repo, pdf=None):
    repo = Path(repo)
    here = repo / 'docs/education'
    receipt = json.loads((here / 'pdf-inputs.json').read_text())
    if receipt.get('schema') != 'rheon-pdf-inputs-v1':
        raise ValueError('Missing or unsupported PDF input receipt.')
    actual = input_hashes(repo)
    expected = receipt['inputs']
    changed = sorted(p for p in actual.keys() | expected.keys()
                     if actual.get(p) != expected.get(p))
    if changed:
        raise ValueError('Stale PDF inputs: ' + ', '.join(changed)
                         + '. Rebuild, rerender and review the PDF before publication.')
    pdf = Path(pdf) if pdf is not None else here / 'downloads/Rheon-expanded-book.pdf'
    if hashlib.sha256(pdf.read_bytes()).hexdigest() != receipt['pdf_sha256']:
        raise ValueError('PDF bytes differ from the qualified input receipt.')
