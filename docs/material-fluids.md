# Material scenes and physical scale

The desktop defaults to water emitted into a **1 m cubic vacuum domain**, with
Earth gravity `[0, -9.80665, 0] m/s²`. Settings are grouped into independently
collapsible sections, retaining expansion state while the app is open. A run copies its settings, so accepted history retains
its own scene scale while the next run is being configured.

## Controls and units

| Control | Meaning |
| --- | --- |
| Domain side | Cube side in metres; volume is side³ |
| Emitter center, length, radius, nozzle depth | Physical metres |
| Pressure heads | Prescribed nozzle overpressure in Pa |
| Spin | Revolutions per simulated second about vertical Y |
| Density | Mass per volume, stored in kg/m³ |
| Viscosity | Dynamic Newtonian viscosity in Pa·s |
| Surface tension | Liquid cohesion coefficient in N/m |
| Wall drag | Tangential contact damping rate in s⁻¹ |
| Gravity | Acceleration vector in m/s² |
| Fluid temperature | Source/reference temperature, displayed in °C, stored in K |
| Ambient / wall temperature | Initial surrounding-fluid temperature and wall temperature |
| Wall heat exchange | Contact temperature-relaxation rate in s⁻¹ |

`1 g/mL = 1 mg/mm³ = 1,000 kg/m³`. Density is not weight. A filled domain's
initial mass is density × physical domain volume; weight also depends on gravity.
Changing the density display unit does not change the stored material value.

Changing the domain side proportionally scales the emitter position and sizes;
its dimensions remain editable individually in metres. Pressure, gravity and
spin are not rescaled. The renderer normalizes the physical cube for navigation,
while integration, parcel mass, neighbor distance and stability use metres.
Changing scene size therefore changes physical behavior, not just camera zoom.

## Initial contents and previews

- **Empty / vacuum:** no initial carrier particles, pressure or fluid mass.
- **Filled with emitted fluid:** seed that material throughout the box.
- **Filled with another fluid:** seed the selected surrounding material at
  the ambient temperature; emitted and surrounding parcels both participate.

Closed stationary walls retain particle mass. Previews highlight emitted
material and exclude surrounding-material parcels from the visible surface,
so a filled gas does not hide the jets. Filled-with-emitted-fluid previews show
that initial fluid too. Appearance is a voxel splat of parcels, not a calibrated
optical-density image. The original smoke view retains its concentration model.

## Solver choice and ownership

The existing carrier solver's admitted slab/column surfaces cannot represent
rotating jets entering vacuum, and its constant-density projection cannot
describe gas expansion. The new `ParticleFluidSimulation` owns a bounded SPH
preview with a distinct physical model; the original pressure implementations
and their replay contracts remain available by disabling **Liquid / gas
material simulation** in Simulation. Pressure-solver comparisons apply to that
original mode. Material controls are disabled there because its legacy smoke
physics does not implement them.

Owning this small preview avoids a C++/FFI engine or a new physics dependency
with different scene, memory and threading contracts. It reuses the established
Rayon pool, batch feedback, worker observer, timeline and renderer. Its basis is
the compact 3D kernels described in the
[Illinois SPH notes](https://cs418.cs.illinois.edu/website/text/sph.html) and the
[SPH tutorial](https://interactivecomputergraphics.github.io/SPH-Tutorial/).
This is an explicit scope choice, not a replacement for a validated multiphase
engineering solver.

Linked-cell neighbor searches visit only occupied material parcels. Density
uses reference-volume weights: `rho_i = rho0_i * sum((mass_j/rho0_j) W_ij)`.
Pressure differentiates compression energy using that same poly6 density
kernel, including the reference-density factors for unlike materials. Each
pair has equal and opposite force. Viscosity damps relative
motion through a positive kernel Laplacian; liquid cohesion uses a symmetric
attractive pair approximation. Gravity is applied in SI acceleration units.

Liquids use a positive, weakly compressible pressure law with reduced acoustic
stiffness. Gases use `p = rho R_specific T` and their isothermal acoustic speed.
Neighbor-estimated density has a small positive regularization floor. Material
interfaces, wall pressure support and surface cohesion are approximate; surface
curvature, contact angles, compressible nozzle choking, shocks and rarefied
vacuum-gas kinetics have not been qualified. Thermal behavior consists of gas
pressure dependence and contact relaxation toward wall temperature. There is
no bulk vacuum cooling, conduction field, latent heat, boiling or freezing.
Phase stays fixed and density/viscosity remain the entered reference values.
Slime is Newtonian and does not stretch elastically.

The nozzle creates fixed-reference-volume parcels, with velocity derived from
pressure head and a nonuniform disk profile. A deterministic disk sequence
samples each end. A 0.43 profile-average approximation determines reservoir
volume flux. Fractional volume credits are retained per end, so a coarse scene
can have several empty emission steps before a parcel is created. Accepted mass
is exactly accounted as retained parcel mass plus emitted parcel mass; the
external reservoir supplies that mass. The visible cylinder remains a source
illustration rather than a moving collision solid.

Each accepted step adapts to acoustic, advective, gravity, viscosity and
cohesion limits. An additional force check can restart preparation at a shorter
interval, at most 16 times. No force or speed is silently clipped. A failed
preparation preserves particles, clock, generation and source credits together.
Only committed states enter history. Wall contact constrains positions and
normal motion, then applies tangential drag and contact heat exchange.

Particle sampling is separate from the preview voxel resolution. More samples
reduce physical parcel spacing and increase work. The particle limit and owned
array cap are explicit; reaching the particle limit stops the run with an
error and retains accepted previews. Numerical buffers reserve bounded capacity
at construction and are reused during stepping. Library callers can supply
their own scoped pool; particle density/force passes parallelize from 512 items,
while cheaper grid/vector passes retain their previous threshold.

Preview defaults retain their adaptive 512-axis ceiling, independently of
physical particle sampling (4–64 per side). New density requests support up
to 1000 samples/m, subject to the particle owner's bounds. Exact preview density
(1–1000/m) and exact preview edge (1–10000) are separate display controls and
also check realized density, history and rendering limits. Increasing preview
resolution does not add physical particle detail. Every accepted step remains
available within the chosen 16–4096 MiB history (default 96 MiB) budget; explicit exact requests
are refused rather than downsampled. For example, 256³ fits a two-frame history
under the default budget; 1025 frames at 128³ require slightly over 2 GiB. Carrier
counts now allow up to 10000/axis, with independent physical-density and 192 MiB
native-array preflight. These settings do not promise a 10000³ allocation.
See [density/resource contracts](api/domain-density.md) for rounding, independent
preview, rendering refusal and retained historical geometry.

## Presets and verification

Water and mercury use approximate room-temperature starting values. Aluminium,
tin and iron presets declare elevated liquid reference temperatures. Air,
argon and helium derive reference density from the ideal-gas equation at
101,325 Pa and 20 °C. Slime-like and syrup presets are adjustable visual
approximations. Presets do not select a temperature-dependent property database.

Primary property references are
[IAPWS liquid-water properties](https://www.iapws.org/relguide/LiquidWater.pdf),
[IAPWS water surface tension](https://www.iapws.org/relguide/Surf-H2O.html),
[NIST fluid properties](https://webbook.nist.gov/chemistry/fluid/),
[NIST gas viscosity measurements](https://www.nist.gov/publications/recommended-viscosities-11-dilute-gases-25degc)
and the [NIST liquid-metal correlation review](https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=956945).
Rounded preview presets are not asserted to reproduce those correlations.

Independent fixtures check SI ballistic acceleration at two scene scales,
cubic mass scaling, reference gas pressure, pair momentum, viscosity energy
decrease, unlike-material inertia, cohesion and wall-temperature relaxation.
Contracts also exercise source mass accounting, long rotating scenes, filled-air
scenes, explicit capacity failures, invalid values, cancellation and exact
thread-count replay. Desktop tests exercise actual egui collapse input and a
gated worker publishing material frames before completion while scrubbing holds
an earlier frame.

Implementation and verification use the Standards MCP guidance already routed
for this workspace; standards Markdown is not treated as authority.

Local verification on 2026-10-12: the full release desktop suite passed 361
tests, including pressure-energy and floor-rebound regressions, physical-nozzle
scaling, particle thread-count replay and real 256³ preview capture. Core-only
pressure-energy and floor-rebound regressions also passed. Desktop and core-only
all-target Clippy, formatting and diff checks passed, and both release binaries
built. An earlier native Linux window capture showed a completed 1,024-step
material run, 1.062 s of simulated time, retained voxel surfaces, mass and
microsecond step timing. The image is `/tmp/rheon-material-desktop.png`.
