"""Resource guards tested with synthetic Python processes; NEVER any solver."""
import sys
sys.dont_write_bytecode=True
import hashlib,json,os,re,resource,signal,subprocess,tempfile,time,unittest
from pathlib import Path
import flat_wall_campaign_guard as guard
from flat_wall_force_source import write as write_source
FIXTURE=Path(__file__).parent/'flat_wall_guard_tests/synthetic_child.py'

class GuardTests(unittest.TestCase):
    def setUp(self):
        retained=os.environ.get('RHEON_GUARD_TEST_EVIDENCE_ROOT')
        if retained:
            base=Path(retained).resolve()
            if base.is_relative_to(Path(__file__).resolve().parents[1]):raise ValueError('test evidence outside Git required')
            self.root=base/self._testMethodName;self.root.mkdir(parents=True,exist_ok=False)
        else:
            self.temporary=tempfile.TemporaryDirectory();self.addCleanup(self.temporary.cleanup)
            self.root=Path(self.temporary.name)
    def run_case(self,mode,seconds=2,rss=256*1024*1024,members=2):
        out=self.root/mode;begin=time.monotonic()
        child=subprocess.run([sys.executable,'-B',str(FIXTURE),'supervisor',mode,str(out),str(seconds),str(rss),str(members)],
                             stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=8)
        elapsed=time.monotonic()-begin
        receipt=json.loads((out/'guard-receipt.json').read_text()) if (out/'guard-receipt.json').exists() else None
        if receipt:
            self.assertFalse(receipt['surviving_live_members'])
            self.assertLessEqual((out/'controller.log').stat().st_size,guard.LOG_CAP)
        return child,receipt,out,elapsed
    def assert_stopped(self,pid):
        try:info=guard.proc_info(pid)
        except FileNotFoundError:return
        self.assertEqual(info['state'],'Z',f'process {pid} still running')
    def test_quota_partition_is_exact_and_limits_validated(self):
        self.assertEqual(sum(guard.QUOTAS.values()),4*1024*1024)
        for limits in (guard.Limits(seconds=.1,cleanup=.2),guard.Limits(sample=float('nan')),guard.Limits(rss=0)):
            with self.assertRaises(guard.GuardViolation):limits.validate()
    def test_writer_refuses_before_exceeding_reserved_slot(self):
        p=self.root/'output'
        with guard.BoundedWriter(p,32) as stream:
            stream.write(b'x'*19)
            with self.assertRaises(guard.GuardViolation):stream.write(b'y'*20)
        self.assertEqual(p.read_bytes(),b'x'*19)
        with self.assertRaises(FileExistsError):guard.write_reserved(p,b'z',32)
    def test_forcing_file_prewrite_cap_and_original_bytes(self):
        p=self.root/'too-small'
        with self.assertRaises(ValueError):write_source(p,output_byte_cap=16)
        self.assertFalse(p.exists())
        p=self.root/'force';receipt=write_source(p)
        self.assertEqual(receipt['sha256'],'d5054d1f5564b69ae118cad0232bf13fd6caa452d26d850e00cfd68adab1775b')
        self.assertLessEqual(p.stat().st_size,guard.SOURCE_CAP)
    def test_success_includes_supervisor_controller_and_child_rss(self):
        child,r,out,_=self.run_case('success')
        self.assertEqual(child.returncode,0,child.stderr.decode());self.assertEqual(r['status'],'completed')
        self.assertGreaterEqual(r['rss_sampled_peak_bytes'],guard.proc_info(os.getpid())['rss']//2)
        self.assertFalse(r['continuous_rss_bound']);self.assertGreater(r['samples'],0)
        self.assertEqual((out/'n6.log').read_text(),'synthetic child\n')
    def test_aggregate_deadline_covers_controller_hang(self):
        child,r,out,elapsed=self.run_case('hang',seconds=.6)
        self.assertNotEqual(child.returncode,0);self.assertIn('aggregate work deadline',r['reason'])
        self.assertLess(elapsed,1.5);self.assert_stopped(json.loads((out/'campaign-report.json').read_text())['controller_pid'])
    def test_aggregate_deadline_covers_postprocessing(self):
        child,r,_,elapsed=self.run_case('postprocess',seconds=.8)
        self.assertNotEqual(child.returncode,0);self.assertIn('aggregate work deadline',r['reason']);self.assertLess(elapsed,1.5)
    def test_child_deadline_covers_git_like_hanging_command(self):
        child,r,out,_=self.run_case('child_timeout')
        self.assertNotEqual(child.returncode,0)
        self.assertTrue('supervised child deadline' in r['reason'] or 'child/controller deadline' in (out/'controller.log').read_text())
    def test_child_deadline_covers_blocked_spawn(self):
        child,r,out,elapsed=self.run_case('spawn_hang')
        self.assertNotEqual(child.returncode,0);self.assertIn('supervised child deadline including spawn',r['reason'])
        self.assertLess(elapsed,1.);self.assert_stopped(json.loads((out/'campaign-report.json').read_text())['controller_pid'])
    def test_controller_exec_delay_is_supervised_with_known_pid(self):
        child,r,_,elapsed=self.run_case('startup_delay',seconds=.7)
        self.assertNotEqual(child.returncode,0);self.assertIn('aggregate work deadline',r['reason'])
        self.assertLess(elapsed,1.5);self.assert_stopped(r['controller_pid'])
    def test_delayed_private_session_setup_is_admitted_without_root_group_signal(self):
        child,r,_,_=self.run_case('session_delay')
        self.assertEqual(child.returncode,0);self.assertEqual(r['status'],'completed')
    def test_cancellation_pending_during_fork_identity_assignment(self):
        child,r,_,_=self.run_case('startup_cancel')
        self.assertNotEqual(child.returncode,0);self.assertIn('supervisor cancellation signal',r['reason'])
        self.assert_stopped(r['controller_pid'])
    def test_raw_fork_child_is_cleaned_when_controller_constructor_fails(self):
        child,r,_,_=self.run_case('constructor_fault')
        self.assertNotEqual(child.returncode,0);self.assertIn('MemoryError',r['reason'])
        self.assertTrue(r['owned_controller_reaped']);self.assert_stopped(r['controller_pid'])
    def test_partial_constructor_close_fault_restores_alarm_mask(self):
        child,r,_,_=self.run_case('partial_constructor_fault')
        self.assertNotEqual(child.returncode,0);self.assertIn('partial constructor failure',r['reason'])
        self.assertTrue(r['owned_controller_reaped']);self.assert_stopped(r['controller_pid'])
        self.assertTrue(r['guard_signals_unblocked_during_receipt']);self.assertTrue(r['cleanup_signalling_errors'])
    def test_raw_fork_child_is_cleaned_when_pidfd_allocation_fails(self):
        child,r,_,_=self.run_case('pidfd_fault')
        self.assertNotEqual(child.returncode,0);self.assertIn('pidfd allocation failure',r['reason'])
        self.assertTrue(r['owned_controller_reaped']);self.assert_stopped(r['controller_pid'])
    def test_private_session_failure_text_stays_in_reserved_log(self):
        child,r,out,_=self.run_case('setsid_fault')
        self.assertNotEqual(child.returncode,0);self.assertEqual(child.stderr,b'')
        self.assertIn('controller exec failed',(out/'controller.log').read_text())
    def test_blocked_alarm_and_nondefault_child_handler_refuse_before_spawn(self):
        old_mask=signal.pthread_sigmask(signal.SIG_BLOCK,{signal.SIGALRM})
        try:
            with self.assertRaisesRegex(guard.GuardViolation,'unblocked'):
                guard.supervise(['never-run'],self.root/'blocked')
        finally:signal.pthread_sigmask(signal.SIG_SETMASK,old_mask)
        old_handler=signal.signal(signal.SIGCHLD,signal.SIG_IGN)
        try:
            with self.assertRaisesRegex(guard.GuardViolation,'SIGCHLD'):
                guard.supervise(['never-run'],self.root/'reaped')
        finally:signal.signal(signal.SIGCHLD,old_handler)
        self.assertFalse((self.root/'blocked').exists());self.assertFalse((self.root/'reaped').exists())
    def cancellation(self,signum):
        out=self.root/'cancellation'
        child=subprocess.Popen([sys.executable,'-B',str(FIXTURE),'supervisor','hang',str(out),'3',str(256*1024*1024),'2'],
                               stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        until=time.monotonic()+1.5
        while not (out/'campaign-report.json').exists():
            if time.monotonic()>=until:child.kill();self.fail('synthetic controller did not start')
            time.sleep(.01)
        pid=json.loads((out/'campaign-report.json').read_text())['controller_pid']
        child.send_signal(signum);child.communicate(timeout=2)
        r=json.loads((out/'guard-receipt.json').read_text())
        self.assertNotEqual(child.returncode,0);self.assertIn('supervisor cancellation signal',r['reason'])
        self.assertFalse(r['surviving_live_members']);self.assert_stopped(pid)
    def test_sigterm_cleans_owned_group_and_refuses(self):self.cancellation(signal.SIGTERM)
    def test_sigint_cleans_owned_group_and_refuses(self):self.cancellation(signal.SIGINT)
    def test_child_stdout_stderr_log_is_bounded_during_writes(self):
        child,r,out,_=self.run_case('log_flood')
        self.assertNotEqual(child.returncode,0);self.assertIn('output write quota',(out/'controller.log').read_text())
        self.assertLessEqual((out/'n6.log').stat().st_size,guard.LOG_CAP)
    def test_controller_log_is_bounded_during_writes(self):
        child,r,out,_=self.run_case('controller_flood')
        self.assertNotEqual(child.returncode,0);self.assertIn('output write quota',r['reason'])
        self.assertLessEqual((out/'controller.log').stat().st_size,guard.LOG_CAP)
    def test_git_like_captured_output_has_its_own_cap(self):
        child,r,out,_=self.run_case('capture_flood')
        self.assertNotEqual(child.returncode,0);self.assertIn('captured child output quota',(out/'controller.log').read_text())
    def test_sampled_memory_threshold_includes_child_allocation(self):
        child,r,_,_=self.run_case('memory',seconds=3,rss=64*1024*1024)
        self.assertNotEqual(child.returncode,0);self.assertIn('sampled whole-group RSS threshold',r['reason'])
        self.assertGreater(r['rss_sampled_peak_bytes'],64*1024*1024)
    def test_descendants_are_killed_when_aggregate_deadline_fires(self):
        child,r,out,_=self.run_case('descendant',seconds=.9,members=4)
        self.assertNotEqual(child.returncode,0);self.assertIn('aggregate work deadline',r['reason'])
        pid=int(re.search(r'DESCENDANT=(\d+)',(out/'n6.log').read_text()).group(1));self.assert_stopped(pid)
    def test_unexpected_extra_group_member_refuses(self):
        child,r,_,_=self.run_case('descendant',members=2)
        self.assertNotEqual(child.returncode,0);self.assertIn('member threshold',r['reason'])
    def test_observed_session_escape_refuses_and_kills_owned_child(self):
        child,r,out,_=self.run_case('escape')
        self.assertNotEqual(child.returncode,0);self.assertIn('escaped controller session/group',r['reason'])
        pid=int(re.search(r'ESCAPED=(\d+)',(out/'n6.log').read_text()).group(1));self.assert_stopped(pid)
    def test_outer_alarm_survives_blocked_watchdog(self):
        child,r,out,elapsed=self.run_case('alarm',seconds=.8)
        self.assertEqual(child.returncode,124);self.assertIsNone(r);self.assertLess(elapsed,1.5)
        self.assert_stopped(json.loads((out/'campaign-report.json').read_text())['controller_pid'])
    def test_local_file_backstop_does_not_change_parent_limits(self):
        before=resource.getrlimit(resource.RLIMIT_FSIZE)
        child,r,out,_=self.run_case('file')
        self.assertNotEqual(child.returncode,0);self.assertLessEqual((out/'n6/acquisition.txt').stat().st_size,guard.NATIVE_CAP)
        self.assertEqual(resource.getrlimit(resource.RLIMIT_FSIZE),before)
    def test_controller_refuses_unguarded_direct_invocation(self):
        with self.assertRaises(guard.GuardViolation):guard.controller_deadline()

if __name__=='__main__':unittest.main()
