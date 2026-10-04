"""Source-only/data-only and retained browser qualification rejection fixtures."""
from pathlib import Path
import hashlib
import json
import tempfile
import unittest
from unittest.mock import patch
import build
from browser_qualification import LABS, REVIEWED_SOURCES, book_sources, source_hashes, verify_browser_qualification
from pdf_freshness import input_hashes
import verify_browser


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


class BrowserBindings(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name)
        self.here = self.repo/'docs/education'
        self.here.mkdir(parents=True)
        for name in set(REVIEWED_SOURCES) | {'pdf_freshness.py'}:
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
                        'page_errors': [], 'http_failures': []}
        self.receipt_path = self.here/'browser-qualification.json'
        self.receipt_path.write_text(json.dumps(self.receipt))
        (self.here/'pdf-inputs.json').write_text(json.dumps({'schema': 'rheon-pdf-inputs-v1',
            'inputs': input_hashes(self.repo), 'pdf_sha256': self.receipt['pdf_sha256']}))

    def test_current_bindings_pass(self):
        verify_browser_qualification(self.repo)

    def test_lab_and_renderer_source_only_changes_reject(self):
        for name in ['labs.js', 'verify_browser.py']:
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
        for key, value in [('labs', []), ('webgl', False), ('page_errors', ['error']), ('http_failures', ['404'])]:
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
