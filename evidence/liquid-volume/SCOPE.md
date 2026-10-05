# Conservative represented liquid-volume transport

Physical assumptions: fixed uniform Cartesian cells; incompressible supplied
MAC carrier velocities held over an explicit interval; constant positive
represented-liquid density. Fractions are dimensionless cell volume occupancy.
Signed shared face transfers use first-order unsplit upwind fractions. Inflow
fractions are specified per complete box side; outflow uses the old adjacent
fraction. Sources/sinks are borrowed m³/s per cell. No post-update clamp or mean
correction is permitted. Global amount balance is checked with an explicit
binary64 reduction budget. Local outflow Courant and raw fraction bounds are
independently gated, as is measured carrier divergence.

This is an independently owned volume-only state. Occupancy labels derive from
accepted fractions; they are not a pressure-unknown, air-velocity or mesh-solid
mask. Failure preserves accepted volume, version and time, but does not roll
back a separate carrier simulation. No free-surface pressure, PLIC normals,
curvature, geometric reconstruction, viscosity, wetting or general moving mesh
wall claim follows. The original smoke tracer and pressure implementations stay
unchanged. This implements the shared-face volume ledger required by Chapter21
before a complete coupled liquid state.

Planned fixtures: signed all-axis donor transfer and inlet/outlet/source ledgers;
closed circulation, anisotropic/oblique flow, constants and density scaling;
axis-aligned slab translation with independent exact cell-overlap volume/spatial
error at useful h/dt refinements; thin fractions and merging supports; CFL/raw
bounds/divergence failures without clamping, partial cancellation/retry, version
and interval/geometry identity, exact capacities and arithmetic underflow.
Full default/core/desktop tests, strict Clippy, original-byte replays and book/
proof source gates are required. Lean will formalize shared-face/source balance,
constant-density mass scaling and conditional convex bounds, distinguishing row
normalization from true volume conservation. It will not certify IEEE execution,
interface geometry, pressure coupling or continuum accuracy.
