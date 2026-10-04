import json
import tempfile
import time
import unittest
from pathlib import Path
from rheon_compare import Comparison, implementations

BINARY=Path('target/release/rheon').resolve()
class ComparisonTest(unittest.TestCase):
    def finish(self,job):
        deadline=time.monotonic()+20
        while job.poll() in ('running','cancelling'):
            if time.monotonic()>deadline:
                job.cancel()
                self.fail('comparison exceeded contract-test deadline')
            time.sleep(.005)
        return job.status
    def test_real_comparison_and_preserved_outputs(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'comparison'
            methods=[m['id'] for m in implementations(BINARY)]
            job=Comparison(BINARY,p,methods,size=4,steps=2,repeats=2)
            self.assertEqual(self.finish(job),'completed')
            result=json.loads((p/'comparison.json').read_text())
            self.assertEqual(len(result['runs']),6)
            self.assertTrue(result['equal_accepted_time'])
            self.assertEqual([s['samples'] for s in result['summary']],[2,2])
            with self.assertRaises(FileExistsError): Comparison(BINARY,p,methods,size=4,steps=2)
            self.assertEqual(json.loads((p/'status.json').read_text())['status'],'completed')
    def test_cancel_before_spawn_and_during_real_work(self):
        with tempfile.TemporaryDirectory() as root:
            for before in [True,False]:
                p=Path(root)/str(before)
                job=Comparison(BINARY,p,['jacobi-pcg-v1'],size=32,steps=100)
                if not before:job.poll()
                job.cancel()
                self.assertEqual(self.finish(job),'cancelled')
                self.assertIsNone(job.process)
                self.assertFalse((p/'comparison.json').exists())
    def test_invalid_selection_creates_no_output(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'invalid'
            with self.assertRaises(ValueError):Comparison(BINARY,p,['unregistered'])
            self.assertFalse(p.exists())

if __name__=='__main__': unittest.main()
