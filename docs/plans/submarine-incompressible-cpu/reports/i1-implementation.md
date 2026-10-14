# Native bounded I1 implementation and handoff

Status: implementation source complete; final corrected-profile checks and fresh independent-validator captures are running. N1/N2 numerical acceptance is not claimed; N3 continuum mesh/domain convergence and experiment rights/model/uncertainty remain pending.

## Authority, readiness and source ownership

Parent transferred native ownership after original worker `01a139a7-da3b-71b3-b155-ddca2f780005` confirmed no files/refs changed during failed provisioning. Current executor filesystem readiness succeeded: original planning worktree was clean on `planning/submarine-incompressible-cpu-20261014` at `9efef18798d48268d9c5f28ea49e968cde55a39f`. Created separate `/workspace/Rheon-submarine-i1`, branch `implementation/submarine-i1-native-20261014`, from current main `447ffc03ebc1f24ba99a221b868b153146b13992`. Rust/Cargo 1.92.0 were available through `/workspace/rheon-setup/env.sh`; existing `cargo check --locked --no-default-features --lib` passed. No AGENTS.md or repository agent skill was found.

This worker owns exactly:

- `experiments/submarine_stokes.rs`
- `examples/submarine_stokes.rs`
- `tests/submarine_stokes_contract.rs`
- this report

Canonical plan/ledger/issues, independent PR79 validation files, all existing solver/session/provider/recording/desktop/browser source, shared exports, Cargo/locks, CI and source-admission inventories remain unchanged. Any needed shared change requires parent serial coordination. Parent's merged PR77 I1 contract remains authoritative; this report does not transition numerical acceptance or authorize later slices.

Parallel implementation used GPT-6.1 Sol High core and exporter workers with disjoint files; root owned contracts/report and serial integration. Separate GPT-6.1 Sol High review is read-only. Independent numerical validator remains owner `01a137af-34d5-7718-89dd-45099419f02c`, PR79; source/expected values are never imported into native assembly or substituted for evolved unknowns.

## Exact corrected contract

Parent accepted `rheon-i1-sector-half-cell-v1` after the independently reviewed corner-decision report `597c3ca02a375cf47e4be083c658857fa2e66ff6`. The corrected validator handoff is `dedf04fb0d094064f5a163a4f7b78e7237e4438f`:

- Fixtures: `rheon-submarine-i1-fixtures-v2`.
- Native record: `rheon-submarine-i1-mac-v2`.
- Operator: `rheon-i1-sector-half-cell-v1`.
- Exact fixture SHA256: `3762540394238820ad47ca7ce4f8fc5b140e11abc0d52458a2451a4fe588f160`.

The example pins these exact fixture bytes before parsing and refuses all stale/unagreed fixtures. All 18 case objects, 34 constitutive controls, caps, pivot floor, residual/divergence thresholds and wall stencils remain the validator's frozen inputs. New corner keys and explicit `reaction_sites` retain actual component, owner and SI point. A read-only detached validator worktree at `/workspace/Rheon-submarine-i1-validator` pins that source for actual execution/provenance; it is not merged or edited into the native branch.

## Numerical behavior and finite limits

`Solver::new` validates immutable grid/body geometry and borrows the existing `StaticObstacleGeometry` for aligned padded cuboids. It verifies source grid, body bounds, connectivity, fluid volumes and shared openings; no-body qualification uses existing `GridGeometry`. Body faces retain two complete fluid layers. Active/nongauge-pressure counts and the hard 4096 combined DOF cap are checked before numerical buffer allocation. Managed allocation uses checked counts, fallible reservations, actual retained Vec capacities and a caller payload limit. Borrowed geometry, allocator overhead, stack, callback/exporter buffers and RSS remain separate.

One dense matrix, bounded work vectors and sparse physical rows implement steady incompressible Newtonian Stokes. No convection, physical dt/trajectory, moving hull or turbulence is invented. All six outer planes prescribe full-vector Dirichlet values; the stationary cuboid is no-slip. Half-dissipation yields normal Hessian weight `2 mu V` and sector shear weight `mu V/4`. Identical interior/flat-wall sectors are aggregated; a three-fluid corner retains all three distinct half-cell rows and full prescribed face/edge columns. No interior zero anchor or former clipped singleton is introduced.

The mixed equations retain prescribed affine terms and known first-fluid-cell pressure-gauge momentum contribution. Column nondimensionalization uses U_ref and mu U_ref/L_ref, then row maxima scale the bounded partial-pivot solve. The fixed 1e-12 pivot floor refuses unresolved conditioning; it is not a condition-number or accuracy certificate. Stored candidate fields are checked against independently traversed sparse physical equations and every wet-cell divergence row, including gauge. Allowances use expanded term magnitudes times 4096 binary64 epsilon, without a unit-sized floor. These in-module checks are acceptance controls, not independent external validation.

Pressure uses the affine-exact `(3 p_near-p_next)/2` wall trace. Normal and tangential no-slip velocity derivatives use the admitted quadratic one-sided stencils and explicit MAC collocation. Physical pressure/shear forces and moments remain distinct from full-functional prescribed face/ghost/corner reactions. Reaction moments use the actual declared boundary sites. Dissipation is 2 Phi; outer/body work and forcing work use the same functional. Pure `integrate_patch` is shared by solved wall loads and the 34 native constitutive-only controls; the independent validator supplies their expectations.

Every solve reuses retained state/workspace without heap allocation. Invalid data, cap/conditioning/arithmetic refusal and cooperative cancellation leave accepted fields, pressure, traces, patches, reactions, loads, request, report and solve ID intact. Candidate and accepted buffers swap only after final gates. Constructor failure never creates a public owner. There is no public-library promotion or production capability flag.

## Exporter and dependencies

The CLI requires external fixtures, one frozen case and a fresh output directory outside the checkout. It runs the native solve and serializes actual accepted arrays, full-vector traces, raw pressure/gradient patches, reactions/sites, separate loads and resource/report data. Exact native binary64 values use lowercase 16-digit numeric `to_bits` hex; canonical sorted compact JSON binds geometry identity. No field is replaced by an analytic reference. Analytical constitutive controls are explicitly labeled and do not count as native body solves.

Input/record size is bounded at 16 MiB; output uses exclusive temporary creation, fsync and rename in a fresh invocation-owned directory. Failure emits no accepted `record.json`; a caught write error removes only its disposable partial record, retaining the fresh invocation directory. The adapter's JSON allocation is separate from the numerical payload cap; no RSS, power-loss or real-time promise.

No dependency or Cargo change was needed. The existing `gltf` 1.4.1 library documents generic JSON Value/deserialization/serialization reexports specifically for non-Root data in `gltf-json` source lines 85–108; it handles JSON semantics rather than introducing a second parser. Existing `sha2` 0.10.9 supplies hashes. Both authoritative bundled terms offer MIT/Apache-2.0; select Apache-2.0 terms consistently with the project and preserve existing notices on distribution. Their consumed identities/features remain in the existing lock. No GPL/unlicensed code, data, mesh asset or paper text was imported. Original box fixture geometry and mathematical formulas are authored inputs.

## Verification and evidence scope

Before corrected serialization alignment, 11 native contracts passed in debug (36.88 s) and release; a stronger populated-body partial-wall/precommit rollback test then raised the suite to 12, passing in release (4.80 s). Combined Clippy on example/test with `-D warnings` and focused formatting passed. Final corrected-schema source passed all 12 release contracts (5.92 s), combined example/test Clippy with `-D warnings`, and focused Rustfmt. Corrected PR79 harness passed 31 normal and 31 optimized-Python tests (9.170/8.931 s) without modification. These harness tests are not native-capture evidence.

Contracts cover 669 mixed unknowns (474 velocity plus 195 pressure), cap/geometry preflight, the independent local corner Phi=1/64 versus old clipped zero, all-axis/oblique no-body uniform flow and pressure datum, nonzero tiny-body drag/reversal, no growth of retained capacities, Newtonian rotation/shear/pressure/moment identities, cancellation across nine stages and populated-body partial reconstruction/precommit rollback. Capacity constancy supports no-growth; source review proves the absence of solve-time allocations. It does not establish allocator/RSS or arbitrary-input performance.

Fresh capture must bind a clean committed native source/tree/archive, exact executable/toolchain/flags, fixture bytes, validator source/file hashes, actual commands and output hashes using the unmodified PR79 runner. Generated source archives, binaries, records, assessments and origins remain outside the checkout. Parent or independent executor pins/rechecks origin integrity separately from numerical correctness; hashes alone are not signed execution authentication.

## Independent review and remaining gates

Read-only GPT-6.1 Sol High source/design review found no blocking numeric/ownership issue in the original complete implementation and requested a populated-body rollback discriminator, now added. Corrected corner serialization/reaction-site and row aggregation alignment received affected re-review: **PASS**, no blocking source finding. The independent reviewer executed pinned Rust 1.92.0 release contracts directly: 12 passed, 0 failed (5.72 s). Exact reviewed source hashes:

- Core SHA256: `e157935c2f3c1e5c832ba6c9365aadeb6cb1cdcdc3b83a5ae784c1d73ba7a251`.
- Exporter SHA256: `0181c29ed7dbca79f8e809c3f5572f841f215de397a4f2b8542b89510ec1115a`.
- Tests SHA256: `f59d757f7e26ac7a5a1098424981dafc9c7c0e602fb8812343cea5700a416b05`.

Actual tiny constructor: 669 unknowns, 1464 physical rows, 780 prescribed columns/sites, 54 unique body trace points, 48 corner keys; retained numerical payload 4,256,913 bytes. This is a finite measured fixture, not a scalable memory guarantee. Final review binds tested source hashes, rather than an unrelated ledger-only commit identity.

The runnable result is tiny steady synthetic-body creeping Stokes, with native fields and explicitly differentiated loads. Coarse wall reconstruction versus algebraic load agreement is not continuum force accuracy. All-column/row checks and constitutive controls cannot close N3 mesh/domain uncertainty, fine MMS order, an experimental SUBOFF gate, turbulence, ascent, moving-boundary GCL, Tuldok/recording promotion or restart. Parent retains full numerical acceptance authority with the independently assigned validator. No tags, merge, force-push or deployment is authorized by this implementation task.

## Capture integration diagnosis

First exact-source attempt at `a486f7c825b7ab3ca66b724a2cb296dae19de304` built the native executable but failed before any accepted record: the example required a pre-existing parent while the PR79 runner passed fresh `cases/CASE`. Origin is terminal `failed` with zero outputs at `/tmp/rheon-i1-capture-a486f7c`; actual source archive, executable and command streams are retained. This was an exporter/runner path mismatch, not a numerical failure or acceptance result.

The owning example now checks an existing canonical ancestor outside its checkout before creating fresh parent directories, rejects unresolved parent traversal, rechecks the canonical parent, then exclusively creates the case directory. This completes the already admitted fresh-output CLI contract. No numerical operator, fixture, schema, pivot/residual threshold or independent validator changed. Combined example/test Clippy and focused formatting passed after the repair; affected independent review precedes a new source freeze and fresh run directory. Failed source history and capture artifacts are not rewritten/resumed.

Affected independent GPT-6.1 Sol High review returned PASS for the runner's absolute `/tmp/RUN/cases/CASE` path and exporter SHA256 `27948d2f45ca555ba8d65bdbdb0c3018a77fb864e122965e9f8b9c6adebd2a06`. Core/test hashes are unchanged. Absolute runner paths are qualified; a relative nested missing parent can explicitly refuse and is not a promised capture path. No retry reuses the failed origin or rewrites shared source history.
