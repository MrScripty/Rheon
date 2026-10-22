# Independent M2 source, mathematics and test review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. Base: locally committed M1 `638008d4f06e885b681428a9d67bd5c4c3e0b01e`, following approved plan `627847915eb97b80da81ec4544bb413bc5f806fa`. Reviewed the complete new transport, viscosity and coupled-step owners; module/state changes; the M2 numerical admission; private kernel fixtures and the public complete-step contract file. The physical scope remains the approved closed, fully occupied incompressible Water/Air box without capillarity.

This is independent static source/mathematical/test-design review. I ran no numerical fixtures, simulations, test suites, native app or benchmark workload, and made no GitHub or Library calls. Root reports six private kernel fixtures and five separately invoked public coupled fixtures passed without adjusting their original assertions/tolerances. That report is execution evidence owned by root, not an independent numerical re-execution by this reviewer. Root is separately fixing routine formatting/lint findings; the semantic round below was collected before any requested semantic correction.

**Current disposition:** targeted independent rereview below resolves R1/R2 in the corrected candidate. No further required correction was found in that scope. The original findings and initial identities are preserved as review history, not current open blockers. Root owns execution/acceptance of M2's bounded numerical fixtures; native/authoring/persistence acceptance and main integration remain separate.

## Complete findings round

### R1 — Medium: strain-work roundoff budget uses a reduced signed scale

Location in the first candidate: `src/voxel_material/viscosity.rs:267–273`, with the expression paths in `strain` and the `work` accumulation in `apply`; corresponding budget claim in `reports/m2-numerical-admission.md`.

The current work check computes `work = sum_i u_i (Ku)_i`, compares it with the separately accumulated nonnegative strain dissipation, and budgets the difference with `gamma_(128N) * (abs(work) + dissipation)`. The magnitude of a final signed reduction is not an adequate absolute roundoff scale for its contributing products. Terms `u_i (Ku)_i` can strongly cancel. Cancellation can also occur in each strain `z = sum_i c_i u_i` and in assembled force components before the final dot product. Positive semidefiniteness of the exact strain form does not bound those absolute intermediate contributions by its final small energy.

For one ordinary rank-one contribution `K = w c c^T`, with `c = (1,-1)` and nearly equal velocities, `z` and `w z^2` can be small while `sum_i |u_i w c_i z|` is much larger. The error in a stored strain also contributes `w * |stored_z| * |exact_z - stored_z|`, whose forward scale includes `sum_i |c_i u_i|`. Multiplying only the small final `abs(work)+dissipation` by a larger operation count does not establish the required inequality. Multiple Cartesian strain rows, anisotropic spacing or nearly cancelling smooth velocities can expose this distinction; no performance experiment or observed failed fixture is needed to establish the missing scale.

Consequence: the current gate can reject valid candidate stress because normal cancellation error exceeds an unsupported budget, and its claimed roundoff certificate is not justified. The original passing bounded fixtures do not resolve that mathematical gap. This finding does not allege an observed wrong accepted simulation or require weakening a test tolerance.

Required correction: retain the fixed operation envelope unless a separately derived dependency count changes it, and derive the work-check scale from the actual absolute expression paths. Accumulate appropriately upward-bounded absolute rank-one strain/force-work contributions, including strain-evaluation sensitivity, and the final absolute dot-product terms where needed; compare work and dissipation against that scale. Merely replacing `abs(work)` by the final `sum |u_i (Ku)_i|` may still omit force-assembly/strain cancellation and needs its own derivation. Update the admission's explanation and add one small predeclared cancellation-sensitive strain/work fixture that checks the chosen absolute scale/identity independently. This is a correction of the mathematical error source, not an observed-output multiplier or relaxed physical gate. Keep direct energy identity, stability and original phase budgets separate.

Owner: numerical implementer. Disposition: required before treating the M2 work/arithmetic admission as accepted; targeted independent rereview after the complete round is corrected.

### R2 — Low: cancellation fixture accepts an arbitrary error

Location: `tests/voxel_material_step_contract.rs:217–243`, particularly `assert!(sim.step(...).is_err())`.

The fixture intends to qualify each named cancellation stage after nonzero accepted state. It checks state preservation and reproducible retry, which are useful, but the negative assertion does not require the declared `VoxelMaterialError::Cancelled` result or explicitly confirm that the selected callback stage was reached. This is narrower evidence than the stated targeted typed-cancellation claim. The immutable accepted-state/private-candidate design itself looks correct; this is a target-bound test issue.

Required correction: record that the callback sees the selected stage and match `Err(VoxelMaterialError::Cancelled)` before the existing identity/retry checks. No new broad harness, stage enum or numerical workload is needed. This does not request every possible callback poll count or treat a generic error as cancellation acceptance.

Owner: implementing test owner. Disposition: small focused correction before presenting the targeted cancellation claim as satisfied.

## Confirmed source and mathematical contracts

**Transport uses one flux generation.** Q is built from old accepted MAC velocity and checked fixed walls. The deterministic descending x-fastest tree visits descendants before parents; increasing the oriented parent-to-child face by the child's outward residual has the correct sign. Propagating the actual stored correction, then independently recomputing residuals, avoids calling binary64 closure exact. The original divergence, tree correction bounds and root sum are checked. The phase residual-volume gate further bounds the absolute residual effect rather than relying on a signed root sum alone.

**The fraction realization is distinct from its conservative reference.** The implementation uses incoming old donor differences with incoming/outgoing volume Courant at most 1/2 and strictly checks stored F in [0,1], without clamps. H uses the same corrected Q and old donor fraction; J uses the declared material mass flux. The local mass check includes `dt*rho_old*d_i`, rather than gas density alone. The per-phase volume budget includes absolute F evaluation scale, gas-complement inheritance, direct reduction allowance and the measured induced residual term. Gas is occupied complement fluid; no ambient refill or vacuum interpretation is introduced. Accepted budgets accumulate from the frozen initial phase-volume reference and failed attempts do not consume them.

The gamma arithmetic helpers use machine epsilon conservatively rather than unit roundoff, upward rounding at the named budget boundaries, and an explicit normal-or-zero policy. The phase/mass envelopes are sufficiently broad for their inspected short local dependency paths and include absolute mass/flux scales. These source arguments do not prove a universal floating-point theorem or extend the fixed numerical fixtures to arbitrary coefficients/conditioning. R1 specifically concerns the signed work expression, whose required absolute scale is missing.

**Staggered momentum matches half-cell masses.** Interior a-face dual masses derive from the same end-candidate F. For a-normal sides, dual J is the average of adjacent a-face primary fluxes; for transverse sides, it averages the matching b-face fluxes from the two a-adjacent cells. Subtracting upper/lower dual fluxes therefore gives the half-sum of the two primary continuity relations. The checked dual defect uses that half-sum. Donor component velocity follows the oriented dual-flux sign, and outgoing mass Courant is measured against old dual mass. Boundary transverse fluxes are zero; adjacent fixed normal face donors remain zero. Wall reactions are not mistaken for unconditional closed-box momentum conservation.

**Variable stress has the declared owner.** Normal strain rows carry `2*mu*V`; interior engineering-shear rows carry harmonic edge mu times V. Skipping outer shear rows implements the declared zero-traction free-slip boundary, while fixed normal faces do not receive mass division or updates. The scaled harmonic expression avoids reciprocal overflow. The force is assembled from weighted rank-one rows; a sum of their absolute mass-normalized row contributions is a conservative Gershgorin bound for the represented positive form, with downward-rounded square-root denominators and upward accumulation. The explicit stage checks dt times that bound against one, then checks both a work relation and the full explicit kinetic-energy identity. Those are distinct claims; R1 must correct the former budget rather than reclassify positive energy as sufficient interface accuracy.

**Complete-step ordering closes the hydrostatic plan correction.** Transport creates candidate F/momentum, viscosity acts on that advected candidate with its current masses, gravity is added afterward, and variable-density projection follows immediately. The public hydrostatic fixture checks pressure differences and small velocity on all three axes using actual unequal, nonzero Water/Air viscosities, including near walls. The sequence does not apply viscosity to the artificial gravity predictor. Arbitrary painted arrangements are not declared hydrostatic equilibrium.

**Atomic ownership and retry are clear.** The accepted state is read throughout numerical work. Transport overwrites all candidate F/velocity values on each attempt; pressure scratch and accepted pressure storage remain separate until checked copying. Only Courant/stress-stability failures may shorten dt, within eight attempts; each retry uses the same accepted fields. After projection, actual divergence, final volume Courant, cumulative phase balances, clock and report construction succeed before the final cancellation callback. The subsequent state swap, cumulative-budget assignment and report assignment are infallible and introduce no allocation/callback. A failed attempt leaves accepted fields/time/generation/report/budgets intact. R2 strengthens evidence of the intended typed cancellation route; it does not identify a source rollback defect.

**Aggregate capacity accounting composes the owners.** Two state buffers, the M1 projection/PCG, transport and stress yield the declared `8*(14*N+9*sum_faces)` minimum. Each constructor checks its remaining managed payload and actual Vec capacities before the next owner is admitted; PCG is not omitted or granted another independent full cap. Caller input, allocator/process overhead and future native/history/recording owners remain excluded explicitly. This is source/capacity accounting, not measured RSS or runtime evidence.

## Evidence adequacy and scope limits

The private dyadic donor fixture independently constructs conservative face sums and checks strict bounds, phase totals, selected contrast cases and continuity reports. The local uniform-velocity patch checks the matching-mass transport property away from incompatible closed walls. The tiny-residual pure-phase fixture reaches actual closure and the deliberately excessive residual reaches divergence rejection. The translated-box fixture has an independent per-cell intersection reference and a predeclared 8/16/32 decreasing-error trend; it does not claim a formal convergence order for a discontinuity or a product periodic boundary.

The stress fixture independently assembles a dense weighted strain matrix and dual masses, compares force/work, bounds the independent row spectral norm, and tests zero/distinct mu and an explicit stability failure. It currently uses well-separated velocity values; it is not a deciding oracle for R1's cancellation scale. The public Water/Air case runs a real coupled step with nonzero motion, changing F, downward liquid center of mass, strict phase bounds and original-reference balances. The scope is the predeclared 8-cubed, 20-step closed-box fixture; it is not calibrated multiphase validation, a continuous density/geometry envelope, a benchmark or a long-duration guarantee.

Fresh reconstruction compares the same numerical constructor and bounded repeated stepping. It is not saved-intent cold rerun in a fresh process. No native brush, save/load, new recording variant, retained material visualization, GPU, thermal/phase-change field or arbitrary-fluid product support is present. The private-kernel test placement is a reasonable refinement of the plan because independent oracles can reach their owner without a production test-mutator API.

The module README was still M1-focused when inspected; updating it to name the new coupled owner and its limits is already in the M2 admitted write set. That routine completion is not a separate numerical finding. The public integration's `bits` helper samples the retained report's key fields rather than every nested diagnostic; source ownership establishes that the whole report is replaced only at commit, so this is not presented as a bytewise snapshot of every report field.

## Initial review disposition and next boundary

M2 source composition is consistent with the selected physical model, but **R1 and R2 remain required corrections in this round**. After serial fixes, targeted independent rereview must inspect the budget derivation, the cancellation-sensitive evidence and the typed cancellation assertion, together with source identities. Root owns affected re-execution and records the original results without erasure. No current result authorizes arbitrary tolerance inflation, ordinary workload rejection as completion, native exposure or main integration. The plan's M2 stop/replan rule remains applicable.

The PR59 frozen baseline and sole integration lead still control shared-source integration; this review does not establish current-main equivalence or merge readiness. No denied coordination action or uncertain Library upload was retried.

### Initial reviewed semantic identities

These identify the first reviewed candidate, before the requested semantic corrections. Routine formatter/lint edits may be made separately by root; a corrected candidate needs the appropriate targeted rereview and updated identities.

| Path | SHA-256 |
| --- | --- |
| `src/voxel_material/mod.rs` | `930c4d4ae1c54e4738e91b9366842dc4a6001853fcf397ad16ac567de861615f` |
| `src/voxel_material/state.rs` | `d58a0f793a6175d564f120687cceb8a5d073dcfca13bde9b2e3c0e1a648d64d9` |
| `src/voxel_material/transport.rs` | `ffa2bc18dca930c85298fc682df13217de7b37081dea90fbf80c1a2b4288305d` |
| `src/voxel_material/viscosity.rs` | `7c28965a7784061d40045136145aa631cb317809ff5eec7f9f38b11131ccb432` |
| `src/voxel_material/step.rs` | `b4aa12a0947a92b57323c9d21659cd38c9d6a5df855fc1c2d21c2c87199f720e` |
| `tests/voxel_material_step_contract.rs` | `9789c10972fc006bb0b1eca484b1a8cd460e03e03c6c37fa481284e913cc056f` |
| `reports/m2-numerical-admission.md` | `524081901ab81424dc8af4bf415b6626841a1e9aa5b8b4ad2859e9bd00b4c019` |

Only this review report was written by the reviewer. No reviewer action was denied, pending or uncertain.

## Correction verification — targeted independent rereview

On 2026-10-22 the same separate reviewer inspected the corrected viscosity source, admission derivation, added cancellation-sensitive fixture, typed cancellation assertion and coupled-owner README. Root reports seven private kernel fixtures plus five separately unfiltered public coupled fixtures passed after these corrections, with the original numerical assertions unchanged. I did not rerun those tests. This is source/mathematical/test-design rereview of the full collected R1/R2 corrections, not independent numerical re-execution or a native/physical qualification campaign.

**R1 — resolved.** For each represented strain row the source now computes an upward T=sum |c_i*u_i|, then upward B=sum W*T². This scale remains large when the signed strain is small. In exact arithmetic |c·u|≤T; the stored strain's forward error also has absolute scale T, so rank-one force/work assembly and stored-dissipation sensitivity have a scale bounded by B with the appropriate short-path gamma factors. Local row arity is at most four; the global dissipation and dot dependency lengths are bounded by the admitted grid/face counts. The unchanged 128N envelope, using machine epsilon conservatively, dominates those dependency paths. This is a dependency-path bound, not a claim that the kernel executes at most 128N total floating operations.

The separately upward C=sum_faces |u_i*(Ku)_i| covers the final products/reduction from the actual stored force, and stored nonnegative D covers its separately accumulated view. The work gate now uses gamma_(128N)*(B+C+D), rather than the final signed work magnitude. Upward T/B/C scale construction uses the checked normal-or-zero arithmetic and refuses unsupported overflow/underflow; it does not silently replace a vanished positive scale by a smaller value. The direct explicit energy-expression allowance remains separately based on E_before+E_after+dt*D+E_update, and the source adds upward dt times the work allowance when substituting D for u·Ku. The spectral threshold, phase/mass gates, pressure limits and original numerical fixture assertions were not relaxed by this repair. The absolute bound may be loose for a nearly rigid high-speed state; it is not a promise of small relative work error in that condition.

The added private `near_rigid_strain_uses_absolute_expression_scale` fixture constructs an independently specified row c=(-1,1), W=0.3 and velocities 2^40, 2^40+2^-10. The dyadic difference and corresponding represented dissipation are known independently. It verifies the accumulated scale against W*T², checks that the observed two-term cancellation defeats the original signed-scale budget, and verifies coverage by the corrected absolute scale with the same operation envelope. It then traverses the actual anisotropic viscous-island `apply` work/energy gates. The artificial unit masses in the isolated row are explicitly a rank-one kernel fixture; the subsequent assembled stage uses actual face masses. The test is correctly labelled a component fixture, not divergence-free authored input or an application workload. This closes the demonstrated missing-error-source concern without tuning a multiplier to an ordinary run's outputs.

**R2 — resolved.** The public cancellation fixture now records `reached` when the selected named stage is seen, asserts it, and requires `Err(VoxelMaterialError::Cancelled)` before the existing accepted-state identity and deterministic-retry comparisons. A different error or failure to reach the intended callback can no longer pass that targeted negative assertion.

The new README now names `VoxelMaterialSimulation::new`/`step`, the matched transport/stress/projection composition, accepted versus disposable ownership, eight-attempt shortening policy, aggregate payload and numerical evidence limits. It still exposes no native option or saved-run format and makes no runtime/physical calibration claim. The other reviewed transport/state/coupled-step source identities below match the initial round; the changes under this rereview are confined to the corrected viscosity/work evidence, typed negative fixture and explanation/documentation.

**Final rereview disposition: no further required correction found within the M2 source/mathematics/test-design scope.** Root must retain and report the executed results at their actual bounded fidelity. This does not accept A4–A6, native UI, durable saved intent, arbitrary fluids, a continuous numerical envelope, runtime performance or shared-source integration. The original stop/replan rule still applies if ordinary supported future fixtures reveal material numerical failure.

### Corrected reviewed semantic identities

| Path | SHA-256 |
| --- | --- |
| `src/voxel_material/mod.rs` | `930c4d4ae1c54e4738e91b9366842dc4a6001853fcf397ad16ac567de861615f` |
| `src/voxel_material/state.rs` | `d58a0f793a6175d564f120687cceb8a5d073dcfca13bde9b2e3c0e1a648d64d9` |
| `src/voxel_material/transport.rs` | `ffa2bc18dca930c85298fc682df13217de7b37081dea90fbf80c1a2b4288305d` |
| `src/voxel_material/viscosity.rs` | `f88412afc7ec7dc0e09c970d9101a43b0b2a186cfd1e1af6636b27661606d084` |
| `src/voxel_material/step.rs` | `b4aa12a0947a92b57323c9d21659cd38c9d6a5df855fc1c2d21c2c87199f720e` |
| `src/voxel_material/README.md` | `0c0af5da344e3768a4bfc014a6bcadc8051a21e5b050dbf2796a9fb98ac42e4f` |
| `tests/voxel_material_step_contract.rs` | `4621a39a3f28faba72e3101bb5eac975a44d25ee1a9aca942e56ee8c5290b11c` |
| `reports/m2-numerical-admission.md` | `746000657f8fa21e58da7e68247f605e40eb33cfef42b79446abfab3ecb0f4a3` |

Only this report was changed by the reviewer. No reviewer test/numerical/native/benchmark workload or external mutation was performed, and no reviewer action was denied, pending or uncertain.
