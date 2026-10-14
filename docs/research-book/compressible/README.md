# Bounded compressible reference fixtures

This directory owns an original educational Python ideal-gas Euler reference
and its reproducible browser data. It does **not** implement or qualify the
production Rust solver. No third-party solver code, GPL code, or GPL dependency
is incorporated. NumPy is the only numerical dependency, using the existing
`numpy==2.3.5` pin in `../companion/requirements.txt`.

Reproduction inputs: [reference solver](reference.py), [independent analytic
oracles](oracles.py), [generator and record verifier](generate.py), [numerical
tests](test_reference.py), and [pinned numerical requirements](../companion/requirements.txt).

## Reproduce and verify

From this directory, using Python 3 with the repository's pinned NumPy:

```sh
python3 -m unittest -v test_reference.py
python3 -O -m unittest -v test_reference.py
python3 generate.py --output /tmp/rheon-compressible-example
```

The output directory must be nonexistent and outside every Git checkout.
Generation computes two 12³ cases, seven snapshots each, and a small refinement
check. It writes `results.json` and `qualification.json`. These generated
records stay outside the checkout. Source code and parameters determine the
records; no random draws or wall-clock timestamps are involved. The declared
seed is zero and is unused. Source SHA-256 values bind the numerical source,
analytic oracles, generator, and tests. Changing those files requires a fresh
generation. JSON keeps binary64 round-trip values and rejects NaN serialization.

`generate.validate_results(results)` checks provenance, reconstructed primitive
fields, conservation diagnostics, and qualification metrics.
`generate.validate_qualification(report)` recomputes acoustic errors, Fourier
phase speeds, and Sod profile errors from recorded probes using continuum
formulas. Neither validator reruns the time-stepping solver. These are local
producer checks; the browser publisher also owns validation of the data it
consumes. Equality with source hashes proves freshness, not physical accuracy.

## Equations and discretization

The state is `U=(rho,rho*u,rho*v,rho*w,E)` on uniform Cartesian cells in the
dimensionless unit cube. The ideal-gas closure is
`p=(gamma-1)*(E-|momentum|²/(2*rho))`, with `gamma=1.4`. The axis-i physical
flux is `(rho*u_i, momentum*u_i+p*e_i, (E+p)*u_i)`.

Each periodic interface uses the conservative local Lax-Friedrichs/Rusanov
flux `0.5*(F_L+F_R)-0.5*a*(U_R-U_L)`, where
`a=max(|u_i,L|+c_L, |u_i,R|+c_R)` and `c=sqrt(gamma*p/rho)`.
An unsplit explicit Euler update takes all three flux divergences together.
The step limit is
`dt=CFL/max_cells(sum_i((abs(u_i)+c)/dx_i))`, with `CFL=0.25` and an API
maximum of `0.3`. Output times can shorten a step. State validation rejects
nonfinite values, nonpositive density, or nonpositive pressure before accepting
a step; there is no floor, clipping, repair, or retry. Caller state is unchanged
on failure. Configurations are bounded to 2–64 cells per axis and dimensionless
end time at most one, with a 10,000-step hard limit.

Initial smooth fields are sampled at cell centers. They serve as cell values
for this first-order fixture; analytic comparisons use the same centers rather
than claiming exact cell-average initialization. All three momenta and total
energy evolve. Periodic paired interface fluxes imply conservation of the five
volume-integrated components up to floating-point reduction error.

The continuum relations used by the separate oracles are documented in
Ketcheson, LeVeque, and del Razo's primary educational reference,
[Euler equations and exact Riemann solutions](https://www.clawpack.org/riemann_book/html/Euler.html)
and [linear acoustics](https://www.clawpack.org/riemann_book/html/Acoustics.html).
Only mathematical relations are used; no code or prose from those references
is copied. `oracles.py` uses self-similar shock/rarefaction relations and scalar
pressure bisection, not the reference solver's numerical flux.

## Cases and limits

- **Planar Sod in a periodic 3D cube:** left density/pressure `(1,1)`, right
  `(0.125,0.1)`, zero velocity, discontinuity at `x=0.5`. Periodicity introduces
  a second discontinuity at `x=0`. The central exact Sod comparison is restricted
  to `0.2<x<0.8`, at `t<=0.08`, before physical waves from the periodic image
  reach that region. Finite-grid numerical diffusion is still present. This
  case is extruded and tests a one-dimensional wave structure in a 3D state;
  it is not a test of general 3D dynamics.
- **Crossing acoustic waves:** two distinct modes `k=(1,1,1)` and `(1,-1,2)`
  with density amplitudes `0.02` and `0.01`, phases zero and `pi/3`, background
  density/pressure one. Their velocity perturbations point along their wave
  vectors, so all three momentum components vary in space and time. Two
  nonparallel modes provide genuinely volumetric variation. Viewer errors
  compare against linear acoustics at finite amplitude and therefore combine
  discretization error with nonlinear discrepancy.

Qualification uses acoustic amplitude `1e-6` on 8³, 12³, and 20³ grids at
`t=0.08`, reducing nonlinear discrepancy. It requires decreasing normalized
density L1 errors, observed refinement orders above `0.6`, and finest-grid
phase-speed error below 8%. Fourier projection onto `k=(1,1,1)` isolates that
mode; its phase shift is under pi so there is no wrapping ambiguity. The report
also shows amplitude attenuation, an important limit of first-order LLF.

Sod refinement uses `[24,2,2]`, `[48,2,2]`, and `[64,2,2]` meshes because the
problem is planar. It requires decreasing density L1 errors and finest-grid
pressure L1 error below `0.04`. These are bounded sanity checks, not an
asymptotic convergence study. All displayed frames must have strictly positive
density/pressure and relative conservation drift below `2e-13`. The drift
denominator is `max(abs(initial_component_total),1)`, so nearly zero momentum
totals use an absolute scale instead of an unstable division.

These tiny inviscid examples make no turbulence, complex geometry, viscosity,
shock-resolution, performance, physical-validation, or production-solver
claim. Strong first-order diffusion visibly smears shocks and damps waves.

## Browser packet contract, version 1

`results.json` contains `schema_version`, `provenance`, `limitations`, `cases`,
and `qualification`. Each case has `id`, `title`, `parameters`, `grid`,
`field_ranges`, and `frames`. Grid shape is `[nx,ny,nz]`; extent is
`[[0,1],[0,1],[0,1]]`; centering is `cell`. Flat array order is C order with
z fastest: index `(i*ny+j)*nz+k`; cell center is
`((i+0.5)/nx,(j+0.5)/ny,(k+0.5)/nz)`.

Every frame includes:

- `time`, flat `density`, flat `pressure`, per-cell `velocity=[u,v,w]`, and
  per-cell `conserved=[rho,rho*u,rho*v,rho*w,E]`;
- `diagnostics.mass`, `.momentum` (three components), `.energy`,
  `.conserved_totals`, `.min_density`, `.min_pressure`, `.max_speed`,
  `.relative_conservation_drift`, `.steps`, and `.density_l1_reference_error`.

Totals integrate over the unit cube (cell volume is `1/(nx*ny*nz)`), and
`field_ranges` gives density, pressure, and speed extrema across every frame.
Qualification contains acoustic and Sod metric summaries and the sampled probe
fields needed to recompute those summaries. Its standalone JSON is the same
object embedded in `results.json`. Version 1 is jointly owned by this producer
and its browser consumer; incompatible changes must update both together.
