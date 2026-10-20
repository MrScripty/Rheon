# Q2 common geometry reference results

Version1 source652383e0b9f59c39bfd6a6367f324fa2c0d4fcf0: retained native exit1,
zero case rows, independent exact-rational tangent diagnosis. See version1-result.md.
Version2 source8b9c3b294c63527db33c8f5faccfb24f19b4217f: separately reviewed and
frozen before selected execution; native exit0,24case rows, native fixed gates pass.
No native retries or threshold changes within either frozen method version.

Four focused contracts pass debug/release. Selected strict Clippy passes. Native
Linux CPU f64 reference uses exact-roundtrip exported source polygons, existing
GridGeometry MAC indexing, shared face intervals and one-metre extrusion. Circle
R.75 source is an8n-sided inscribed polygon on n32/64/128 grids; rotated fluid
squares at0/17/31degrees use n16/32/64. Both viscosity scales .1 and .001002Pa s
are geometry/stress integration checks, not water MMS or a coupled solve.

| Native normalized maximum | Measured | Fixed ceiling |
|---|---:|---:|
| Closed normal measure | 1.422e-14 | 1e-10 |
| First-moment closure | 6.721e-13 | 1e-10 |
| Cubic flux/divergence moment defect | 2.379e-13 | 1e-10 |
| Quadratic solenoidal flux defect | 1.009e-12 | 1e-10 |
| Linear full-stress traction defect | 1.429e-12 | 1e-10 |
| Independent shared-face/cell-edge mismatch | 1.766e-14 | 1e-10 |
| Velocity/shear dual partition defect | 1.885e-15 | 1e-10 |

Smallest retained positive volume1.27963e-7m³; no volume floor or global mass
renormalization. Other unresolved topologies and finite source-edge/grid-plane
coincidence remain refused. No certified arbitrary underresolved geometry claim.

| Disk relative geometry error | n32 | n64 | n128 |
|---|---:|---:|---:|
| Volume / linear-stress integrated force | 1.00396e-4 | 2.50995e-5 | 6.27491e-6 |
| Radial second moment / cubic flux | 2.00779e-4 | 5.01983e-5 | 1.25498e-5 |

All pairwise geometry orders lie near2 (1.99990–1.99999). Finest errors meet the
fixed1e-4 prerequisite budget. Pressure p=1+mu*x−3mu*yPa is retained; p=0 negative
control gives the predeclared wrong-minus-correct force(mu,−3mu)*Vsource, showing
that dropping pressure is detectable in this integration fixture.

These flux/stress convergence curves largely measure polygon moment geometry
error, not independent PDE discretization error. No velocity/pressure unknowns
were solved; no B/M/S/W assembly, pressure-component gauge, small-cell solver,
free-surface MMS/annulus, exterior-obstacle hydrostatics or physical boundary
traction convergence is admitted. Independent post-run correspondence and final
qualification are recorded separately. Only a bounded geometry prerequisite can
advance; canonical full Q2 criteria remain unchanged and pending.
