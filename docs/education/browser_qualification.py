"""Read-only validation of the retained browser/source/book/PDF qualification."""
from pathlib import Path
import hashlib
import json

REVIEWED_SOURCES = ['build.py', 'labs.js', 'style.css', 'package-lock.json',
                    'teaching.py', 'reader.js', 'reader.css', 'navigation.js', 'teaching_browser.py',
                    'transport_reference.js', 'transport_labs.js', 'transport_browser.py', 'test_transport_reference.py',
                    'flow_labs.js', 'flow_worker.js', 'flow_field.js', 'flow_volume.js', 'flow_browser.py', 'flow_field_oracles.py', 'test_flow_browser.py', 'test_flow_field.py',
                    'timestep_labs.js', 'timestep_worker.js', 'timestep_browser.py', 'test_timestep_browser.py', 'build_solver.py', 'test_build_solver.py', 'BROWSER_TIMESTEP.md',
                    '../research-book/companion/experiments.py', '../research-book/companion/depth_experiments.py',
                    '../../browser/viewer/view.js', '../../browser/viewer/shell.css',
                    'verify_browser.py', 'browser_qualification.py', 'native_browser.py',
                    'native_sequence.py', 'native-sequence.json', 'markdown_bundle.py', 'requirements.txt',
                    'static_obstacle.py', 'obstacle.js', 'obstacle_browser.py', 'obstacle_flow.py', 'obstacle_flow.js', 'obstacle_flow_browser.py', 'aligned_strain_packet.py', 'aligned-strain-native-sources.json', 'aligned_strain_html.py', 'aligned_strain.js', 'aligned_strain.css', 'aligned_strain_browser.py', 'compressible_lab.py', 'compressible.js', 'compressible_browser.py']
LABS = ['projection', 'collision', 'hydrostatic', 'viscous', 'slip', 'cap']


def transport_source_hashes(here):
    sources = {'transport_reference.js': 'transport_reference.js',
               'transport_labs.js': 'transport_labs.js',
               'transport-companion.py': '../research-book/companion/experiments.py',
               'sampling-companion.py': '../research-book/companion/depth_experiments.py'}
    return {published: hashlib.sha256((here/source).read_bytes()).hexdigest()
            for published, source in sources.items()}


def validate_transport_receipt(observed, expected_sources=None):
    if not isinstance(observed, dict) or observed.get('schema') != 'rheon-transport-browser-v1':
        raise ValueError('Missing sampling/transport browser qualification')
    expected = {'sampler_states':12, 'transport_states':10, 'sampler_donors_checked':96,
                'transport_curves_checked':30, 'sine_refinements':[32,64,128,256],
                'affine_wrong_offset_bias':-.07, 'fixed_plot_scale':True,
                'symmetric':{'raw_min':-.125,'raw_max':1.125,'fallback':2,'limited_sum':2},
                'asymmetric':{'fallback':1,'initial_sum':1.6,'limited_sum':1.61875,'sum_change':.01875}}
    if observed.get('numerical') != expected:
        raise ValueError('Incomplete sampling/transport numerical comparisons')
    for key, value in [('desktop_scroll_presets',4),('chapter_scroll_presets',7),('direct_links',11),('fluid_solves',0),('fluid_advances',0)]:
        if type(observed.get(key)) is not int or observed[key] != value:
            raise ValueError('Incomplete sampling/transport count: '+key)
    for flag in ['camera_preserved_during_scroll_and_controls','manual_comparison_pauses_follow',
                 'follow_resumes_current_section','one_period_button']:
        if observed.get(flag) is not True:
            raise ValueError('Incomplete sampling/transport check: '+flag)
    mobile=observed.get('mobile',[])
    if len(mobile)!=2 or [entry.get('width') for entry in mobile]!=[390,320]:
        raise ValueError('Incomplete sampling/transport phone widths')
    for entry in mobile:
        if entry.get('modes')!=['text','split','example'] or any(entry.get(flag) is not True for flag in
                ['readable_text','no_horizontal_overflow','split_scroll_presets','control_comparison','mode_preserves_lesson_and_scroll','direct_link_selects_preset']):
            raise ValueError('Incomplete sampling/transport phone checks')
    sources=observed.get('source_sha256',{})
    if set(sources)!={'transport_reference.js','transport_labs.js','transport-companion.py','sampling-companion.py'} or any(
            not isinstance(value,str) or len(value)!=64 or any(c not in '0123456789abcdef' for c in value) for value in sources.values()):
        raise ValueError('Missing sampling/transport source bindings')
    if expected_sources is not None and sources!=expected_sources:
        raise ValueError('Stale sampling/transport source bindings')


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


def validate_timestep_publication(repo, artifact, observed):
    from build_solver import verify_published as verify_solver_published
    from timestep_browser import validate_receipt as validate_timestep_receipt
    solver=verify_solver_published(repo,artifact)
    validate_timestep_receipt(observed,solver['included'])
    expected={name:hashlib.sha256((artifact/'solver'/name).read_bytes()).hexdigest()
              for name in ['rheon_solver.js','rheon_solver_bg.wasm','native-reference.json','build-receipt.json']} if solver['included'] else {}
    if observed.get('packet_hashes')!=expected:
        raise ValueError('Served timestep packet differs from the retained edition')


def validate_flow_publication(repo, artifact, observed):
    from build_solver import verify_published as verify_solver_published
    from flow_browser import validate_receipt as validate_flow_receipt
    solver = verify_solver_published(repo, artifact)
    validate_flow_receipt(observed, solver['included'])
    names = ['rheon_solver.js', 'rheon_solver_bg.wasm', 'native-flow-reference.json',
             'native-flow-fields.bin', 'build-receipt.json']
    expected = {name: hashlib.sha256((artifact/'solver'/name).read_bytes()).hexdigest()
                for name in names} if solver['included'] else {}
    if observed.get('packet_hashes') != expected:
        raise ValueError('Served 3D flow packet differs from the retained edition')


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
    validate_transport_receipt(receipt.get('transport'), transport_source_hashes(here))
    if artifact_dir is not None:
        validate_timestep_publication(repo,artifact,receipt.get('timestep'))
        validate_flow_publication(repo,artifact,receipt.get('flow'))
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
        from compressible_lab import verify_published as verify_compressible_published
        from compressible_browser import validate_receipt as validate_compressible_receipt
        compressible=verify_compressible_published(repo,artifact)
        validate_compressible_receipt(receipt.get('compressible',{}),compressible)


if __name__ == '__main__':
    verify_browser_qualification(Path(__file__).resolve().parents[2])
    print('PASS retained browser source/book/PDF bindings and qualification')
