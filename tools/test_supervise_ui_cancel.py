"""No numerical/native work: cap/identity contracts and owned waiting-process teardown.

Kernel cgroup/tmpfs enforcement is not emulated as an integration pass. All own
fixture files remain under /tmp/rheon-ui-supervisor-tests-*; no cleanup occurs.
"""
import errno
import hashlib
import json
import os
from pathlib import Path
import signal
import tempfile
import unittest
from unittest.mock import patch

import supervise_ui_cancel as s


class ContractTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="rheon-ui-supervisor-tests-"))

    def test_caps_reject_unlimited_cpu_memory_swap_and_pids(self):
        good = {"cpu.max": "10000 10000", "cpu.max.burst": "0", "memory.max": str(512*s.MIB),
                "memory.swap.max": "0", "pids.max": "16"}
        s.check_cgroup_values(good)
        for key, value in [("cpu.max", "max 10000"), ("cpu.max.burst", "1"),
                           ("memory.max", "max"), ("memory.swap.max", "1"), ("pids.max", "17")]:
            with self.subTest(key=key), self.assertRaises(s.AdmissionError):
                s.check_cgroup_values(dict(good, **{key: value}))

    def test_ordinary_files_cannot_impersonate_cgroup_or_bounded_volume(self):
        with self.assertRaises(s.AdmissionError):
            s.Cgroup(self.root)
        with self.assertRaises(s.AdmissionError):
            s.output_volume(self.root)

    def test_output_pipe_capacity_has_an_actual_kernel_bound(self):
        read_fd, write_fd = os.pipe()
        try:
            s.fcntl.fcntl(write_fd, s.F_SETPIPE_SZ, s.PIPE_BYTES)
            self.assertEqual(s.fcntl.fcntl(write_fd, s.F_GETPIPE_SZ), 4096)
        finally:
            os.close(read_fd)
            os.close(write_fd)

    def test_per_process_limits_are_applied_before_exec(self):
        with patch.object(s.resource, "setrlimit") as call, \
             patch.object(s.resource, "getrlimit", return_value=(s.resource.RLIM_INFINITY, s.resource.RLIM_INFINITY)):
            s.child_limits()
        self.assertEqual(len(call.call_args_list), 5)
        call.assert_any_call(s.resource.RLIMIT_CPU, (5, 6))
        call.assert_any_call(s.resource.RLIMIT_AS, (1024*s.MIB, 1024*s.MIB))
        call.assert_any_call(s.resource.RLIMIT_FSIZE, (8*s.MIB, 8*s.MIB))
        call.assert_any_call(s.resource.RLIMIT_CORE, (0, 0))

    def test_stricter_inherited_limits_are_never_raised(self):
        with patch.object(s.resource, "getrlimit", return_value=(1, 2)), \
             patch.object(s.resource, "setrlimit") as call:
            s.child_limits()
        call.assert_any_call(s.resource.RLIMIT_CPU, (1, 2))
        call.assert_any_call(s.resource.RLIMIT_AS, (1, 2))
        call.assert_any_call(s.resource.RLIMIT_CORE, (0, 0))

    def test_binary_copy_is_immutable_and_growth_or_memfd_failure_is_refused(self):
        repo = self.root / "repo"
        repo.mkdir()
        (repo / "Cargo.lock").write_bytes(b"locked fixture")
        binary = self.root / "binary"
        raw = b"\x7fELFexplicit inert bytes; never executed"
        binary.write_bytes(raw)
        seal = {"test": s.TEST, "toolchain": "1.92.0", "commit": "a"*40, "tree": "b"*40,
                "cargo_lock_sha256": hashlib.sha256(b"locked fixture").hexdigest(),
                "binary": str(binary), "binary_sha256": hashlib.sha256(raw).hexdigest(), "binary_bytes": len(raw)}
        seal["supervisor_sha256"] = hashlib.sha256(Path(s.__file__).read_bytes()).hexdigest()
        path = self.root / "seal.json"
        path.write_text(json.dumps(seal))
        def git(_repo, *args):
            return {("status", "--porcelain", "--untracked-files=all"): "",
                    ("rev-parse", "HEAD"): "a"*40, ("rev-parse", "HEAD^{tree}"): "b"*40}[args]
        with patch.object(s, "git", git):
            _, fd = s.seal_inputs(repo, path)
            try:
                with self.assertRaises(OSError) as raised:
                    os.write(fd, b"altered")
                self.assertEqual(raised.exception.errno, errno.EPERM)
                binary.write_bytes(b"changed input pathname after sealing")
                self.assertEqual(os.read(fd, len(raw)), raw)
            finally:
                os.close(fd)
            binary.write_bytes(raw)
            with patch.object(s.os, "memfd_create", side_effect=OSError("fixture allocation refusal")):
                with self.assertRaises(OSError):
                    s.seal_inputs(repo, path)
            with patch.object(s.os, "read", side_effect=[raw, b"growing input", b""]):
                with self.assertRaisesRegex(s.AdmissionError, "grew"):
                    s.seal_inputs(repo, path)

    def test_exit_zero_or_incomplete_receipt_cannot_pass(self):
        seal = {"commit": "a"*40, "binary_sha256": "c"*64}
        with self.assertRaises(s.AdmissionError):
            s.result_complete(self.root, seal)
        value = {"contract": "genuine-cancel-v1", "test": s.TEST, "source": "a"*40,
                 "binary_sha256": "c"*64, "gate_held": True, "ui_cancel": True, "joined": True,
                 "integrity_verified": True, "cancelled": True, "progress": 1,
                 "record_count": 2, "accepted_generation": 1, "requested_complete": False}
        (self.root / "result.json").write_text(json.dumps(value))
        self.assertEqual(s.result_complete(self.root, seal), value)
        for key, bad in [("record_count", 3), ("ui_cancel", False), ("binary_sha256", "d"*64),
                         ("requested_complete", True), ("test", "some_other_test")]:
            (self.root / "result.json").write_text(json.dumps(dict(value, **{key: bad})))
            with self.subTest(key=key), self.assertRaises(s.AdmissionError):
                s.result_complete(self.root, seal)

    def test_owned_non_numerical_process_gets_term_then_kill_and_is_reaped(self):
        read_fd, write_fd = os.pipe()
        pid = os.fork()
        if pid == 0:
            os.close(read_fd)
            signal.signal(signal.SIGTERM, signal.SIG_IGN)
            os.write(write_fd, b"R")
            os.close(write_fd)
            while True:
                signal.pause()
        os.close(write_fd)
        fd = os.pidfd_open(pid)
        events = []
        class OwnedProcessFixture:
            def populated(self):
                return Path(f"/proc/{pid}").exists()
            def term(self):
                events.append("TERM")
                signal.pidfd_send_signal(fd, signal.SIGTERM)
            def kill(self):
                events.append("KILL")
                signal.pidfd_send_signal(fd, signal.SIGKILL)
        try:
            self.assertEqual(os.read(read_fd, 1), b"R")
            result = s.teardown(OwnedProcessFixture(), pid, fd, None, None, None)
            self.assertEqual(events, ["TERM", "KILL"])
            self.assertTrue(result["child_reaped"] and result["cgroup_empty"])
            (self.root / "owned-process-result.json").write_text(json.dumps(result))
        finally:
            os.close(read_fd)
            try:
                signal.pidfd_send_signal(fd, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                os.waitpid(pid, 0)
            except ChildProcessError:
                pass
            finally:
                os.close(fd)

    def test_pre_go_receipt_failure_closes_gate_reaps_child_and_restores_handlers(self):
        # Fault-injected admission only: no executable or numerical code is run.
        fake_path = self.root / "leaf"
        fake_path.mkdir()
        instances = []
        class WaitingChildGroup:
            values = {"fixture": "NOT kernel containment evidence"}
            initial_cpu = 0
            path = fake_path
            fd = None
            pid = None
            def __init__(self, _path):
                instances.append(self)
            def admit(self, pid):
                self.pid = pid
                self.fd = os.pidfd_open(pid)
            def populated(self):
                return self.pid is not None and Path(f"/proc/{self.pid}").exists()
            def cpu_usec(self):
                return 0
            def term(self):
                signal.pidfd_send_signal(self.fd, signal.SIGTERM)
            def kill(self):
                signal.pidfd_send_signal(self.fd, signal.SIGKILL)
        real_write = s.write_receipt
        writes = []
        def fail_before_go(fd, value):
            writes.append(value["stage"])
            if len(writes) == 2:
                raise OSError("injected pre-GO receipt failure")
            real_write(fd, value)
        binary_fd = os.memfd_create("inert-never-executed", os.MFD_CLOEXEC)
        before = {sig: signal.getsignal(sig) for sig in [signal.SIGTERM, signal.SIGINT]}
        try:
            with patch.object(s, "Cgroup", WaitingChildGroup), \
                 patch.object(s, "output_volume", return_value=self.root), \
                 patch.object(s, "seal_inputs", return_value=({"binary": "never-executed"}, binary_fd)), \
                 patch.object(s, "write_receipt", fail_before_go):
                with self.assertRaisesRegex(OSError, "pre-GO"):
                    s.supervise(self.root, self.root, self.root, fake_path)
            value = json.loads((self.root / "run/supervisor.json").read_text())
            self.assertFalse(value["accepted"])
            self.assertTrue(value["child_reaped"] and value["cgroup_empty"])
            self.assertIn("pre-GO", value["reason"])
            self.assertEqual(before, {sig: signal.getsignal(sig) for sig in before})
        finally:
            for instance in instances:
                if instance.fd is not None:
                    os.close(instance.fd)

    def test_echild_without_pidfd_never_signals_a_recycled_numeric_pid(self):
        events = []
        class LostOwnership:
            def populated(self):
                return True
            def term(self):
                events.append("TERM")
            def kill(self):
                events.append("KILL")
        with patch.object(s.os, "waitpid", side_effect=ChildProcessError("ECHILD")), \
             patch.object(s.os, "kill") as numeric_signal, patch.object(s.time, "sleep"), \
             patch.object(s.time, "monotonic", side_effect=range(100)):
            result = s.teardown(LostOwnership(), 12345, None, None, 0.0, None)
        numeric_signal.assert_not_called()
        self.assertEqual(events, ["KILL"])
        self.assertFalse(result["child_reaped"])

    def test_cleanup_does_not_restart_expired_term_or_kill_deadlines(self):
        for kill_at in [None, 25.0]:
            events = []
            class ExpiredDeadline:
                def populated(self):
                    return True
                def term(self):
                    events.append("TERM")
                def kill(self):
                    events.append("KILL")
            def sleep(_seconds):
                self.assertIn("KILL", events, "expired TERM grace must not restart")
                self.assertIsNone(kill_at, "expired post-KILL observation must not restart")
            with patch.object(s.os, "waitpid", side_effect=ChildProcessError("ECHILD")), \
                 patch.object(s.signal, "pidfd_send_signal"), patch.object(s.time, "sleep", sleep), \
                 patch.object(s.time, "monotonic", side_effect=range(100, 200)):
                result = s.teardown(ExpiredDeadline(), 12345, 9, None, 24.0, kill_at, 25.0)
            self.assertEqual(events, ["KILL"])
            self.assertFalse(result["child_reaped"] or result["cgroup_empty"])

    def test_receipt_short_writes_are_completed_before_fsync(self):
        fd = os.open(self.root / "receipt.json", os.O_CREAT | os.O_RDWR, 0o600)
        original = os.pwrite
        def short_write(target, data, offset):
            return original(target, data[:97], offset)
        try:
            with patch.object(s.os, "pwrite", short_write):
                s.write_receipt(fd, {"accepted": False, "reason": "fixture"})
            self.assertEqual(os.fstat(fd).st_size, 65536)
            self.assertEqual(json.loads((self.root / "receipt.json").read_text())["reason"], "fixture")
        finally:
            os.close(fd)

    def test_unreadable_population_and_reap_error_do_not_skip_kill(self):
        events = []
        class BrokenObservation:
            def populated(self):
                raise OSError("injected events read error")
            def term(self):
                events.append("TERM")
                raise OSError("injected TERM refusal")
            def kill(self):
                events.append("KILL")
        with patch.object(s.os, "waitpid", side_effect=ChildProcessError("injected reap error")), \
             patch.object(s.signal, "pidfd_send_signal"), patch.object(s.time, "sleep"), \
             patch.object(s.time, "monotonic", side_effect=range(100)):
            result = s.teardown(BrokenObservation(), 12345, 9, None, None, None)
        self.assertEqual(events, ["TERM", "KILL"])
        self.assertFalse(result["child_reaped"] or result["cgroup_empty"])
        self.assertTrue(result["teardown_errors"])


if __name__ == "__main__":
    unittest.main()
