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
