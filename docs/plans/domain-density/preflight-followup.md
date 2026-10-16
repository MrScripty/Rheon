# Independent comparison preflight correction review

Approved the exact seven-file correction/documentation delta below for explicit source admission. No remaining material finding. This is source review only, not a test execution receipt, committed-head qualification, hosted acceptance or merge approval.

Subject: uncommitted delta atop historical `94062f6e75cdbdd8828fcbe4e9dd3a4d01f1426f`, original density base `ad888bdbbbd5cae362439458af342196f51c58e2`. The four implementation/test/API paths and subsequently appended plan/issues/ledger changes were all read. Original review.md and approved-pins.json remain immutable. The94062 native qualification, GUI controls, binaries/stamps, release and hosted receipts keep their historical subject; none is relabelled to cover the corrected head.

## Pressure owner and accounting

engine.rs now owns PRESSURE_ITERATION_LIMIT=3000. Its sole native pressure solve owner uses the constant where the unchanged literal3000 appeared. Relative/absolute/divergence thresholds and corePressureWorkspace algorithms remain untouched. Moving constant ownership does not change numerical behavior. The core loop remains1..=settings.max_iterations, including residual/restart handling inside that cap.

Traced every native_solve reference: exactly two call sites. The paired native reference runs on every entered interval. Warmup returns before the second site. Afterwards independent typed requests additionally solve the exact AI request for a native witness; legacy creates no typedrequest/witness, whileteacher reuses the already solved native result. prepare/finish and construction add no other native pressure solves.

The policy helper validatesScene before subtraction and computes upper native solve counts: steps forlegacy/teacher; steps+(steps−warmup) forindependent-stop-on-reject. Early cancellation/rejection/failure can only reduce this count. For16³/64steps/15warmup there are64/64/113solves, giving total native pressure cell-iteration proxies786432000/786432000/1388544000 at3000iterations. These are iteration-slot proxies, not FLOP/runtime bounds. Matrix/preconditioner/reduction/check work and viscosity/advection/model work are additional. Validated counts/steps keep theu128 products safely bounded.

The CLI removes the misleading per-track2000 and native-array-per-track keys. It reports policy/steps/warmup, explicit native iterationcap, solve budget and totalpressure iterationproxy. The renamed native_simulation_array_payload_reference_bytes is honestly documented as the existing coreSimulation BufferPlan reference for this grid, not actual diagnosticEngine allocation or a memory cap. Engine tracks/workspaces/preparation/model allocation remain outside that reference; paired history is separate.

## Regression and documentation

The cfgdesktop actualCLI test invokes the realcomparison executable at16³/1000m⁻¹/64steps/15warmup forlegacy/teacher/stop-on-reject. It independently asserts3000,64/64/113solves, exacttotalbudgets,333824-byte coreSimulation reference, falsemodel/support/wholeprocess-cap flags, removal ofoldambiguouskeys, and absence ofthe requestedoutputroot. No deployment is supplied; source preflight returns before Worker/model/simulation launch. This meaningfully covers the externally visible undercount and mode distinction; this reviewer did not execute it.

API documentation gives the exact formulas/examples and exclusions. Appended plan/issues/ledger text preserves the original numerical failure and A1 blocker, identifies the late undercount, and keeps94062 evidence historical. Ledger wording that explicitadmission now covers15/109 is approved as the intended atomic review/inventory update: the inventory is still stale at this review point, and Root must apply the exactpins before treating that sentence as currentadmission proof. Prior reviews/receipts stay unchanged. Root reports the oldhostedset passed; no newhosted or test pass is inferred here.

## Complete selected closure

Reconstructed the unchanged conservative selector:124paths,15changed/109unchanged versusad888,0added/removed. Since94062admission onlyengine.rs andrheon-ai-compare.rs need selected rebindings. All other selected bytes, historicalv1/v2 inventories, selector/gates, Cargo/lock/toolchain and protected numerical/qualification inputs remain exact. The existing inventory intentionally refuses these two changed source paths until Root explicitly rebinds their approved values and newreviewSHA.

[preflight-approved-pins.json](preflight-approved-pins.json), SHA256`4b36340a69d1a02b7a22fd74df121c83cd34eb3b5f6452f276cc718e3d50c6ee`, contains all124currenthashes, all15changed-versusad888pins, the2currentselectedrebindings and7exactdelta pins. This supplements the original source reviews, superseding only corrected CLI/test/API bytes and adding the reviewedengine change and ownerhistory appendices. It supplies no corrected-head execution evidence. Fresh committed-source qualification, requiredtests/builds and exact-headhostedgates remain required beforemerge. A1 remains blocked; no training or model-support/accuracy claim is supplied.

Read-only checks included exactscope/hashes/15–109split, protectedinputs/immutablepriorcopies, Markdownfences/localreferences andgitdiff--check. No repository/ref/target changes or application/build execution. Only these task-owned `/tmp` artifacts were written.

## Exact correction pins

| Path | SHA256 |
| --- | --- |
| `docs/api/domain-density.md` | `81f0a5baa8fdc38a07356da3309e0ecbcc308cd89caa838f59242c197a31e90d` |
| `docs/plans/domain-density/execution-ledger.md` | `0465d5277ddeab155cc6ab9278add5022eaed06e5f9187b6b28472102af8e982` |
| `docs/plans/domain-density/issues.md` | `9fecb86b42a39f9588a67005a32a1f20c8cabb89ebf494e0db97bec2fe2337fa` |
| `docs/plans/domain-density/plan.md` | `7e79f54919b4e6de065d08911877b4224774f9e64dd7a4e307a1fc4f97c8da16` |
| `src/bin/desktop/ai_comparison/engine.rs` | `610ba29f66eb91083f9bbd26b823b14e4bde4887eb057e929c089d6cd4592e55` |
| `src/bin/rheon-ai-compare.rs` | `00000ec93161c04d9ff0515e5ae2ab6855a5467f56f23a4d6fc6c1059ef8ada9` |
| `tests/cli_contract.rs` | `83d3c9a839dd2c722b3c509e051350a6ca1c754cbe0ae5dbb521e7d3346c6aef` |
