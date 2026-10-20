# Bounded method decision

Select candidateC for a private OPERATOR/TRACE reference and deciding experiment,
not for production or a native coupled step. Derive all operators from the
2017supplement's continuum variational functional using an explicit continuous
staggered-bilinear velocity basis, consistent full mass and primalP0pressure.
This changes discrete quadrature and support semantics and has its own version;
it is not a claim to reproduce the published weighted-difference implementation.

A publishedVFD implementation is a legitimate comparator, but its minimum weights,
support elimination, missing-velocity backsubstitution and boundary-commutator
trace correspondence are not resolved under Rheon's fixed no-floor/actual-flux
requirements. Do not import them as guessed defaults or call clipped hdual volume
an exact mass for a different reconstruction. RT0 is a useful exact local flux
bridge for VFD, but its shear/traction trace does not automatically equal the
staggered strain trace. CandidateB's aperture-only active set plus aligned strain
therefore has no admitted variational/physical boundary correspondence.

C makes active-basis support and physical trace explicit, eliminating extension
values guessed from aperture flags. FullM represents the declared field's kinetic
energy and permits physical linear/angular momentum tests. Exact common geometry
and a single field supply B, K and nodal traction. Its new pressure pair has no
assumed cut-uniform inf-sup theorem. Exact rank/positive mass, full nodal affine
stress/load and physical flux identities must pass per declared instance.
No pressure gauge is removed for an open-traction boundary. No arbitrary mass
floor, basis deletion, stabilization coefficient or threshold adaptation is allowed.

Smallest admitted implementation CV1: exact-cut support and positive quadrature,
fullM/B/K and independently integrated physical load/flux/moment maps on four
static represented polygons, with source-bound independent reconstruction. No
linear solver/timestep/provider/transport/solid wall implementation in this slice.
CV2 coupled-step/backends depend on the CV1 rank/conditioning results and a new
bounded prospective protocol. Generic stability/PDE/solid Dirichlet lifting
remain explicit method gates. This is a useful closed reference builder rather
than a production capability or a pretense that aligned tests qualify cuts.

No essential physical/user input is missing for these synthetic fixtures: rho,
mu, extrusion, source geometry, fields and prescribed liquid-outward traction
are explicitly fixed by protocol. Oblique no-slip Dirichlet, moving geometry,
capillarity/atmosphere choice for an actual application and measured experimental
observations are not silently inferred from this synthetic natural-traction test.

User implementation authority is explicit in this delegation and prior grid-first
repair direction. Independent researchreviewa2d7483 accepts the mathematical
candidate subject to source/protocol review and actual deciding gates. The
bounded direction and its full-mass/new-quadrature consequence were explained
before code, with no separate approval ceremony or root-plan/source-owner edit.

Prospective protocol review accepted mathematical CV1 scope subject to explicit Duffy map, fixed work-test vectors and omitted-minus-full load sign; all three clarifications were incorporated before implementation. M/B conditioning diagnostics only; no hidden CV2 timestep selected.
