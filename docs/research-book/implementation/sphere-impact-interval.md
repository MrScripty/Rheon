# Bounded remaining-time sphere impacts

This explicit successor of PR36 executes repeated isolated sphere/static-triangle
impacts within one caller-requested interval. It reuses PR36's finite-feature
detector, frictionless response and PR35's actual mesh/pose free coast. The
COM-centered analytic sphere remains the collision shape; retained moving
triangles remain render/traction geometry. Static facets form a two-sided union,
not an inferred solid interior. No loads, gravity, friction or fluid coupling
are introduced. Restitution is one declared constant in [0,1].

[Baraff's original rigid-body notes](https://www.cs.cmu.edu/~baraff/sigcourse/notesd2.pdf),
§§6,8, motivate stopping/re-evaluating at events and distinguish separating,
colliding and resting contacts. [Fauerby's original finite-feature geometry](https://www.peroxide.dk/papers/collision/collision.pdf),
§§3.3–3.4, remains the geometry reference. The departure certificate, bounded
driver, rounding refusals and prefix accounting below are newly derived here;
no paper is claimed to supply this implementation or a global contact solver.

## Departure without a pushout or time epsilon

The previous accepted impact supplies its static point p and actual current
center c. Define the exact-real radial a=c-p. To exclude that ONE facet during
unchanged straight free flight, require

    ||a||² >= R²,  a·(x_i-p) <= 0 for all three facet vertices,
    a·V >= 0,  R > 0.

Every point of the closed triangle is a convex combination of its vertices,
so it satisfies the support inequality. For every t>=0 and triangle point x,
a·(c+tV-x)>=||a||². Cauchy–Schwarz then gives
||c+tV-x||²>=||a||²>=R². The conclusion includes stationary or tangent
departure; it assumes no external acceleration or impulse during that flight.
At an edge or vertex this is the radial supporting plane, not a winding plane.

The implementation compares these predicates on the EXACT dyadic values of
represented input coordinates. Binary64 significands are multiplied as u128,
then accumulated in separate positive/negative 66-word fixed integer arrays.
At most 16 products are allowed; no heap or rounded subtraction is used. Expanded
differences use 13 products for clearance, 12 for each support test, and 12 (six nonzero terms) for
velocity. This finite integer kernel is independently checked against Fraction;
the Lean proof does not verify Rust bit decoding or IEEE execution.

The private certificate binds actual body/moving/static identities, complete
snapshot, radius and last accepted point. It is invalidated after every accepted
event. All OTHER facets keep PR36's strict initial-clearance admission. Negative
clearance, nonsupport or actual inward post-impact velocity returns an explicit
Stopped(InvalidDeparture) stop with the accepted prefix. There is no spatial pushout,
clamp, repeated zero-time impulse or tolerance-based time consumption. Initial
touching without a preceding private accepted witness also stops explicitly.
For e=0, exact stationary/tangent free coast can proceed without a resting-force
solver; an oblique response rounded inward refuses.

Before publishing a flight endpoint, also require on its ACTUAL proposed center
c_new that a·(c_new-c)>=0, using 12 exact products. This prevents rounded endpoint
regeneration from moving inward across the certified support plane. Failure
preserves the previous accepted owner. The PR36 floating detector remains a
conditioned nearest-rounded method; this departure proof does not turn its
roots/gap diagnostics into a global error enclosure or nonpenetration theorem.

## Bounded control and accepted-prefix semantics

At most 64 impacts are permitted, with caller-preallocated record slots for the
impact budget plus one terminal miss/coast. Every iteration scans at most the
declared static facet cap. It queries first, reserves the next record, and
prechecks a strictly positive requested event duration and strictly decreasing
finite remaining duration before an atomic PR36 publication. Each accepted
segment also advances the actual clock strictly. Completion requires exactly
zero remaining requested time; a small positive remainder is never discarded.

When the budget is reached, a subsequent definite miss may finish the interval.
A subsequent hit returns ImpactBudgetExhausted at the last accepted state,
leaving all remaining time explicitly unadvanced. Cancellation, zero-time or
absorbed progress, geometric ambiguity, unqualified departure and later motion
refusal retain earlier accepted events and return their record count plus the
positive remaining duration. Admission errors before execution mutate neither
owner nor record buffer. No all-interval rollback is claimed.

Per-event PR36 momentum/energy/clock reports are preserved. Aggregate impulse,
energy loss and actual-endpoint residuals are nearest-rounded, unenclosed
diagnostics; if their arithmetic is unrepresentable the accounting is explicitly
unavailable, never replaced by zeros or used as a stepping authority. Requested
segment durations, actual elapsed clock and their defects are distinct. A
completed nominal interval need not have exact IEEE clock or geometric agreement.
PR35's caller-selected maximum coast duration and 0.25 rad per-coast spin limit
remain in force; this driver supplies no adaptive accuracy or stability policy.

Simultaneous impacts still refuse under PR36's declared window. A sphere tangent
to one facet that later hits another can stop on the older touching facet: one
departure witness is deliberately not a manifold/resting-contact formulation.
Finite feature conditioning, thin/far facet refusal and all PR36 gap/time policy
limitations remain. Physical units, measured fixture accuracy and fixed algebra
test tolerance are reported separately. The renderer records actual owner
vertices after each accepted segment, without another pose integrator.


## Reviewable API and numerical boundaries

`SphereIntervalRequest` pins caller body/moving/static stamps, radius, restitution,
interval, PR36 settings and an impact budget in 0..=64. The caller supplies at
least budget+1 `Option<SphereIntervalSegment>` slots. The driver adds no heap
allocation; records, retained PR35 mesh capacities and observers' storage are
caller-visible separate payloads. The fixed exact-sign workspace has two arrays
of 66 u64 limbs (1056 bytes) plus fixed headers; it is stack storage, not RSS.

`advance_static_sphere_interval` returns admission errors without mutation.
During execution, `Complete`, `ImpactBudgetExhausted`, `TimeProgressStalled`,
`Cancelled` between segments, and `Stopped(SphereContactError)` distinguish every
termination. Within-segment cancellation preserves its original stage/index in
`Stopped(Cancelled { .. })`. Observer/cancellation callback panics are outside the
returned-result contract. An immutable observer sees the published owner after
record insertion. Only the accepted prefix of record slots is written.

The certificate is internal to one call; separately restarting at a contact
still refuses. It binds the borrowed static mesh as well as its stamp through
the call's exclusive owner/immutable mesh borrows, and pins the complete actual
pose snapshot. A zero-time contact never receives another impulse.

Lean's support/Cauchy/convex-triangle theorems establish the exact-real exclusion
with explicit radius, squared-clearance, vertex-support and outward premises.
Finite ledgers telescope only under correspondence of their increments to actual
physics or time. The completion statement requires every requested duration to
be consumed. No theorem identifies rounded diagnostic sums with exact ledgers.

## Reproducible local qualification

Run `tools/qualify_sphere_interval.py --output /absolute/fresh/external/path
--lean-bin /absolute/lean --lean-dependencies /absolute/pinned/packages` in a clean
committed checkout with Rust 1.92.0, Lean 4.19.0, Python with mpmath and matplotlib.
This opt-in local script never requests hosted CI. It freshly builds every Rheon
proof from current source, checks all nine pinned third-party revisions, audits
axioms (including injected axiom/sorry rejection), and runs native debug/release
contracts, new sequence labs plus the original PR36/PR35 actual-native oracles.
The 27 sequence fixtures compare Fraction corridor ledgers, independently found
finite edge/face roots and Rodrigues mesh poses, including 64 accepted impacts,
63-impact budget exhaustion, e=0 tangent coast, a two-impact edge/face conservative
stop and the floor/wall resting-manifold boundary. Twelve deliberate schema,
geometry, pose, accounting and completion forgeries must reject under normal and
optimized Python. The fixed 1e-12 algebra test tolerance is separate from measured
physical fixture errors and the API's gap/time policies. Native raw inputs,
JSON, independent reference ledgers, fresh proof logs, binary hashes and actual
stored sequence JPEG stay outside Git. All prior PR36 tracked files are protected
except the declared exports/private adapter, root proof imports/audit/inventory
and one narrow ignore; the adapter has an explicit frozen-source hash gate.

Independent review additionally compiles the actual exact-product Rust module
into an external harness and compares it to Fraction across normal/subnormal,
carry, cancellation and extreme bit patterns. Its final receipt is bound to the
final source HEAD; development evidence is not inherited as qualification.
