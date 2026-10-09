import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from build import external

class OutputBoundary(unittest.TestCase):
    def test_checkout_relative_symlink_and_inherited_git_overrides(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td); repo = root / 'repository'; repo.mkdir()
            subprocess.run(['git', 'init', '-q', str(repo)], check=True)
            for path in [repo / 'generated', repo / 'nested/generated']:
                with self.assertRaisesRegex(ValueError, 'outside Git'): external(path)
            alias = root / 'alias'; alias.symlink_to(repo, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, 'outside Git'): external(alias / 'generated')
            with patch.dict(os.environ, {'GIT_DIR': str(repo / '.git'), 'GIT_WORK_TREE': str(repo)}):
                self.assertEqual(external(root / 'outside'), root / 'outside')
                with self.assertRaises(ValueError): external(repo / 'generated')
            self.assertFalse((repo / 'generated').exists())

    def test_uncertain_probe_refuses(self):
        with tempfile.TemporaryDirectory() as td:
            with patch('build.subprocess.run', return_value=subprocess.CompletedProcess([], 128, '', 'permission denied')):
                with self.assertRaisesRegex(ValueError, 'uncertain'): external(Path(td) / 'output')

if __name__ == '__main__': unittest.main()
