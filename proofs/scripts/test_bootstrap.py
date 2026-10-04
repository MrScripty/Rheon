"""Exercise the actual workflow shell with offline download/execution stubs."""
import os
from pathlib import Path
import re
import subprocess
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[2]


class ElanIntegrity(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        archive = os.environ.get('ELAN_TEST_ARCHIVE')
        if archive is None:
            raise unittest.SkipTest('ELAN_TEST_ARCHIVE must name the official downloaded archive')
        cls.payload = Path(archive).read_bytes()
        workflow = (ROOT / '.github/workflows/lean-rheon.yml').read_text()
        match = re.search(r'- name: Install official pinned elan.*?run: \|\n(.*?)      - name:',
                          workflow, re.S)
        if match is None:
            raise ValueError('Workflow bootstrap step missing')
        cls.script = textwrap.dedent(match.group(1))

    def test_correct_archive_reaches_extraction_and_installer(self):
        self.exercise(self.payload, accepted=True)

    def test_corrupted_archive_never_reaches_extraction_or_installer(self):
        payload = bytearray(self.payload)
        payload[len(payload) // 2] ^= 1
        self.exercise(payload, accepted=False)

    def exercise(self, payload, *, accepted):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive, bin_dir = root / 'input.tar.gz', root / 'bin'
            archive.write_bytes(payload)
            bin_dir.mkdir()
            stubs = {
                'curl': '#!/bin/bash\nfor arg in "$@"; do output="$arg"; done\ncp "$BOOTSTRAP_INPUT" "$output"\n',
                'tar': '''#!/bin/bash
touch "$RUNNER_TEMP/extracted"
cat > "$RUNNER_TEMP/elan-init" <<'SH'
#!/bin/bash
touch "$RUNNER_TEMP/executed"
SH
chmod +x "$RUNNER_TEMP/elan-init"
''',
            }
            for name, code in stubs.items():
                path = bin_dir / name
                path.write_text(code)
                path.chmod(0o700)
            environment = os.environ | {'PATH': str(bin_dir) + os.pathsep + os.environ['PATH'],
                                        'BOOTSTRAP_INPUT': str(archive), 'RUNNER_TEMP': str(root),
                                        'GITHUB_PATH': str(root / 'github-path')}
            result = subprocess.run(['bash', '-c', self.script], env=environment,
                                    text=True, capture_output=True, timeout=10)
            self.assertEqual(result.returncode == 0, accepted, result.stdout + result.stderr)
            self.assertEqual((root / 'extracted').exists(), accepted)
            self.assertEqual((root / 'executed').exists(), accepted)
            self.assertIn('OK' if accepted else 'FAILED', result.stdout)


if __name__ == '__main__':
    unittest.main()
