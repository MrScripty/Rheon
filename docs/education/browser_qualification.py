"""Read-only validation of the retained browser/source/book/PDF qualification."""
from pathlib import Path
import hashlib
import json

REVIEWED_SOURCES = ['build.py', 'labs.js', 'style.css', 'package-lock.json',
                    'verify_browser.py', 'browser_qualification.py']
LABS = ['projection', 'collision', 'hydrostatic', 'viscous', 'slip', 'cap']


def source_hashes(here):
    return {name: hashlib.sha256((here/name).read_bytes()).hexdigest()
            for name in REVIEWED_SOURCES}


def book_sources(repo):
    book = repo/'docs/research-book'
    paths = sorted((book/'chapters').glob('*.md')) + sorted((book/'appendices').glob('*.md'))
    return [{'slug': p.stem, 'title': p.read_text().splitlines()[0].removeprefix('# '),
             'source': str(p.relative_to(repo)),
             'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths]


def verify_browser_qualification(repo):
    repo = Path(repo)
    here = repo/'docs/education'
    receipt = json.loads((here/'browser-qualification.json').read_text())
    if receipt.get('schema') != 'rheon-education-browser-v1':
        raise ValueError('Missing or unsupported browser qualification.')
    if receipt.get('reviewed_sources') != source_hashes(here):
        raise ValueError('Stale browser source bindings; requalify before publication.')
    if receipt.get('book_sources') != book_sources(repo):
        raise ValueError('Stale browser book bindings; requalify before publication.')
    for pdf in [here/'downloads/Rheon-expanded-book.pdf',
                here/'_site/downloads/Rheon-expanded-book.pdf']:
        if hashlib.sha256(pdf.read_bytes()).hexdigest() != receipt.get('pdf_sha256'):
            raise ValueError('Stale browser PDF binding: ' + str(pdf))
    labs = receipt.get('labs', [])
    if [lab.get('lab') for lab in labs] != LABS or not all(
            lab.get('control_changes_metrics') is True for lab in labs):
        raise ValueError('Incomplete six-lab browser qualification.')
    for flag in ['webgl', 'mobile_navigation_search', 'mobile_no_horizontal_overflow',
                 'katex_no_errors', 'planar_statement_hit_time_rendering']:
        if receipt.get(flag) is not True:
            raise ValueError('Missing browser qualification check: ' + flag)
    if receipt.get('page_errors') != [] or receipt.get('http_failures') != []:
        raise ValueError('Retained browser qualification contains page/network failures.')


if __name__ == '__main__':
    verify_browser_qualification(Path(__file__).resolve().parents[2])
    print('PASS retained browser source/book/PDF bindings and qualification')
