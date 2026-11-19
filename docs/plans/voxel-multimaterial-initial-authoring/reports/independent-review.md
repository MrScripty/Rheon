# Independent review of the voxel multi-material implementation plan

Review date: 2026-10-21. Reviewer: separate agent `/root/voxel_plan_independent_review`, distinct from the plan author, numerical adviser and visual-document author. Reviewed the authored candidate plan and reports in the working tree based on production source `d67aec1a4575e0a61aa8cd931c8eeec33c8dc6a3`, with actual Coding-Standards source at `0fc92a24279f3a4574c80af0a2f31ea347a21597`.

This is a **static source, mathematical and plan-contract review**. It is not numerical acceptance, native interaction qualification, a standards-engine certificate or independent re-execution of previous evidence. I inspected the plan, numerical/editor/persistence/verification/design reports, issue/ledger text and local HTML, and checked relevant pressure, viscosity, native selection, saved-source/comparison and recording source. No simulation, test suite, benchmark cohort, native window, production edit, staging, commit, push or publication was performed. Only this review report was written by this reviewer.

## Required corrections

### R1 — Major numerical inconsistency: viscosity of the gravity predictor spoils the stated hydrostatic pressure equilibrium

Locations in the initial candidate: `reports/numerical-contract.md:44–49` (especially step 4), `:122–128`, and the pressure-equilibrium acceptance row in `reports/verification.md:16`. The HTML's numerical stage list also presents pressure before viscosity and does not define the same sequence.

The proposed step currently performs momentum transport, gravity, viscosity, then projection. The plan separately requires the discrete hydrostatic pressure difference `p_R-p_L=rho_f g_d h_d`. Those claims are not mutually consistent when viscosity is nonzero in a closed box.

From rest, applying gravity first makes an internal normal-velocity predictor `g dt`, while impermeable outer normal faces stay zero. This predictor has normal strain in cells next to those walls. The proposed positive symmetric-strain operator then changes that predictor before pressure sees it. Projection can remove the resulting one-dimensional velocity, but its pressure gradient balances the **viscously modified predictor divided by dt**, rather than exactly the captured gravity. Consequently a near-wall pressure difference is generally different from `rho_f g h`, even though final velocity can be zero. A visually stationary frame cannot distinguish the error.

Source corroboration: `src/viscosity.rs:247–268` forms normal strain from neighboring face velocities, including fixed outer zeros, with nonzero normal-strain stress. The proposed variable-coefficient form retains this structure. The counterexample does not depend on a specific solver implementation or a simulated measurement: even uniform positive rho/mu in a one-dimensional column with multiple cells has the artificial predictor-wall strain. Distinct Water/Air viscosities do not remove it.

Required correction: choose an ordering that satisfies the stated balance. The minimal correction is transport → viscosity of the advected candidate → gravity → variable-density projection, using the same end-candidate masses throughout. Alternatively an explicitly pressure-balanced predictor could work, but adds scope and needs its own derivation. Keep the numerical prose and visual in agreement. Add a focused future hydrostatic regression with nonzero, unequal viscosity and gravity, checking pressure differences as well as velocity; it must detect the original ordering. This does not request any simulation execution in the planning task.

### R2 — Medium integration/write-set gap: new saved variants have existing comparison consumers

Locations: M4 write set in `plan.md:83`, new saved variant in `reports/authoring-persistence.md:57–59`, and the composed-design/caller-consumer discussion.

The proposed `SavedRunSet`/`SavedMember` extension through `run_sets/source.rs` reaches existing exhaustive consumers outside the listed M4 population. Relevant source includes:

- `src/bin/desktop/comparison.rs:450–469`: Native/Configured `SavedRunSet` match, plus `SavedMember` matches around `:588–598`.
- `src/bin/desktop/comparison_field_view.rs:593–596`: Native/Configured member geometry match; its payload estimate around `:620–628` also assumes current f32 face fields and existing cell inventories.
- `src/bin/desktop/comparison_archive.rs:137–170`, `:442` onward: capture/readiness and selected-source validation over the same saved-source abstraction.
- Existing `configured_comparison_tests.rs` covers that source composition; `rheon-desktop.rs` and `run_set_view.rs` are already listed.

Adding the new variant requires coordinated consumer disposition even if comparing voxel-material runs is intentionally unsupported. Otherwise the “exact write set” does not preserve compilation/behavior, and an accidental generic route could apply old field inventories or memory estimates to f64 two-material publications.

Required correction: include the actual comparison consumer population in the M4 write set and contract traceability, narrowly for an explicit typed unsupported/refusal path and focused regression. Do not add a new comparison feature to meet the plan. Alternatively choose and justify a different integration boundary that avoids those consumers, rather than assuming it does. The new source dispatcher must distinguish the new supported version intentionally and preserve old malformed/integrity/I/O failures; broad parse-failure fallback is not a migration policy. This follows actual source and Coding-Standards' coordinated internal-contract/evolution rules.

### R3 — Low interaction specification gap: keyboard dab lacks an in-plane center

Locations: `reports/authoring-persistence.md:19`, `:31–35`, and `reports/verification.md:37`.

The contract promises an explicit keyboard single-dab operation “at a numeric slice position,” but the visible geometry controls specify only slice axis and its normal/depth coordinate. A point in a slice still needs two in-plane coordinates. Pointer users can supply them by picking; a keyboard-only path cannot obtain them from the stated depth control alone.

Required correction: define a minimal precise center mechanism, such as a focused in-plane cell cursor moved by keys with an Enter/Apply dab action, or two in-plane numeric coordinates in Details. It must share the same world/cell conversion, mask, preview and atomic transaction as a pointer dab. Specify how focus/cancel works. No new sculpting framework or additional brush is required. If keyboard dabbing is deliberately excluded, the plan must remove the stronger promised acceptance claim; preserving the existing named-controls accessibility intent is preferable.

## Confirmed design and source findings

**The voxel requirement is retained.** The actual `LiquidTransportConfig` and carrier keep one constant inertia; `PressureOperator::new` accepts one rho, current velocity transport is semi-Lagrangian, and current viscosity takes scalar rho/mu. The plan therefore correctly requires variable-density projection, matched phase/mass/momentum transport and variable viscosity. It does not pass a particle initialization adapter off as voxel support. The separate ideal-gas Euler owner is not a ready liquid/gas replacement.

**The minimal physical model is coherent as a proposal.** Two immiscible incompressible Newtonian phases, full occupancy, one velocity/pressure and no capillary force are a defensible bounded model. Gas has positive density and viscosity; it is not Vacuum. The model does not supply EOS gas compression, thermal/phase change, arbitrary fluids or all existing preset properties. Full occupancy, incompressible gas and no surface tension are correctly visible parent-review decisions while status remains Deferred. They are not yet accepted user preferences.

**Binary conversion is explicit.** Captured Water IDs map exactly to F=1 and Air to F=0 on the numerical lattice; holes produce an actionable refusal rather than automatic gas fill. Transported mixed fractions are numerical interface data, not a promise of miscible-fluid dynamics. Add/Remove/Replace and explicit whole-domain Air initialization have clear initial mass effects.

**The finite-arithmetic proposal is not disguised as exact conservation.** The report correctly distinguishes conservative face-flux transport from its constant-preserving realization, with difference `dt F_i d_i/V`. Independently substituting the latter into rho(F) gives the stated cell-mass continuity defect `dt rho_i_old d_i`; the half-cell dual relation and `-U R_f/m_new` constant-velocity defect follow. Tree closure is not declared exact in binary64. Direct phase totals, strict bounds, density-scaled cumulative arithmetic budgets and a stop/replan gate are appropriate. Whether this proposed realization passes Water/Air cases remains unverified; it must not be described as scientifically validated or permitted to reject normal supported workloads indefinitely.

**Pressure reuse is feasible with the stated audit.** Existing f64 `PressureWorkspace::solve_rhs` can consume a correctly extended weighted operator. `neighbor_weight`, `fixed_box_weights`, `apply_full`, `diagonal`, correction and the SGS consumer really do contain current scalar/full-box assumptions. The report correctly names them and declines untested SGS parity. Equal-density tests alone would miss these defects.

**Variable viscosity has an appropriate mathematical owner.** A positive weighted strain form with the same dual mass matrix and a derived explicit stability bound is materially different from painting viscosity labels or smoothing by scalar nu. The harmonic shear-site choice is explicit and gets an independent layer/stress test; positive energy alone is correctly not equated with continuum interface accuracy.

**Undo, persistence and run ownership are proportionate.** One committed document, private candidate overlay, exact ordered before/after cells, one stroke transaction, no-op/cancel preserving redo, bounded whole-command eviction and separate retained selection address the main gesture hazards. New immutable saved revisions, explicit uncertain-publication reconciliation, session-only undo and no unrequested recovery journal are clear. Captured input bytes/coefficients/method—not mutable presets, preview voxels or current Draft—own cold rerun. Recording is not claimed to be complete mid-run restart state.

**Standards status and evidence limits are honest.** Actual Core/Router/Planning/template/architecture/evolution/persistence/oracle/GUI guidance is used with a complete composed-design probe. Deferred status, one next possible slice and parent admission/publication authority are explicit. Proposed HTML is labelled and grants no approval. No external library is imported, and the comparison of difficult numerical ownership versus existing/foreign alternatives has a real stopping condition. The plan does not fake native, performance or numerical acceptance.

## Review disposition

R1 must be corrected before this numerical sequence is treated as an implementation-ready proposal. R2 should close the real consumer/write population; R3 should make the promised keyboard operation executable. After the author collects and corrects this complete round, a targeted independent rereview can verify those changes without starting numerical/native work. Parent still decides whether the proposed physical restrictions meet the desired first product. All A1–A6 feature claims remain pending regardless of plan-review approval.

No approval denial, uncertain execution or other action blocker occurred during this review.

## Correction verification — targeted independent rereview

On 2026-10-21 the same independent reviewer reread the corrected technical documents and HTML. The original R1–R3 findings above record the initial candidate; the following dispositions supersede their requested-correction status. This round checked the corrections and their cross-document consistency, not a new numerical or native evidence cohort.

| Finding | Actual correction inspected | Disposition |
|---|---|---|
| R1 | `reports/numerical-contract.md` §2 now performs transport → viscosity of the advected candidate → gravity → immediate variable-density projection with the same end-candidate mass. §5 explicitly requires nonzero unequal viscosities and near-wall hydrostatic pressure checks. `reports/verification.md` A3 requires the wrong gravity-before-viscosity order to fail pressure even if final velocity is zero. The HTML stage list now has the same corrected order. | **Resolved at plan-contract level.** The rest-state counterexample is addressed. Correctness of the future discrete implementation and its pressure/stress tolerances remains pending. |
| R2 | `plan.md` M4 explicitly includes `comparison.rs`, `comparison_field_view.rs`, `comparison_archive.rs` and narrow `configured_comparison_tests.rs` changes. `reports/authoring-persistence.md:61`, the A5 consumer row in verification, and the composed-design caller paragraph require typed unsupported comparison handling before legacy f32 inventory/byte estimation, explicit supported version/kind dispatch, and preservation of malformed/integrity/I/O failures. | **Resolved at plan-contract/write-set level.** This closes the identified real consumer population without granting a new comparison feature. Implementation must still enumerate every actual consumer and verify the refusal/error paths. |
| R3 | `reports/authoring-persistence.md:37` defines Y/Z, X/Z or X/Y world coordinates for the corresponding slice axis, with slice depth supplying the third coordinate. Details has a named Apply dab action; pointer-equivalent geometry, validation, one atomic transaction, Escape/focus behavior and active-pointer exclusion are explicit. The A4 verification row and HTML keyboard note agree. | **Resolved at interaction-contract level.** No keyboard or pointer operation has been executed; native accessibility and geometry equivalence remain acceptance work. |

The candidate still says `Deferred`, with M0 review/parent admission as its current phase and M1 as the single next possible implementation slice after admission. The HTML visibly identifies the fixed Water/Air pair and the pending full-occupancy, incompressible-gas and no-surface-tension restrictions. The institutional Rudman reference repair is present in the current standards/design report; it changes no reviewed numerical contract. This rereview did not independently re-fetch that bibliography entry.

No further required correction was identified within this targeted scope. That is a review disposition, **not implementation admission or physical/numerical qualification**. Parent must still accept or change the product restrictions; all A1–A6 evidence remains pending. In particular, conservative bounded transport at Water/Air contrast, finite-residual defect budgets, interface stress accuracy, native editing and immutable cold rerun have not been demonstrated by this review. The M2 stop/replan condition remains consequential.

### Reviewed semantic document identities

SHA-256 values captured after the corrections above. Issue and execution-ledger status are excluded from these semantic identities, as is this review report. Later semantic edits require appropriate review of the changed content; factual issue/ledger bookkeeping does not itself invalidate these content hashes.

| Topic-relative file | SHA-256 |
|---|---|
| `plan.md` | `7732fdf67bd81ba15c6d13aee7cd660d21266d27ca22ad2d793760a91cdcdcaf` |
| `reports/numerical-contract.md` | `12acc57af8ee4e39920d4a989bca823766cb0b43fd9f1c449a3c5b999bc21c1b` |
| `reports/authoring-persistence.md` | `4eeac42bfe40b56ef39d0064915c0584d636deeb5bd4adf6fd7f9bfa9b1d74df` |
| `reports/verification.md` | `cb144d2efa6556beb221e498003f8f61145af0b09c2049abfb715d2ff3d6973b` |
| `reports/standards-and-design.md` | `d7f3e1d06b124c011f8b399bf9eb8aa1faae76befe7fa20ad4df92d48645df51` |
| `reports/proposed-flow.html` | `1a2ce9ad95722207913acae4f5a145660784857d31593c646708f8e396c17222` |

Only this report was changed by this reviewer. No tests, simulation, benchmark, native execution, staging, commit or external publication was performed. No action was denied, pending or uncertain.
