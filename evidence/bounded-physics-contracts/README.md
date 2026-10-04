# Bounded collision and viscosity contracts

Additive branch: research/bounded-collision-viscosity-contracts, based on reviewed
book publication 2bc2a84fd4c562062ad592426794e8382f098a77. This changes no
production solver, manuscript, root import, shared audit or source inventory.
The prerequisite Rheon/Physics.lean is preserved byte-for-byte at SHA256
26cbb4f75076c7e70a90da126f96622b75044befce26a4c17a50066c76e96050.

The single new module is proofs/Rheon/BoundedPhysics.lean, namespace
Rheon.BoundedPhysics. Its helper statements establish the two requested
contracts rather than assuming their key conclusions.

## Oriented planar collision

wallValue(n,c,x) = sum(n_i*x_i)-c defines the permitted half-space by >=0.
The normal points toward that half-space and need not have unit length. The
start must have strictly positive wall value and the proposed end strictly
negative wall value. These assumptions also exclude a zero normal/denominator.
The defined hit is wallValue(a)/(wallValue(a)-wallValue(b)).

- wall_segment_affine derives the affine wall value along the actual 3D segment.
- wall_hit_range proves the hit parameter lies strictly between zero and one.
- wall_hit_on_surface proves the computed hit has wall value zero.
- wall_first_hit proves every segment parameter before that hit has strictly
  positive wall value: this is a first contact, not merely an endpoint check.
- clipped_segment_in_halfspace proves every point on the segment from the
  original start to that hit remains in the permitted half-space, including
  contact. The quantified clipping parameter lies in [0,1].

This is an infinite planar-wall contract. It does not establish triangle
containment, first intersection among multiple faces, watertightness, mesh
query correctness or floating-point collision robustness. Using it for a
finite mesh facet still requires a separately certified facet hit and query
ordering. Starting on the wall, grazing and same-side endpoints are outside
the strict crossing assumptions; no policy for them is claimed here.

## Finite implicit viscosity

strain(E,v)=E*v, viscousOperator(E,mu,v)=E^T*diag(mu)*E*v, and
dissipation(E,mu,v)=sum(mu_e*(E*v)_e^2). E is a fixed finite real matrix;
mu contains nonnegative material/quadrature weights fixed during the step.

- viscous_work derives v^T*K*v = dissipation from those definitions.
- dissipation_nonnegative derives nonnegativity from nonnegative mu.
- backward_euler_work derives the work identity from the actual coordinate
  equations mass_i*v_i + dt*(K*v)_i = mass_i*u_i.
- backward_euler_energy_nonincrease connects that derived identity to the
  existing Physics.implicit_energy_nonincrease theorem. With mass_i>=0 and
  dt>=0, sum(mass_i*v_i^2)<=sum(mass_i*u_i^2).

u is the old state, v the exact updated state. Zero masses/weights are allowed;
the theorem is an energy-quadratic bound, not a uniqueness or solvability proof.
The step is unforced and homogeneous: nonzero prescribed boundary motion or
external forcing would need their work terms. No symmetric-gradient stencil,
boundary traction assembly, nonlinear material iteration, approximate linear
solve or IEEE implementation is certified by this algebraic result.

## Checks and integration handoff

From proofs/, with the pinned Lean 4.19.0 and dependency manifest:

```sh
lake build +Rheon.BoundedPhysics
lake env lean ../evidence/bounded-physics-contracts/AuditBoundedPhysics.lean
python3 ../evidence/bounded-physics-contracts/witness.py
python3 -O ../evidence/bounded-physics-contracts/witness.py
```

The focused auditor requires all 15 public definitions/theorems and also scans
all theorem/axiom declarations in the new namespace, including generated
equation/proof helpers. This is the same logical-declaration selection as the
published root auditor; compiler-specialized executable bodies are excluded.
Only propext, Classical.choice and Quot.sound are allowed transitive axioms.
The check receipt binds actual compilation/audit logs to exact source hashes
and commands.

The final module build passed without warnings, and the focused audit passed
for 15 declarations (six definitions and nine theorems). The unchanged Physics
prerequisite and its pinned dependencies were compiled locally from source;
the previously unavailable Mathlib binary cache was not retried. The complete
910-target prerequisite build log is retained as prerequisite-build.log.
The first module build had one unnecessary tactic-sequence warning, corrected
before the final build. The first audit scanned compiler-specialized executable
bodies too; its rejection is retained alongside the final logical-declaration
audit, which follows the published root auditor and keeps the same axiom
allowlist. Neither failed attempt is presented as passing evidence.

witness.py uses exact rational arithmetic. It checks a crossing at t=1/2 and
shows the unclipped continuation enters the forbidden half-space. Removing
endpoint sign assumptions yields out-of-range hit parameters. A two-velocity
backward-Euler example decreases the quadratic from 2 to 1/2; negative viscosity
instead increases it to 32/9. An arbitrary candidate with positive viscosity
but no update equation can also increase energy. These are small witnesses,
not production or physical-validation experiments.
The normal and Python -O runs produced identical output; the checks use explicit
exceptions, so optimization does not disable them.

After independent review, the book author should add the root import, register
the 15 public names listed in AuditBoundedPhysics.lean, and review the new module
hash in source-inventory.json. Those shared files are intentionally untouched;
the existing source-inventory gate will reject the unregistered new file until
that coordinated integration. The standalone module check does not qualify a
future root/inventory edit or a whole-book publication. No merge, PR update,
external review request, production feature or dependency/pin change is made.
