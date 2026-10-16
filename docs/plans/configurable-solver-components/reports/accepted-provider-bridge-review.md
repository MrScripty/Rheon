# Accepted CPU provider bridge independent source review

Reviewed the current uncommitted bridge in `/workspace/rheon-accepted-runtime-bridge`, base `aac620db3abb652019dc9c44884928e75bab06ee`. Read-only review; no Cargo, subprocess, model, numerical or GUI execution performed. `git diff --check` passed. The author/lead's validation receipts remain separate evidence.

## Assessment and exact scope

No material source defect found in the reviewed provider bridge, tests, and one-line module registration. Source approval is bounded to this composition over the existing process owner and public Session provider seam. Registration makes the provider available internally; it does not wire a production worker, recording loop, frozen rerun, or UI to use it. The tests directly instantiate this production provider, rather than a duplicate test-only implementation.

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/ai_comparison/accepted_provider.rs` | `f86d624e4324af9f257ef2b9643b2f116443c18d3f39e34cafd2a0de6a92868f` |
| `src/bin/desktop/ai_comparison/accepted_provider_tests.rs` | `35607bb996208614933e8b1aa2c462403dba7f9c99ec322038bc4c791afaf371` |
| `src/bin/desktop/ai_comparison/mod.rs` | `81ea11b59ca3bd78efcc224be946fd55c4672d049a0b94d901b798a237a8ddb4` |

Relevant retained dependencies read in this review:

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/ai_comparison/model_process.rs` | `77070b2ae78dabf8f2d046a0b83f97053ef6460f8c2f19f47dcc3264e34f9754` |
| `tests/fixtures/accepted_pressure_adapter.py` | `958ba74ebde596aac8375eb436898a1e2342ba1b7868c08bfee99c09c02dc27c` |
| `src/solver_provider/configured.rs` | `124ca65dd1dec3a8a89043198fb085453a42ca8aff0dc87c67b04b6a986109be` |
| `src/solver_provider/context.rs` | `763debfefc6f9375849c8f503995e20660fc25efef9e827d5942d980e14fe693` |

## Source conclusions

`start` validates bounded identity and loads the bounded adapter manifest before launching anything. It requires exact id/version/manifest hash and refuses caller checkpoint/training claims that adapter v1 cannot independently bind. The existing pinned process startup repeats the resolved binding check before executing model code. Interpreter and package files are locally hash/size checked through the existing adapter loader; runtime and source strings remain caller declarations. Concurrent mutation of trusted local package files is explicitly excluded, rather than solved or authenticated by these hashes.

The two retained f64 scratch arrays are checked with multiplication overflow detection against an explicit caller cap and allocated fallibly before child creation. Their size is the package's bounded maximum cell count. This cap covers those arrays only, not process protocol/serialization/output allocations, Session workspaces, Python allocations, descendants or total RSS. Existing separate bounds and resource ownership continue to apply.

`supports` is inert and checks accepted mode, scratch capacity, actual grid/rho/mu/duration, required prior and native-prefix support. `predict` repeats support, checks output length, copies only the authoritative accepted prior (or explicit absent-history zero guess), and computes `b-A*prior` with the core full operator. The reconstructed context retains actual run/generation/interval/RHS/history. ModelProcess reconstructs correction outputs by adding the prior exactly once; full-pressure outputs are copied directly. Session retains host qualification, fallback, all transport gates, accepted history and final publication authority.

Typed process failures are retained separately from the coarse public provider rejection, including dispatch and invalid consumed-payload digest. Host quality rejection is correctly not called a process failure. `last_failure` is explicitly the most recent prediction failure; an inert support decline can leave an older failure visible. A future interval recorder/worker must associate it with the current actual prediction and must not label an older failure as a new dispatch merely because it remains present. Successful predictions clear it.

Cancellation/deadline polling uses the existing ModelProcess callback and immediate-child slot. The existing loops poll in cooperative windows around 20 ms waits; this is not a hard deadline bound, especially during serialization, bounded residual computation or OS scheduling. Session's post-prediction/final commit gates remain necessary. Callback cancellation latches the shared flag and kills the existing child; that owner is not reusable for a retry. Drop retains immediate-child kill/wait plus writer/reader joins. These are best-effort OS ownership operations, not descendant termination or an external hard-shutdown watchdog guarantee.

## Test source assessment

Eight tests exercise real bridge/Session composition with independently authored stdlib CPU fixtures. The two-cell sealed-box fixture uses cubic spacing 1/4, rho 2, nonzero Newtonian mu 1/128, requested and actual duration 1/16, binary forcing, full liquid fractions, ordinary unchanged acceptance limits, tight pressure tolerances, and a real native first interval. It analytically checks the nonzero compatible RHS, `b-A*accepted_prior`, physical coordinates/scalars, accepted status, contiguous generation and prior endpoint. Full-pressure and correction paths are compared byte-for-byte against existing native viscous field evolution; this establishes this controlled small physical case, not general model accuracy or a trained deployment claim.

The test source also covers inert unsupported duration with native fallback/no request, bad pins/metadata/scratch cap before child marker creation, host quality rejection followed by native fallback history, actual runtime exit with dispatch retained and a fresh owner retry, malformed consumed-payload digest and explicit runtime context decline, in-flight cancellation/deadline with exact publication/receipt preservation and remaining quota, and a non-two-cell direct bridge shape with zero guess, diagnostic-mode/output-shape refusal and pre-dispatch cancellation. The direct arbitrary-shape fixture tests adapter plumbing, not successful Session physical qualification. Linux teardown checks use the established kill-zero helper after owned wait; they do not rely on the invalid `/proc` namespace oracle.

No execution result, strict compiler result, inventory admission, trained-model parity, or full A2/A3/A4 acceptance is asserted by this review. Production integration must preserve the separate caller/executor/package identities, resource budgets, fresh process-owner retries, current-attempt failure association, and existing accepted publication ownership described above.
