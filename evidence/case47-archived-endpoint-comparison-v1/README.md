# Case47 archived endpoint comparison

This packet compares the already completed pure-E2 original case47 endpoint
with the preserved pressure_state/nonconstant/reversed paired DOP853 endpoints.
It reads existing data only and uses the original convergence metrics and strict
reference-consistency criterion. No trajectory, native test/build, reference
integration, finite-difference column or instantaneous momentum equation runs.

The original native accepted constructor must match bitwise, and the reference
canonical q/eta/third coefficients must match bitwise. Independent nodal
reconstruction need not have identical floating-point bits; its differences are
reported and checked against the original constructor consistency gates. The
initial q/eta check is stronger than the old 1e-14 comparison. Pressure is an
algebraic DAE output, not an independently specified reference evolution state.
Physical fixture/geometry helper sources, loads, model and reference provenance
are checked before endpoint errors. Any mismatch stops the comparison.

The endpoint is the original prescribed 128 steps of h=0.00078125, nominal
t=0.1. The actual repeated-addition native clock is reported separately. Neither
endpoint is retimed, interpolated, integrated again or relabelled bitwise equal.

The original metrics are full velocity lumped L2, third velocity lumped L2, and
max absolute cap-coordinate error after x=(q0,q2,q1,2.25-q2). All velocity
comparisons use the original fine-reference endpoint lumped masses, including
the coarse-reference diagnostic and the coarse/fine consistency gap. The strict
criterion remains gap < full fine-reference endpoint error / 1000. Additional
coordinate and third coarse/fine gaps are diagnostics with no invented gates.

Only static original geometry/chart algebra is used to reconstruct the original
initial fixture and fine endpoint mass weights. The exact pressure fixture is
read from its preserved rational archive; its defining saddle system is not
solved again. Integration and trajectory/equation entry points are disabled.
This is endpoint error evidence for one restricted family. It establishes no
temporal order, new refinement ratio/band, broader geometry/material claim,
repair of eight old geometry-band failures, memory certificate, production
adoption or Lean theorem. Both reviewed packets remain frozen and unmodified.
