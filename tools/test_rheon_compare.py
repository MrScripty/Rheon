import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock
from pathlib import Path
from rheon_compare import Comparison, implementations

BINARY=Path('target/release/rheon').resolve()
class ComparisonTest(unittest.TestCase):
    def finish(self,job):
        deadline=time.monotonic()+20
        while job.poll() in ('running','cancelling','timing_out'):
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
            self.assertIsNone(result['environment']['run_timeout_seconds'])
            with self.assertRaises(FileExistsError): Comparison(BINARY,p,methods,size=4,steps=2)
            self.assertEqual(json.loads((p/'status.json').read_text())['status'],'completed')
    def test_cancel_before_spawn_and_during_real_work(self):
        with tempfile.TemporaryDirectory() as root:
            for before in [True,False]:
                p=Path(root)/str(before)
                job=Comparison(BINARY,p,['jacobi-pcg-v1'],size=32,steps=100,run_timeout=20)
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

    def test_explicit_dt_and_rejection_of_wrong_or_missing_manifest_dt(self):
        with tempfile.TemporaryDirectory() as root:
            for missing in (False, True):
                with self.subTest(missing=missing):
                    p=Path(root)/str(missing)
                    job=Comparison(BINARY,p,['jacobi-pcg-v1'],size=4,steps=2,repeats=1)
                    with mock.patch('rheon_compare.subprocess.Popen', wraps=subprocess.Popen) as spawn:
                        job.poll()
                        job.process.wait(timeout=10)
                        command=spawn.call_args.args[0]
                    # Check separate argv tokens, not only the unchanged binary default.
                    self.assertIn('--dt',command)
                    self.assertEqual(command[command.index('--dt')+1],'0.02')
                    manifest=job.run_dir/'run.json'
                    data=json.loads(manifest.read_text())
                    self.assertEqual(data['requested_dt'],0.02)
                    if missing: del data['requested_dt']
                    else: data['requested_dt']=0.04
                    manifest.write_text(json.dumps(data))
                    self.assertEqual(job.poll(),'failed')
                    self.assertEqual(job.index,1)
                    self.assertEqual(job.rows,[])
                    self.assertIsNone(job.process)
                    self.assertFalse((p/'comparison.json').exists())

    def test_invalid_deadlines_create_no_output(self):
        with tempfile.TemporaryDirectory() as root:
            for index,value in enumerate((0,-1,float('nan'),float('inf'),-float('inf'),True,'1')):
                with self.subTest(value=value):
                    p=Path(root)/str(index)
                    with self.assertRaises(ValueError):
                        Comparison(BINARY,p,['jacobi-pcg-v1'],run_timeout=value)
                    self.assertFalse(p.exists())

    @unittest.skipUnless(os.name=='posix','POSIX TERM/KILL behavior')
    def test_timeout_terminates_or_kills_then_reaps_without_more_jobs(self):
        real_spawn=subprocess.Popen
        with tempfile.TemporaryDirectory() as root:
            for ignore_term in (False,True):
                with self.subTest(ignore_term=ignore_term):
                    p=Path(root)/str(ignore_term)
                    job=Comparison(BINARY,p,['jacobi-pcg-v1'],run_timeout=2)
                    script='import signal,time\n'
                    if ignore_term:
                        script+="signal.signal(signal.SIGTERM,lambda *_: print('TERM received',flush=True))\n"
                    script+="print('measurement started',flush=True)\ntime.sleep(60)\n"
                    def spawn(_command,**kwargs):
                        return real_spawn([sys.executable,'-c',script],**kwargs)
                    with mock.patch('rheon_compare.subprocess.Popen',side_effect=spawn) as start:
                        job.poll()
                        child=job.process
                        try:
                            self.assertEqual(self.finish(job),'timed_out')
                            self.assertEqual(job.poll(),'timed_out')
                            self.assertEqual(start.call_count,1)
                            self.assertEqual(child.returncode,-signal.SIGKILL if ignore_term else -signal.SIGTERM)
                            self.assertIsNone(job.process)
                            self.assertIsNone(job.log)
                            self.assertEqual(job.rows,[])
                            status=json.loads((p/'status.json').read_text())
                            self.assertEqual(status['status'],'timed_out')
                            self.assertEqual(status['run_timeout_seconds'],2)
                            self.assertIn('deadline',status['error'])
                            self.assertFalse((p/'comparison.json').exists())
                            log=(p/'000-stderr.log').read_text()
                            self.assertIn('measurement started',log)
                            if ignore_term: self.assertIn('TERM received',log)
                        finally:
                            if child.poll() is None: child.kill()
                            child.wait(timeout=5)
                            if job.log is not None: job.log.close()

    def test_cli_deadline_is_explicit_and_recorded(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'cli'
            result=subprocess.run([sys.executable,'tools/rheon_compare.py','--binary',str(BINARY),
                                   '--output',str(p),'--size','4','--steps','2','--repeats','1',
                                   '--run-timeout','20'],capture_output=True,text=True,timeout=10)
            self.assertEqual(result.returncode,0,result.stderr)
            data=json.loads((p/'comparison.json').read_text())
            self.assertEqual(data['environment']['run_timeout_seconds'],20)
            self.assertEqual(data['workload']['requested_dt'],0.02)

    def test_cli_timeout_returns_failure_without_completion(self):
        with tempfile.TemporaryDirectory() as root:
            p=Path(root)/'timeout'
            result=subprocess.run([sys.executable,'tools/rheon_compare.py','--binary',str(BINARY),
                                   '--output',str(p),'--size','64','--steps','100','--repeats','1',
                                   '--run-timeout','0.001'],capture_output=True,text=True,timeout=10)
            self.assertEqual(result.returncode,1,result.stderr)
            self.assertEqual(json.loads(result.stdout)['status'],'timed_out')
            self.assertEqual(json.loads((p/'status.json').read_text())['completed_runs'],0)
            self.assertFalse((p/'comparison.json').exists())

if __name__=='__main__': unittest.main()
