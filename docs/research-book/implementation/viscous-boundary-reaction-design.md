# Proposed immutable aligned viscous boundary reaction

This contract is a derivation plan, not an implemented or qualified operator.
It preserves the stationary bounded domain in
[reconstructed aligned strain](reconstructed-aligned-strain.md). It selects no
pressure boundary closure or new advancing solver. See the
[dated roadmap](next-physics-roadmap-20261008.md) for capability and decision scope.

## Finite work derivation

Let `u` be the existing active face speeds, `E` the existing retained sparse row
matrix and `W=diag(w_r)` the existing nonnegative row weights. Normal strain
weights are `2V`; engineering-shear weights are the actual fluid-sector volumes.
The exact-real algebra uses the stored coefficients and weights as real
constants; this interpretation does not prove binary64 execution refinement.

Introduce a six-component virtual solid twist `xi=(V,omega)` about declared
reference `c`, and a separate virtual outer-wall twist `eta`. Restoring actual
boundary traces must derive row lifts `C_s,C_o` so that

```
s = E u + C_s xi + C_o eta
Phi = (mu/2) s^T W s
D = mu s^T W s
f = -mu E^T W s
g_s = -mu C_s^T W s
g_o = -mu C_o^T W s
u^T f + xi^T g_s + eta^T g_o = -D <= 0.
```

The last equality follows by distributing finite sums and transposing the
three gather maps. Nonnegativity requires `mu>=0` and every `w_r>=0`. It is a
conditional exact-real work statement, not a stepping or convergence theorem.
At physical `xi=eta=0`, every existing row action, fluid force and dissipation
must be recovered with exactly the same stored `E`, `W`, active map and masses.
The first API evaluates only this stationary physical field; virtual twists
identify the wrench derivative. It does not accept moving physical walls.

Differentiation with respect to translational speed gives force in N, and with
respect to angular speed gives torque in N m. Reference covariance must follow
from actual `r(x)=V+omega cross (x-c)` samples: after changing reference by `a`,
`tau_new=tau-a cross F` and `V_new=V+omega cross a` preserve pairing.

Work alone does not prove total force/torque conservation. A sufficient extra
premise is full common-rigid reproduction, `E R_f+C_s+C_o=0`, where `R_f` samples
the same rigid field at every active face. Under this premise,
`R_f^T f+g_s+g_o=0`. Both lifts use the same reference `c` and twist basis.
Derive the lifts from boundary geometry before checking this
identity; defining `C_s=-E R_f` merely to force cancellation supplies no physical
trace derivation. Fixed sealed outer walls forbid a global rotating fluid as an
actual admissible stationary test. Local compatible rotation and virtual
outer reactions must be distinguished. If the full reproduction premise is
not established, expose its residual and limit claims to assembled work/wrench.

## Boundary derivation obligations

The current sparse rows discarded stationary traces; `E` alone cannot identify
their physical reaction. Reconstruct every eliminated trace at its represented
location and distinguish obstacle from outer wall. Preserve all rows, including
old rows with no active coefficients: they can have nonzero lift coefficients.

* Normal families restore missing normal face values with the existing signed
  inverse represented cell width. Record the actual face point and wall owner.
* Interior four-fluid-quadrant shears keep their current coefficients and zero
  boundary lift.
* Flat obstacle walls need both derivatives in engineering shear. For `y=0`,
  fluid `y>0`, rigid rotation `u=-Omega*y,v=Omega*x`, the existing stationary
  row gives `U/delta=-Omega`. Subtracting tangential wall speed alone still
  falsely dissipates rotation. Restored `partial_x v_wall=Omega` cancels it.
* On each reflected anisotropic three-fluid-quadrant corner, derive the rigid
  affine background plus the current positive-part reconstruction of residual
  samples. For example, add `r(x)` to component residuals
  `(U-r_a(x_U))*max(y,0)/ell_b` and
  `(V-r_b(x_V))*max(x,0)/ell_a`, in consistently reflected local coordinates.
  Verify both solid half-wall traces, actual component locations and all sectors.
  This is a proposed new derived lift, not a formula attributed to Batty.
* Outer free-slip shear remains analytically zero under the admitted sealed
  traces. Restore and account for outer normal traces separately where needed;
  a virtual moving-outer interpretation must enforce relative free-slip shear,
  not reinstate no-slip tangential traces.

Use represented center-to-wall distances without snapping. Flat conductance is
`sum(v_sector)/delta^2`; rewriting it as geometric area divided by distance
requires the separate exact-product-volume premise. Rounded inverse-distance
and coordinate products may violate exact affine reproduction: report actual
native defects and state exact-real premises instead of asserting IEEE equality.

## Proposed output and qualification gates

Return an immutable stamped solid wrench, outer-reaction diagnostics where
derived, dissipation and fluid/solid/outer work diagnostics. Reuse existing
bounded geometry/material identities and resource limits. Six lift columns and
bounded scratch require explicit ownership, memory and cancellation accounting.
No nearest-rounded diagnostic authorizes stepping. No state/time advancement,
pressure solve or impulse application is included. A total wrench does not
uniquely determine facet P1 traction; local load distribution is a separate
physical quadrature contract.

Before native implementation acceptance:

1. Finish a row-by-row geometric lift derivation and independently review it.
   Freeze stationary row/weight/map/mass recovery and every zero row.
2. Compile fresh Lean finite gather/transpose, nonnegative dissipation, matched
   work and reference-covariance statements with explicit reconstruction and
   closure premises. Audit axioms and reject custom axioms/sorry. Generic matrix
   algebra does not prove the geometry premises.
3. Independently rebuild exact Fraction expectations from geometry, including
   nonmidpoint flat walls, reflected anisotropic corners, cross components,
   affine symmetric strain, compatible local rotation and old empty rows.
   Include the wrong tangential-only rotation lift as a failing control.
4. Compare actual Rust actions and all six wrench components with that oracle.
   Check stamps, immutability, unsupported geometry, bounded allocation,
   cancellation, shifted references and stationary compatibility. Use new
   source/binary/evidence hashes; no inherited theorem or test count is a gate.
5. Perform an independent frozen-source implementation review and save durable
   source/evidence checkpoints. Publication still requires its separate hold
   to be lifted; parent owns CodeRabbit.

The current design checkpoint satisfies research/roadmap review only. It does
not satisfy these new implementation gates. Pressure remains held at the
explicit trace/closure/accuracy choice in the roadmap.
