"""Source-only/data-only and retained browser qualification rejection fixtures."""
from pathlib import Path
import hashlib
import json
import tempfile
import subprocess
import unittest
from unittest.mock import patch
import build
from browser_qualification import validate_transport_receipt, transport_source_hashes, validate_teaching_receipt, validate_strain_browser_receipt, LABS, REVIEWED_SOURCES, book_sources, source_hashes, verify_browser_qualification
from pdf_freshness import input_hashes
import verify_browser
from browser_qualification import validate_timestep_publication
from timestep_browser import source_hashes as timestep_source_hashes


class OutputAdmission(unittest.TestCase):
    def test_existing_output_and_checkout_ancestors_are_preserved(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);repo=root/'repo';repo.mkdir();output=root/'edition';output.mkdir()
            marker=output/'retained';marker.write_text('unchanged')
            for target in (root,repo,repo/'output',output):
                with self.subTest(target=target),self.assertRaises(ValueError):build.validate_output(target,repo)
            self.assertEqual(marker.read_text(),'unchanged')

    def test_other_git_checkout_is_refused_and_fresh_external_path_is_admitted(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);repo=root/'repo';repo.mkdir();other=root/'other';other.mkdir()
            subprocess.run(['git','init','--quiet',str(other)],check=True)
            with self.assertRaisesRegex(ValueError,'every Git checkout'):build.validate_output(other/'edition',repo)
            build.validate_output(root/'fresh'/'edition',repo)


class TimestepOmissionGate(unittest.TestCase):
    def test_omitted_solver_requires_disabled_controls_and_no_invented_execution(self):
        with tempfile.TemporaryDirectory() as tmp:
            edition=Path(tmp)
            (edition/'solver-presentation.json').write_text('{"schema":1,"included":false}')
            observed={'schema':'rheon-timestep-browser-v1','included':False,'fluid_advances':0,
                      'rejected_candidates':0,'numerical_states':0,'unavailable_controls_disabled':True,
                      'no_synthetic_state':True,'source_sha256':timestep_source_hashes(),
                      'packet_hashes':{},'screenshots':[]}
            validate_timestep_publication(build.ROOT,edition,observed)
            for key,value in [('included',True),('fluid_advances',1),('unavailable_controls_disabled',False)]:
                with self.subTest(key=key),self.assertRaises(ValueError):
                    validate_timestep_publication(build.ROOT,edition,{**observed,key:value})


class ReferenceQualification(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name)
        self.reference = self.repo/'docs/research-book/expansion'
        self.reference.mkdir(parents=True)
        for name, content in [('reference.py', '# checked source'), ('reference-data.json', '{"checked": true}')]:
            (self.reference/name).write_text(content)
        receipt = {key: hashlib.sha256((self.reference/name).read_bytes()).hexdigest()
                   for name, key in [('reference.py', 'source_sha256'), ('reference-data.json', 'data_sha256')]}
        (self.reference/'reference-qualification.json').write_text(json.dumps(receipt))

    def test_current_source_and_data_pass(self):
        build.verify_reference(self.reference)

    def test_source_only_and_data_only_reject_before_copy(self):
        for name in ['reference.py', 'reference-data.json']:
            with self.subTest(name=name):
                original = (self.reference/name).read_bytes()
                (self.reference/name).write_bytes(original+b'\nchanged')
                output = self.repo/'docs/education/_site'
                output.mkdir(parents=True, exist_ok=True)
                marker = output/'retained-site'
                marker.write_text('unchanged')
                with patch.object(build, 'BOOK', self.reference.parent), patch.object(build, 'ROOT', self.repo), patch.object(build, 'OUT', output):
                    with self.assertRaisesRegex(ValueError, name+' does not match reference qualification'):
                        build.build()
                self.assertEqual(marker.read_text(), 'unchanged')
                self.assertFalse((output/name).exists())
                (self.reference/name).write_bytes(original)


class AlignedStrainReceiptRejection(unittest.TestCase):
    def test_source_only_receipt_cannot_claim_computation(self):
        empty={'included':False,'dynamic_states':0,'independent_rational_comparisons':0,
               'fluid_solves':0,'fluid_advances':0,'mobile_no_overflow':True,'screenshots':[]}
        validate_strain_browser_receipt(empty,None)
        for key,value in [('fluid_solves',1),('included',True),('screenshots',['invented'])]:
            with self.subTest(key=key),self.assertRaises(ValueError):
                validate_strain_browser_receipt({**empty,key:value},None)

    def test_actual_receipt_retains_corner_zero_and_solver_guarantees(self):
        strain={'active':list(range(48)),'rows':[{'terms':[]}]*6+[{'terms':[1]}]*204,
                'cornerEdges':list(range(12))}
        observed={'included':True,'dynamic_states':46,'active_faces':48,'rows':210,
                  'independent_rational_comparisons':46,'retained_zero_rows':6,
                  'reflected_corners':12,'fluid_solves':0,'fluid_advances':0,'mobile_no_overflow':True}
        validate_strain_browser_receipt(observed,strain)
        for key in observed:
            changed=dict(observed);changed.pop(key)
            with self.subTest(missing=key),self.assertRaises(ValueError):
                validate_strain_browser_receipt(changed,strain)
        for key,value in [('fluid_solves',1),('fluid_advances',1),('retained_zero_rows',0),
                          ('reflected_corners',0),('dynamic_states',8),('included',1)]:
            with self.subTest(changed=key),self.assertRaises(ValueError):
                validate_strain_browser_receipt({**observed,key:value},strain)


def teaching_receipt():
    return {'schema':'rheon-teaching-browser-v1', 'reference_sha256':'reference',
            **dict.fromkeys(['desktop_columns','desktop_sticky_example',
                'manual_comparison_pauses_follow','follow_resumes_current_section',
                'camera_drag_and_keyboard','camera_preserved_during_scroll_and_controls',
                'camera_reset_and_zoom','wheel_does_not_zoom',
                'compact_proof_disclosure_and_deep_link','chapter_source_links'],True),
            'scroll_stops':12,'fluid_solves':0,'fluid_advances':0,
            'numerical':{'states':dict(zip(LABS,[3,9,3,9,3,9])), 'total_states':36,
                'arrow_states':27,'cap_render_invariance_comparisons':6,
                'independent_equations':['graph incidence and kinetic norm','segment interpolation',
                    'two-layer hydrostatic pressure','implicit periodic eigenmode and continuum decay',
                    'one-slip-wall Couette speed, traction and dissipation',
                    'spherical cap volume, Young–Laplace pressure and adhesion work']},
            'mobile':[{'width':width,'modes':['text','split','example'],
                'chapter_search_and_navigation':True,'no_horizontal_overflow':True,
                'readable_text':True,'example_control_comparison':True,
                'mode_preserves_lesson_and_scroll':True,'split_scroll_selects_preset':True,
                'follow_in_example_restores_reading_preset':True} for width in [390,320]]}


def transport_receipt(sources=None):
    return {'schema':'rheon-transport-browser-v1',
            'source_sha256':sources or dict.fromkeys(['transport_reference.js','transport_labs.js','transport-companion.py','sampling-companion.py'],'a'*64),
            'numerical':{'sampler_states':12,'transport_states':10,'sampler_donors_checked':96,
                'transport_curves_checked':30,'sine_refinements':[32,64,128,256],
                'affine_wrong_offset_bias':-.07,'fixed_plot_scale':True,
                'symmetric':{'raw_min':-.125,'raw_max':1.125,'fallback':2,'limited_sum':2},
                'asymmetric':{'fallback':1,'initial_sum':1.6,'limited_sum':1.61875,'sum_change':.01875}},
            'desktop_scroll_presets':4,'chapter_scroll_presets':7,'direct_links':11,'fluid_solves':0,'fluid_advances':0,
            **dict.fromkeys(['camera_preserved_during_scroll_and_controls','manual_comparison_pauses_follow',
                            'follow_resumes_current_section','one_period_button'],True),
            'mobile':[{'width':width,'modes':['text','split','example'],
                **dict.fromkeys(['readable_text','no_horizontal_overflow','split_scroll_presets',
                                'control_comparison','mode_preserves_lesson_and_scroll','direct_link_selects_preset'],True)} for width in [390,320]]}


class TransportReceiptRejection(unittest.TestCase):
    def test_complete_and_missing_fields(self):
        good=transport_receipt()
        validate_transport_receipt(good,good['source_sha256'])
        for key in good:
            changed=dict(good);changed.pop(key)
            with self.subTest(missing=key),self.assertRaises(ValueError):validate_transport_receipt(changed)
        with self.assertRaisesRegex(ValueError,'Stale'):
            validate_transport_receipt(good,{**good['source_sha256'],'transport-companion.py':'b'*64})

    def test_false_conservation_and_incomplete_mobile_checks_reject(self):
        changed=transport_receipt();changed['numerical']['asymmetric']['limited_sum']=1.6
        with self.assertRaises(ValueError):validate_transport_receipt(changed)
        changed=transport_receipt();changed['mobile'][1]['split_scroll_presets']=False
        with self.assertRaises(ValueError):validate_transport_receipt(changed)
        changed=transport_receipt();changed['fluid_advances']=1
        with self.assertRaises(ValueError):validate_transport_receipt(changed)


class TeachingReceiptRejection(unittest.TestCase):
    def test_complete_evidence_and_reference_binding(self):
        validate_teaching_receipt(teaching_receipt(),'reference')
        with self.assertRaisesRegex(ValueError,'reference data'):
            validate_teaching_receipt(teaching_receipt(),'changed')

    def test_missing_checks_and_damaged_numeric_or_phone_evidence_reject(self):
        good=teaching_receipt()
        for key in good:
            if key=='reference_sha256':continue
            changed=dict(good);changed.pop(key)
            with self.subTest(missing=key),self.assertRaises(ValueError):
                validate_teaching_receipt(changed)
        for section,key in [('numerical','total_states'),('numerical','arrow_states')]:
            changed=teaching_receipt();changed[section][key]=0
            with self.subTest(changed=key),self.assertRaises(ValueError):
                validate_teaching_receipt(changed)
        changed=teaching_receipt();changed['mobile'][1]['mode_preserves_lesson_and_scroll']=False
        with self.assertRaises(ValueError):validate_teaching_receipt(changed)


class BrowserBindings(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name)
        self.here = self.repo/'docs/education'
        self.here.mkdir(parents=True)
        for name in set(REVIEWED_SOURCES) | {'pdf_freshness.py'}:
            (self.here/name).parent.mkdir(parents=True,exist_ok=True)
            (self.here/name).write_text(name)
        self.chapter = self.repo/'docs/research-book/chapters/01-example.md'
        self.chapter.parent.mkdir(parents=True)
        self.chapter.write_text('# Original')
        self.pdf = self.here/'downloads/Rheon-expanded-book.pdf'
        self.pdf.parent.mkdir()
        self.pdf.write_bytes(b'qualified PDF fixture')
        self.site_pdf = self.here/'_site/downloads/Rheon-expanded-book.pdf'
        self.site_pdf.parent.mkdir(parents=True)
        self.site_pdf.write_bytes(self.pdf.read_bytes())
        self.receipt = {'schema': 'rheon-education-browser-v1', 'browser': 'fixture', 'reviewed_sources': source_hashes(self.here),
                        'book_sources': book_sources(self.repo), 'pdf_sha256': hashlib.sha256(self.pdf.read_bytes()).hexdigest(),
                        'labs': [{'lab': lab, 'control_changes_metrics': True} for lab in LABS],
                        'webgl': True, 'mobile_navigation_search': True, 'mobile_no_horizontal_overflow': True,
                        'katex_no_errors': True, 'planar_statement_hit_time_rendering': True,
                        'page_errors': [], 'http_failures': [], 'teaching':teaching_receipt(), 'transport':transport_receipt(transport_source_hashes(self.here))}
        self.receipt_path = self.here/'browser-qualification.json'
        self.receipt_path.write_text(json.dumps(self.receipt))
        (self.here/'pdf-inputs.json').write_text(json.dumps({'schema': 'rheon-pdf-inputs-v1',
            'inputs': input_hashes(self.repo), 'pdf_sha256': self.receipt['pdf_sha256']}))

    def test_current_bindings_pass(self):
        verify_browser_qualification(self.repo)

    def test_lab_and_renderer_source_only_changes_reject(self):
        for name in ['labs.js', 'verify_browser.py', 'reader.js', 'navigation.js', 'reader.css', 'teaching.py', 'transport_reference.js', 'transport_labs.js', '../research-book/companion/experiments.py', '../research-book/companion/depth_experiments.py', '../../browser/viewer/view.js', '../../browser/viewer/shell.css']:
            with self.subTest(name=name):
                original = (self.here/name).read_bytes()
                (self.here/name).write_bytes(original+b' changed')
                with self.assertRaisesRegex(ValueError, 'Stale browser source bindings'):
                    verify_browser_qualification(self.repo)
                (self.here/name).write_bytes(original)

    def test_book_and_pdf_binding_changes_reject(self):
        self.chapter.write_text('# Revised')
        with self.assertRaisesRegex(ValueError, 'Stale browser book bindings'):
            verify_browser_qualification(self.repo)
        self.chapter.write_text('# Original')
        for pdf in [self.pdf, self.site_pdf]:
            original = pdf.read_bytes()
            pdf.write_bytes(b'changed PDF')
            with self.assertRaisesRegex(ValueError, 'Stale browser PDF binding'):
                verify_browser_qualification(self.repo)
            pdf.write_bytes(original)

    def test_added_book_source_rejects(self):
        self.chapter.with_name('02-new.md').write_text('# New')
        with self.assertRaisesRegex(ValueError, 'Stale browser book bindings'):
            verify_browser_qualification(self.repo)

    def test_incomplete_or_failed_retained_checks_reject(self):
        for key, value in [('labs', []), ('webgl', False), ('page_errors', ['error']), ('http_failures', ['404']), ('teaching', {}), ('transport', {})]:
            receipt = self.receipt.copy()
            receipt[key] = value
            self.receipt_path.write_text(json.dumps(receipt))
            with self.assertRaises(ValueError):
                verify_browser_qualification(self.repo)

    def test_check_mode_does_not_write_or_requalify(self):
        retained = [self.pdf, self.here/'pdf-inputs.json', self.receipt_path]
        before = [p.read_bytes() for p in retained]
        with patch.object(verify_browser, 'HERE', self.here), patch.object(verify_browser, 'exercise_browser', return_value=self.receipt) as run, patch.object(verify_browser, 'write_receipt') as write:
            verify_browser.verify(base_url='http://unused.invalid')
        run.assert_called_once_with('http://unused.invalid', False)
        write.assert_not_called()
        self.assertEqual([p.read_bytes() for p in retained], before)

    def test_check_mode_cannot_bless_stale_pdf(self):
        self.chapter.write_text('# Revised')
        before = self.receipt_path.read_bytes()
        with patch.object(verify_browser, 'HERE', self.here), patch.object(verify_browser, 'exercise_browser') as run, patch.object(verify_browser, 'write_receipt') as write:
            with self.assertRaisesRegex(ValueError, 'Stale PDF inputs'):
                verify_browser.verify(base_url='http://unused.invalid')
        run.assert_not_called()
        write.assert_not_called()
        self.assertEqual(self.receipt_path.read_bytes(), before)


if __name__ == '__main__':
    unittest.main()
