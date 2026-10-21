# I1 implementation and independent acceptance handoff

Status: parent-admitted `Active` bounded I1, numerical acceptance pending. Canonical decisions and exact disjoint write authority are [plan.md](../plan.md). Implementation owner is `01a139a7-da3b-71b3-b155-ddca2f780005`; independent validator is `01a139a7-fc1b-7384-9200-e10dd033f016`. This lifecycle update executes no numerical code and changes no numerical scope, cap, method, fixture or gate.

## Native behavior and explicit finite limits

A host constructs one immutable padded aligned synthetic cuboid in a fixed physical box, supplies positive constant rho/mu and a uniform straight/oblique exterior vector, and requests a **steady Stokes solve**. It can observe accepted native f64 MAC velocity, cell pressure with declared gauge, pressure/shear forces and moments, geometry/boundary identities, solve generation, actual residuals/work and payload use. No physical dt, elapsed trajectory, convective wake or submarine accuracy is invented for a stationary solve.

Internal verification fixture input must also support the prescribed analytic nonuniform outer velocity and volume forcing in V07, plus the no-body/rest controls. Keep this mode private to the experimental lab and feed it through the same affine assembly and solve as uniform demonstrations; do not create a second solver or publish a generic boundary API just for testing. Arbitrary unsupported boundary/forcing policies remain unavailable.

First fixture: body dimensions L=1 m, breadth=height=L/4, COM at origin. Original geometry is an axis-aligned cuboid; it is not SUBOFF. Base finite-domain half-widths `(2L,L,L)` with uniform full-vector Dirichlet data on all six planes and stationary no-slip body. Base grid `(32,16,16)` at isotropic h=L/8 keeps cuboid faces aligned; fine `(64,32,32)` and `(128,64,64)` at h=L/16 and L/32 are prospective refinement fixtures, not guaranteed runnable within I1's direct-solve budget. Tiny runnable fixture: counts `(8,5,5)`, origin `(-1,-0.625,-0.625)` m, isotropic h=0.25 m, same body bounds `(-0.5,-0.125,-0.125)` to `(0.5,0.125,0.125)` m. It retains two full fluid-cell layers on each body side; its smaller domain and one-cell body breadth are explicitly discrete verification, not a resolved continuum drag specimen. Verify the resulting exact DOF count before allocating. It supplies the first bounded runnable host case. Straight vector `(-U,0,0)`, oblique `(-U,0,-U/4)`; use rho=1 kg/m³, mu=1 Pa s and U=0.01 m/s (nominal Re_L=0.01). Report that Stokes neglects inertia rather than claiming this approximation solves Navier–Stokes at any Re.

I1 dense backend is limited prospectively to **4096 total active velocity+pressure degrees of freedom**, checked before allocation; lower caller caps can refuse earlier. Store one matrix plus work/pivot/candidate/accepted arrays with checked capacity accounting and fallible reservations. At that maximum a dense f64 matrix alone is 128 MiB; the complete retained payload is larger and must be reported. No cubic-scaling speed promise, no per-request unbounded temporary matrix, no background work. The larger mesh/domain qualification grids above require an independently reviewed sparse/iterative extension or a smaller valid qualification family preserving wall padding; they must remain pending rather than being silently reduced after output. The native tiny-system path is useful as an affine boundary/load verification slice even while full load convergence is pending.

Before admitting a solve, independently review affine velocity elimination, divergence/gradient adjointness, symmetric Newtonian strain, component compatibility/gauge elimination, and pressure/stress wall traces. Derive the matrix, dimensions, corner treatment and nonzero boundary contributions explicitly. Reuse `StaticObstacleGeometry` and qualified `experiments/outward.rs` arithmetic; do not call sealed `StaticObstaclePressure` with nonzero farfield or use `AlignedStrain`'s omitted outer rows as a complete external operator.

Select bounded direct pivoted elimination for the tiny mixed system only after deriving rank/gauge/conditioning and scaled pivot refusal. Solve stored equations independently after elimination; never accept based on pivots or an internal iteration flag alone. Compare with mature PETSc adoption facts in [standards/design](standards-and-design.md); original finite direct code is justified only by this auditable tiny-domain resource/transaction contract. A need for scalable solver semantics, different factorization or new dependency is a replan, not a hidden fallback.

Accept all fields, load quadrature, report and solve identity together. Reject finite-invalid input, unsupported geometry, incompatible prescribed flux, singular/ill-conditioned unresolved solve, nonfinite/lost-positive arithmetic, capacity/allocation failure and cancellation. On each refusal preserve accepted bits and last accepted report. Attempt diagnostics have a different identity. Clock/generation of later time-stepping sessions belongs to I3/I7, not the steady solver.

## Load definitions and controls

With outward body normal n_b, `sigma=−p I+mu*(grad u+grad u^T)` and traction `sigma*n_b`. Sum `F_p`, `F_mu`, and `M_p`, `M_mu` about COM separately. Preserve the pressure datum and distinguish reconstructed physical traction from algebraic boundary reaction until consistency is proved.

Deciding controls include:

- Closed constant pressure: force and moment zero within declared floating-point closure bounds; do not use it as sole load evidence.
- Affine pressure `p=p0+a·x`: `F_p=−V_body*a`; `M_p=−V_body*(x_centroid−x_ref)×a`. Shift moment reference by d and require `M_new=M_old−d×F`. Include the documented naive adjacent-cell factor-two counterexample as a negative.
- Hydrostatic `a=(0,0,−rho*g)`: `F_p=(0,0,rho*g*V_body)`, torque about centroid zero; resting velocity/divergence and shear remain zero. Test shifted world depth and p_ref; neither changes buoyancy.
- Newtonian affine shear / independently prescribed analytic Couette and Poiseuille wall traces, including shear sign, area, units and mechanical work. No free-slip substitute for no-slip.
- Stokes linearity and reversal, all-axis mapping, oblique force components, symmetry, boundary momentum closure, viscous dissipation and external-boundary work. Force-momentum balance cannot by itself prove wall trace accuracy.

Threshold definitions and MMS/refinement uncertainties are owned by [validation design](validation-design.md). Tiny-system residual tolerances must be scale/operation-based, not a unit-sized absolute allowance. A passing dense solve is discrete verification; body drag still needs wall/mesh/domain convergence before continuum accuracy is accepted.

## Native export contract and independent owner

Assigned independent validation owner `01a139a7-fc1b-7384-9200-e10dd033f016` (GPT-6.1 Sol High) owns the exact separate `validation/submarine/` and I1 validation report write set in the canonical plan; `01a139a7-da3b-71b3-b155-ddca2f780005` owns only its admitted experiment/example/test/implementation-report paths. `/root/validation_design` authored the prospective acceptance design and is not the active executor. It does not share flux, row assembly, stress quadrature or expectations with the numerical implementation. The parent pins both revisions and agrees the export interface before parallel implementation/validation source work. A later independent executor must be assigned if this worker is unavailable; self-review never closes a deciding gate.

Fixture exporter writes to fresh ignored output. Each accepted **solve** supplies actual velocity/pressure and geometry/wall trace data, dimensions, x-fast ordering and face offsets, rho/mu, outer vector, pressure convention, load reference/frame, accepted solve ID, matrix/physical residuals and cap. Export actual wall patch positions, body normals and areas, raw reconstructed pressure/gradient, and discrete reactions so a verifier can integrate traction and compare it independently. Do not export only the already summed force or derive expectations from current fields.

A parent-controlled run manifest binds repository/source tree, native executable bytes, toolchain/flags, exact initial fixture/control bytes, invocation and output hashes. A same-case correct copied field may pass analytics; provenance must be a separate check. Tiny exact/rational oracle assembles from declared geometry/equations independently, not exported matrix coefficients alone. Exported coefficients can diagnose agreement but do not replace independent assembly. A stored-record hash proves integrity, not authentic origin by itself.

## Independently executable future procedure

The following paths/CLI are proposed source work, **not commands available in this planning PR**. Implementer and validator must expose this contract or prospectively review a replacement. `run.py` builds/executes the pinned native fixture, captures origin separately, then runs the independent oracle; missing native capability is an explicit unavailable failure, never a skip/pass.

```sh
cargo test --locked --no-default-features --test submarine_stokes_contract
cargo run --locked --release --no-default-features --example submarine_stokes -- --case tiny-straight --output .generated/submarine-i1/straight
cargo run --locked --release --no-default-features --example submarine_stokes -- --case tiny-oblique --output .generated/submarine-i1/oblique
python3 validation/submarine/run.py --suite stokes --source <implementation-revision> --output .generated/submarine-i1/validation
python3 -m unittest discover -s validation/submarine -p 'test_*.py'
python3 -O -m unittest discover -s validation/submarine -p 'test_*.py'
cargo fmt --all --check
cargo clippy --locked --no-default-features --example submarine_stokes --test submarine_stokes_contract -- -D warnings
```

The independent suite must discriminate wrong body normals/area, pressure offset/depth, affine trace bias, omitted shear, torque arm/axis swaps, incompatible outer flux, nonfinite/corrupt fields, altered accepted generation/control/geometry and mismatched origin. Each negative changes one otherwise valid condition and identifies the intended numerical/profile failure; a generic nonzero exit is insufficient.

## Completion boundary

I1 implementation can reach Implemented when the tiny external Stokes lab runs with its complete transaction and actual load output. Acceptance of N1/N2 requires the independent controls above. N3 body drag convergence remains pending until a valid resource-bounded mesh/domain family executes. I2 cannot inherit accurate hull loads from an unqualified I1 trace. No experiment, turbulent closure, full submerged translation, Tuldok training eligibility or real-time guarantee follows from this slice.
