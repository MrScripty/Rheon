"""Exercise composition preservation and refusal without numerical providers."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import compose_viewer as c


class Composition(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name); self.book = self.root / 'book'; self.book.mkdir()
        self.viewer = self.root / 'viewer'; self.viewer.mkdir(); self.out = self.root / 'out'
        self.raw = {'index.html': b'historical book', 'proof-qualification.json': b'{"source":"original"}'}
        for name, raw in self.raw.items(): (self.book / name).write_bytes(raw)
        hashes = {name: hashlib.sha256(raw).hexdigest() for name, raw in self.raw.items()}
        self.digest = hashlib.sha256(json.dumps(hashes, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        p = patch.object(c, 'BOOK_DIGEST', self.digest); p.start(); self.addCleanup(p.stop)

    def test_preserves_original_bytes_and_links_separate_workspace(self):
        with patch.object(c, 'inventory'), patch.object(c, 'publish', return_value={'rheon_head': '1'*40}):
            result = c.compose(self.book, self.viewer, self.out)
        for name, raw in self.raw.items():
            self.assertEqual((self.out/name).read_bytes(), raw)
            self.assertEqual((self.book/name).read_bytes(), raw)
        self.assertEqual(result['historical_book_content_sha256'], self.digest)
        self.assertFalse(result['new_research_qualification'])
        self.assertIn('src="viewer/index.html"', (self.out/'workspace.html').read_text())
        with self.assertRaisesRegex(ValueError, 'fresh'): c.compose(self.book, self.viewer, self.out)

    def test_changed_book_or_invalid_viewer_refuses_before_output(self):
        (self.book/'index.html').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'exact reviewed'): c.compose(self.book, self.viewer, self.out)
        self.assertFalse(self.out.exists())
        (self.book/'index.html').write_bytes(self.raw['index.html'])
        with patch.object(c, 'inventory', side_effect=ValueError('invalid viewer')):
            with self.assertRaisesRegex(ValueError, 'invalid viewer'): c.compose(self.book, self.viewer, self.out)
        self.assertFalse(self.out.exists())

    def test_symlink_refuses(self):
        (self.book/'link').symlink_to(self.book/'index.html')
        with self.assertRaisesRegex(ValueError, 'symlink'): c.book_inventory(self.book)

if __name__ == '__main__': unittest.main()
