# Periodic z-invariant three-component successor

The physical domain and derived third equations are stated in
`docs/research-book/implementation/extruded-third-velocity.md`. Both horizontal
and extrusion directions are periodic with period one, every field is z
independent, the bottom is impermeable/free-slip and cap traction is natural in
the existing restricted weak space. This is not general 3D liquid motion.

`composition/` freezes actual tests of the normal composition of the independent
oracle and equation-audit histories. `third_exact.py` derives exact conservative
third momentum and full strain/work on both existing xy fixtures. `reference.py`
reassembles the third scalar block independently and integrates the combined
semidiscrete DAE with declared bounded reference calls. Native replay/refinement
must use actual published full velocity, geometry/masses, pressure, time and stamp;
missing fields, false identities and fabricated convergence remain rejections.

Old packets and the original `.00078125` host refusal remain byte-preserved.
No new Lean, continuum traction/spatial, sign-isolation, mesh-family, positivity,
variable-material, adhesion or surface-reconstruction claim is made.
