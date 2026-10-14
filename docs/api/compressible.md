# CPU periodic ideal-gas Euler

`CompressibleSimulation` is the native S1 cell-centered conservative Euler owner.
It uses a serial shared-face local Lax–Friedrichs (LLF/Rusanov) flux, piecewise
constant reconstruction and unsplit forward Euler on fixed periodic Cartesian
cells. It is independent of Rheon's incompressible pressure projection.
Numerical acceptance is pending independent validator qualification; see the
[implementation report](../plans/cpu-compressible-solver/reports/implementation.md).

This scope is inviscid single-material ideal gas. It supplies no open or solid
boundary, viscosity, turbulence, moving geometry, recording/session adapter,
learned model, GPU path, aircraft or submarine qualification. The same serial
method is available under core-only, default and parallel Cargo configurations;
`parallel` does not parallelize this owner.

## State and units

`ConservativeState = [f64; 5]` stores **cell averages**, ordered
`[rho, mx, my, mz, E]`. Their SI units are kg/m³, kg/(m² s) for each momentum,
and J/m³ for total energy density. Energy includes kinetic and thermal energy.
Index is `x + nx*(y + ny*z)` using validated `GridGeometry`; origin and spacing
are in metres. `GridGeometry` also validates unused MAC counts; S1 reuses only
its Cartesian metadata and cell indexing. One/two-cell-wide periodic axes work.

`IdealGas::new(gamma, gas_constant)` requires finite `gamma>1`, finite `R>0`
(J/(kg K)), and finite positive `cv=R/(gamma-1)`.
`conservative(rho, velocity, pressure)` converts finite SI primitive values.
`primitive(U)` derives velocity, internal energy density, absolute EOS pressure,
temperature in K and sound speed in m/s:

```
velocity = momentum / rho
internal_energy_density = E - dot(momentum, velocity)/2
pressure = (gamma-1) * internal_energy_density
temperature = pressure / (rho*R)
sound_speed_squared = gamma*pressure/rho
```

Every stored/derived component and intermediate must be finite; density,
internal energy, pressure, temperature and squared sound speed must be positive.
Overflow, underflow yielding zero thermodynamics and lost internal energy are
errors. There are no floors, clipping, energy repair, pressure gauge or modified
sound speed. High-Mach subtraction and extreme-scale arithmetic remain limited
by binary64 conditioning; successful retries are not high-Mach accuracy evidence.
`physical_flux(U, axis, positive)` returns `(rho*un, momentum*un+p*n,
(E+p)*un)` with a Cartesian unit normal, reversing every component for a negative
normal. It checks flux arithmetic separately from EOS validity.

## Using the owner

```rust
use rheon::{CompressibleSimulation, GridGeometry, IdealGas};

let grid = GridGeometry::new([12, 8, 4], [0.1, 0.2, 0.3], [0.0; 3])?;
let gas = IdealGas::new(1.4, 287.0)?;
let uniform = gas.conservative(1.2, [10.0, 0.0, 0.0], 101325.0)?;
let mut simulation = CompressibleSimulation::new(grid, gas, 0.5, 4_000_000, uniform)?;
let report = simulation.step(0.001, |_| false)?;
let accepted = simulation.view();
assert_eq!(accepted.time, report.time);
assert!(report.accepted_dt > 0.0);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Construction accepts a caller's explicit uniform state at t=0/generation=0.
`from_fn(grid, gas, cfl, payload_cap, callback)` constructs validated nonuniform
conservative cell averages at the same initial clock/generation.
`initialize(&[U], time)` or `initialize_with(time, |coordinate, world_center| U)`
replaces the complete accepted state after validation and finite volume-integrated
diagnostics. Time must be finite and nonnegative. The callback must supply
conservative averages; the center is a convenience coordinate, not an averaging
operation. Initialization success increments generation, resets the initial
conservation budget and clears the last report. Failure leaves the previous
accepted arrays, clock, generation, diagnostics and report intact. Callbacks
must return normally; caller panics are outside the typed error contract.

`view()` borrows immutable accepted fields and the last accepted report. No
scratch or mutable array escapes. A view cannot coexist with a mutating call.
`allocated_bytes()` measures actual retained vector capacities: three five-f64
state buffers plus one f64 incident-rate buffer (normally 128 bytes/cell).
Checked capacity arithmetic, fallible reservations and actual capacity checking
obey the caller's payload cap. This excludes caller inputs, stack/geometry/report
values, allocator metadata, process RSS and export/storage/UI resources. Steps
and initialization perform no vector allocation. Allocation failure is a typed
outcome, without an operating-system overcommit/OOM-killer guarantee.

## Time, cancellation and failure

`step(requested_dt, cancel)` requires a finite positive interval. CFL coefficient
is finite in `(0,0.5]`. Each unique positive-axis periodic face has
`alpha=max(abs(un_left)+c_left, abs(un_right)+c_right)` and one LLF value applied
with opposite signs to both neighboring cells. Self faces contribute twice to
the CFL bound and exactly zero to divergence. Two-cell axes have two distinct
periodic faces. All three momentum components and pressure work evolve.

The bound is `C / max_cell(sum_incident_faces(alpha/spacing))`. On uniform
speeds `C=.5` gives `.25 / sum_axis((abs(u)+c)/spacing)`.
This is the admitted engineering bound, not an invariant-domain theorem.
Accepted dt starts at `min(requested_dt, bound)`. The caller hits a desired time
by requesting `min(normal_request, target - view.time)` repeatedly, observing
actual accepted increments. No clock is overwritten to conceal nonadvance.

Candidate admissibility and candidate-dependent diagnostic arithmetic failures
halve dt from the same accepted state, at most eight times (nine attempts).
Raw flux/divergence arithmetic is dt-independent and terminates immediately;
preflight/EOS input, cancellation and nonadvancing clock failures do not retry.
Every candidate is fully checked before publication. No failed call alters
accepted arrays, clock, generation, diagnostics or last accepted report.
`RetryExhausted` carries attempt count, last attempted dt and failure context;
success reports rejections and the last rejection separately from accepted
field diagnostics. Flux fallback count is zero in S1.

Cancellation callback returns true to cancel at a `CompressibleCheckpoint`.
Preflight occurs before CFL work; CFL visits every face endpoint pair;
candidate work visits each unique face and each candidate cell; diagnostics
visits each cell; publication checks once immediately before swapping state.
Checkpoints identify stage, candidate attempt (1..9, or 0 for preflight/CFL),
and cell when relevant. Cancellation during any attempt discards scratch and
returns `Cancelled` without retries. Cooperatively checked callbacks provide
no hard deadline or real-time guarantee.

`CompressibleError` distinguishes invalid parameters/state/length, allocation,
capacity overflow/limit, arithmetic, cancellation, nonadvancing time, generation
overflow and exhausted retries. State/arithmetic context carries stage, optional
x-fast cell index, optional component index and cause. EOS utility conversions
have no grid cell; initialization and step failures identify one when applicable.

## Diagnostics and arithmetic allowance

Accepted reports identify requested/accepted dt, clock/generation, actual
incident rate/CFL bound/controlling cell (first maximum in x-fast order), method,
EOS, shape/spacing/origin and managed capacity. They report volume-integrated
mass, three momenta and total energy, minima of rho/p/internal-energy/T,
maximum speed/Mach, attempts/rejections and zero flux fallbacks.

Neumaier compensated summation reduces `volume*U` in x-fast order. The periodic
ledger residual is candidate totals minus previous accepted totals; drift is
candidate minus initialization totals. They are measured binary64 residuals,
not exact conservation certificates. The frozen tiny S1 fixtures require
normalized drift `<=2e-13`, scale `max(abs(initial total),1)` in the stated SI
unit system; this is not an arbitrary-scale tolerance or clipping policy.
Formula/one-step roundoff allowances and continuum accuracy have distinct gates.
`host_step_seconds` is a monotonic elapsed observation covering preflight,
retries, diagnostics and cancellation callbacks, excluding exports.
`host_kernel_seconds` accumulates candidate flux/update time across attempts.
Both observations are separate from physical time and imply no speedup or real-time claim.

## Disposable native reference output v1

Parent admitted the independent validator's format at commit
`4621d69747740a950a90d78755fe3bb4e220efe4`,
`docs/plans/cpu-cfd-validation/reports/s1-export-proposal.md` in draft PR73.
It is a verification format, with no recording or dataset compatibility promise.
Changed layouts require coordinated producer/validator updates; stale outputs
are disposable and rejected. The producer emits actual fields and reports;
independent acceptance recomputes thermodynamics/budgets from raw U.

```
cargo run --locked --release --no-default-features --example compressible_reference -- \
  --output .generated/compressible-suite-fresh
```

The only command option is `--output FRESH_DIRECTORY`; parent directories must
exist. Existing outputs are preserved. The fixed suite contains eight cases:

| Kind | Shape | Geometry | Initial data | Final time |
| --- | --- | --- | --- | --- |
| uniform | 3x2x1 | h=(.2,.3,.4), origin=(.25,-.5,1) | rho=1.7, u=(.2,-.4,.3), p=.9 | .01 s |
| differential | 3x4x2 | same asymmetric geometry | smooth nonuniform all-axis field; bounded rho,p and transverse velocities | .001 s, one step |
| acoustic | 8³, 12³, 20³ | unit cube, zero origin | two linear conservative cell-average modes | .08 s |
| sod | 24x2x2, 48x2x2, 64x2x2 | unit cube, zero origin | face-aligned periodic Sod jump | .08 s |

Gamma=1.4, R=287 and C=.5. Each actual request is the remaining target interval;
accepted steps are capped by CFL/retries. No exact-time overwrite occurs.
At most 10,000 steps per case and a conservative 2GB raw-field budget bound
example output. Failures retain partial files and never publish `suite.json`.
The temporary manifest is `suite.partial.json`; it is renamed only after all
field and manifest writers flush. This is a completion convention, not an
atomic archive, fsync durability or concurrent-writer promise. Only the acquired
fresh output directory belongs to this run; no outputs belong in commits.

`suite.json` contains `schema_version=1`, `method="llf-euler-periodic-v1"`,
gamma/R, SI component units, output bounds and `cases`. Each case identifies
kind, id, shape, spacing/origin, periodic boundary, x-fast layout, initialization,
CFL/target and managed capacity. Every accepted frame (including initial) carries
actual native `generation`, `time_s`, `requested_dt_s`, `accepted_dt_s`, a
relative `file` path and accepted diagnostics. Initial generation/time/dt are
zero. Subsequent frames carry actual rate/CFL/control, attempts/rejections,
zero flux fallbacks, step/kernel host seconds, ledger/initial drift and optional
last rejection stage/cell/component/cause. Fields use case-directory six-digit
ordinals; CSV header is exactly `rho,mx,my,mz,E`, followed by one row per x-fast
cell and `.17e` binary64 round-trip values. No index or coordinates duplicate
layout metadata. Primitives are independently derivable from U.

The uniform initializer is `uniform-v1`. Differential `exported-initial-v1`
uses native bounded smooth primitives sampled at centers; this case tests one-step
translation from its exported initial values, not continuum cell-average accuracy.
Its rho/p stay in [.5,2], velocity components in [-1,1], and all axes vary.

Acoustic `linear-conservative-cell-average-v1` uses modes
`(k,w,phi)=((1,1,1),1,0),((1,-1,2),.5,pi/3)`, epsilon=1e-6 and c=sqrt(gamma).
Let each `s=epsilon*w*sin(2*pi*k dot x+phi)` multiplied by
`product(sinc(pi*k_d*h_d))` for its exact cell average. Initial U is
`rho=1+sum(s)`, `momentum=c*sum(s*k/|k|)` and
`E=1/(gamma-1)+gamma/(gamma-1)*sum(s)`.
This is linear conservative initial data: the finite-amplitude nonlinear Euler
residual is O(epsilon²). It has no quadrature error and supplies no finite-amplitude
exact-Euler claim. The validator independently measures its linearization defect.

Sod `sod-cell-average-v1` has `(rho,u,p)=(1,0,1)` left of x=.5 and
`(.125,0,.1)` right; the jump aligns to cell faces and a second jump is periodic.
The independent isolated-fan oracle integrates conservative U in each cell,
splitting at waves, then derives pressure. Comparisons use cells centered in
`.2<x<.8` at t=.08 before periodic-image waves reach those cell extents.
The validator measures root/quadrature uncertainty without changing frozen gates.

Origin admission is separate: the validation owner executes a pinned clean
committed binary and externally binds source commit/tree/build/executable,
exact command controls, artifact hashes and actual clocks. Metadata alone cannot
prove which physics ran; copied solver reports are not independent verification.
