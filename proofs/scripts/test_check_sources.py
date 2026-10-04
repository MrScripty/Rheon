"""Source/pin gate rejection tests, including actual optimized subprocesses."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class SourceGate(unittest.TestCase):
    def run_gate(self, root, optimized):
        return subprocess.run([sys.executable, *(['-O'] if optimized else []),
                               str(root / 'scripts/check_sources.py')],
                              text=True, capture_output=True, timeout=10)

    def test_reviewed_sources_pass_in_both_modes(self):
        for optimized in (False, True):
            with self.subTest(optimized=optimized):
                result = self.run_gate(ROOT, optimized)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn('PASS exact reviewed', result.stdout)

    def test_changed_sources_and_pins_fail_in_both_modes(self):
        def replace(root, name, before, after):
            path = root / name
            path.write_text(path.read_text().replace(before, after))
        def change_transitive(root):
            path = root / 'lake-manifest.json'
            data = json.loads(path.read_text())
            next(p for p in data['packages'] if p['name'] != 'mathlib')['rev'] = 'branch-name'
            path.write_text(json.dumps(data))
        changes = (
            lambda root: (root / 'Rheon/Discrete.lean').write_text('-- changed\n'),
            lambda root: (root / 'Rheon/New.lean').write_text('-- extra\n'),
            lambda root: (root / 'Rheon/Indexing.lean').unlink(),
            lambda root: (root / 'lean-toolchain').write_text('leanprover/lean4:v4.20.0\n'),
            lambda root: replace(root, 'lakefile.toml', 'c44e0c8ee63ca166450922a373c7409c5d26b00b', '0' * 40),
            lambda root: replace(root, 'lake-manifest.json', 'c44e0c8ee63ca166450922a373c7409c5d26b00b', '1' * 40),
            lambda root: replace(root, 'lake-manifest.json', 'c44e0c8ee63ca166450922a373c7409c5d26b00b', 'branch-name'),
            change_transitive,
        )
        for index, change in enumerate(changes):
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory) / 'proofs'
                shutil.copytree(ROOT, root, ignore=shutil.ignore_patterns('.lake', '__pycache__'))
                change(root)
                for optimized in (False, True):
                    with self.subTest(change=index, optimized=optimized):
                        result = self.run_gate(root, optimized)
                        self.assertNotEqual(result.returncode, 0)
                        self.assertNotIn('PASS exact reviewed', result.stdout)


if __name__ == '__main__':
    unittest.main()
