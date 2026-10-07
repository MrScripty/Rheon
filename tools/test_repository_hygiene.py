"""Clean source-tree policy: output ignores and necessary inputs stay explicit."""
import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]

class RepositoryHygiene(unittest.TestCase):
    def test_tracked_tree_has_no_bulk_outputs_or_pdfs(self):
        paths = subprocess.check_output(['git', 'ls-files'], cwd=ROOT, text=True).splitlines()
        forbidden = []
        for name in paths:
            path = Path(name)
            if path.suffix == '.pdf' or (name.startswith('evidence/') and path.suffix in
                    {'.csv','.jsonl','.log','.png','.gif','.html','.zip','.gz'}):
                forbidden.append(name)
        self.assertEqual(forbidden, [])
        for name in ['proofs/Rheon/Physics.lean', 'proofs/lake-manifest.json', 'LICENSE',
                     'docs/research-book/companion/experiments.py',
                     'docs/research-book/companion/baseline-sources.json',
                     'docs/research-book/expansion/reference.py',
                     'docs/research-book/expansion/test_reference.py']:
            self.assertIn(name, paths)
        for name in ['docs/research-book/companion/results.json',
                     'docs/research-book/companion/depth-results.json',
                     'docs/research-book/companion/reproduction/haswell-openblas-0.3.30.json',
                     'docs/research-book/expansion/reference-data.json',
                     'docs/research-book/expansion/reference-qualification.json']:
            self.assertNotIn(name, paths)

    def test_precise_output_ignores_do_not_hide_sources(self):
        outputs = ['.generated/probe/frames.jsonl', 'rheon-output/steps.csv',
                   'rheon-demo/steps.csv',
                   'evidence/probe/cells.csv', 'evidence/probe/render.png',
                   'docs/education/downloads/new.pdf', 'docs/education/pdf-inputs.json',
                   'docs/education/browser-qualification.json']
        outputs += ['docs/research-book/expansion/reference-data.json',
                    'docs/research-book/expansion/reference-qualification.json',
                    'docs/research-book/companion/results.json',
                    'docs/research-book/companion/depth-results.json',
                    'docs/research-book/companion/reproduction/haswell-openblas-0.3.30.json']
        result = subprocess.run(['git', 'check-ignore', '--stdin'], cwd=ROOT,
                                input='\n'.join(outputs)+'\n', capture_output=True, text=True)
        self.assertEqual(set(result.stdout.splitlines()), set(outputs))
        sources = ['src/new.rs','proofs/New.lean','evidence/new/reproduce.py',
                   'evidence/new/requirements.txt','evidence/new/source-manifest.json',
                   'evidence/new/config.json','docs/new.md','tools/new.py']
        sources.append('docs/research-book/companion/baseline-sources.json')
        result = subprocess.run(['git','check-ignore','--stdin'],cwd=ROOT,
                                input='\n'.join(sources)+'\n',capture_output=True,text=True)
        self.assertEqual(result.stdout, '')

    def test_retained_receipts_do_not_embed_simulation_measurements(self):
        paths = subprocess.check_output(['git', 'ls-files', 'evidence/*.json'],
                                        cwd=ROOT, text=True).splitlines()
        def measurements(value, key=''):
            if isinstance(value, dict):
                return any(measurements(v, k) for k, v in value.items())
            if isinstance(value, list):
                return any(measurements(v, key) for v in value)
            # Command durations and hosted CI timing are provenance, not fields.
            return isinstance(value, float) and key != 'elapsed_seconds'
        for name in paths:
            if name == 'evidence/coupled-terminal-main/receipt.json':
                value = json.loads((ROOT/name).read_text())
                value.pop('hosted_log_span_seconds', None)
            else:
                value = json.loads((ROOT/name).read_text())
            self.assertFalse(measurements(value), name)

    def test_audit_records_original_identities_without_payloads(self):
        audit = json.loads((ROOT/'docs/generated-output-audit.json').read_text())
        self.assertEqual(audit['schema'], 'rheon-generated-output-audit-v1')
        self.assertEqual(sum(len(row[2]) for row in audit['content']), audit['removed_files'])
        self.assertEqual(sum(row[1]*len(row[2]) for row in audit['content']),audit['removed_bytes'])
        self.assertTrue(all(len(row[0])==64 and row[1]>=0 for row in audit['content']))
        self.assertGreater(audit['removed_bytes'], 160_000_000)

if __name__ == '__main__':
    unittest.main()
