"""Immutable baseline retrieval must reject corruption before comparison."""
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import historical_baselines as archive


class BaselineRetrieval(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.history = archive.historical_baselines()

    def setUp(self):
        self.config = json.loads((archive.HERE/'baseline-sources.json').read_text())
        self.payloads = {name: (self.history/name).read_bytes()
                         for name in self.config['files']}
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.here = self.root/'companion'
        self.here.mkdir()
        (self.here/'baseline-sources.json').write_text(json.dumps(self.config))

    def download(self, url, **kwargs):
        prefix = ('https://raw.githubusercontent.com/'+self.config['repository']
                  +'/'+self.config['commit']+'/'+self.config['path']+'/')
        self.assertTrue(url.startswith(prefix))
        return io.BytesIO(self.payloads[url.removeprefix(prefix)])

    def retrieve(self):
        with mock.patch.object(archive, 'HERE', self.here), mock.patch.object(archive, 'ROOT', self.root), \
             mock.patch.object(archive.subprocess, 'run', return_value=mock.Mock(returncode=1)), \
             mock.patch.object(archive, 'urlopen', side_effect=self.download):
            return archive.historical_baselines()

    def test_shallow_archive_preserves_exact_bytes_and_commit(self):
        directory = self.retrieve()
        for name, raw in self.payloads.items():
            self.assertEqual((directory/name).read_bytes(), raw)

    def test_corrupt_existing_cache_rejects_without_replacement(self):
        directory = self.retrieve()
        target = directory/'results.json'
        target.write_bytes(target.read_bytes()+b'\n')
        before = target.read_bytes()
        with self.assertRaisesRegex(ValueError, 'identity mismatch'):
            self.retrieve()
        self.assertEqual(target.read_bytes(), before)

    def test_changed_archived_payload_is_never_published(self):
        self.payloads['results.json'] += b'\n'
        with self.assertRaisesRegex(ValueError, 'identity mismatch'):
            self.retrieve()
        self.assertFalse((self.root/'.generated/research-baselines'/self.config['commit']/'results.json').exists())

    def test_archive_failure_does_not_generate_expected_results(self):
        with mock.patch.object(archive, 'HERE', self.here), mock.patch.object(archive, 'ROOT', self.root), \
             mock.patch.object(archive.subprocess, 'run', return_value=mock.Mock(returncode=1)), \
             mock.patch.object(archive, 'urlopen', side_effect=OSError('archive unavailable')):
            with self.assertRaisesRegex(OSError, 'archive unavailable'):
                archive.historical_baselines()
        self.assertFalse((self.root/'.generated').exists())


if __name__ == '__main__':
    unittest.main()
