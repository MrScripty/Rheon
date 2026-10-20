# G1 implementation qualification

Status: **Verifying; native workflow gate remains open**. Source checkpoint `bc494c180175fa66120d1249679179b577b39bca` implements the contract in [g1-contract.md](g1-contract.md). Criteria and workload were frozen in [g1-protocol.md](g1-protocol.md) before evaluation. PR122 remains draft. This report concerns inspection and custody of the existing sealed-MAC execution; it does not establish new water accuracy, physical validation, GPU support, browser parity, universal accessibility or packaged-release qualification.

## CPU and archive contracts

`cargo test --no-default-features --lib --tests --locked` completed 74 suites: **597 passed, zero failed, four ignored**. The ignored cases include explicitly opt-in qualification helpers, not passing evidence. A subsequent current-source `cargo test --locked --no-default-features --lib workflow` passed all **15** workflow contracts with zero ignored. Desktop workflow cases, legacy exact-side comparison, strict all-target library/desktop Clippy and formatting passed. Independent architecture and persistence review approved the corrected source; independent native UI source review approved it subject to the actual display gate. See [review dispositions](g1-review.md).

Coverage includes exact native input and actual control/RHS identity, actual shortened intervals, viscosity disabled/applied zero/nonzero, provider acceptance/refusal/rejection/fallback, cancellation at real owner boundaries, generation-zero terminal evidence, immutable accepted state, additive RCP4/RCT2 custody, old profile meanings, invalid/cross-linked/unknown evidence, increasing attempt IDs and explicit accepted-but-unrecorded tails. Existing persistence integration passed 60 tests independently. The actual producer-exit/model-absent reader test passed.

Raw CPU evidence remains outside Git in `/workspace/rheon-g1-evidence`; native/event evidence is under `/tmp/rheon-native-display`. The final compact manifest will bind retained logs, source and executables after the native gate is disposed.

## Fixed release capture-plus-projection budget

The first actual measurement passed the unchanged **≤5% median** criterion: **0.403723%** median paired added time across exactly 30 pairs after five warmup intervals. Every pair asserted bitwise identical accepted native fields, stamps, pressure, fractions and times outside the timers. Baseline and observed paths alternated first position. The candidate timer includes actual `WorkflowInspector::set_attempt` validation and cached projection. Disk I/O and rasterization are outside this metric.

Median baseline time was 18,040,111.5 ns and median candidate time 17,918,459 ns; the criterion uses the median of paired ratios, not a ratio of these separate medians. Paired overhead quartiles were −2.3943%, 0.4037%, 2.3332%; extrema were −16.0484% and 19.8920%. Raw pairs and variability are retained. Timing was performed with other owned CPU jobs and the native App stopped, on Intel Xeon Platinum 8573C under a four-CPU cgroup limit and 16 GiB memory limit, Linux x86_64, Rust 1.92.0 optimized release. This is one fixed representative CPU workload, not a universal performance claim.

Measured source: `bc494c180175fa66120d1249679179b577b39bca`; source tree `9b4173cc9441781a942ba2396a9d9fc905d4d57c`. Release test executable SHA-256: `3a375a2b3c45a4befb4a16ce2b970a2866039b3cc1e6fe9298bffa2ec9bc6821`. Raw log SHA-256: `31ea8d6f959e1334dca744335adcdeb910b1a49a512b03c49a756dfd2d8e6633`. Build used `RHEON_SOURCE_REVISION` set to that source checkpoint, with `RHEON_BUILD_MATERIAL_SHA256` unset; this is a source declaration, not a hermetic-build assertion. Independent architecture reviewer recomputed the budget and hashes and approved this gate. The sorted tenth–twenty-first paired-ratio bracket is −0.946% to 1.940%, below the limit; timing variability does not obscure the threshold margin.

## Native interaction

The first normal-App display run was retained as a failed evaluation at `/tmp/rheon-native-display/failure-bc494c1`. It reached an actual live configured pressure attempt with Jacobi-PCG quality-gate fallback, but the End/Left selection observation failed. Investigation found the dummy Xorg server using a legacy keyboard map; stronger real-egui tests with key-release and idle repaint frames pass. The product-versus-display-input cause is not yet established. No native workflow pass is claimed from this run, screenshots or startup.

The actual normal-App procedure must still pass pointer/keyboard/focus, actual provider/interval/units and fallback inspection, accepted interval change, retained evidence, cancellation/close, provider-executable removal and cold reopening of the identical retained graph, followed by owned-process cleanup. G1 acceptance and C1 dependency readiness remain pending until that evidence and independent review are complete.
