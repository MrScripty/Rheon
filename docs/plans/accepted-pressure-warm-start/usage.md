# Opt-in accepted pressure initialization

Enable the non-default Cargo feature `accepted-pressure-warm-start` only for a
sealed coupled-liquid CPU consumer, then select an explicit initialization policy:

```toml
rheon = { path = "../Rheon", features = ["accepted-pressure-warm-start"] }
```

Default builds exclude warm solver branches, owner state, report fields and entry
points. The original default-particle +2.20% confirmation remains failed performance
evidence; this build boundary narrows exposure and does not erase that observation.
Historical gains apply only to their frozen binaries and workloads. Feature-enabled
mixed particle applications have no performance admission from those gains.

The native sealed liquid route accepts:

```rust
let report = session.step_with_initialization(
    inputs,
    rheon::PressureInitializationPolicy::AcceptedPreviousV1,
    None, // optional cooperative deadline
    |_| false,
)?;
let initialization = report.pressure_initialization;
```

`LiquidTransportSimulation::step_with_initialization` exposes the same policy
without the Session deadline/quota wrapper. Existing step/provider/boundary APIs
stay cold. Standalone `Simulation` does not retain a warm pressure array.

The first request uses a cold solve and reports `Constructor`. A subsequent
eligible request borrows pressure from the preceding accepted native interval,
in Pa, without timestep rescaling. The report distinguishes requested policy,
actual initialization, predecessor generation/operator revision/stage/incarnation,
and cold reason. Native cold publications can establish future eligibility.

PressureWorkspace's explicit `*_with_initial_pressure` methods only borrow a
caller-supplied guess; they do not certify accepted provenance. They still check
finite size/gauge, full RHS compatibility, current-RHS thresholds and full true
residual, including the eliminated gauge equation. No directions, residuals or
preconditioner recurrence survive from an earlier solve.

Only complete liquid publication changes accepted history. A failed/cancelled
carrier, projection, transport, reconstruction or final deadline check leaves
accepted fields, pressure, metadata, clock and quota unchanged. There is no
implicit warm-to-cold retry or enlarged iteration budget. An abrupt RHS change
can make previous pressure more expensive than cold initialization; the retained
switch pilot demonstrates this tradeoff. Policy selection is explicit.

Successful provider (including numerical fallback), boundary, viscous, surface or
column publication invalidates native history. A failed unsupported attempt does
not. New owners receive distinct process-local incarnations even with repeated
caller IDs. Current coupled geometry/density are immutable; future reset/import
or operator-changing APIs must invalidate through this same owner boundary.

Native projection is a distinct immutable Session intent. Its qualified projection
path stays Cold and never establishes reusable legacy warm history. Both initialized
legacy policies, including explicit Cold, refuse on a projection Session before
callbacks or state changes. Use that Session's native projection entry instead;
the warm feature does not add a projection seed or projection restart contract.

Session capabilities advertise API version3 and set `accepted_previous_pressure_v1`
to whether this Cargo feature is enabled.
Initialization reports are separate from provider reports. Existing dense3D frames
do not serialize warm history/provenance; process-local stamps are not portable
replay IDs. Numerical replay requires matching policy, accepted history and inputs.

The existing accepted pressure array is reused; managed-array budgets are unchanged.
Benchmark `capture_s` denotes scanning/hashing borrowed fields and an oracle, not
rendering/export. CPU cloud measurements do not establish GPU/device performance.

Explicit seeded solves retain conjugate directions when only the excluded gauge
row causes true-residual rejection. Recurrence replacement uses the reduced
system's true residual; success still requires the unchanged full residual and
divergence gates. Cold recurrence remains the reference implementation. This
opt-in distinction addresses a reproduced64³ localized restart stagnation; the
original candidate and its lost horizon are retained as rejected evidence.

The configured-provider recording API requires a current configured receipt.
A successful initialized Session publication clears that receipt, so existing
configured recording refuses it as unsupported rather than attributing it to a
stale provider. Frame export still captures accepted arrays; it does not create a
warm-history restart or a portable warm replay contract.
