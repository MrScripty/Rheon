# PR59 independent exact-source review

Date: 2026-10-21. Reviewer: independent source-review worker, read-only repository review.

Decision: **no blocking source defect found in the exact material listed below**. This is a source review, not final performance admission or merge authorization.

## Source binding and method

Reviewed the frozen working source against main `15026a966265cf9eaedf464e2b5a066bd0fef3d8`, then independently checked that every listed file is byte-identical to commit `0b3e3fc0e83524fcc9064a2158030f2ee4994a1a`. That merge commit has parents `7f1733f0dc956ef8f25d5b110d990117fb765903` and `15026a966265cf9eaedf464e2b5a066bd0fef3d8`. Working tree was clean at commit-binding verification. The source/test diff is confined to the declared production write set and focused contracts. Additive force fields and configured-provider code from current main are retained in the reviewed diff.

Read the integration protocol, original owner contract, verification and results, usage documentation, production delta, relevant owner/solver/provider publication context, and all newly added or amended test cases. No builds, tests, benchmarks, cloud task tools, artifact downloads, network writes, or repository mutations were performed. Scoped `git diff --check` passed. Runtime results reported by the primary worker are not independently reproduced by this review.

## Findings

No blocking findings.

1. **Default construction boundary is explicit.** Cargo's default remains `png-export`; neither it nor `parallel`/`desktop` enables `accepted-pressure-warm-start`. The warm module and exports, borrowed-seed methods, solver parameter and seeded recurrence branch, coupled owner state and constructor identity allocation, initialization report field, and warm Session entry point are all feature gated. Ordinary standalone Simulation entry points pass no initial pressure even in feature builds. API version 3 and the capability boolean are intentional always-visible API changes; the capability is false without the feature. This source exclusion does not establish identical executable layout or runtime.

2. **Numerical success gates are preserved.** Seeds are length/finite validated and require a sealed gauge-zero operator. Full compensated RHS compatibility remains unchanged and no mean shift occurs. Thresholds use the current full RHS. Initial and terminal reports compute full `b-Ap`, including the eliminated gauge equation; predicted divergence, actual f32 divergence, transport bounds and later acceptance gates remain in force. The seeded-only recurrence replacement decision uses the reduced system residual, without modifying the full success predicate, max iterations, or cold recurrence. No hidden warm-to-cold retry or timestep scaling was introduced.

3. **Accepted provenance is owned and staged correctly.** Only immutable accepted pressure is borrowed; no additional seed array is allocated. Owner incarnation uses checked nonwrapping identity allocation. The key includes immutable geometry, density and implementation; unsupported boundary/provider/viscosity/surface/column routes have no native key. Generation/key checks prevent reuse across invalid publications. History preparation and overflow checks occur before the final callback, followed by the existing infallible pressure copy and owner commits, then assignment of staged history. Failed attempts leave accepted pressure and metadata untouched. Successful provider routes, including numerical fallback, invalidate native eligibility. Configured-provider history remains a distinct Session field.

4. **Session publication composes correctly.** The warm path captures the actual pre-step carrier clock, then updates interval start, quota, accepted report and receipt/history clearing only after the simulation call succeeds. It clears both provider/configured receipts and configured pressure history. Deadline checks bracket cancellation callbacks, including the final publication checkpoint. Failures return before Session publication changes. This follows the existing current-main publication pattern.

5. **Focused tests cover the relevant seams.** Borrowed-seed tests include independent exact/approximate chains, zero/current RHS thresholds, gauge/full residual rejection, incompatible RHS, invalid seeds, cancellation and dirty-scratch replay, independent Neumann refinement, and full-gauge acceptance. Owner tests cover opt-in and dt changes, distinct incarnations, observed cancellation checkpoints, exact retry, unsupported success/failure transitions, memory limits, saturated rejection and physical accuracy/flux balance. The added configured transition checks preserve receipts/clock on a final cancellation and clear receipts on acceptance, with subsequent native reuse and configured invalidation. Interval-start coverage exercises 64 nonbinary timesteps for both implementations, proves a subtraction counterexample, and checks failure/quota/stop preservation. Provider success/fallback stays cold. The Session unit test adds a deadline reached inside the final callback. Test source inspection cannot replace execution of the scoped feature/default matrix and real-model contracts.

## Performance and admission limits

The integration protocol appropriately retains the historical 1000-particle +1.83% screening and **+2.20% confirmation** as unresolved adverse default-path observations. Binary layout is only a hypothesis. The narrower build feature is a supported exposure boundary; it does not erase those outcomes or admit feature-enabled mixed particle consumers. Historical warm gains remain bound to original binaries, workloads and PR55 base, with no new-main universal speedup claim.

The prospective default control is bounded in advance: final pre-PR59 composed main versus integrated candidate, release/locked Rust 1.92.0, separate source/target roots, identical `--features parallel` with observation off, existing 1000/4096-particle single-worker harness, 64 steps at dt 0.001, twelve balanced shuffled pairs per case, complete outcomes and exact horizon/fingerprint/managed-byte checks. It retains median/range/sign count and paired uncertainty, disallows cohort extension and invented equivalence margins, and refuses a default non-regression claim from positive or inconclusive results. This is an appropriate scoped strategy. Actual executable inspection and those measurements remain necessary; no timing was assessed here.

Before a normal merge, final source must follow #142 → #138 → #143, match or receive renewed independent review, obtain current native source admission without bypassing inventory/provenance gates, pass required exact-head CI and local checks, and receive lead authorization. This report binds the current initial-main composition only. Changes to reviewed production/test bytes require review of the delta and refreshed hashes. A different final base also requires composition assessment and the protocol's final controls. No blocker arose from a denied action in this review; no prohibited operation was attempted.

## Exact reviewed material (SHA256)

All hashes below were recomputed and matched committed bytes at the bound commit.

```text
c122573542e3367761aab241a1de89a3002142360415c2d154e3b294acdf31c4  Cargo.toml
9dad0c45ef7b000b9f560da05efbf6ccf6241574a42e416d056439b8ca0dc39b  src/lib.rs
fb70a35516764666acbedffb3c8cb2fbdc9c63690cfe87701085a9a5bf4c8827  src/pressure.rs
dc81cb8c5b9dd1719a8e513e87d655098688b18e32bbea576b72746cadd1a233  src/pressure_initialization.rs
4500be8ac73dafb99b31408a5b81a358230af4a244bcde337986b0969adeb2c4  src/liquid_step.rs
79a64bbfd8774da9903046566271a6551fd2290606210d4bef4160eb6a09696a  src/simulation.rs
217b7ab5eb379aeed7d6fea5381b49c4c5974f7e41b1a5383bdfe9b5a36250c1  src/session.rs
7d30b0d10acfb461d7c09798728b2e6c5bde72c17a45447df54ea80a15f08cc8  tests/accepted_pressure_history.rs
5ad1f1ac8aad421cef53af6aa7a3db71440cb2a0eb2d3d8f223e395c691aee14  tests/pressure_initial_guess.rs
06e1e1ed6b0b20278ef44e01daa5585b1095fb4850ff834b4c2b7257c87eeea7  tests/configured_provider_session_contract.rs
e5f6784f3ead54238f76f77bc6400d9007cd0e930a4381df22300fda4d012bf5  tests/provider_step_contract.rs
4503116a845296afcd058a988a3d343811944fa479cff3c489427c43db09e3c0  tests/session_contract.rs
40b64f4709fbe597431021e95e94ae2a1279d189b009b33323a8da362f9aea31  tests/session_interval_start_contract.rs
d99ae990e9c7ddc956f95f414a25832411a4dc84ab434df95ff9f8f68cf52a83  src/session/configured.rs
cd98684d5204908e5344b2628a8f93ea2a55a90c6678e03a71f2d75ab1f2be38  src/operator.rs
67f8f0ba89310b728f4d261d50c7b0cddfe25c17b4a3a7d3ee3bf9601e849b5a  examples/accepted_pressure_lab.rs
7c509870878006887cc91a5808fa553f9a7632d114c578202e9ab4808d60a3b0  docs/plans/accepted-pressure-warm-start/integration-20261021/protocol.md
90b9a3abc555a54562cd1868bde685a500db28d9a6d72f638b4f2a4f4268c9b7  docs/plans/accepted-pressure-warm-start/owner-contract.md
b675c67439ac2f21d56093e14a75c7e925b550860ae2f41917320c169b991b66  docs/plans/accepted-pressure-warm-start/verification.md
e643af581fdaae04fb831c57f352508c3e23543e7f28e478c1c439c5a8e9b12d  docs/plans/accepted-pressure-warm-start/results.md
87546fae9349a238b6795cf7d519d22015746edeaefeb4a440b1bd066d6e8ae2  docs/plans/accepted-pressure-warm-start/usage.md
```
