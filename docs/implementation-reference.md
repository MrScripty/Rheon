# Rheon implementation reference

Detailed entrypoints originally collected from the repository README at audit snapshot `c85dfee85597dd75acff5622c07d96b0368d8b89`. Run commands from the repository root. The scope notes below follow the current API contracts; follow each linked contract for its details.

## Coherent carrier and liquid-volume transport

`LiquidTransportSimulation` publishes the projected carrier velocity/pressure
and conservative represented liquid fractions together. Failure or cancellation
preserves both accepted states; candidates reuse the existing owners and capped
workspaces. The optional PNG path exports accepted fraction-integral guidance.
The base `step` is transport-only on a fixed all-fluid box. A separate
`step_viscous` path supports fully filled Newtonian liquid in a sealed box with
stationary free-slip walls. Free-surface pressure, variable-density feedback,
adhesion and geometric surface reconstruction remain open. See the [API and equations](../docs/research-book/implementation/coupled-liquid-transport.md)
and [native tests, replay and 3D numerical evidence](../evidence/liquid-step/README.md).

## Implementation comparison

The Rust fixed-box core has two preserved pressure implementations selectable
from an optional native desktop UI and the headless CLI. The original Jacobi
PCG remains the default; symmetric Gauss–Seidel PCG is an explicit alternative.
See [comparison contracts, controls and verification](../docs/COMPARISON.md).

```sh
cargo run --locked --release --features desktop --bin rheon-desktop
cargo run --locked --release --bin rheon -- --list-implementations
```

The desktop displays accepted simulation steps live in a navigable 3D voxel
view. Scrub or play the timeline while calculation continues; each recorded step
shows its calculation time. Orange wire boxes sample the actual voxel positions
of CPU workers, with idle workers dimmed. Left or middle drag orbits,
Shift+drag pans, and Ctrl+drag zooms, without requiring Alt. Wheel and touchpad
scroll/pinch controls are also supported.

The horizontal cylinder sweeps two nonuniform pressure-head jets around
the vertical axis at one revolution per simulated second. Each end's pressure
and spin speed are adjustable. Material mode emits liquid or gas parcels; the
original carrier comparison prescribes momentum and appearance concentration.
The visible cylinder illustrates the source and has no solid collision boundary.
See the
[viewer, timeline and emitter contracts](../src/bin/desktop/README.md).
See [emitter vector, motion and seeded perturbation controls](../docs/emitter-controls.md)
for shared SI controls and a headless material-generation example.
Use `default-features = false` to omit the default PNG-export feature; mandatory
dependencies and explicitly selected features still apply.

The desktop's **CPU threads** control detects the available logical CPU threads.
**Automatic** selects the detected count and refreshes the limit before each
run; turn it off to choose a smaller count. Both Jacobi and SGS parallelize grid
work, including dependency-preserving SGS tile sweeps. Small grids stay
sequential to avoid scheduling overhead. For a threaded headless run:

```sh
cargo run --locked --release --features parallel --bin rheon -- --threads 8 --output rheon-threaded
```

See [CPU execution and verification](../docs/COMPARISON.md#cpu-execution) for solver
limits, repeatability and thread ownership.

Worker batches now tune from measured kernel time, and large-grid advection
splits work within planes. Exact zero-tracer regions skip interpolation; carrier
pressure still includes clear fluid. SGS compares tile sizes at runtime. See
[scheduling measurements and limits](../evidence/scheduling/README.md).
The fixed-box pressure kernels reuse boundary coefficients; the desktop's batch
panel also reports cumulative kernel timings to identify the remaining costs.

The desktop also supports material scenes in physical metres: liquid or gas
presets, density, viscosity, surface cohesion, wall drag, gravity, temperature,
and initial vacuum or filled domains. Its particle preview preserves free
liquid surfaces and gas expansion; the original carrier comparison is still
available. Settings sections collapse independently. See
[material controls, units and model limits](../docs/material-fluids.md).

## Explicit body forces

The library's `step_with_forces` adds acceleration or force density in all three
axes, with optional box support and force-stage work diagnostics. See the
[book implementation addendum](../docs/research-book/implementation/explicit-forces.md)
for units, ownership, acceptance fixtures and limits.

```sh
cargo run --locked --release --no-default-features --example forced_smoke
```

## Static triangle-surface queries

The library can query the first segment hit on an immutable indexed triangle
surface and return a clipped endpoint with geometric normals and versioned
identity. See the [book implementation addendum](../docs/research-book/implementation/static-surface-queries.md)
for numerical tolerance, ownership and the remaining solid-boundary work.

```sh
cargo run --locked --release --no-default-features --example surface_queries
```

## Prescribed surface translation

`TranslationInterval` queries a linearly moving point against a translating
surface over one immutable interval, reporting world contact and wall velocity.
See the [book implementation addendum](../docs/research-book/implementation/translating-surfaces.md)
for coordinate equations, ownership and the remaining fluid boundary work.

```sh
cargo run --locked --release --no-default-features --example translating_surface
```

## Static passive-tracer barriers

`step_with_tracer_barrier` checks trace legs and donor visibility in the passive
concentration stage, while velocity and pressure retain the fixed-box model.
See the [book implementation addendum](../docs/research-book/implementation/static-tracer-barriers.md)
for the reversion policy, normalized interpolation and ownership limits.

```sh
cargo run --locked --release --no-default-features --example tracer_barrier
```

## Prescribed normal box flux

`BoxFluxWorkspace` projects a fixed rectangular control volume with prescribed
normal inlet/outlet speeds, independent net-flux rejection and explicit boundary
pressure work. See the [book implementation addendum](../docs/research-book/implementation/prescribed-box-flux.md)
for affine equations, output ownership and physical limits.

```sh
cargo run --locked --release --no-default-features --example box_flux
```

## Prescribed box-flux simulation steps

`step_with_box_flux` publishes inlet/outlet fields and complete step work together,
using a separately capped workspace and an explicit appearance-tracer policy.
See the [book implementation addendum](../docs/research-book/implementation/box-flux-simulation.md)
for conservation assumptions and transactional ownership.

```sh
cargo run --locked --release --no-default-features --example box_flux_step
```

## Conservative represented liquid volume

`LiquidVolumeState` transports bounded cell fractions with shared face transfers,
explicit inlet fractions and volume sources, preserving an accepted volume ledger.
See the [book implementation addendum](../docs/research-book/implementation/conservative-liquid-volume.md)
for interface-error evidence and the remaining free-surface pressure coupling.

```sh
cargo run --locked --release --no-default-features --example liquid_volume
```

## Bounded static-obstacle field operators

`StaticObstaclePressure` projects sealed stationary face fields using the same
retained obstacle volumes, openings and components as collision geometry.
`StaticObstacleShear` uses that owner for a separately admitted fully developed
extruded channel, including no-slip/Navier friction, density/force response and
work/impulse ledgers. These different boundary models are not yet a composed 3D
fluid solver. See the [equations, controls and recorded lab](../docs/research-book/implementation/static-obstacle-flow.md).

