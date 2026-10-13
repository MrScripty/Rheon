# Bounded obstacle-flow integration milestone

## Goal and provenance

Integrate and freshly qualify one finite viscous flow experiment around an
obstacle on main `2209012bc5f902693d0f695ec1c2d854126af317`, without changing the
production liquid/gas, renderer or API owners. This is a discrete physics
experiment, not calibrated engineering accuracy or a general 3D solver.

The existing aligned Stokes implementation, interval arithmetic, contracts,
proof sources and rational oracle were selectively retained from
`03dd79561cbba6b5b81737b3f4b4cf5af1e64740`
(`implementation/static-mesh-sphere-contact-20261008`). The integration adds an
encircling-flow specimen and independent cell-cycle control, qualifies against
current main, and binds CI to the imported strain oracle. No historical test
receipt is accepted as evidence for this integration.

No AGENTS.md, repository .agents/skills, or separate coding-standard file was
present in the inspected workspace or main tree. Follow
[repository output policy](REPOSITORY_OUTPUTS.md), pinned Rust 1.92.0 and the
existing strict formatting, Clippy, contract and source-bound evidence checks.
Generated records/logs remain outside Git. A native persistent-goal facility was
not available; this milestone and the execution task plan retain the goal.

## Candidate inventory and integration decision

Remote comparisons against main, inspected 2026-10-13:

| Candidate branch | Relation to main | Decision |
| --- | --- | --- |
| implementation/static-obstacle-operators | 0 ahead, 51 behind | Already integrated; reuse current geometry/pressure/strain. |
| implementation/rigid-mesh-force-impulse-20261008 | 98 ahead, 40 behind | Retain for later rigid wrench work; do not merge its entire stack. |
| implementation/spherical-rigid-kick-drift-20261008 | 97 ahead, 40 behind | Existing bounded sphere motion; does not supply fluid boundary work. |
| implementation/static-mesh-sphere-contact-20261008 | 96 ahead, 40 behind | Select only its aligned Stokes experiment and qualification dependencies. |
| implementation/bounded-sphere-impact-interval-20261008 | 95 ahead, 40 behind | Existing isolated impact work; not general resting/multiple contact. |
| research/bounded-force-wall-stress-foundation-20261009 | 94 ahead, 40 behind | Preserve research direction; do not promote wall-load accuracy claims. |

Branch names describe earlier stages; the compared histories overlap heavily.
The donor's `next-physics-roadmap-20261008.md` records a pressure-load
counterexample: adjacent-cell pressure on a unit embedded cube produces twice
the affine physical force at that grid spacing, even though gauge cancellation
and a discrete work identity pass. Therefore no pressure-to-rigid-body coupling
is inferred from this step. Its existing rigid, contact and wall-stress sources
remain candidates for separate qualification, rather than being reimplemented.
PR43's CAD research likewise distinguishes proposed analytic lessons from
calibrated field/load measurements. This milestone makes no CAD measurement or
stress accuracy claim and does not modify that PR's book or UI.

## Finite acceptance criteria

1. Preserve main's geometry and numerical owners. One retained internal,
   grid-aligned box has at least one full fluid-cell padding; no-slip stationary
   obstacle and sealed free-slip outer walls use the same active face space.
2. Execute actual nonzero circulation around all four sides of the obstacle,
   with positive constant density/viscosity, then three consecutive accepted
   steps of the same field. The rational cell-cycle fixture has initial energy
   1 J; at dt = 1/32 s, rho = 1 kg/m³ and mu = 1 Pa s, the first discrete step
   has energy 731/1024 J and zero cell flux. Symmetry makes pressure zero here;
   the separate asymmetric curl must execute a nonzero pressure correction.
3. Independently verify stored coefficients, all wet-cell divergence, actual
   component momentum defects, and both stage energy changes using exact
   rational arithmetic. Check the actual native fields, not substituted ideal
   trajectories. Reject corrupt records in normal and optimized Python.
4. Preserve caller velocity and previously accepted pressure/reports on every
   refusal and cancellation stage. Verify invalid inputs, oversized steps,
   unresolved arithmetic, payload caps and repeatability.
5. Pass pinned Rust contracts, formatting and strict Clippy; run the source-bound
   native/rational qualification on clean committed source. Keep conditional
   Lean kernel checks separate from claims about Rust/IEEE implementation.
6. Independently review the integration, push a dedicated feature branch and
   open a draft PR. Do not merge or deploy.

## Controls and data for the API owner

The opt-in `experiments/aligned_stokes.rs` workspace borrows `AlignedStrain` and
its immutable `StaticObstacleGeometry`. No public production API is changed.
Proposed eventual adapter inputs are geometry identity, metres-based grid and
box bounds, constant rho/mu, full binary64 MAC face velocities in X/Y/Z order,
requested positive dt, pressure iteration/tolerance settings, divergence limit,
relative momentum-defect limit, managed-payload cap and cancellation callback.

Successful outputs are accepted velocities, gauge-fixed pressure in Pa,
requested/accepted dt (no hidden reduction), stage energy-change intervals in J,
all-cell divergence bound in 1/s, component momentum-defect certificates and
geometry identity supplied by the owner. The example's accepted-step counter
is exporter metadata; the workspace has no simulation clock or generation ID.
A future API owner must attach its own accepted generation/time atomically.
Pressure-proposal residuals are diagnostics; independent certificates decide
acceptance. Rejected attempts must not appear as accepted frames. Keep the
experimental capability explicitly labeled and return typed unsupported-shape
or arithmetic refusal rather than silently changing a boundary model.

## Staged boundaries and coupled bodies

- **This milestone:** fixed aligned-box viscous decay plus pressure proposal.
  The 3³ fixture has 26 wet cells; it is a discrete acceptance case, not a
  spatial/temporal convergence study or inflow/outflow obstacle benchmark.
- **Next geometry/transport gate:** derive a common interaction geometry for
  embedded symmetric strain and pressure; qualify nonaligned shapes, refinement,
  transport conservation and manufactured/continuum references independently.
  Static triangle hits alone do not establish a fluid-domain discretization.
- **Prescribed motion gate:** reuse translating-surface queries, but first add
  swept-volume/geometric conservation, moving-wall normal flux, wall work and
  accepted moving geometry/state. Require rest under common translation,
  impermeability, and volume/work balances before enabling motion.
- **Two-way body gate:** qualify existing rigid impulse/motion and sphere contact
  candidates separately; resolve pressure trace bias and matched force/velocity
  maps. Require equal/opposite impulse, torque about the same origin, coupled
  work balance and transactional rollback. Multiple/contacting bodies and
  resting contact require separate assumptions and fixtures.

Unsupported: advection, forcing, inflow/outflow, liquid fractions/free surfaces,
variable/two-phase density or viscosity, capillarity, adhesion/contact angles,
moving or rotating obstacles, two-way solids, contact resolution, arbitrary
meshes, calibrated drag/lift or engineering stress. No GPU or real-time claim.
The interval backend admits Linux x86_64 with default scalar binary64 controls;
ARM, Windows, macOS, GPU and altered floating-point controls are unqualified.
Lean statements assume sound enclosures and do not prove compiler/IEEE behavior.

## Reproduction

From a clean committed checkout, with pinned Rust on PATH:

```sh
python3 tools/qualify_aligned_stokes.py /tmp/rheon-obstacle-new-evidence
cargo run --locked --no-default-features --example aligned_stokes -- unit-ring 3 /tmp/rheon-obstacle-new-ring.json
```

Destinations must be new and outside Git. The qualification receipt records
exact commands, source hashes, platform, binary hashes and native/rational case
results. `--lean` additionally needs the pinned Lean/Mathlib environment.
See the [step contract](research-book/implementation/experimental-aligned-stokes.md)
for equations and the [experiment entry point](../experiments/README.md).
