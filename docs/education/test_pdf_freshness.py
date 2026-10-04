"""Regression: fresh HTML must not publish an old reading PDF."""
from pathlib import Path
import hashlib
import json
import tempfile
import unittest
import shutil
import subprocess
import sys
from pdf_freshness import input_hashes, verify_pdf, write_receipt


class PdfFreshness(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.repo = Path(self.tmp.name)
        self.here = self.repo / 'docs/education'
        self.chapter = self.repo / 'docs/research-book/chapters/01-example.md'
        self.figure = self.repo / 'docs/research-book/figures/example.svg'
        for path, content in [(self.chapter, '# Original manuscript'),
                              (self.figure, '<svg/>'),
                              *[(self.here/name, name) for name in
                                ['build.py', 'verify_browser.py', 'style.css', 'package-lock.json']]]:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        self.pdf = self.here / 'downloads/Rheon-expanded-book.pdf'
        self.pdf.parent.mkdir(); self.pdf.write_bytes(b'qualified PDF fixture')
        self.receipt = {'schema': 'rheon-pdf-inputs-v1',
                        'inputs': input_hashes(self.repo),
                        'pdf_sha256': hashlib.sha256(self.pdf.read_bytes()).hexdigest()}
        (self.here/'pdf-inputs.json').write_text(json.dumps(self.receipt))

    def test_unchanged_inputs_pass(self):
        verify_pdf(self.repo)

    def test_changed_manuscript_rejects_old_pdf_despite_new_html(self):
        self.chapter.write_text('# Revised manuscript')
        site=self.here/'_site';site.mkdir()
        (site/'chapter.html').write_text('<h1>Revised manuscript</h1>')
        with self.assertRaisesRegex(ValueError, 'Stale PDF inputs.*01-example.md'):
            verify_pdf(self.repo)
        self.assertEqual(self.pdf.read_bytes(), b'qualified PDF fixture')

    def test_added_and_removed_chapters_reject_old_pdf(self):
        extra=self.chapter.with_name('02-added.md');extra.write_text('# Added')
        with self.assertRaisesRegex(ValueError, '02-added.md'):verify_pdf(self.repo)
        extra.unlink();self.chapter.unlink()
        with self.assertRaisesRegex(ValueError, '01-example.md'):verify_pdf(self.repo)

    def test_changed_figure_or_pdf_is_rejected(self):
        self.figure.write_text('<svg>revised</svg>')
        with self.assertRaisesRegex(ValueError, 'example.svg'):verify_pdf(self.repo)
        self.figure.write_text('<svg/>');self.pdf.write_bytes(b'different PDF')
        with self.assertRaisesRegex(ValueError, 'PDF bytes differ'):verify_pdf(self.repo)

    def test_renderer_only_change_rejects_retained_pdf(self):
        (self.here/'verify_browser.py').write_text('changed print options')
        with self.assertRaisesRegex(ValueError, 'Stale PDF inputs.*verify_browser.py'):
            verify_pdf(self.repo)

    def test_stale_html_cannot_issue_fresh_receipt(self):
        site=self.here/'_site';site.mkdir()
        (site/'build-receipt.json').write_text(json.dumps({'pdf_inputs':self.receipt['inputs']}))
        self.chapter.write_text('# Revised manuscript')
        with self.assertRaisesRegex(ValueError, 'Rendered HTML has stale PDF inputs'):
            write_receipt(self.repo, input_hashes(self.repo))
        self.assertEqual(json.loads((self.here/'pdf-inputs.json').read_text()), self.receipt)

    def test_publication_entry_point_rejects_changed_manuscript_and_old_pdf(self):
        source=Path(__file__).resolve().parent
        for name in ['verify_site.py','pdf_freshness.py']:
            shutil.copy2(source/name,self.here/name)
        site=self.here/'_site';(site/'downloads').mkdir(parents=True)
        shutil.copy2(self.pdf,site/'downloads/Rheon-expanded-book.pdf')
        self.chapter.write_text('# Revised manuscript')
        (site/'chapter.html').write_text('<h1>Revised manuscript</h1>')
        result=subprocess.run([sys.executable,str(self.here/'verify_site.py')],
                              text=True,capture_output=True)
        self.assertNotEqual(result.returncode,0)
        self.assertIn('Stale PDF inputs:',result.stderr)
        self.assertIn('01-example.md',result.stderr)


if __name__ == '__main__':unittest.main()
