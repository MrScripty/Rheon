#!/usr/bin/env python3
"""Admit one sealed, ignored numerical cancellation test; never build or provision.

Linux only. Requires separate live authority, an exclusive preconfigured cgroup
and an empty dedicated <=32MiB tmpfs. No fallback, mount, cleanup or limit changes.
This contains a trusted exact test, not hostile code with cgroup/filesystem access.
"""
import argparse
import ctypes
import fcntl
import hashlib
import json
import os
from pathlib import Path
import resource
import selectors
import signal
import stat
import subprocess
import sys
import time

# Linux UAPI <linux/fcntl.h>; some Python builds omit these named constants.
F_ADD_SEALS, F_GET_SEALS, IMMUTABLE_SEALS = 1033, 1034, 0x000F
F_SETPIPE_SZ, F_GETPIPE_SZ, PIPE_BYTES = 1031, 1032, 4096
TEST = "setup_ui_tests::bounded_genuine_cancel_contract"
MIB = 1024 * 1024
WALL = 25.0                 # KILL disposition deadline, not guaranteed D-state exit.
GRACE = 1.0
LOG_LIMIT = MIB             # Each of stdout/stderr; combined <=2MiB retained.
LIMITS = {"cpu_soft_s": 5, "cpu_hard_s": 6, "address_space_bytes": 1024 * MIB,
          "file_bytes": 8 * MIB, "open_files": 64, "output_volume_bytes": 32 * MIB,
          "memory_bytes": 512 * MIB, "pids": 16, "wall_s": WALL, "term_grace_s": GRACE}


class AdmissionError(ValueError):
    pass


def require(ok, message):
    if not ok:
        raise AdmissionError(message)


def encoded(value):
    return (json.dumps(value, sort_keys=True, indent=2) + "\n").encode()


def git(repo, *args):
    env = {"PATH": "/usr/bin:/bin", "LANG": "C", "GIT_CONFIG_NOSYSTEM": "1",
           "GIT_CONFIG_GLOBAL": os.devnull, "GIT_NO_REPLACE_OBJECTS": "1",
           "GIT_OPTIONAL_LOCKS": "0"}
    return subprocess.check_output(["git", "-c", "core.fsmonitor=false", "-C", str(repo),
                                    *args], env=env, timeout=10).decode().strip()


def mount_for(path):
    # Mountinfo escapes are deliberately refused: no guessed decoding/alias path.
    matches = []
    for line in Path("/proc/self/mountinfo").read_text().splitlines():
        before, after = line.split(" - ", 1)
        fields = before.split()
        mount = fields[4]
        if "\\" in mount:
            continue
        if path == Path(mount) or Path(mount) in path.parents:
            matches.append((len(mount), Path(mount), after.split()[0], after.split()[2]))
    require(matches, "no actual mount identity")
    return max(matches, key=lambda item: item[0])[1:]


def check_cgroup_values(values):
    require(values["cpu.max"].split() == ["10000", "10000"], "one-CPU cgroup quota required")
    require(values["cpu.max.burst"].strip() == "0", "CPU burst must be zero")
    for key, cap in [("memory.max", 512 * MIB), ("pids.max", 16)]:
        require(values[key].strip().isdigit() and 0 < int(values[key]) <= cap, key + " cap")
    require(values["memory.swap.max"].strip() == "0", "swap must be disabled in leaf")


class Cgroup:
    def __init__(self, path):
        self.path = path.resolve(strict=True)
        mount, kind, _ = mount_for(self.path)
        require(kind == "cgroup2" and self.path != mount, "dedicated cgroup-v2 leaf required")
        require(not any(p.is_dir() for p in self.path.iterdir()), "leaf has child cgroups")
        keys = ["cpu.max", "cpu.max.burst", "memory.max", "memory.swap.max", "pids.max"]
        self.values = {key: (self.path / key).read_text().strip() for key in keys}
        check_cgroup_values(self.values)
        require(not self.populated() and not self.members(), "cgroup must be exclusive and empty")
        require(all(os.access(self.path / key, os.W_OK) for key in ["cgroup.procs", "cgroup.kill"]),
                "leaf admission/kill rights unavailable")
        self.initial_cpu = self.cpu_usec()

    def members(self):
        return [int(p) for p in (self.path / "cgroup.procs").read_text().split()]

    def populated(self):
        return dict(line.split() for line in (self.path / "cgroup.events").read_text().splitlines())["populated"] != "0"

    def cpu_usec(self):
        return int(dict(line.split() for line in (self.path / "cpu.stat").read_text().splitlines())["usage_usec"])

    def admit(self, pid):
        (self.path / "cgroup.procs").write_text(str(pid))
        require(self.members() == [pid], "exclusive admitted leaf must contain only the owned child")

    def term(self):
        # pidfds avoid signaling a recycled PID outside this exclusive subtree.
        for pid in self.members():
            try:
                fd = os.pidfd_open(pid)
                try:
                    if pid in self.members():
                        signal.pidfd_send_signal(fd, signal.SIGTERM)
                finally:
                    os.close(fd)
            except ProcessLookupError:
                pass

    def kill(self):
        (self.path / "cgroup.kill").write_text("1")


def output_volume(path):
    path = path.resolve(strict=True)
    mount, kind, options = mount_for(path)
    require(path == mount and kind == "tmpfs", "dedicated tmpfs mount root required")
    require("noswap" in options.split(","), "dedicated output tmpfs must use noswap")
    require(not any(path.iterdir()), "output mount must be empty; never reuse/delete evidence")
    fs = os.statvfs(path)
    require(0 < fs.f_blocks * fs.f_frsize <= 32 * MIB, "hard <=32MiB output filesystem required")
    require(0 < fs.f_files <= 1024, "hard <=1024 output inodes required")
    return path


def seal_inputs(repo, seal_path):
    require(seal_path.stat().st_size <= 65536, "seal too large")
    seal = json.loads(seal_path.read_text())
    require(seal["test"] == TEST and seal["toolchain"] == "1.92.0", "wrong test/toolchain seal")
    require(hashlib.sha256(Path(__file__).read_bytes()).hexdigest() == seal["supervisor_sha256"],
            "supervisor source seal mismatch")
    require(not git(repo, "status", "--porcelain", "--untracked-files=all"), "source must be clean")
    require(git(repo, "rev-parse", "HEAD") == seal["commit"], "commit seal mismatch")
    require(git(repo, "rev-parse", "HEAD^{tree}") == seal["tree"], "tree seal mismatch")
    require(hashlib.sha256((repo / "Cargo.lock").read_bytes()).hexdigest() == seal["cargo_lock_sha256"],
            "lock seal mismatch")
    binary = Path(seal["binary"]).resolve(strict=True)
    source_fd = os.open(binary, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    fd = None
    try:
        fd = os.memfd_create("rheon-ui-cancel-sealed", os.MFD_ALLOW_SEALING | os.MFD_CLOEXEC)
        info = os.fstat(source_fd)
        require(stat.S_ISREG(info.st_mode) and 0 < info.st_size <= 512 * MIB, "512MiB binary input limit")
        digest = hashlib.sha256()
        copied = 0
        while block := os.read(source_fd, MIB):
            copied += len(block)
            require(copied <= info.st_size, "binary grew while sealing")
            digest.update(block)
            view = memoryview(block)
            while view:
                view = view[os.write(fd, view):]
        require(digest.hexdigest() == seal["binary_sha256"], "binary hash mismatch")
        require(os.fstat(fd).st_size == seal["binary_bytes"] == info.st_size, "binary size seal")
        fcntl.fcntl(fd, F_ADD_SEALS, IMMUTABLE_SEALS)
        require(fcntl.fcntl(fd, F_GET_SEALS) & IMMUTABLE_SEALS == IMMUTABLE_SEALS,
                "immutable executable seals unavailable")
        os.lseek(fd, 0, os.SEEK_SET)
        require(os.read(fd, 4) == b"\x7fELF", "compiled ELF test binary required")
        os.lseek(fd, 0, os.SEEK_SET)
        return seal, fd
    except BaseException:
        if fd is not None:
            os.close(fd)
        raise
    finally:
        os.close(source_fd)


def planned_limits():
    values = []
    for key, cap in [(resource.RLIMIT_CPU, (5, 6)),
                     (resource.RLIMIT_AS, (1024 * MIB, 1024 * MIB)),
                     (resource.RLIMIT_FSIZE, (8 * MIB, 8 * MIB)),
                     (resource.RLIMIT_NOFILE, (64, 64)), (resource.RLIMIT_CORE, (0, 0))]:
        inherited = resource.getrlimit(key)
        effective = tuple(bound if old == resource.RLIM_INFINITY else min(old, bound)
                          for old, bound in zip(inherited, cap))
        values.append((key, effective))
    return values


def child_limits():
    for key, value in planned_limits():
        resource.setrlimit(key, value)          # Never raise a stricter inherited limit.


def write_receipt(fd, value):
    raw = encoded(value)
    require(len(raw) <= 65536, "receipt size")
    view = memoryview(raw + b" " * (65536 - len(raw)))
    offset = 0
    while view:
        written = os.pwrite(fd, view, offset)
        require(written > 0, "receipt write made no progress")
        offset += written
        view = view[written:]
    os.fsync(fd)


def result_complete(root, seal):
    path = root / "result.json"
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 65536, "missing bounded result")
    value = json.loads(path.read_text())
    require(value.get("contract") == "genuine-cancel-v1" and value.get("source") == seal["commit"]
            and value.get("test") == TEST and value.get("binary_sha256") == seal["binary_sha256"], "result identity")
    for field in ["gate_held", "ui_cancel", "joined", "integrity_verified", "cancelled"]:
        require(value.get(field) is True, "incomplete result: " + field)
    require(value.get("progress") == 1 and value.get("record_count") == 2
            and value.get("accepted_generation") == 1 and value.get("requested_complete") is False,
            "retained-prefix/completeness result")
    return value


def teardown(group, pid, pidfd, status, term_at, kill_at, wall_deadline=None):
    """Bounded observation/kill disposition; never broaden beyond owned resources."""
    errors = []
    ownership_known = True
    def attempt(operation):
        try:
            return operation()
        except (OSError, ValueError, KeyError) as error:
            if len(errors) < 16:
                errors.append(str(error))
            return None
    def populated():
        value = attempt(group.populated)
        return True if value is None else value
    def reap():
        nonlocal status, ownership_known
        if status is None:
            try:
                value = os.waitpid(pid, os.WNOHANG)
                if value[0]:
                    status = value[1]
            except ChildProcessError as error:
                ownership_known = False
                if len(errors) < 16:
                    errors.append(str(error))
            except OSError as error:
                if len(errors) < 16:
                    errors.append(str(error))
    def signal_child(sig):
        if status is None:
            if pidfd is not None:
                signal.pidfd_send_signal(pidfd, sig)
            elif ownership_known:
                os.kill(pid, sig)  # Suppressed after ECHILD/ownership loss.
    reap()                                     # Establish ownership before any numeric-PID fallback.
    if status is None or populated():
        if term_at is None:
            term_at = time.monotonic()
            attempt(group.term)
            attempt(lambda: signal_child(signal.SIGTERM))
        grace = min(term_at + GRACE, wall_deadline if wall_deadline is not None else float("inf"))
        if kill_at is not None:
            grace = kill_at
        while time.monotonic() < grace:
            reap()
            if status is not None and not populated():
                break
            time.sleep(0.01)
        if status is None or populated():
            if kill_at is None:
                kill_at = time.monotonic()
            attempt(group.kill)
            attempt(lambda: signal_child(signal.SIGKILL))
        deadline = (kill_at if kill_at is not None else time.monotonic()) + 2.0
        while time.monotonic() < deadline:
            reap()
            if status is not None and not populated():
                break
            time.sleep(0.01)
    return dict(child_reaped=status is not None, child_status=status,
                cgroup_empty=not populated(), teardown_errors=errors)


def supervise(repo, seal_path, volume, group_path):
    require(signal.getsignal(signal.SIGCHLD) == signal.SIG_DFL,
            "normal SIGCHLD direct-child ownership required")
    require(sys.platform == "linux" and hasattr(os, "pidfd_open")
            and hasattr(signal, "pidfd_send_signal"), "Linux pidfd support required")
    fds, log_fds = set(), []
    state = {"limits": LIMITS, "stage": "preflight", "reason": None, "accepted": False}
    group = selector = pid = pidfd = child_status = receipt_fd = gate_w = None
    term_at = kill_at = None
    handlers, interrupted = {}, []
    def own(fd):
        fds.add(fd)
        return fd
    def close(fd):
        if fd in fds:
            fds.remove(fd)
            try:
                os.close(fd)
            except OSError as error:
                state["accepted"] = False
                state.setdefault("close_errors", []).append(str(error))
    def pipe():
        pair = os.pipe2(os.O_CLOEXEC)
        return tuple(own(fd) for fd in pair)
    try:
        group = Cgroup(group_path)              # Never provision/change any limits.
        volume = output_volume(volume)         # No mount/quota fallback.
        seal, binary_fd = seal_inputs(repo, seal_path)
        own(binary_fd)
        root = volume / "run"
        root.mkdir(mode=0o700)
        (root / "recordings").mkdir()
        (root / "tmp").mkdir()
        (root / "contract.json").write_bytes(encoded(dict(seal, limits=LIMITS)))
        receipt_fd = own(os.open(root / "supervisor.json", os.O_CREAT | os.O_EXCL | os.O_RDWR, 0o600))
        os.posix_fallocate(receipt_fd, 0, 65536)  # Reserve evidence even after ENOSPC.
        state.update(seal=seal, cgroup=str(group.path), cgroup_values=group.values,
                     effective_rlimits=planned_limits(), stage="prepared")
        write_receipt(receipt_fd, state)
        gate_r, gate_w = pipe()
        out_r, out_w = pipe()
        err_r, err_w = pipe()
        for fd in [out_w, err_w]:
            fcntl.fcntl(fd, F_SETPIPE_SZ, PIPE_BYTES)
            require(fcntl.fcntl(fd, F_GETPIPE_SZ) == PIPE_BYTES, "bounded output pipe unavailable")
        selector = selectors.DefaultSelector()
        def stop(number, _frame):
            interrupted.append(number)         # Never raise during fork/FD ownership changes.
        for sig in [signal.SIGTERM, signal.SIGINT]:
            handlers[sig] = signal.signal(sig, stop)
        started = time.monotonic()
        pid = os.fork()
        if pid == 0:
            try:
                os.setsid()
                libc = ctypes.CDLL(None, use_errno=True)
                if libc.prctl(1, signal.SIGKILL, 0, 0, 0) != 0:
                    os._exit(125)
                child_limits()
                null_fd = os.open(os.devnull, os.O_RDONLY | os.O_CLOEXEC)
                os.dup2(null_fd, 0)
                os.dup2(out_w, 1)
                os.dup2(err_w, 2)
                # Parent's write end must not keep this own GO pipe alive.
                os.close(gate_w)
                if os.read(gate_r, 1) != b"G":
                    os._exit(125)
                os.chdir(root)
                env = {"LANG": "C.UTF-8", "HOME": str(root), "TMPDIR": str(root / "tmp"),
                       "RHEON_UI_CANCEL_ROOT": str(root), "RHEON_UI_CANCEL_ADMISSION": "genuine-cancel-v1"}
                os.execve(binary_fd, [seal["binary"], "--exact", TEST, "--ignored", "--nocapture", "--test-threads=1"], env)
            except BaseException:
                os._exit(125)
        pidfd = own(os.pidfd_open(pid))
        close(gate_r)
        close(out_w)
        close(err_w)
        group.admit(pid)                        # Test cannot exec before membership confirmed.
        state["output_observed_bytes"] = {}
        for stream, name in [(out_r, "stdout.log"), (err_r, "stderr.log")]:
            os.set_blocking(stream, False)
            log_fd = own(os.open(root / name, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600))
            log_fds.append(log_fd)
            selector.register(stream, selectors.EVENT_READ, [log_fd, 0, name])
            state["output_observed_bytes"][name] = 0
        require(not interrupted, "supervisor interrupted before GO")
        require(not state.get("close_errors"), "control descriptor close failed before GO")
        state["stage"] = "admitted"
        write_receipt(receipt_fd, state)
        os.write(gate_w, b"G")
        close(gate_w)
        gate_w = None
        while True:
            require(not interrupted, "supervisor interrupted: " + str(interrupted))
            now = time.monotonic()
            if child_status is None:
                reaped, value = os.waitpid(pid, os.WNOHANG)
                if reaped:
                    child_status = value
            if child_status is not None and not group.populated() and not selector.get_map():
                break
            cpu = group.cpu_usec() - group.initial_cpu
            trigger = None
            if now - started >= WALL - GRACE:
                trigger = "wall deadline"
            elif cpu >= 5_000_000:
                trigger = "aggregate CPU observation trip"
            elif child_status is not None and group.populated():
                trigger = "direct child exited; terminate remaining descendants"
            if trigger and term_at is None:
                state["reason"] = trigger
                term_at = now
                group.term()
            if term_at is not None and kill_at is None and (now - term_at >= GRACE or now - started >= WALL):
                group.kill()
                kill_at = now
            if kill_at is not None and now - kill_at >= 2.0:
                raise AdmissionError("containment unresolved after KILL")
            for key, _ in selector.select(0.02):
                data = os.read(key.fd, min(65536, max(1, LOG_LIMIT - key.data[1] + 1)))
                if not data:
                    selector.unregister(key.fd)
                    close(key.fd)
                    continue
                log_fd, count, name = key.data
                kept = memoryview(data[:max(0, LOG_LIMIT - count)])
                while kept:
                    kept = kept[os.write(log_fd, kept):]
                key.data[1] += len(data)
                state["output_observed_bytes"][name] = key.data[1]
                if key.data[1] > LOG_LIMIT:
                    selector.unregister(key.fd)
                    close(key.fd)              # No continued drain/flood during TERM grace.
                    if term_at is None:
                        state["reason"] = "captured output limit"
                        term_at = time.monotonic()
                        group.term()
        state.update(stage="finished", elapsed_s=time.monotonic() - started,
                     cpu_usec=group.cpu_usec() - group.initial_cpu)
        require(child_status == 0 and state["reason"] is None, "bounded test failed/terminated")
        state["result"] = result_complete(root, seal)
        require(not state.get("close_errors"), "control descriptor close failure")
        state["accepted"] = True               # Final teardown may still revoke acceptance.
    except BaseException as error:
        state["reason"] = type(error).__name__ + ": " + str(error)[:4096]
        raise
    finally:
        try:
            close(gate_w)
            if pid is not None:
                final = teardown(group, pid, pidfd, child_status, term_at, kill_at, started + WALL)
                state.update(final)
                if not final["child_reaped"] or not final["cgroup_empty"] or final["teardown_errors"]:
                    state["accepted"] = False
                    state["reason"] = "teardown unresolved; preserve owned leaf/evidence and escalate"
            if selector is not None:
                selector.close()
            for fd in log_fds:
                try:
                    os.fsync(fd)
                except OSError as error:
                    state["accepted"] = False
                    state.setdefault("log_errors", []).append(str(error))
            for fd in list(fds - {receipt_fd}):
                close(fd)
            if receipt_fd is not None:
                write_receipt(receipt_fd, state)
        finally:
            # Even receipt/close errors cannot skip remaining FD release/handler restoration.
            for fd in list(fds):
                close(fd)
            for sig, handler in handlers.items():
                signal.signal(sig, handler)
    return state


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--seal", type=Path, required=True)
    parser.add_argument("--output-volume", type=Path, required=True)
    parser.add_argument("--cgroup", type=Path, required=True)
    parser.add_argument("--admit-genuine-cancellation", action="store_true", required=True,
                        help="Execution authority must be granted separately; this flag is not approval.")
    args = parser.parse_args()
    try:
        value = supervise(args.repo.resolve(), args.seal.resolve(), args.output_volume, args.cgroup)
        require(value["accepted"], "final teardown/evidence check failed")
        print(json.dumps(value, sort_keys=True))
    except (AdmissionError, OSError, ValueError, subprocess.SubprocessError) as error:
        print("UI cancellation contract refused/failed: " + str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
