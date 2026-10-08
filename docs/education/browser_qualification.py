"""Read-only validation of the retained browser/source/book/PDF qualification."""
from pathlib import Path
import hashlib
import json

REVIEWED_SOURCES = ['build.py', 'labs.js', 'style.css', 'package-lock.json',
                    'verify_browser.py', 'browser_qualification.py', 'native_browser.py',
                    'native_sequence.py', 'native-sequence.json', 'markdown_bundle.py', 'requirements.txt',
                    'static_obstacle.py', 'obstacle.js', 'obstacle_browser.py', 'obstacle_flow.py', 'obstacle_flow.js', 'obstacle_flow_browser.py', 'aligned_strain_packet.py', 'aligned_strain_html.py', 'aligned_strain.js', 'aligned_strain.css', 'aligned_strain_browser.py']
LABS = ['projection', 'collision', 'hydrostatic', 'viscous', 'slip', 'cap']


def source_hashes(here):
    return {name: hashlib.sha256((here/name).read_bytes()).hexdigest()
            for name in REVIEWED_SOURCES}


def book_sources(repo):
    book = repo/'docs/research-book'
    paths = sorted((book/'chapters').glob('*.md')) + sorted((book/'appendices').glob('*.md')) + sorted((book/'implementation').glob('*.md'))
    return [{'slug': p.stem, 'title': p.read_text().splitlines()[0].removeprefix('# '),
             'source': str(p.relative_to(repo)),
             'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths]


def verify_browser_qualification(repo, artifact_dir=None):
    repo = Path(repo)
    here = repo/'docs/education'
    artifact = Path(artifact_dir) if artifact_dir is not None else here
    receipt = json.loads((artifact/'browser-qualification.json').read_text())
    if receipt.get('schema') != 'rheon-education-browser-v1':
        raise ValueError('Missing or unsupported browser qualification.')
    if receipt.get('reviewed_sources') != source_hashes(here):
        raise ValueError('Stale browser source bindings; requalify before publication.')
    if receipt.get('book_sources') != book_sources(repo):
        raise ValueError('Stale browser book bindings; requalify before publication.')
    pdfs = [artifact/'downloads/Rheon-expanded-book.pdf'] if artifact_dir is not None else [here/'downloads/Rheon-expanded-book.pdf',here/'_site/downloads/Rheon-expanded-book.pdf']
    for pdf in pdfs:
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
    if artifact_dir is not None:
        native=receipt.get('native_labs',{})
        presentation=json.loads((artifact/'native-presentation.json').read_text())
        expected={'recorded_bundles_included':presentation['recorded_bundles_included'],
                  'hub_models':3,'mobile_no_overflow':True,'native_numerical_calls':0,
                  'native_snapshot_checks':50 if presentation['recorded_bundles_included'] else 0,
                  'wall_final_profile_checks':6 if presentation['recorded_bundles_included'] else 0}
        if native!=expected:
            raise ValueError('Incomplete native playback/source-only browser qualification.')
        from static_obstacle import verify_published
        obstacle=verify_published(repo,artifact)
        included=obstacle is not None
        expected={'included':included,'cases':9 if included else 0,'cells':240 if included else 0,
                  'flux_pairs':27 if included else 0,'mobile_no_overflow':True,'fluid_advances':0}
        if receipt.get('static_obstacle')!=expected:
            raise ValueError('Incomplete static geometry browser qualification')
        from obstacle_flow import verify_published as verify_flow_published
        flow=verify_flow_published(repo,artifact)
        included=flow is not None
        expected={'included':included,'pressure_views':24 if included else 0,'shear_frames':54 if included else 0,'mobile_no_overflow':True,'browser_fluid_solves':0}
        if receipt.get('obstacle_flow')!=expected:
            raise ValueError('Incomplete obstacle-flow browser qualification')

        from aligned_strain_packet import verify_published as verify_strain_published
        strain=verify_strain_published(repo,artifact)
        included=strain is not None
        observed=receipt.get('aligned_strain',{})
        if observed.get('included') is not included or observed.get('fluid_advances') != 0 or observed.get('mobile_no_overflow') is not True:
            raise ValueError('Incomplete aligned-strain browser qualification')
        if included and (observed.get('independent_rational_comparisons',0) < 8 or observed.get('active_faces') != len(strain['active']) or observed.get('rows') != len(strain['rows'])):
            raise ValueError('Missing actual aligned-strain control comparisons')


if __name__ == '__main__':
    verify_browser_qualification(Path(__file__).resolve().parents[2])
    print('PASS retained browser source/book/PDF bindings and qualification')
