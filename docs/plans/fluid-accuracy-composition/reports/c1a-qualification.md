# C1-A typed native boundary qualification

Status: **C1-A locally qualified / integration pending**. Frozen protocol `e5b1f8811ce3c4f6519d70fc2c8e2de149f587bc`; implementation checkpoint `5c9fb96d7ebbc49638afbe812ac1db69b5dfc643`; final source `ac6ed17003279e7078751a4390f2ee28200d4d5f`. [Draft PR125](https://github.com/MrScripty/Rheon/pull/125) is stacked on unmerged draft PR122. Neither local qualification nor this report means merged/release acceptance.

## Implemented boundary

`PressureStageV1` delegates the actual existing pressure-v1 request, settings, provider and cancellation to ProviderWorkspace. `QualifiedPressureV1` borrows the pressure slice and retains the original ProviderReport. This preserves old error, candidate and fallback semantics; it does not validate new metadata or promise corrected velocity, Session publication, native generation authority or arbitrary provider state rollback.

`NativeSession` borrows either the existing SimulationSession or MaterialSession owner. Requests and snapshots retain their native family, fields and precision. Family, declared SessionIdentity (run id and source revision), and generation guards precede callbacks/workspace/native stepping. MAC calls the existing pressure-provider path with no preferred candidate; particle preview calls the existing MaterialSession. Owners retain quota, deadline, actual shortened interval, cancellation and accepted-history publication. No accepted history is copied or converted. Subgraph/whole-family candidates and both versioned transfer directions return explicit unavailable errors.

Existing production changes are only module declarations in lib.rs and solver_provider.rs. No numerical formula, private storage, geometry, existing particle method/physical fixture, persisted profile, process or UI changes. New adapter tests reuse the fixed native fixtures. Q1/Q2 numerical work remains independent.

## Source-bound evidence

Rust 1.92.0, unchanged Cargo.lock; fixed CPU fixtures and tolerances are in the protocol. Raw command logs are retained outside Git at `/workspace/rheon-c1a-evidence`; the compact manifest records their SHA256 and source identities.

| Check | Result | Source / scope |
| --- | --- | --- |
| Initial focused no-default C1-A tests | 12 passed, 0 failed, 49.05 s | Pre-5c9fb96 working tree; differs only by module declaration ordering; includes 100 bitwise MAC intervals |
| Focused default-feature C1-A tests | 12 passed, 0 failed, 53.49 s | 5c9fb96; same fixed fixtures |
| Existing eight contract targets | 33 passed, 0 failed, 3 ignored | 5c9fb96; configured/material/pressure/provider-step/Session/configuration/interval/solver-provider |
| Borrow compile-fail Rustdoc | 2 passed | 5c9fb96; pressure view and native snapshot prevent overlapping mutation |
| Final affected test reruns | 11 passed in each configuration, 0 failed | ac6ed17; no-default 2.34 s, default 1.17 s; test refinements and annotation only |
| Final exact-source 100-interval equivalence | 1 passed in each configuration, 0 failed | ac6ed17; no-default 46.94 s, default 48.26 s; completes all 12 final-source cases per configuration |
| Strict no-default/desktop all-target Clippy and formatting | PASS / PASS / PASS | ac6ed17; both Clippy commands use `-D warnings` |
| Independent architecture and contracts review | PASS / PASS | ac6ed17; [dispositions](c1a-review.md) |

The final patch contains only a local report-enum lint allowance, a named test observation struct and a stronger exact cancellation assertion. It changes no runtime operation. Final affected-test runs cover 11 cases; separate passed 100-interval runs bind the deciding equivalence case to exact ac6ed17 source in each feature configuration. The initial no-default run preceded a declaration-order formatting repair, so it is retained with its precise attribution rather than treated as an exact committed-source result. No physical or G1 GUI suite is presented as new evidence.

Pressure tests compare the original dispatcher on both native implementations: exact classical candidate, short/nonfinite writes, wrong gauge/field, support/execution/cancellation refusal, same-input fallback, invalid inputs/settings and original path-sensitive zero-iteration behavior. Native tests fingerprint every accepted native field and publication/source/interval/frame/report fact, particle accepted controls and diagnostics; they check wrong family/run/revision/generation, invalid native input, quota/stop/deadline, early/final cancellation and retry. Pressure report timings are excluded only for cross-run numerical equivalence; unchanged-history fingerprints retain them. No arbitrary external side-effect rollback is asserted.

Three ignored existing tests require `RHEON_AI_EXPORT` from the hash-pinned checkpoint: real checkpoint replay/corruption, frozen-model selection/fallback and 24 production intervals. No real-model result is claimed. Classical adversarial providers are identified as test fixtures, not AI. C1-B/C, transfer qualification, broader family substitution/C1-R, native GUI, GPU, performance and physical accuracy remain outside this result.

## Integration custody

Branch `implement/c1a-typed-native-boundary-20261020` preserves G1 PR122 at67ef1e9 and tested f6b7c8c source. Main bc2f472, plan120 at8422f77, frozen audit119 and pressurewarmstart59 remain untouched. No merge, rebase or branch cleanup. Repository lead owns stacked dependency handling, hosted CI and all merge decisions.
