# Conservative represented liquid-volume transport

This slice implements Chapter 21's shared-face volume ledger as a separate,
transactional `LiquidVolumeState` on a fixed Cartesian grid. It stores dimensionless
cell-average liquid fractions, a constant positive represented-liquid density,
version and time. Dry/mixed/full occupancy is derived from accepted fractions.
Those labels are not a pressure-unknown or mesh-solid mask: supplied carrier
velocity is held over an explicit interval, with no free-surface feedback. The
existing smoke tracer, sampler, force and pressure implementations are unchanged.

## Face transfers, boundaries and sources

For each Cartesian face compute one signed, positive-axis interval transfer
T=dt*A*u*f_upwind in m³. Interior upwind fractions come from old cells. Inflow
uses explicit constant outside fractions on the complete box side; outflow uses
the old adjacent cell. Zero velocity transfers zero. Adjacent cells reuse the
same stored transfer with opposite signs; all axes update together from old
fractions. This is first-order unsplit donor-cell transport, not PLIC or another
geometric interface reconstruction.

Each cell starts with amount m=f*V, subtracts outward transfers, adds inward
transfers and adds dt*s, with signed borrowed source rates s in m³/s per cell.
Source saturation or post-update fraction clamping would change the volume ledger
and is forbidden. Raw candidate fractions must be finite and within [0,1]. The
independently checked local summed outward Courant number is at most the declared
limit in (0,1]. Direct carrier divergence is independently measured and gated.
For exact divergence-free flow, zero sources and compatible inflow donors, the
update is a convex combination when outward Courant is at most one. Courant alone
does not ensure bounds for nonzero divergence or arbitrary sources. Even a small
accepted carrier residual can trigger raw fraction rejection near full cells.

The accepted report verifies

V_liquid_after - V_liquid_before + V_outward - V_inward - V_source = error.

Compensated reductions and a budget of 64*epsilon times the sum of absolute old,
new, boundary and source amounts make the binary64 acceptance convention explicit.
This is not a certified floating-error bound. Nonzero products or amounts that
underflow out of representation, nonfinite arithmetic and unresolved time are
rejected. The constant density scales represented mass as rho*V_liquid; it does
not change a carrier pressure coefficient or introduce two-phase inertia.

## State and ownership

`LiquidFlowInterval` borrows the exact immutable grid and all three finite full
face fields, together with an explicit start and dt. Its end is start+dt; start
must match the accepted volume clock exactly. Velocity, inlet and source stamps
are caller-owned identities copied into successful reports. The accepted volume
stamp version increments only on publication. Geometry, timing, input shape and
settings failures precede callbacks. The inlet snapshot specifies outside liquid
fractions rather than silently extending an appearance tracer at inflow.

The state owns two f64 cell arrays and three f64 face-transfer arrays. Its limit
covers actual retained Vec capacities, including the transferred initial Vec:
nominal payload 16*Ncell+8*Nface bytes. Flow/source arrays, allocator overhead and
RSS are excluded. Advances allocate no heap buffers. Cancellation or failure may
leave scratch partial but preserves accepted fraction bits, volume stamp and
time. Retry overwrites scratch and agrees with fresh execution.

Publication is transactional for this volume-only state. It does not roll back a
separate accepted carrier simulation or constitute a complete liquid step that
publishes pressure, geometry and phase together. Tests explicitly demonstrate
that separation. Future coupling must establish the appropriate liquid/air
pressure mask, velocity-validity mask and unified accepted-state transaction.

## Volume and spatial evidence

Signed transfers in all axes, an anisotropic closed circulation, oblique constant
transport, signed sources and thin merging supports exercise conservation and
bounds. Both accepted PCG implementations supply a readonly 1D projected carrier
in a separate fixture; liquid fractions do not feed back into that pressure solve.
A 3D translated unit cube remains bounded and retains volume one, yet its
cell-average L1 volume error is 0.65625 after one unsplit step. Conservation does
not establish geometric accuracy.

An independent binomial oracle checks axis-aligned slab transport. The slab starts
at x=[0.25,0.5], moves at 0.25 m/s for 0.5 s, and is compared with exact translated
cell overlaps at x=[0.375,0.625]. With outward Courant 0.5, 16/32/64 cells give
cell-average L1 volume errors 0.09375, 0.068359375 and 0.04909515380859375. Total
volume remains exactly 0.25 m³ and centroid is correct. Halving dt at fixed 64
cells (Courant 0.25) increases the error to 0.06039030345194791: this first-order
scheme becomes more diffusive as dt decreases at fixed spatial stencil. Record
both refinements instead of claiming that smaller dt alone improves interface
shape. These mechanisms are not calibrated pouring, thin-film or droplet physics.

VolumeLedger.lean supplies eight exact finite theorems and three definitions for
shared-face/source/boundary conservation, constant-density mass scaling,
conditional nonnegative/unit-interval donor updates and constant preservation.
Explicit Courant and post-clamp counterexamples name missing assumptions. Its
incidence/weights are supplied; it does not certify Rust assembly, IEEE execution,
geometry or continuum accuracy. The historical proof inventory and accepted
book/PDF inputs are preserved. See the [executed evidence](../../../evidence/liquid-volume/README.md).

Remaining work includes geometric reconstruction and interface normals, active
free-surface pressure/air extension, density-dependent correction, unified liquid
step publication, mesh-solid classification/cut cells, moving swept volume,
viscosity/traction, wetting and capillarity. This milestone earns a represented
volume-transport claim, not a general liquid solver or moving mesh wall claim.
