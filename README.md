# Rheon
Real-time lowish resolution 3D fluid physics for creating refrence images to drive AI image diffusion. You would use this to when you want positionaly acurite AI rendered fluid effects. When using AI rendering it is often not nessisary to provide high accuracy visual instruction to get good looking reasults, but it can be difficult to achive deliberate precision of object placment and distribution without providing a guiding image. Instead of using more traditional 3D simulation tools and plugins Rheon provides an speedy and simple to use simulation toolkit whos only concern is to be good enoough to pass guidance to AI while using minimal memory to maintain space for loaded weights. 

This repo will include a standalone demonstration GUI and a modular simulation framework that can be implemented into other apps. Idealy you would integrate Rheon into your own AI design tool as part of your seamless production pipeline.

# About the name
Derived from rheology (study of flowing/deforming matter, especially non-Newtonian liquids) + Greek rhein (to flow)

## Implementation comparison

The Rust fixed-box core has two preserved pressure implementations selectable
from an optional native desktop UI and the headless CLI. The original Jacobi
PCG remains the default; symmetric Gauss–Seidel PCG is an explicit alternative.
See [comparison contracts, controls and verification](docs/COMPARISON.md).

```sh
cargo run --locked --release --features desktop --bin rheon-desktop
cargo run --locked --release --bin rheon -- --list-implementations
```

The UI displays solver results and opacity projections. It is an initial
comparison laboratory, not a complete interactive 3D editor or real-time claim.
Use `default-features = false` for a standard-library-only library consumer.

## Explicit body forces

The library's `step_with_forces` adds acceleration or force density in all three
axes, with optional box support and force-stage work diagnostics. See the
[book implementation addendum](docs/research-book/implementation/explicit-forces.md)
for units, ownership, acceptance fixtures and limits.

```sh
cargo run --locked --release --no-default-features --example forced_smoke
```

## Static triangle-surface queries

The library can query the first segment hit on an immutable indexed triangle
surface and return a clipped endpoint with geometric normals and versioned
identity. See the [book implementation addendum](docs/research-book/implementation/static-surface-queries.md)
for numerical tolerance, ownership and the remaining solid-boundary work.

```sh
cargo run --locked --release --no-default-features --example surface_queries
```

## Prescribed surface translation

`TranslationInterval` queries a linearly moving point against a translating
surface over one immutable interval, reporting world contact and wall velocity.
See the [book implementation addendum](docs/research-book/implementation/translating-surfaces.md)
for coordinate equations, ownership and the remaining fluid boundary work.

```sh
cargo run --locked --release --no-default-features --example translating_surface
```

## Static passive-tracer barriers

`step_with_tracer_barrier` checks trace legs and donor visibility in the passive
concentration stage, while velocity and pressure retain the fixed-box model.
See the [book implementation addendum](docs/research-book/implementation/static-tracer-barriers.md)
for the reversion policy, normalized interpolation and ownership limits.

```sh
cargo run --locked --release --no-default-features --example tracer_barrier
```

## Prescribed normal box flux

`BoxFluxWorkspace` projects a fixed rectangular control volume with prescribed
normal inlet/outlet speeds, independent net-flux rejection and explicit boundary
pressure work. See the [book implementation addendum](docs/research-book/implementation/prescribed-box-flux.md)
for affine equations, output ownership and physical limits.

```sh
cargo run --locked --release --no-default-features --example box_flux
```

## Prescribed box-flux simulation steps

`step_with_box_flux` publishes inlet/outlet fields and complete step work together,
using a separately capped workspace and an explicit appearance-tracer policy.
See the [book implementation addendum](docs/research-book/implementation/box-flux-simulation.md)
for conservation assumptions and transactional ownership.

```sh
cargo run --locked --release --no-default-features --example box_flux_step
```

## Conservative represented liquid volume

`LiquidVolumeState` transports bounded cell fractions with shared face transfers,
explicit inlet fractions and volume sources, preserving an accepted volume ledger.
See the [book implementation addendum](docs/research-book/implementation/conservative-liquid-volume.md)
for interface-error evidence and the remaining free-surface pressure coupling.

```sh
cargo run --locked --release --no-default-features --example liquid_volume
```
