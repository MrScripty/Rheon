# Independent CPU CFD qualification boundary

This boundary measures elementary numerical correctness through Rheon's public
CPU APIs. It owns benchmark measurements, fixed-suite acceptance and exact-source
run receipts. It does not implement a CFD solver or establish experimental
agreement, aircraft/submarine capability, turbulent closure or training-label
quality. The [staged program](../../docs/plans/cpu-cfd-validation/reports/staged-validation.md)
owns those subsequent gates.

## Run and reproduce

Use repository Rust 1.92.0 and Python 3 (executed locally with 3.12.14):

```sh
cargo fmt --manifest-path validation/cpu-cfd/Cargo.toml --check
cargo clippy --manifest-path validation/cpu-cfd/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path validation/cpu-cfd/Cargo.toml --locked
python3 -m unittest discover -s validation/cpu-cfd -v
python3 validation/cpu-cfd/run.py --output /tmp/rheon-cpu-cfd-new-run
```

Output must be a new directory outside the checkout. It contains raw
`measurements.csv`, `cargo.log`, and either `receipt.json` for accepted elementary
numerics or `failed.json`. Process/metadata failures, timeouts, invalid metrics
and source changes cannot produce accepted completion. Failed runs are retained;
retry with a new directory. Default timeout is 600 s including compilation.
Interrupt/kill before receipt completion leaves a partial run, never acceptance.

Schema v1 binds the Git commit, dirty paths, SHA-256 of every root source file,
root/harness Cargo manifests and locks, toolchain file and validation source,
reference README, exact command, Rust/Cargo/Python/platform versions and raw CSV.
Dirty execution is explicitly represented, so the commit alone does not identify
uncommitted sources. Rerun from a clean commit for model acceptance. Hashes identify
bytes, not correctness or authenticity. Elapsed time includes build overhead and
is not a throughput benchmark. Results are CPU-only, single-process sequential
kernel execution; no GPU/remote-worker evidence is produced.

The standalone crate has only the existing Rheon path dependency, default
features disabled. Its committed lock is seeded from the admitted root lock,
then resolved for this consumer, preserving existing dependency versions. No
third-party code or experimental dataset is copied here. Existing Rheon native
import dependencies remain in the resolved graph; disabling PNG does not mean
the current core has no transitive dependencies. No GPL dependency is introduced.

## Independent references and fixed inputs

`src/main.rs` owns all fixed numerical inputs; there are no runtime numerical
overrides. Positions/spacing are metres, time seconds, density kg/m³, viscosity
Pa s, velocity m/s, pressure Pa, energy J. The initial velocity scale is 1 m/s
and pressure amplitude is 1 Pa. Fields are stored f32; pressure is solved f64.
All boxes start at origin zero. Density is 2; dynamic viscosity is 0.1, hence
kinematic viscosity 0.05 m²/s. All reported norms are absolute maxima.

**Pressure mesh:** unit cube at 6³, 12³, 24³, closed Neumann walls and gauge at
cell zero, with both Jacobi and SGS PCG. Derive
`p=cos(pi*x) cos(pi*y) cos(pi*z)` and `-laplacian(p)=3*pi²*p` by differentiation.
Feed continuum midpoint RHS `h³*3*pi²*p/rho`; compare to `p-p(cell_zero)`.
This tests continuum truncation, unlike forming RHS with the production matrix.
An independently indexed six-neighbour flux stencil measures full residual.
Settings: relative 1e-11, absolute 1e-13, predicted divergence 1e-9, 5000 maximum
iterations, dt 0.02 s. The CSV's pressure `dt_s=0` identifies the stationary
mesh study; a separate gradient-projection observation uses 0.02 s.

For that observation, sample `u*=0.02*grad(p)/rho` on component faces, with exact
zero normal walls, call `solve_velocity` and `correct_candidate_in_place`, then
independently differentiate the resulting f32 velocities. CSV pressure residual
and iterations belong to the continuum solve; divergence/flux/wall/energy belong
to the separate projection. Predicted divergence is not substituted for actual
stored-field divergence. A unit-cube constant-density volume balance is net
boundary flux zero; this is not a conservative compressible mass/energy test.

**Viscosity mesh/time:** box 1 by 1 by 0.75 m, n by n by 1 cells, stationary
impermeable free-slip walls, `u=(sin(pi*x)cos(pi*y),-cos(pi*x)sin(pi*y),0)`.
It is divergence-free and has zero tangential traction. With advection omitted,
the continuum viscous solution is `u(t)=u(0)*exp(-nu*2*pi²*t)`; this is a Stokes
decay test, not a complete nonlinear Taylor–Green solver test. Final time 0.1 s.
Mesh study uses n=8,16,32 and n² steps. Time study holds n=12 with 16,32,64 steps.
`continuum_linf` compares against the continuum exponential; `time_linf` compares
against the independently derived MAC modal eigenvalue
`lambda_h=8*sin(pi/(2*n))²/h²`. The latter isolates time error relative to the
semidiscrete problem and is explicitly not continuum accuracy. Both compare
against the stored initial f32 samples, excluding initialization rounding error.
Diagnostics include every step's worst divergence, wall/flux error and energy
growth, plus final energy. One-cell Z extent is an extruded two-dimensional case.

Acceptance requires all 12 unique cases and finite, correctly signed metrics;
mesh orders 1.8–2.2; temporal orders 0.75–1.25; nonzero observable errors;
viscosity mesh temporal contamination at most 10% of measured continuum error;
pressure residual at most 1e-10; projected divergence at most 1e-7 s⁻¹ and viscous
divergence at most 1e-4 s⁻¹; wall normal velocity exactly zero; net boundary flux
at most 1e-12 m³/s; positive viscous energy and at most 1e-10 J per-step growth.
These budgets qualify this fixed modest-grid fixture; they are not universal
solver/experimental tolerances. Three points expose two observed slopes; no GCI
or asymptotic certification is inferred. Finer runs need roundoff-floor review.

## Pressure-load oracle

`oracle.pressure_wrench` integrates pressure traction on supplied surface samples
using outward body normals: `F=-sum(p*n*A)` and
`M=sum((x-origin) cross (-p*n*A))`. Unit normals/areas/finiteness are checked.
Tests use original rectangular geometry and independent divergence-theorem
values `F=-V*grad(p)`, `M=(center-origin) cross F`, pressure-gauge cancellation,
moment-origin change and hydrostatic buoyancy. This reference-only evaluator is
reusable for future solver surface exports. It does not test Rheon's wall
pressure reconstruction, viscous traction or body-load accuracy; receipt body
forces/moments remain `not_assessed`. Synthetic policy rows and fake failure
processes in tests exercise receipt decisions, never solver qualification.
