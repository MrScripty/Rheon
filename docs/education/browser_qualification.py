"""Read-only validation of the retained browser/source/book/PDF qualification."""
from pathlib import Path
import hashlib
import json

REVIEWED_SOURCES = ['build.py', 'labs.js', 'style.css', 'package-lock.json',
                    'teaching.py', 'reader.js', 'reader.css', 'navigation.js', 'teaching_browser.py',
                    '../../browser/viewer/view.js', '../../browser/viewer/shell.css',
                    'verify_browser.py', 'browser_qualification.py', 'native_browser.py',
                    'native_sequence.py', 'native-sequence.json', 'markdown_bundle.py', 'requirements.txt',
                    'static_obstacle.py', 'obstacle.js', 'obstacle_browser.py', 'obstacle_flow.py', 'obstacle_flow.js', 'obstacle_flow_browser.py', 'aligned_strain_packet.py', 'aligned_strain_html.py', 'aligned_strain.js', 'aligned_strain.css', 'aligned_strain_browser.py']
LABS = ['projection', 'collision', 'hydrostatic', 'viscous', 'slip', 'cap']


def validate_teaching_receipt(observed, reference_sha256=None):
    """Retained checks must include measured teaching behavior, not just assets."""
    required = ['desktop_columns', 'desktop_sticky_example',
                'manual_comparison_pauses_follow', 'follow_resumes_current_section',
                'camera_drag_and_keyboard', 'camera_preserved_during_scroll_and_controls',
                'camera_reset_and_zoom', 'wheel_does_not_zoom',
                'compact_proof_disclosure_and_deep_link', 'chapter_source_links']
    if not isinstance(observed, dict) or observed.get('schema') != 'rheon-teaching-browser-v1':
        raise ValueError('Missing teaching browser qualification')
    for flag in required:
        if observed.get(flag) is not True:
            raise ValueError('Incomplete teaching browser check: '+flag)
    for key, value in [('scroll_stops', 12), ('fluid_solves', 0), ('fluid_advances', 0)]:
        if type(observed.get(key)) is not int or observed[key] != value:
            raise ValueError('Incomplete teaching browser count: '+key)
    expected = {'states': dict(zip(LABS, [3, 9, 3, 9, 3, 9])), 'total_states': 36,
                'arrow_states': 27, 'cap_render_invariance_comparisons': 6,
                'independent_equations': ['graph incidence and kinetic norm', 'segment interpolation',
                    'two-layer hydrostatic pressure', 'implicit periodic eigenmode and continuum decay',
                    'one-slip-wall Couette speed, traction and dissipation',
                    'spherical cap volume, Young–Laplace pressure and adhesion work']}
    if observed.get('numerical') != expected:
        raise ValueError('Incomplete teaching numerical comparisons')
    mobile = observed.get('mobile', [])
    if len(mobile) != 2 or [entry.get('width') for entry in mobile] != [390, 320]:
        raise ValueError('Incomplete teaching phone widths')
    for entry in mobile:
        if entry.get('modes') != ['text', 'split', 'example'] or any(entry.get(key) is not True for key in
                ['chapter_search_and_navigation', 'no_horizontal_overflow', 'readable_text',
                 'example_control_comparison', 'mode_preserves_lesson_and_scroll',
                 'split_scroll_selects_preset', 'follow_in_example_restores_reading_preset']):
            raise ValueError('Incomplete teaching phone interaction checks')
    if reference_sha256 is not None and observed.get('reference_sha256') != reference_sha256:
        raise ValueError('Stale teaching reference data binding')


def source_hashes(here):
    return {name: hashlib.sha256((here/name).read_bytes()).hexdigest()
            for name in REVIEWED_SOURCES}


def book_sources(repo):
    book = repo/'docs/research-book'
    paths = sorted((book/'chapters').glob('*.md')) + sorted((book/'appendices').glob('*.md')) + sorted((book/'implementation').glob('*.md'))
    return [{'slug': p.stem, 'title': p.read_text().splitlines()[0].removeprefix('# '),
             'source': str(p.relative_to(repo)),
             'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths]


def validate_strain_browser_receipt(observed, strain):
    if strain is None:
        expected={'included':False,'dynamic_states':0,'independent_rational_comparisons':0,
                  'fluid_solves':0,'fluid_advances':0,'mobile_no_overflow':True,'screenshots':[]}
        if observed != expected:
            raise ValueError('Incomplete source-only aligned-strain browser qualification')
        return
    expected={'included':True,'dynamic_states':46,'active_faces':len(strain['active']),
              'rows':len(strain['rows']),'independent_rational_comparisons':46,
              'retained_zero_rows':sum(not r['terms'] for r in strain['rows']),
              'reflected_corners':len(strain['cornerEdges']),
              'fluid_solves':0,'fluid_advances':0,'mobile_no_overflow':True}
    if any(type(observed.get(key)) is not type(value) or observed.get(key)!=value
           for key,value in expected.items()):
        raise ValueError('Incomplete actual aligned-strain control comparisons')


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
    reference = artifact/'reference-data.json'
    validate_teaching_receipt(receipt.get('teaching'),
                             hashlib.sha256(reference.read_bytes()).hexdigest() if artifact_dir is not None else None)
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
        validate_strain_browser_receipt(receipt.get('aligned_strain',{}),strain)


if __name__ == '__main__':
    verify_browser_qualification(Path(__file__).resolve().parents[2])
    print('PASS retained browser source/book/PDF bindings and qualification')
