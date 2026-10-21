# Independent Worker-path runtime test plan review

## Disposition and scope

The amended two-case plan is proportionate for preparing actual Request →
Worker::start dispatch/effect/custody/readback probes. It improves on the earlier
direct-loop proposal by including the real owned worker boundary. Conditional
admission is for test preparation only. No numerical/SQLite execution, production
force launch or second-step rerun is admitted by this review.

Final re-read resolves the draft conditions below. No new plan blocker remains;
full helper/source review, final seals and separately admitted execution remain
gates. SQLite transient-store peak/count is explicitly unsealed and must remain
disclosed to the execution owner, rather than replaced by a new OS quota or
provisioning requirement.

Reviewed the parent's proposed amendment, the existing validation contract and
the temporary draft `/tmp/rheon-force-runtime-integration-tests.rs` as design
material. Temporary draft SHA-256:
`2ee5de610102351183f9c87b3da4e9f1129891c4e4731121907b184cf57724dd`.
Existing contract SHA-256:
`7327a76ef2d66525badf61d73518ad56009c2e54149ad181f703516890c3f2c5`.
Later reviewed prepared bodies now reside at
`src/bin/desktop/force_runtime_integration_tests.rs`, SHA-256
`da753230b81df85227ab348c39ea4ac97e2ff093cf2da09ff9ac5dce225d0584`;
amended contract SHA-256
`6bac634aa89780bed5dba840f9cd64aca71c314f417f28da05811d0f2a4c4455`.
This was not a complete independent audit of the production helper closure.
No build, test, numerical/SQLite/native/supervisor action or source edit occurred.

## Draft conditions and final disposition

| Priority | Finding | Required disposition |
| --- | --- | --- |
| High | The draft material effect guard accepted any positive net X momentum, including tiny pair-force roundoff. | Resolved: finite positive mass/accepted dt and finite momentum change, within 0.5–1.5 times mass × 0.25 m/s² × actual dt. Fixed coarse dispatch guard, not scientific accuracy acceptance; no tuning/baseline run. |
| High | Post-run `files <= 8` could be confused with a proven peak bound. | Resolved by qualification: seven native/two material named application paths are enumerated, while SQLite FILE transient count/peak remains unsealed. Existing per-file/time/process limits and final inventory are the proposed proportionate envelope; no quota/tmpfs/mount/cgroup prerequisite. |
| Medium | The helper required an empty root, but a supervisor might pre-create capture/seal files there. | Resolved: DATA and capture roots are separate; all three data/temp environment paths are checked against the canonical fresh DATA root. |
| Medium | Draft mutation proved snapshot distinction without checking archive contents afterward. | Resolved: both tests reread constructor and accepted metadata after separate draft mutation and compare exact frozen bytes. |
| Medium | A manually cloned native prepared-rerun Request retained original output directory/receipt. | Resolved: comments/contract explicitly call it non-executable prepared intent; no second Worker or production rerun claim. |

The final native guard checks all eight finite interval-pressure values against
the cell0-gauge hydrostatic pattern (index%2) × 0.25 Pa with 0.05 Pa tolerance,
using the explicit density/acceleration/spacing. This discriminates zero-input or
metadata-only dispatch as a coarse symptom, without a second numerical run. No
claim that the expectation has actually passed or supplies scientific validation
is made; thresholds are predeclared and must not be tuned after execution.

## Amended execution envelope

The parent's new envelope supersedes the old direct-loop/1 GiB proposal: one
individually exact ignored test per disposable process, one libtest test thread,
one owned Worker thread and one owned Rayon pool worker from Request threads=1.
Desktop still enables Rayon; no zero-parallel or zero-worker claim is valid.
These are the intended numerical/test ownership roles, not a measured total OS
thread count. The external owner retains process-group supervision.

The proposed limits are 512 MiB address space, 30 s wall, 20 s CPU, 1 MiB aggregate
captured log, existing per-file 1 MiB enforcement and a reviewed finite source
envelope for named application paths, followed by post-exit inventory. TERM then
owned process-group KILL after 1 s remains supervisor-owned. Do not infer peak
resource use from limits. If an already supported supervisor control is absent,
report that exact prerequisite; do not invent new security/provisioning work.

This explicitly replaces the earlier review's aggregate-filesystem-enforcement
prerequisite with the parent's qualified finite-source design. Native named paths
are four manifests, at most one serial staging link, one database and its DELETE
journal (seven); material has database/journal (two). SQLite temp_store=FILE can
create additional transient stores, including anonymous/unlinked files. Their
count/peak is not statically sealed. The visible success inventory <=8 files/8 MiB
is an observation/assertion, not a hard aggregate or historical peak cap. Existing
per-file/process/time controls are the disclosed proportionate proposal to the
execution owner; no new aggregate storage mechanism is required. Unexpected
output/bound failure, timeout or numerical failure preserves disposition with no
tuning/retry/fallback. Limits do not become measured consumption.

## Fixture and ownership boundaries

Native remains one track, eight cells, one requested interval, at most 100 pressure
iterations, no source/emitter/model/provider/configured owner. Its explicit saved
v2 spec controls density, tolerances, memory and fixed dt; no normal-preview
defaults may replace those values. The worker returns before outcome/readback
assertions. A shortened fixed dt refuses normally and preserves its archive.

Material remains 64 initially stationary particles, one public step, at most 16
internal preparations, no outer retry or pressure-solver workload. Verify the
constructor/fill and explicit config against the final helper source before
sealing. Actual accepted time/dt, rather than a literal requested endpoint,
governs preview/record/result association. Force metadata, units and physics
remain canonical; the test does not implement an alternate evaluator.

Worker::start must receive the real validated per-track snapshot and only the
private cfg(test) temporary-gate bypass. Freeze, eligibility, budgets and exact
run/publication binding stay enforced. No App update/start, GUI, observer,
progress/failure rendezvous, collision asset, model/provider or subprocess is
needed. Keep all captured owner handles/lifetimes explicit in full source review.
The external supervisor bounds join; do not add cleanup cancellation or another
scheduler/worker after a failure.

## Readback, rerun intent and retained artifacts

Native can verify the finalized v2 member through the existing cold-open/pin path,
then prepare a child intent with retained spec hash/force bytes and cleared attempt
without a second numerical call. That is a prepared-intent check, not production
rerun execution or proof of rerun numerical equivalence.

Material has no serialized material FrozenSpec/cold-rerun API in the application.
Retaining a Request clone and rebinding its force snapshot from cold recorded
metadata is an honest, narrower check. Keep that limitation explicit: it does not
reconstruct all material controls from an archive or establish a complete cold
material rerun. Do not create such a format to satisfy this test.

Use a fresh explicit absolute supervisor-owned data root, with no temp fallback,
replacement or cleanup guard. Preserve outputs on success/error/panic/timeout.
The test inventory is only reached on success; the supervisor must still collect
post-exit file names/sizes/hashes and survivor/disposition evidence after failure.
No test Drop cleanup or automatic root deletion is permitted. Normal production
SQLite commits may unlink journals; successful manifest publication unlinks its
staging link. This is distinct from test cleanup, and those normal transient paths
cannot be claimed as retained artifacts or historical peak-storage evidence.

## Preparation gates and documentation alignment

Both ignored bodies now exist. The amended contract reports list-only compilation
of 201 tests including the two exact names; this reviewer did not repeat that
enumeration/build. Before execution, independently review the complete production/
helper closure and seal final source/test/lock/toolchain/features and binary. The
older executable supplies no integration evidence. The final contract now uses
real Worker::start, 512 MiB and qualified transient-storage semantics; the earlier
documentation-alignment finding is resolved. List-only/compile are preparation,
never workload or runtime acceptance.

Passing future cases could establish tiny real-worker dispatch/effect/custody and
cold-readback association. They do not establish live Cancel/join behavior under
interruption, native/UI interaction, mixed-member runtime, scientific accuracy,
legacy equivalence or full cold rerun. Production launch stays gated until the
separately authorized execution evidence is reviewed for its exact scope.

## Future capability-aware gate-removal candidate review

Re-read contract SHA-256
`b76c5ebd188fbeb37165a56d3e1828f769e7fa695a503a6223f081a6bdef9654`.
The four intended thread roles are now explicitly distinguished from a measured/
enforced count, and the 30-second wall timeout deadline excludes teardown latency.
These are accurate qualifications, not new resource measurements.

The final gate-removal section defines a suitable **future candidate scope**:
main/SceneUi readiness and the smallest focused eligibility/fixture tests, with
Request validation/launch checks adjusted only where the temporary gate lives.
No solver, force evaluator, recording schema, saved-spec migration, new worker or
alternate runtime switch belongs in that change. No gate code is admitted now.
Removing a single blanket check is insufficient because visible setup readiness
and consumed Request/dispatch readiness must agree.

The future candidate should satisfy these criteria:

- One typed capability decision uses the effective target/workflow and immutable
  consumed request intent, rather than remembered material mode, viewport display,
  worker presence or status strings. Native carrier and particle-material preview
  are the only force-enabled positive cases. The visible launch projects the same
  result that is rechecked before Worker/output admission.
- Direct `start_configured`, Accepted Session, AI comparison, transport/provider/
  free-surface and unsupported lab routes retain explicit refusal before any
  worker, model/provider startup, directory, manifest or recorder side effect.
  Unsupported metadata must not disappear through an empty/disabled/zero shortcut:
  distinguish None from Some(empty), and validate disabled/zero objects normally.
- Freeze valid draft intent once before launch and retain per-track custody.
  A failure preserves draft/current retained history and a concise visible reason.
  Positive native/material readiness does not bypass any existing geometry, budget,
  numerical-policy, immutable-binding or owner eligibility validation.
- Saved native reruns resolve each stored member's exact optional snapshot,
  independent of the current draft. Cover v1 None, v2 Some, explicit empty and
  mixed independently valid members; no `.or(current_draft)` substitution or
  historical migration. Preparation-only rerun clones remain non-executable.
- Remove the private `cfg(test)` bypass and obsolete fixture fields only with
  equivalent positive/negative production eligibility coverage. No replacement
  hidden flag, test-only production routing or environment escape hatch remains.
- Focused inert tests exercise each supported/unsupported readiness decision and
  demonstrate no dispatch/output intent for refusal, including direct methods
  rather than only a disabled button. Full helper inspection precedes any exact
  test execution; do not run numerical fixtures to prove the decision table.

Prerequisite ordering must avoid a circular native gate. Before preparing code,
the two exact runtime cases need separately admitted passing evidence, and the
currently reachable native authoring/gesture/retained presentation requirements
need exact-source acceptance. Actual force-enabled UI launch cannot precede the
candidate code that enables it; it belongs to a subsequent separately bounded
candidate-source native check, before broader feature acceptance. Native launch,
progress/Cancel/join and unsupported-route behavior are not established by the
two one-step ignored probes. No duplicate native owner/session is implied.

**Disposition:** the identified candidate is reviewable and proportionate with
these criteria. Prerequisite runtime/native evidence is still absent from this
review; no implementation or workload is authorized by this future plan review.

Final re-read of contract SHA-256
`1b96cb00db82e92978282c213c4abea54ee06e382c5f994edb6185f99fd5ad71`
confirms the owner incorporated the shared typed effective-target policy,
unsupported empty/disabled/zero refusal, immutable saved intent and noncircular
native ordering. The candidate-design conditions are resolved. No remaining
plan/design blocker exists for that later bounded candidate once its stated
runtime/reachable-native prerequisites are met. No gate code is admitted now.

## Ignored-list correction — preparation only

The owner reports that the ignored-only enumeration contains **four** tests:
the two newly prepared integration cases plus two pre-existing ignored cases.
An owner-side list-validation assertion incorrectly expected two total entries
and failed. No test body ran; preserve that failed preparation receipt and do not
report it as a numerical/runtime failure or a passing membership check.

Correct validation should check the two exact fully qualified integration names
are present in the ignored-only list, without asserting that the entire ignored
suite has only two entries. The other two entries do not gain execution admission.
This review did not rerun list commands. Never replace the intended exact-case
commands with a broad `--ignored` execution to resolve a listing mismatch.

Subsequent receipt read: `/tmp/rheon-ui-force-worker-enumeration-check.log`
confirms both requested fully qualified names are ignored, four ignored cases
exist in total, and the earlier expected-total-two assertion was an enumeration
mistake with no body execution. This reviewer read that corrected receipt without
repeating commands. The owner additionally reports 201 total tests and exactly
one occurrence of each new name in both lists; those raw-list counts were not
independently recomputed by this review. The corrected receipt resolves the
reported listing mismatch and supplies no runtime acceptance.
