# Concrete route recommendation

**Stop promotion of P1 particle production after this single material-map gate.
Prioritize the plan's existing Q1→Q2 grid reference route for accurate water.**
This is a recommendation to the repository lead, not a canonical plan edit or
authorization to change Q/G1 owners in this slice. Existing particle preview and
both frozen negative campaigns stay explicit. A future particle proposal must
supply a credible bounded method and pass unchanged gates; no indefinite kernel
correction campaign follows this reference.

The material-map formulation has a coherent shared volume, first-moment, gravity,
wall and acoustic-displacement construction in1D. That distinguishes an actual
coupled representation from independent pressure/visibility/density corrections.
But it replaces free particle sites with ordered connected cell interfaces and
material reference mass; it cannot honestly be integrated as an SPH wall patch.
One n8 reference computation is justified only to close that causal question.

| Obligation | Material-FE production route | Existing grid Q1→Q2 route |
|---|---|---|
| State/geometry | New material connectivity, reference density/moments, nodal state and3D Jacobian validity | Existing MAC/grid/obstacle/pressure/transport owners; common cut geometry still needs qualification |
| Large deformation | New distortion control, topology/remap and positive-volume policies | Fixed-grid transport/interface flux accuracy; original coarse failure retained |
| Stress/boundaries | New3D compatible pressure/viscosity, physical no-slip/free-surface traction and geometry | Already selected unified Stokes B/M/S/W reference, still to implement/verify on shared geometry |
| Stability/model | Weakly compressible acoustic stiffness plus nonlinear positive-J/wet-state dt | Incompressible pressure gauges/components, Stokes solve and cut-cell conditioning; transport CFL |
| Integration/custody | New native solver family/state schema, recording/replay, graph semantics | Reuse existing family/state owners and D1/G1 seams; add qualified capability/version |
| Scope evidenced here | Static planar algebra only; no viscosity,3D,geometry or dynamics | Existing native baseline and plan-specific references; no new accuracy acceptance claimed |

Both routes require substantial numerical work. Grid is recommended because that
work advances the family already selected in PR120 without inventing a third
moving-mesh production owner. This is an ownership/dependency comparison, not a
measured effort estimate or a claim that grid cut cells are solved.

Sequence: Q1 first measures native channel/transport resolution and Courant gates
before selecting transport changes; Q2 then implements common embedded geometry
and unified pressure–viscosity Stokes, with geometric interface transport only as
justified by Q1 and its own selection. Start with the plan's2D/extruded MMS,
physical wall loads and shared B/M/S/W work checks. Preserve stationary planar
versus arbitrary-mesh limits. G1/C1 integration stays with its owner. Do not
silently reinterpret P2 variable-volume particle initialization as a material
mesh seeder, or claim this reference discharges P1/P2/Q2.

The selected grid family and limitations come from PR120 at8422f77 and
[Larionov–Batty–Bridson2017](https://elrnv.com/projects/variational-stokes-a-unified-pressure-viscosity-solver-for-accurate-viscous-liquids/),
including its [coupled-stress MMS supplement](https://cs.uwaterloo.ca/~c2batty/papers/Larionov2017/supplemental.pdf).
The reference is not independent proof of Rheon accuracy; native assembly,
geometry/refinement/traction/transport gates still have to be measured.
