"""Clean source-tree policy: output ignores and necessary inputs stay explicit."""
import ast
import fnmatch
import json
from pathlib import Path
import re
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]

FIELD_KEYS = {'pressure', 'velocity', 'fraction', 'mass', 'volume', 'density',
              'liquid_mass', 'liquid_volume', 'kinetic_energy', 'energy_before',
              'energy_after', 'final_time', 'accepted_time', 'residual_max',
              'true_residual_l2', 'true_residual_max', 'predicted_divergence_max'}
RECORDED_FIELDS = re.compile(
    r'\b(?:(?:Pressure|Transport|Liquid|Velocity|Projection|Step|Momentum)\w*Report\s*\{|(?:' + '|'.join(sorted(FIELD_KEYS))
    + r'|reported_z|reconstructed_z|divergence_max|momentum_residual)'
    + r'\s*[:=]\s*\[?\s*[-+]?(?:\d|\.\d))')


def measurements(value, key=''):
    if isinstance(value, dict):
        return any(measurements(v, k) for k, v in value.items())
    if isinstance(value, list):
        return any(measurements(v, key) for v in value)
    if isinstance(value, str):
        if RECORDED_FIELDS.search(value):
            return True
        if key in FIELD_KEYS:
            try:
                float(value)
                return True
            except ValueError:
                pass
        if value.lstrip().startswith(('{', '[')):
            try:
                return measurements(json.loads(value), key)
            except json.JSONDecodeError:
                pass
    # Command durations and hosted CI timing are provenance, not fields.
    return ((isinstance(value, float) and key != 'elapsed_seconds')
            or (type(value) is int and key in FIELD_KEYS))

class RepositoryHygiene(unittest.TestCase):
    def test_tracked_tree_has_no_bulk_outputs_or_pdfs(self):
        paths = subprocess.check_output(['git', 'ls-files'], cwd=ROOT, text=True).splitlines()
        forbidden = []
        for name in paths:
            path = Path(name)
            if path.suffix == '.pdf' or (name.startswith('evidence/') and path.suffix in
                    {'.csv','.jsonl','.log','.png','.jpg','.jpeg','.gif','.html','.zip','.gz'}):
                forbidden.append(name)
            if name.startswith(('docs/research-book/figures/',
                                'docs/research-book/expansion/figures/')):
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
                    'docs/research-book/companion/reproduction/haswell-openblas-0.3.30.json',
                    'evidence/probe/plot.jpg',
                    'docs/research-book/figures/pressure-residual.svg',
                    'docs/research-book/expansion/figures/projection.jpg',
                    'docs/research-book/expansion/contact/_plot_cache/fontlist.json']
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
        paths = subprocess.check_output(['git', 'ls-files', 'evidence/*.json',
                                         'docs/research-book/evidence/*.json'],
                                        cwd=ROOT, text=True).splitlines()
        for name in paths:
            if name == 'evidence/coupled-terminal-main/receipt.json':
                value = json.loads((ROOT/name).read_text())
                value.pop('hosted_log_span_seconds', None)
            else:
                value = json.loads((ROOT/name).read_text())
            self.assertFalse(measurements(value), name)

    def test_receipt_scan_rejects_encoded_outputs_and_allows_provenance(self):
        outputs = [
            {'paired_probes': {'before': {'result': 'Err(IterationLimit { iterations: 120, residual_max: 6e-6 })'}}},
            {'result': 'Ok(PressureReport { iterations: 35, true_residual_l2: 1.7e-9 })'},
            {'actual_result': 'parameter=0.5 reported_z=0 reconstructed_z=1\nassertion failed'},
            {'pressure': '0.5'}, {'mass': 100},
            {'stdout': '{"velocity": [0, 0, 0]}'},
        ]
        for value in outputs:
            with self.subTest(value=value):
                self.assertTrue(measurements(value))
        self.assertFalse(measurements({'source_sha256': '1'*64, 'run_exit': 101,
                                      'elapsed_seconds': .5,
                                      'compiler': 'Rust 1.92.0',
                                      'command': 'probe --relative-residual=1e-11'}))

    def test_pages_triggers_cover_current_publication_sources(self):
        workflow = (ROOT/'.github/workflows/research-book-pages.yml').read_text()
        # Companion programs and pins execute during publication generation.
        inputs = ['docs/research-book/companion/'+name for name in
                  ['experiments.py', 'depth_experiments.py', 'make_figures.py',
                   'requirements.txt']]
        for event in ['pull_request', 'push']:
            section = re.split(r'\n  [a-z_]+:', workflow.split(
                '  '+event+':\n', 1)[1], maxsplit=1)[0]
            patterns = ast.literal_eval(re.search(r'paths:\s*(\[[^\n]+\])', section)[1])
            for name in inputs:
                self.assertTrue(any(fnmatch.fnmatchcase(name, p) for p in patterns),
                                event+': '+name)

    def test_audit_records_original_identities_without_payloads(self):
        audit = json.loads((ROOT/'docs/generated-output-audit.json').read_text())
        self.assertEqual(audit['schema'], 'rheon-generated-output-audit-v1')
        self.assertEqual(sum(len(row[2]) for row in audit['content']), audit['removed_files'])
        self.assertEqual(sum(row[1]*len(row[2]) for row in audit['content']),audit['removed_bytes'])
        self.assertTrue(all(len(row[0])==64 and row[1]>=0 for row in audit['content']))
        self.assertGreater(audit['removed_bytes'], 160_000_000)

if __name__ == '__main__':
    unittest.main()
