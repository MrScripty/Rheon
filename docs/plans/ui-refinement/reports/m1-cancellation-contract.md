# M1 bounded cancellation contract — no live admission

The implementation is a **conditional, unexecuted genuine-test contract**, not a
cancellation result. Production UI is unchanged from native-validated `426fd25`.
This continuation starts at `e1097581d751c061eaf8c2df45d83de62911a74d` on new local
branch `implementation/ui-m1-cancel-contract-20261020`. The original local branch
is preserved; parent reports remote `implementation/ui-hierarchy-20261020` now
points to `426fd25` after separate user action. No fetch/push/reset/rebase occurs.

## Exact execution and identity

Only `setup_ui_tests::bounded_genuine_cancel_contract` may execute, via
`tools/supervise_ui_cancel.py`. It is `#[ignore]`, excluded from `ui_setup_`, and
requires runtime admission/root plus a compiled source revision before
`Worker::start`. Direct `cargo test --ignored` or invented environment values are
not an admitted execution route. The supervisor flag is acknowledgement of a
separate authorization, not authorization by itself. No numerical/model test was
run in this turn; no runtime provisioning or global settings were changed.

After this checkpoint is committed, use its exact full commit/tree and clean
tracked source. Build with existing trusted Rust/Cargo 1.92.0, locked dependencies,
explicit existing tool homes and a fresh target outside every checkout. Set
`RHEON_SOURCE_REVISION` to that exact commit and use only:
`cargo test --locked --features desktop --bin rheon-desktop --no-run --message-format=json`.
Record the actual emitted executable, complete build log/tool versions and
Cargo.lock SHA256. This builds tests without running them. The ordinary qualified
application-binary sidecar does not automatically qualify a test binary.

Create an external JSON seal with fields `test` (exact name above), `toolchain`
(`1.92.0`), full `commit`, full `tree`, `cargo_lock_sha256`, absolute `binary`,
`binary_bytes`, `binary_sha256`, and `supervisor_sha256` (exact script bytes).
The supervisor compares clean git identity/lock/script, hashes and copies the
actual ELF into a write/grow/shrink/seal-locked memfd, then executes those exact
immutable bytes, not a re-resolved path. Input <=512 MiB; growth during copy is
refused. The memfd allocation belongs to the supervisor before child admission;
its <=512 MiB input memory is **outside** the child's memory cgroup limit.
The compiled revision is a caller-supplied declaration checked by the test,
not hermetic build attestation. Preserve the original sealed build artifact/log.

## Mandatory preconfigured resources — fail before GO if unavailable

The execution owner must already have separate authority/custody for:

| Boundary | Enforced contract |
| --- | --- |
| Process subtree | Genuine cgroup-v2 dedicated leaf, initially empty, no child groups, excludes supervisor; write rights to `cgroup.procs`/`cgroup.kill`; at GO membership exactly the owned waiting PID |
| Exclusive custody | Leaf and output filesystem reserved externally for this supervisor's entire lifetime; no concurrent admission, reconfiguration, remount or other users' work. Empty-at-preflight is not a lease |
| CPU | Preconfigured `cpu.max=10000 10000`, `cpu.max.burst=0`: at most one CPU-equivalent bandwidth. Inherited per-process RLIMIT_CPU soft/hard at most5/6 s. Aggregate `cpu.stat` 5s is a **monitored trip**, not an exact hard cumulative CPU budget |
| Memory/processes | Preconfigured `memory.max` <=512 MiB, `memory.swap.max=0`, `pids.max` <=16 including threads. Inherited RLIMIT_AS at most1 GiB/process, NOFILE at most64, CORE0 |
| Individual file | Inherited RLIMIT_FSIZE at most8 MiB/process |
| Aggregate retained storage | Empty dedicated tmpfs mount root, `noswap`, kernel capacity <=32 MiB, <=1024 inodes. Includes logs, receipts, TMPDIR, SQLite database/journal; no shared-filesystem free-space substitute. This bounds **allocated storage**, not sparse logical lengths or cumulative rewritten bytes |
| Captured output | Each stdout/stderr file retains <=1 MiB (2 MiB combined). Each pipe is kernel-sized4096 bytes; read at most remaining+1 then close on overflow, with no continued drain during grace. Observed counts retained; trusted test never resizes pipes. OS-accepted pipe bytes are bounded by 2 MiB +8194 bytes across both pipes, not arbitrary application strings constructed in memory |
| Wall/disposition | Start clock before fork; TERM by 24s observation, at most 1s grace, subtree KILL at 25s deadline; monitor polls20ms. These are supervisor kill-disposition deadlines subject to OS scheduling, not a guarantee a kernel D-state task retires by25s |

Stricter inherited rlimits are retained, never raised; effective values are
recorded in the receipt. Such stricter limits may refuse/fail the test without
any budget increase.

Linux semantics: [cgroup v2](https://docs.kernel.org/admin-guide/cgroup-v2.html)
defines bandwidth/memory/subtree control;
[getrlimit](https://man7.org/linux/man-pages/man2/getrlimit.2.html) defines inherited
per-process limits; [tmpfs](https://docs.kernel.org/filesystems/tmpfs.html) defines
allocated capacity/inode/noswap bounds. No claim that polling disk sizes or
per-file limits alone enforce a hard aggregate quota.

This is containment of a trusted pinned test with reviewed output paths, not a
hostile-code/filesystem/network sandbox. Cgroup kill includes detached process
groups while they remain in its subtree; the test does not migrate cgroups or
write outside its root. Cwd/HOME/TMPDIR alone do not restrict filesystem access.
The supervisor never mounts, provisions, raises limits, changes credentials or
network settings, deletes evidence, or signals a broad PID/process-group match.
If the dedicated leaf/volume cannot be supplied, live execution remains blocked;
do not substitute weaker containment or increase limits to make it pass.

## Actual test inputs and observations

One native carrier track: JacobiPcgV1, 4×4×4 cells, 1 m cube, zero origin, density
1 kg/m³, two requested steps, dt0.02 s, tight=false, threads1, observe=false;
material/emitter/collision/model/configured/saved-set absent. Existing ordinary
smoke source semantics remain, so this is real numerical work. Pressure uses
unchanged relative1e-9, absolute1e-12, predicted divergence1e-7, actual1e-5,
Courant1 and max2000 iterations/solve. No larger grid, extra restart, comparison,
producer, corpus or scientific experiment is included.

Preview: exact edge4, one track,16 MiB, at most3 frame slots. Recording: one
fresh `run/recordings` directory,8 MiB database,1 MiB publication,max_records3.
Rollback journal may use roughly another database plus overhead, within the
kernel output volume; these are component limits, not process RSS. No preserved
483 evidence files or three original databases are inputs or targets.

1. Validate admission and compiled revision before starting one actual Worker.
2. Reuse existing ProgressGate at first acknowledged accepted step. Poll
   `try_recv`, render the real sidebar and yield while waiting, <=10s observation;
   never block UI on recv/join. Assert progress1/2 and still-live handle.
3. While the gate holds the Worker, send real egui pointer press/release to
   **Cancel simulation**. Assert actual shared cancel flag and held handle;
   preserve the reached event, then release gate. No direct substitute call to
   `app.cancel` in the test.
4. Keep painting while observing `is_finished`, <=10s. Poll/join only after true.
   Require no completed result rows/textures, progress1, cancelled status.
5. Open only the new recording: verify integrity,2 publications (constructor +
   first accepted step), accepted_generation1 and terminal outcome cancelled.
   Record requested_complete=false separately from integrity_verified=true.

`events.jsonl` is synced at reached stages. A bounded new `result.json` contains
source/binary identity, gate/UI/join and retained-prefix facts. Supervisor success
requires exit0 **and** that exact complete contract result; exit0 with zero tests
or a missing/incomplete result fails. This is genuine worker/egui interaction
when eventually run, still not native rendered Cancel/focus verification.

## Teardown and evidence

Child waits on a private GO pipe until membership and receipt are confirmed.
Limits are applied before exec. Parent TERM/INT set a stop flag without throwing
across fork/FD ownership changes; controlled cleanup closes GO, attempts TERM,
then cgroup.kill/SIGKILL only for the admitted subtree or its exact unreaped
owned child. pidfds avoid recycled-PID signaling. All normal/exception paths
require both `cgroup.events populated=0` and direct-child reaping before success.
Normal SIGCHLD ownership is required before fork. Read/reap/TERM errors are
recorded conservatively and do not skip KILL attempts; ECHILD suppresses any
numeric-PID fallback after ownership is lost.

Observe cleanup for2s after KILL. If retirement/emptiness cannot be established,
mark accepted=false and teardown unresolved, retain the owned cgroup/volume and
escalate to its authorized owner; never broaden killing or claim success.
Existing absolute TERM/KILL/observation deadlines are reused during exception
cleanup; no second grace or post-KILL wait is added. Catchable supervisor
interrupts enter cleanup. Uncatchable supervisor SIGKILL,
host failure, lost cgroup authority or uninterruptible kernel tasks are outside
a userspace teardown guarantee; resource containment remains dependent on kernel
and reserved-resource custody. Do not silently turn these into a successful
hard-wall or no-leftover claim.

A preallocated64 KiB supervisor receipt preserves final errors even after child
ENOSPC; finalization attempts remaining descriptor release and restores handlers
even on receipt failure. Logs, partial recordings, stage events and all input
seals remain; no retry or cleanup. Output tmpfs is volatile: its owner must retain
it/copy authorized evidence before unmount/reboot; no automatic transfer occurs.

## Workload-free checks in this turn

Long-status egui stress covers original long path, a16 KiB unbroken token and
multiline Unicode/combining text at both logical sizes, full exact copy/end
navigation/edit refusal, pointer-acquired focus and Shift+Tab exit. These remain
headless tests and do not close native accessibility/usability acceptance.
Supervisor unit checks cover cap refusal, ordinary filesystem rejection, actual
immutable memfd refusal, missing/incomplete result, owned non-numerical waiting
process TERM→KILL/reap, injected observation/reap failures and pre-GO receipt
failure cleanup/handler restoration. Mocked cgroups are explicitly not kernel
containment integration evidence. No genuine ignored case is run here.
