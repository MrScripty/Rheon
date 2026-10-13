# Rich emitter controls

Status: Complete. Objective acceptance: satisfied for this bounded emitter milestone. Phase: delivered.
Delivered: [draft PR57](https://github.com/MrScripty/Rheon/pull/57), stacked on fixed PR55.
No merge or deployment. Remote CI is reported separately from local acceptance.
User authorizes implementation, tests, independent review, push and draft PR; no merge.
Base: PR55 qualified head `3525c979e16629b295a6865e3abbb2a36803cc93`; keep its branch fixed.
Branch: `feat/emitter-vector-turbulence`. No AGENTS.md or .agents skills found locally.
Standards: Coding-Standards `188beda1`; Core/Router, planning, implementation,
verification, development proportionality, commit/documentation, Rust/library/API,
contracts, architecture and desktop UI applicability. Existing routed standards
remain applicable. No Standards Engine tool is available; no engine approval claimed.

## Binding decisions and owners

API owner owns emitter types/sampling, emission-only particle-fluid integration,
carrier-emitter integration, emitter UI controls/direction overlay, matching tests
and docs. Collision task `01a13509-edaf-7008-af00-4b8c1e418031` owns collision/mesh;
CPU task `01a13511-af6d-7023-8c01-26adff87302c` owns scheduling/performance kernels.
Write overlap is limited to additive emitter portions of particle_fluid.rs,
rheon-desktop.rs and viewport.rs; no collision or worker/kernel changes. Parent
relays scope because cross-thread messaging is not exposed here.

Retain pressure-head mode and defaults. Optional relative emitted-fluid velocity
vector (m/s) defines positive end axis/peak speed; negative end is opposed. Zero
vector switches emission off. Rotation rotates this initial vector about world Y.
Separate motion velocity (m/s) translates the original center from simulated time0;
world injected velocity includes this motion. Moving geometry is not a solid collider.

Seeded smooth value-noise inlet perturbations have strength (maximum vector speed,
m/s), spatial scale (m) and evolution time scale (s). They are attached to emitter
coordinates, stateless in absolute simulated time, bounded and reproducible on the
same numerical platform. Strength0 adds exactly zero perturbation. Nominal source
volume is determined by the original pressure or vector speed, never random draws.
These authored inlet perturbations do not constitute validated turbulent physics.

Particle birth velocity uses the selected profile, motion and perturbation. Carrier
velocity mode uses bounded residence-time relaxation toward that velocity before
normal pressure projection; pressure-head mode retains its original forcing at zero
new controls. Reject nonfinite controls/invalid scales, extreme unresolved sampling,
and motion leaving the supported domain before accepted state commits. Respect
existing timestep, volume-credit, capacity, cancellation and transaction semantics.

Native controls expose vector XYZ plus direction/speed editing, separate motion XYZ,
seed/strength/scales, and an illustrative direction overlay. Accepted MaterialSession
controls and desktop frozen Request retain all fields. No dense3D wire-format change.

## Milestones and acceptance

| Slice | State | Gate |
| --- | --- | --- |
| E1 shared API and emission | Complete | defaults unchanged; units/bounds; coherent rotated geometry; both carrier and material paths |
| E2 native controls/recording | Complete | compact controls, vector/speed consistency, direction overlay, retained accepted controls |
| E3 validation/review | Complete | deterministic replay, zero perturbation, seeds/scales/time, rotation/motion, liquid/gas, dt and rollback; independent review |
| E4 delivery | Complete | strict checks, dedicated pushed branch/draft PR, exact hardware limitations and coordination handoff |

Scope excludes arbitrary imported geometry collision, SPH interaction algorithm
changes, CPU optimizations, validated turbulence spectra, cross-platform bitwise
replay and real-time guarantees. Replan for conflicting write ownership or incompatible
recorded-control semantics. Blockers: none currently. See [ledger](execution-ledger.md),
[issues](issues.md), [design probe](reports/design.md).
