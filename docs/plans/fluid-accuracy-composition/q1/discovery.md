# Q1 discovery checkpoint (before any new computation)

Existing actual channel owner StaticObstacleShear uses the retained
StaticObstacleGeometry volumes/open areas and true centroid/wall distances.
The original audit channel uses gap1mm, rho998.2, mu.001002, a.01, backward Euler
20steps of1s at8/16/32 levels. Preserve its coarse2.139313%>2% failure and the
original semidiscrete sine-decay8/16/32step evidence at n32,T.05s.

LiquidVolumeState is the existing f64 shared-face donor-cell fraction owner;
LiquidFlowInterval supplies f32 face velocities. It currently supports uniform
outside inlet fractions per side, not periodic wrapped donors. A periodic sphere
cannot be honestly substituted by outflow/refill. New bounded periodic support
must mirror one seam transfer exactly and demand paired boundary-normal velocities.

Smooth analytic divergence-free face sampling in f32 can lose discrete zero
divergence and make raw full fractions exceed1. Do not clamp them, loosen their
bounds, refill, or hide a projection. Proposed reversible vortex uses a dyadic
integer vertex streamfunction curl at n16/32/64, giving exactly discrete-solenoidal
f32 faces. Define actual summed-outward multidimensional Courant from these faces.
Quantization must be stated separately; autonomous forward/reverse target remains
the original set. A 2D vortex extruded through a3D sphere is a narrower explicit
case, not an arbitrary3D vortex or velocity qualification.

Proposed independent sphere cell average: integrate exact disk/rectangle YZ area
along X; freeze analytic primitive, integration knots/tolerance and separate
oracle/uncertainty tests. Source initialization and final sphere reference use
volume fractions, not point-center binary masks. Total analytic sphere volume,
periodic image semantics and independent quadrature checks must precede any
positive shape claim. Unresolved oracle uncertainty cannot pass a threshold.

No scientific Q1 output is present at this checkpoint. Independent reviewer is
assessing topology/CFL/ledger, f32 solenoidal construction, geometry oracle and
smallest Q2 reference scope. Exact protocols/code will be frozen before evaluation.
