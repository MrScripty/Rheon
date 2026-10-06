# Native bounded coupled endpoint-donor step

This implements the selected plan in `fitted-discrete-work-refinement.md` after
freezing research source `ef9c3dc3875f7a1008ba2edb35589c43a72bb97e` and evidence
`598e9a0647e33c908ec95a841b58c67a372635ed`. All prior failures and proof limitations
remain frozen. The native API is `CoupledDiscreteFlow`, a narrowly supported
periodic two-column fitted height graph, fixed bottom `[0,.5,1]`, width one,
constant density three, viscosity `.05`, height sum `2.25`, invariant zero third
velocity. It advances all 22 xy velocity coefficients and 16 pressure modes.
It does not enable general legacy liquid stepping or make a validated continuum
free-surface/traction, variable-density, adhesion, capillarity, positivity or
surface-reconstruction claim.

## One accepted owner and simultaneous publication

`TranslatedViscousFlow` supplies the sole accepted fitted frame, nodal velocity,
clock, stamp and candidate velocity. A crate-private full-xy initialization path
preserves the public translating-flow constructor's original restrictions.
Pressure coefficients belong to the coupled wrapper and are replaced at the same
infallible final publication barrier as that frame and velocity. No independent
phase geometry, carrier authority, advancing pressure owner or accepted-state
history is introduced. Liquid volume is fitted nodal mass divided by the fixed
density; the physical face integrals must reproduce each accepted mass change.

The constructor checks actual accepted embedding/strong divergence and initializes
pressure from the instantaneous assembled DAE. The step derives q and eta from
the accepted frame and one-hot velocity coefficients. One candidate frame is
reassembled in place. The fifteen-row chart assigns the seven known coefficients
exactly. All 24 strong velocity and differentiated rows plus material cap are
checked throughout actual quadrature and endpoints. The native node layout probe
records raw-node/periodic-node indexing explicitly; their initial confusion and
rejected run are preserved.

The nonlinear solve starts with the actual 38-row instantaneous momentum/moving
constraint system. Its six acceleration columns use bounded finite differences;
its sixteen pressure columns use exact assembled endpoint B transpose. The first
zero-seed attempt exhausted the original sign-search budget and is preserved.
No sign threshold, flux clipping, tolerance increase or fitted mass correction
repairs it. Iteration count stays at most seven, true rate stopping target
`1e-13`, equation calls at most 200. Original direct momentum is checked as well
as stable changing-mass inertia using the actual accepted velocity.

## Physical mass and discrete momentum/work

Numerically discovered sign partitions keep the original 17 samples, at most
64 roots, 20 bisections per root and 4096 face evaluations. Actual native relative
fluid/PS-mesh face flux is integrated with fixed 16/32 Gauss rules; refinement
must be below `1e-15`. The native fitted frame computes real material PS motion,
full symmetric-gradient strain forces and pressure adjoint forces.

The finite momentum equation remains the **first-order endpoint donor** equation
from the research chapter. The actual varying-donor path momentum is measured
separately and differs. Exact continuous momentum, pointwise free-surface
traction and continuum convergence are not thereby validated. Separate BE,
face-mixing, viscous, pressure, local-GCL and residual work terms use the actual
accepted endpoints. Momentum/divergence/material gates remain `1e-11`, local
GCL uses `128 eps (m0+m1)`, work/ledger/pressure use `128 eps` of the original
energy scale. Endpoint publication cannot occur before all gates pass.

## Bounded storage and failure contracts

The budget includes both frames' actual vector capacities, the reused accepted
owner's vector capacities, the coupled wrapper's fixed arrays, and a conservative
sum of every simultaneously live bounded stack object (caller/callee equation,
point, quadrature, root and dense linear scratch). This is a payload/storage
budget, not a process RSS, allocator metadata or compiler ABI stack-size theorem.
Existing fitted reconstruction clears and repopulates preallocated topology
vectors; the fixed topology never exceeds their constructor capacities. Native
step source contains no heap allocation, growth request or accepted snapshots.
Explicit endpoint histories in the example and Python replay are evidence exports.

Every arithmetic operation follows the existing zero-or-normal, checked-product
and checked-division guards. Nonzero subnormal results, underflow-to-zero products
and bad divisions refuse the candidate. Six cancellation barriers are tested
after a nonzero accepted state, including a seventh actual quadrature callback;
invalid/subnormal/underflow intervals and bounded Newton failure preserve accepted
geometry, masses, full velocity, pressure, clock and stamp. Continuation matches
an uninterrupted native owner bit for bit. Memory below the declared budget and
unsupported third velocity/divergence refuse initialization.

## Numerical/proof scope

Qualify the actual Rust formatting, full feature test matrix, three strict Clippy
modes, focused release contracts, two complete native refinement exports,
independent host accepted-state replay and corruption controls, and accepted
native endpoint renders. The same spatial DAE reference resolves the errors;
this is temporal qualification of this bounded discretization only. Research
whole-real-path certificates cover two first coarse host polynomial paths, not
all native paths, IEEE evaluation or an arbitrary mesh family. Numerical sign
discovery/refinement is not certified sign isolation. No Lean theorem is added.
The original Jacobi fixtures, scale guards and all historical evidence stay
unchanged. Parent coordinates reviews, PRs and merges.

The inherited settings expose memory and iteration budgets for this slice.
The constructor rejects changes to the inherited numerical tolerance fields;
all four default values and the fixed physical gates above are preserved.
The reported `allocated_bytes` includes a conservative scratch reservation of
450,320 total bytes on the qualified target. It does not assert exact optimized
machine stack consumption. The accepted native/host endpoint coefficients differ
by less than `3.78e-15` on the preliminary complete two-field export; a final
source-bound export and replay supersede that preliminary observation.
