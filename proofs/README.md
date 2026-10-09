# Rheon exact discrete contracts

This project owns exact finite-dimensional reference definitions and 106 checked public theorem statements in eleven modules. It does not own the production solver, geometry assembly, IEEE arithmetic, continuum physics or performance claims.

## Reproduce

Use the exact lean-toolchain and lake-manifest.json. From this directory run:

    python3 scripts/check_sources.py
    lake exe cache get  # optional cache; normal source build works without it
    lake build
    lake env lean AxiomAudit.lean
    python3 scripts/test_audit.py

The source inventory records reviewed bytes. Do not regenerate it in CI. Review changed statements and the audit inventory before updating digests. The kernel audit allows only propext, Classical.choice and Quot.sound. It rejects admitted and custom assumptions; negative fixtures verify both rejection paths. Native-evaluation axioms are outside the allowlist.

## Qualified evidence

Source head ecf97d3a943ebe30d20acc1cf01b94dc08ff96a2 passed hosted run https://github.com/MrScripty/Rheon/actions/runs/37141752646. The run compiled all modules and audited 31 declarations including generated equation/proof declarations. The retained log records the actual pull-request merge checkout.

## Assumptions and limits

Discrete.lean uses arbitrary finite real matrices. Balanced columns are required for conservation and constant nullspace; nonnegative weights for positive semidefiniteness; exact RHS and exact solution hypotheses for exact projection. Geometry validity, complete nullspace characterization, pressure existence/uniqueness and solver convergence are not proved.

Transport.lean proves convex interpolation bounds, one-dimensional upwind and explicit-diffusion positivity under stated step restrictions, and exact rational counterexamples. It does not prove general mass conservation for semi-Lagrangian transport.

Indexing.lean uses unbounded natural numbers. Machine integer overflow and allocation limits remain implementation obligations.

## Expanded physics contracts

Physics.lean adds thirteen finite statements for force work, positive coefficients, weighted energy, source/flux conservation, implicit dissipation, slip power and Young adhesion. `docs/research-book/expansion/proof-qualification.json` binds their current source inventory to local pinned Lean 4.19.0 qualification: all modules compile, 45 declarations pass the axiom audit, and negative/source gates pass normally and under optimized Python. The official cache returned HTTP403; a normal pinned source build completed. The historical evidence above remains unchanged.

## Bounded collision and viscosity integration

BoundedPhysics.lean adds nine theorems and six definitions. Its strict-crossing assumptions describe an infinite stationary planar wall; it derives the first-contact range, surface hit, permitted prefix and clipped segment. Its fixed finite strain/weight definitions derive dissipative work from exact coordinate backward-Euler equations, then nonincrease under nonnegative masses, time and weights. Zero masses/weights are permitted: existence and uniqueness are not proved. Arbitrary mesh queries, assembled symmetric-gradient/free-surface traction, forcing, approximate solves and IEEE code are outside these statements.

The integrated root import, fifteen-name audit registration and reviewed source inventory are qualified together in `docs/research-book/expansion/bounded-proof-qualification.json`: 42 public theorems and 60 logical/expected declarations, with the original allowlist. The earlier receipts remain tied to their original source bytes. The additive source/evidence commits and exact rational witnesses are retained in `evidence/bounded-physics-contracts/`.


## Static obstacle geometry algebra

`StaticObstacle.lean` adds seven public exact-real statements and two definitions
for overlap, volume complement, shared incidence, component-constant jumps and
residual/divergence scaling. The complete geometry-stage source locally compiled with
pinned Lean 4.19.0: 64 public theorems, 87 audited declarations and the unchanged
allowlist. Three actual negative audit probes reject custom axioms, sorry and a
missing expected declaration. `source-inventory.json` records current reviewed
bytes; `source-inventory-pr24.json` preserves the earlier accepted 57/77 source.
The PR24 CI receipt does not qualify this new module. Rust mesh admission,
binary64 arithmetic, component construction and obstacle solver refinement
remain outside these statements.


## Stationary pressure and reduced extruded shear

`ObstacleOperators.lean` adds 24 public theorems and 15 definitions. The entire
current source was checked locally on 2026-10-08 with pinned Lean 4.19.0 and
mathlib `c44e0c8ee63ca166450922a373c7409c5d26b00b`: all ten modules and the root
compile, and the unchanged axiom audit passes 135 declarations, including
118 explicitly expected declarations and generated proof/equation declarations.
All three real negative audit cases reject custom axioms, admitted proofs and a
missing expected declaration in both normal and optimized Python. The four
source/pin-gate tests also pass in both modes. No `sorry`, custom axioms or
native-evaluation axioms are accepted.

The current reviewed `source-inventory.json` SHA256 is
`d33f1ee1614084b5fce8d6d4a93ed7f539b07c9bea2e14c563618f3720064c61`.
This qualification used the pinned compiler directly with a private artifact
search path: existing pinned dependency artifacts were read-only, 205 missing
imports were compiled from their pinned sources, and all current Rheon modules
were rebuilt. The audit rejection cases used that same compiler/search path
instead of `lake env`. This is local kernel evidence, not a new hosted-CI or
`lake build` receipt. Historical inventories and qualification receipts remain
unchanged and tied to their original sources.

Pressure definitions match outward flux `Q=-B(A*u)`, weights `A/(rho*d)`, the
correction `u'=u-dt*(B^T*p)/(rho*d)`, and declared inertia `rho*A*d`.
`pressure_flux_residual` derives `Q'=-dt*(b-Lp)` from the defined RHS, with no
exact-solve assumption. Positive volume and time give the signed and absolute
divergence scaling. Component compatibility follows from equal endpoint labels
on each shared edge. Correction energy is derived from the coordinate update
and matched mass. Exact-solve nonincrease additionally needs positive inertia
and zero residual. Neither a clipped-dual-volume identity nor a complete
nullspace, gauge-elimination or PCG-convergence theorem is claimed.

Shear work and forced momentum/energy follow from the defined graph-plus-wall
operator and exact coordinate backward-Euler equations with mass `rho*V` and
body force `q*V`. Lower and upper wall diagonals add even for a singleton layer.
Nonnegative coefficients give nonnegative dissipation and unforced energy
nonincrease. Navier conductance, eliminated wall trace, equal traction and the
fluid-half-layer/wall-friction loss split are explicit, including zero friction.
The 3-by-3 symmetric-gradient evaluation is for the specified mode
`u=(u(y),0,0)` only. It does not identify arbitrary flows with this mode.

These are exact finite reference statements. Geometry admission, owner-volume
and face-area assembly, centroid distances, Rust/IEEE refinement, linear-solver
correctness, continuum convergence, moving walls, free-surface traction and
general embedded Newtonian tensor viscosity remain outside the proofs.


## Reconstructed aligned strain (fresh local qualification)

`AlignedStrain.lean` adds eight finite definitions and eighteen public
exact-real theorems. The new gather and its transpose prove work, symmetry and
nonnegative loss. Weighted finite Cauchy and sum exchange derive
`forceNorm <= B*strainLoss` directly from the per-face coefficient bounds,
including zero rows. `rowBound` is the finite maximum with a zero default;
its bound premises are proved. The defined unforced coordinate Euler step
then decreases energy for nonnegative time and viscosity with
`dt*mu*rowBound <= 2`. The pressure composition uses precisely
`faceMass rho area distance`, positive density/areas/distances and an exact
solve of the defined full pressure residual. There is no assumed abstract
operator-norm theorem, approximate-solve guarantee or IEEE refinement.

Fresh 2026-10-08 qualification rebuilt the pinned sources with Lean 4.19.0 and
mathlib `c44e0c8ee63ca166450922a373c7409c5d26b00b` through normal `lake build`.
All eleven Rheon modules and the root compile; the unchanged axiom allowlist
passes 161 actual declarations, with 144 explicitly expected declarations.
The three real audit rejection probes and four source/pin-gate tests pass in
both normal and optimized Python. These numbers were obtained from this
source and run, without reusing historical qualification counts.

The generated external `lean-qualification.json` binds current source and log
digests. Generated receipts and logs are retained outside Git in the durable
qualification bundle. The official mathlib cache endpoint
returned HTTP 403. No cache artifacts were obtained; dependency artifacts
were compiled normally from the pinned Git sources. This is a local compiler
and kernel receipt, not hosted CI. The only build warning is the pre-existing
unused `hz0` premise in `StaticObstacle.lean`.

The restricted assembly and newly chosen reflected anisotropic corner
reconstruction are specified in
[reconstructed-aligned-strain.md](../docs/research-book/implementation/reconstructed-aligned-strain.md).
Formal geometry admission, stencil assembly/refinement, machine arithmetic,
force work, moving boundaries and global solver convergence remain external.

For combined fresh native, rational, source-policy and Lean qualification,
place the pinned Rust/Lean tools on PATH, set `CARGO_TARGET_DIR` outside Git,
and run from the repository root:

    python3 tools/qualify_aligned_strain.py /tmp/rheon-aligned-fresh --lean

The destination must be new and outside every Git worktree. The receipt binds
the clean committed source, executable, actual native records and all logs.
