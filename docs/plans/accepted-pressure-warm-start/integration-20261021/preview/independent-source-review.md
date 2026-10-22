# PR59 preliminary PR142 composition source review

2026-10-21. Workspace `/workspace/pr59-composition-preview`. Review base is published PR142 `5eb8bf5607c01f1d59242c604b1d1204882d2eac`. Frozen preview HEAD `d0452c79794c33bb0f9af14de0213a26bb5b1cd0`, pending MERGE_HEAD `5eb8bf5607c01f1d59242c604b1d1204882d2eac`. This report binds the working bytes below; it is preliminary and not a review of a completed merge commit or final main composition.

**Decision: no blocking source defect found in the reviewed composition.** No material unresolved source conflict was found. The index still records the expected in-progress merge entries for `src/liquid_step.rs` and `src/simulation.rs`; this is not a completed Git merge. Production/test source has no conflict markers and scoped `git diff --check` passed.

## Scope and evidence

Read the warm-start delta against PR142, the native projection carrier/publication path, immutable Session intent guard, projection producer/qualification/sealing context, workflow publication path, recording profile guards, and new test additions/helpers. Confirmed the ten supporting projection/workflow/recording production files listed below are byte-identical to PR142. Confirmed the pressure solver/history module and six original warm contract files listed below remain identical to reviewed source `0b3e3fc`. This is a composition review, not renewed qualification of every scientific or desktop change already owned by PR142.

No builds, tests, benchmarks, binary rewrites, GitHub writes, cloud tasks, artifact downloads or repository mutations were performed. Primary-worker test results, source-admission outcomes and campaign records were not independently executed here. No source inventory or CI gate was changed by this reviewer.

## Assessment

1. **Warm eligibility is correctly separated from native projection.** `native_sealed` now requires `projection.is_none()` in addition to all prior sealed-route checks. Projection therefore yields no operator key, borrows no accepted pressure seed and stages publication with no reusable seed. The private Liquid native-projection entry explicitly passes policy Cold. Both builtin and classical-selection producer routes use the existing cold ProviderWorkspace/qualification path; their supplied current-attempt candidate is not reinterpreted as accepted-history initialization. Ordinary native legacy publications can still establish eligibility. Any successful unsupported projection publication invalidates legacy eligibility through the staged history; a failed attempt returns before history assignment.

2. **Immutable Session intent cannot be bypassed by the new API.** `initialized_step_with_clock` calls `require_legacy_intent()` before stopped/quota/deadline/work/callback handling. That same guard already protects the existing legacy/provider/configured entries. Both explicit Cold and AcceptedPreviousV1 initialized requests refuse on a projection Session before callbacks, state changes, quota changes or receipt adoption. Intent remains constructor-only/private. The new warm path therefore cannot replace a projection receipt with an unqualified legacy step.

3. **Final sealing and publication preserve transaction boundaries.** Native projection prepares/qualifies disposable pressure and velocity, then runs complete carrier/transport gates. Warm history revision/exhaustion is staged before the final callback. The existing projection path then seals a prospective full-state digest/validated attempt before any accepted pressure copy or owner swap. Seal/alias/validation failure returns without committing fields or history. After sealing, pressure copy, carrier/volume commits and history assignment remain infallible owner updates without callbacks. Session adopts the already sealed attempt and updates actual interval start/report/provider metadata/quota only after Liquid succeeds. Failed attempts retain the accepted Arc/state; separate failure evidence does not replace publication. Deadline semantics remain the existing cooperative checkpoint semantics, including the final callback; no hard wall-time guarantee is implied.

4. **Workflow composition is preserved.** Workflow capture enters through configured calls, which remain cold and invalidate native warm eligibility only on successful publication. Warm initialized calls have no configured workflow capture and clear stale provider/configured receipts/history only after acceptance. The new initial-pressure parameter is forwarded alongside, rather than replacing, workflow and projection parameters; force/workflow completion and provider selection branches from PR142 remain present. Workflow receipt preparation/allocation remains before the existing infallible swaps. No additional warm recurrence is introduced into configured/projection dispatch.

5. **Recording boundaries remain explicit.** Projection has distinct immutable-intent run/publication/terminal profiles and validates its selected native role/attempt. Configured capture refuses projection Sessions. Workflow wrappers validate their configured profile and do not silently interpret projection archives. Accepted warm legacy steps have no configured receipt, so configured publication capture cannot invent one. Existing dense3D writing explicitly does not encode warm history/provenance. No archive/wire format or replay capability was expanded by this merge; no claim of portable warm restart is supported.

6. **New tests address the integration seam.** The public projection contract tests both initialized policies refusing before callbacks and retaining the accepted Arc/frame/pressure, then exercises projection retry through the existing continuation. The added private test covers both pressure implementations: actual projection report stays Cold, no legacy seed appears, final cancellation retains accepted state/receipt, both initialized policies refuse, and a later projection succeeds without establishing seed eligibility. Shared state helpers include field/time/stamp/pressure digest and Arc identity. Existing projection fault/alias/deadline and recording tests remain inherited from PR142; original warm cancellation/dirty-scratch/numerical/provider contracts remain byte-identical. Source inspection does not replace execution of default and feature-enabled projection, recording, workflow and warm suites. The new cold-only test fixture uses BuiltinNativeV1; classical selection is protected by the common guard/eligibility code and existing selection tests, rather than a duplicate new warm-specific selection fixture.

7. **Default feature exclusion and numerical gates are preserved.** All warm API exports, initialization parameters/branches, owner history/identity and report fields remain Cargo-feature-gated. The default feature set remains unchanged, with capability false without the feature and Session API version 3 explicit. Pressure source bytes match the independently reviewed candidate, preserving full compatibility/full-gauge residual/current-RHS/divergence gates, iteration budget, cold recurrence and absence of an implicit warm-to-cold retry. PR142 numerical/receipt source is not loosened in the preview diff.

## Admission limitations

This is only the preliminary PR142 composition. Bind these hashes to the eventual commit and re-review changed material before relying on the report. Planned #138/#143 merges require source/dependency comparison and any necessary renewed review; the final actual ordered main baseline controls all final qualification. An unchanged native source inventory currently does not admit the new warm module; lead-owned inventory/CI work and warm CI coverage remain outstanding and must pass without bypass or weakened gates. No final source admission, timing acceptance, scientific campaign recertification or normal-merge authorization is given here.

The original 1000-particle **+2.20%** adverse confirmation remains retained and unresolved. Earlier initial-main executable inspections and raw/diagnostic artifact distinctions do not qualify this new composition. Final prospective controls retain the existing fixed cohort and strict positive/inconclusive stopping rule unless an explicitly authorized protocol change occurs. No new-main universal speedup, default equivalence or feature-enabled mixed-particle claim is admitted.

No denied action or uncertain mutation arose during this review. The remaining blockers are final commit/source binding, actual predecessor composition, execution of required checks, lead-owned source admission/CI, prospective controls and lead authorization; they are not source defects found in the frozen preview.

## Exact reviewed material (SHA256)

Supporting files are hashed as whole files even where this composition review inspected their relevant production/profile portions rather than repeating every inherited test or scientific path.

```text
c122573542e3367761aab241a1de89a3002142360415c2d154e3b294acdf31c4  Cargo.toml
c05e7887e95a97d1db1e5407a9b6d76d4cc94b419b1fb0265d42d4120ab38c3c  src/lib.rs
c3c64645e0af3b11bf279c0ba4fb2740db1b785f046f115d084d136f436d43f5  src/liquid_step.rs
fb70a35516764666acbedffb3c8cb2fbdc9c63690cfe87701085a9a5bf4c8827  src/pressure.rs
dc81cb8c5b9dd1719a8e513e87d655098688b18e32bbea576b72746cadd1a233  src/pressure_initialization.rs
69a152d5b2b8dfe36888b2dd243b4069fdc2cd17268a201f06027d5c1bd4bb8f  src/session.rs
509c853d7232d22fecbd14066178befb3174b34e0aedaf4bdc27e0b8263d86fe  src/simulation.rs
360e7feb3f4955903510c82d0711820ad4aa922ffea075fd7084028d07f57619  src/session/projection_tests.rs
7d30b0d10acfb461d7c09798728b2e6c5bde72c17a45447df54ea80a15f08cc8  tests/accepted_pressure_history.rs
06e1e1ed6b0b20278ef44e01daa5585b1095fb4850ff834b4c2b7257c87eeea7  tests/configured_provider_session_contract.rs
5ad1f1ac8aad421cef53af6aa7a3db71440cb2a0eb2d3d8f223e395c691aee14  tests/pressure_initial_guess.rs
1ad133448f734a3a576cfef3dfc63c46e655d83039c6728413e38cfccc573239  tests/projection_session_contract.rs
e5f6784f3ead54238f76f77bc6400d9007cd0e930a4381df22300fda4d012bf5  tests/provider_step_contract.rs
4503116a845296afcd058a988a3d343811944fa479cff3c489427c43db09e3c0  tests/session_contract.rs
40b64f4709fbe597431021e95e94ae2a1279d189b009b33323a8da362f9aea31  tests/session_interval_start_contract.rs
b94b144d441730a50f641656dbfd069e7182b8471a73e8aba8b4df17c53cf0a4  src/session/projection.rs
25a3478d1408ee98f55fc072fcef97631df5283bd01f13e3ac81d947450e5ce4  src/session/configured.rs
51fe13c95772d1e3b68f2c261b50d5e2ec49bbc7adf8ff2573e3f36c2c34df1c  src/projection_step.rs
c122744dfc3b4fccd4c5755df10fef1c12033e51d87c191d73efdb0cdaab28a3  src/solver_provider/composition/projection.rs
5d902d075f784c9c062d1d5372873b76844d1f8afb06218bff5c619c9d006eb9  src/workflow.rs
4273be1af82f7a1ab9c0f141fc58b663354e5fc96e625835ad1dbcef2588af34  src/recording.rs
49323046bc72879a80fae248a3f5f05018f461df4d26c185b7458a05fe225588  src/recording/projection.rs
3daa4c37b3f78dcc1c02454e00d5586b3e859c1671766ae0f7fc6984190b7826  src/recording/provider.rs
2cff480b60cdae14107541ce07d72afe2e1627684b82d427211df1bff023d381  src/recording/provider/runtime.rs
020b8cd9d63dc5afedfb1b99b316fe584a5f226241abe4be159af960b13d2cc1  src/recording/workflow.rs
```
