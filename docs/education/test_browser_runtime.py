"""Real Chromium publication rejection probes; run after building/qualifying _site."""
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch
from playwright.sync_api import Page
import verify_browser

HERE = Path(__file__).resolve().parent


class BrowserPublication(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.site = Path(self.tmp.name)/'site'
        shutil.copytree(HERE/'_site', self.site)
        self.retained = [HERE/'downloads/Rheon-expanded-book.pdf', HERE/'_site/downloads/Rheon-expanded-book.pdf',
                         HERE/'pdf-inputs.json', HERE/'browser-qualification.json']
        self.before = [p.read_bytes() for p in self.retained]
        self.addCleanup(self.check_retained)

    def check_retained(self):
        self.assertEqual([p.read_bytes() for p in self.retained], self.before)

    def verify(self):
        with verify_browser.serve_site(self.site) as url, patch.object(Page, 'pdf', side_effect=AssertionError('Publication must never render a PDF')):
            verify_browser.verify(base_url=url)

    def test_current_six_labs_pass_without_pdf_or_receipt_writes(self):
        self.verify()

    def test_lab_javascript_syntax_error_blocks_publication(self):
        with (self.site/'labs.js').open('a') as out:
            out.write('\nthis is deliberately invalid javascript !!!\n')
        with self.assertRaisesRegex(RuntimeError, 'Browser page errors'):
            self.verify()

    def test_http_failure_blocks_publication(self):
        with (self.site/'index.html').open('a') as out:
            out.write('<img src="missing-browser-probe.png">')
        with self.assertRaisesRegex(RuntimeError, 'Browser network failures.*404'):
            self.verify()

    def test_mobile_page_error_blocks_publication(self):
        with (self.site/'index.html').open('a') as out:
            out.write('<script>if(innerWidth<400) throw Error("mobile-only probe");</script>')
        with self.assertRaisesRegex(RuntimeError, 'Browser page errors.*mobile-only probe'):
            self.verify()

    def test_unresponsive_controls_block_publication(self):
        with (self.site/'labs.js').open('a') as out:
            out.write('\nfor (const event of ["input","change"]) document.querySelector("#controls").addEventListener(event, e=>e.stopImmediatePropagation(), true);\n')
        with self.assertRaisesRegex(RuntimeError, 'Controls did not change metrics'):
            self.verify()


if __name__ == '__main__':
    unittest.main()
