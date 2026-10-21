# T5 bounded worker validation contract — prepared, NOT EXECUTED

This amendment supersedes the earlier direct-loop/1GiB proposal reviewed at5eab3ac.
Two ignored bodies now exist in desktop/force_runtime_integration_tests.rs; list-only
compilation enumerates201 tests including their exact names below. No numerical,
SQLite, Worker or model workload has executed in this preparation slice. Source
review, final seals and separate execution admission must precede any invocation.
Production authored-force launches stay gated until runtime and relevant native
acceptance pass. Do not treat these source/compile receipts as runtime acceptance.

## Scope and custody

Canonical backend ancestor43ec505218c43594641ecbb80873bec8c88d7d2c; fetched backend
a4326a78a747bc868874fd16e434d780fd5a6d35 only changes its admission documents, not
core code. UI owns Request/worker dispatch/Inspector wrappers/frozen desktop intent;
backend owns physics and accepted recording custody. No legacy hash migration.

Both tests call real Worker::start, not an alternate direct solver loop. It validates
Request, creates bounded in-memory history, starts the existing one Worker thread
and a scoped Rayon pool with threads1. Libtest also uses threads1; main/test/Worker/
Rayon are the4 intended thread roles here, not a measured or OS-enforced thread cap. No new process/model/provider/emitter,
collision mesh, native window/GPU, observer, progress gate or failure gate exists.
The helper joins the worker before outcome assertions; normal/error/panic outcomes
therefore leave no detached numerical worker. External process timeout bounds a
blocked join. No second worker, baseline run or external retry is included.

## Exact future execution and process bounds

Execute each exact ignored name ONCE, serially, from the sealed test binary:

```
SEALED_BINARY force_runtime_integration_tests::native_force_dispatch_records_and_cold_binds_v2 --exact --ignored --test-threads=1 --nocapture
SEALED_BINARY force_runtime_integration_tests::material_force_dispatch_records_immutable_scene --exact --ignored --test-threads=1 --nocapture
```

These lines are the contract, not commands run by this task. No substring/broad
filter, fallback suite, tuning or automatic retry. Seal final source commit/tree,
affected source manifest, test bytes, Cargo.lock, Rust/feature configuration and
binary SHA256 before execution. The earlier197/199-name binaries lack these cases.

Use the execution owner's established disposable-child supervision, with no new
cgroup/mount/permission/network prerequisite:

- wall timeout deadline30s, CPU20s, address space512MiB, captured combined log1MiB;
- child per-file limit1MiB and core0; one owned process/session group, TERM at wall
  expiry then KILL after1s if needed, wait/reap and inspect owned-process disposition;
  the30s deadline excludes teardown latency and is not a hard reap guarantee;
- one fresh canonical absolute empty DATA root per case; supervisor stdout/stderr,
  receipts and seals are in a SEPARATE capture directory, never pre-created in DATA;
- set RHEON_UI_FORCE_TEST_ROOT, TMPDIR and SQLITE_TMPDIR to that same DATA root;
  the helper checks all three. No temp fallback or production recording directory;
- preserve root and capture directory on success, assertion, panic, timeout, signal
  or uncertain outcome. No Drop deletion, remove_dir_all or test-success teardown;
- after child exit, the supervisor inventories/hashes all remaining DATA/capture
  artifacts EVEN on failure (the test's inventory helper runs only on success).
  Report actual disposition, bytes and any remaining processes. Limits are not
  measurements. Stop on an unexpected selection/file/bound failure; preserve evidence.

512MiB is the proposed hard cap, not an observed runtime result. Current enabled
UI tests already ran in512MiB; these bounded8cell/64particle fixtures allocate1MiB
solver arrays plus a small16MiB-capped preview plan and1MiB SQLite cache/connection.
If that cap rejects execution, retain failure; do not silently raise it.

## Data-file bound and existing protocol behavior

The program has no dynamic step/retry loop around public execution: each case has
one track, one requested step and at most2 publications. Source paths are fixed:

| Data path class | Native maximum simultaneous entries | Material maximum | Bound/owner |
| --- | ---: | ---: | --- |
| intent.json, member-0-started.json, member-0-finished.json, completed.json |4|0| Each manifest <=1MiB via publish_new; no extra revision loop |
| .rheon-manifest-PID-SEQUENCE.tmp |1|0| One serial publisher at a time; an uncertain publication returns immediately rather than continuing to another staging path |
| One generated carrier/material SQLite database |1|1| max_page_count caps database at1MiB; no second archive, ATTACH or VACUUM |
| Its DELETE-mode rollback journal |1|1| One writer/one main database; per-file1MiB hard limit; no WAL/SHM mode |
| SQLite transient stores | Not statically sealed | Not statically sealed | temp_store=FILE is canonical; child TMPDIR/SQLITE_TMPDIR point into DATA. Fixed indexed/scalar queries and integrity/foreign-key checks, <=2records; readback runs serially |
| Application-owned path envelope |7|2| Each file <=1MiB under child limit; at most7MiB/2MiB for these named application paths. SQLite transient files are additional |
| Success-only observed inventory guard |8|8| Test refuses visible final inventory >8files/8MiB; this is NOT a peak/anonymous-temp bound |

The named application paths have the finite source envelope above, not an OS
aggregate filesystem quota. SQLite transient-store count/peak is not independently
sealed here: the8-file post-exit assertion must not be described as a proven
simultaneous/open/anonymous-temp limit. Existing per-file/process/time controls
remain the proposed proportionate bounds for this trusted fixed fixture; disclose
this limitation to the execution owner rather than adding mounts/cgroups. No
numerical execution is admitted by this preparation.
Normal SQLite commits may unlink journals and successful manifest publication
unlinks its own staging hard link under the existing production protocol. The tests
add NO cleanup; abandoned paths from protocol errors remain. Do not claim that
normal transient files deleted by those existing protocols were retained. Verify
all unexpected files after any failure rather than claiming the success-only
inventory proved peak historical disk use. No spill workload/user SQL is added.

Each publication payload <=65536bytes; constructor+accepted logical payload
<=131072bytes. The single force metadata document is146bytes by its literal codec
layout. Database/manifests' actual sizes are unavailable before execution and must
be reported from retained output, never fabricated from these bounds.

## Case1 — exact name

`force_runtime_integration_tests::native_force_dispatch_records_and_cold_binds_v2`

Native JacobiPcgV1,2×2×2 grid (8cells), spacing[0.5;3]m, origin[0;3], density2kg/m³,
zero initial velocity/tracer. One requested1e-6s interval. Tolerances: relative1e-11,
absolute1e-12, predicted divergence1e-9/s, actual divergence1e-6/s, Courant1,
pressure max100iterations; retained-array cap1MiB. No source/emitter/gravity input.
Preview edge2, history budget16MiB, steps1/threads1. Recording cap1MiB database,
65536bytes/publication,2records. Snapshot frozen before Worker/output binding.

Scene version1: enabled id1 “Desktop custody probe”, translation[0.5;3]m, identity
xyzw quaternion, Acceleration, Directional[0.25,0,0]m/s², Global, activation[0,∞).
Bind the same immutable snapshot to Request and FrozenSpec v2 in manifest v1.
Only private cfg(test) force_validation permits this otherwise gated request.

Expected: joined worker succeeds with progress1/one row/two ordered previews; final
native member cold-opens Ready, verifies SQLite status Completed/count2/generation1,
pin/hash/spec/declaration/header and both exact force documents. Interval pressure
is present from start0, all8values finite. Predeclared coarse effect guard:
cell0-gauge p[index] is within0.05Pa of (index%2)×0.25Pa, using dp=rho*a*dx.
Zero force/metadata-only dispatch cannot pass this20% guard; it is not a pressure
precision or general scientific acceptance test. Native fixed-dt policy remains:
accepted dt must equal1e-6; shortened dt refuses and preserves the actual archive.
Preview/result times must match accepted record bits and force-qualified controls.

Prepare a constructor-zero rerun intent from the cold v2 member, remove its attempt,
retain exact force bytes and validate. NO second Worker/step; the prepared clone still
has prior output/receipt paths and is explicitly NOT an executable rerun request.
Production rerun must allocate fresh paths through App. Mutate a separate draft,
then reread constructor+accepted archive metadata and compare exact original bytes.

## Case2 — exact name

`force_runtime_integration_tests::material_force_dispatch_records_immutable_scene`

Same1m domain/grid2×2×2/previewedge2. EmittedFluid fill,4³=64 stationary water
particles; samples_per_axis4/max_particles64, no emitter/collision. Default water
at293.15K, Air ambient293.15K, gravity[0,-1,0]m/s², sound speed20m/s, wall heat
exchange1; retained-array cap1MiB. Same force document and Request/recording limits.
Exactly ONE public particle step1e-6s, <=16 internal preparation/retry attempts;
no grid pressure loop and no external retry. Actual accepted dt may shorten.

Expected: joined worker progress1/one row; both SQLite records verify under the
same immutable run-bound scene,64particles, generation1, constructor zero velocities,
force-qualified actual dt/emitter/report controls. All times associate with the
actual accepted dt; no literal-time substitution. Fixed coarse effect guard:
X momentum change lies in[0.5,1.5]×(total initial mass×0.25m/s²×actual dt), with
finite positive mass/dt and finite momentum. This rejects tiny positive roundoff;
it is not a general SPH force-equivalence/accuracy certificate. Verify preview and
result actual time/dt plus64accepted diagnostic entries.

Material has NO existing serialized FrozenSpec/cold-rerun API. The test preserves
the typed Request, checks its configuration/geometry against recorded header facts,
rebinds the cold force document and validates that retained future-run intent.
It does not reconstruct a complete executable material request from opaque header
text or claim a new production rerun format. After separate draft mutation, reread
both records and verify original exact scene bytes. No second Worker/step.

## Gate-removal checkpoint, identified but NOT IMPLEMENTED

Only after separately admitted exact runtime results AND relevant reachable native
acceptance on the gated source (idle authoring, controls and retained/synthetic
presentation), prepare a small independently reviewed eligibility change in main/
SceneUi plus focused tests. A real force-enabled UI launch is a subsequent bounded
candidate-source validation gate; it cannot precede eligibility implementation. Removing just the Request gate would leave SceneUi blocking, while
removing just SceneUi's blanket refusal would silently let configured/AI setup ignore
forces. The future patch must use one typed effective-target readiness policy shared by
visible actions and last pre-output/start checks (native carrier, particle material,
unsupported); avoid independently passed booleans or a general capability framework.
That policy permits valid frozen
forces only in native-carrier/particle-material preview; preserve explicit refusal
in start_configured, Accepted Session, AI comparison, transport/provider/free-surface
and unsupported labs. Unsupported paths refuse authored metadata even if empty,
disabled or zero-valued. Saved native reruns retain stored per-member metadata and
never depend on current draft. Remove the private test-bypass field/temporary gate
only with equivalent production eligibility coverage, not a hidden alternate flag.

Native enabled hover/close/picking/outline/handles/Escape/camera ownership/property
sync and bounded actual force launch/retained view remain separate from these tests.
Genuine active Cancel/join, mixed members, long status and full material cold-rerun
capability are not supplied by two one-step probes. Existing native owner validates
exact1b37774/timeline repair; no duplicate native session or workload is authorized.
