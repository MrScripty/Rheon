# Configured accepted pressure sessions

`SimulationSession::step_with_configured_pressure` composes an optional stateless
`ContextPressureProvider` with the existing accepted pressure gate and transport
transaction. An optional existing `ViscosityWorkspace` applies the sealed-box
constant-density Newtonian stage before the pressure RHS is assembled. This is a
finite admitted composition; it is not a configurable stage graph.

The numerical owner supplies `ContextPressureRequest`: native geometry, actual
density, the actual shortened timestep and interval, the post-viscosity RHS,
accepted prior pressure with run/generation/time lineage, the actual dynamic
viscosity, and the initial native accepted prefix for that composition. Adapters
receive borrowed immutable context. They neither advance history nor publish
fields. Package-specific channels, preprocessing and candidate reconstruction
belong behind this interface; genuinely new physical or hidden-state semantics
require a separate contract and qualification.

`supports` runs before `predict`. Pressure-stage support, execution and candidate
quality rejection use the existing native same-request fallback. Candidate
pressure passes the existing gauge, finiteness, pressure-residual and
predicted-divergence gates. The subsequent actual corrected-velocity and volume
gates reject the entire interval on failure; they do not retry the full step.
`BeforeCommit` must also pass before any accepted field, time, report, history or
quota changes. Cancellation, deadline and post-pressure failure publish no new
receipt. Workspaces may contain failed scratch; only the Session snapshot is
accepted state.

Changing the viscosity coefficient invalidates the previous composition's
history. A legacy boundary/transport-only call clears the typed pressure history.
If a later-generation context lacks its immediately preceding accepted pressure,
the call falls back natively without invoking the configured provider. That
accepted fallback establishes history for the next call. Native prefix counts
stop increasing after a configured candidate is selected; rejected predictions
and support refusals retain genuinely native accepted history.

`ConfiguredProviderReport` accompanies only a completely accepted interval:

| Field | Meaning |
|---|---|
| `binding` | Bounded validated owned package identity declared by the runtime; absent for native-only calls and invalid identities |
| `context` | Exact host request generation/interval/dt/prefix and SHA256 of little-endian IEEE-f64 RHS/prior values, when a valid context reached provider support checking |
| `dynamic_viscosity` | Actual host stage coefficient, zero for inviscid calls |
| `prediction_called` | Host entered the trait's prediction method; it does not prove an external process received bytes |
| `pressure` | Existing selected/attempted/rejected pressure report, diagnostics and scoped timings |

The stable attempted role `configured-pressure-context-v1` is distinct from the
actual package identity. Identity declarations and hashes do not authenticate an
executable. A concrete runtime owner must verify its files/pins before execution
and separately retain actual IPC dispatch/failure facts. Invalid bounded identity
metadata never enters an accepted binding receipt.

The Session retains metadata and borrows its existing accepted pressure array;
it adds no pressure-history volume. The step's array-byte report includes the
existing provider and viscosity workspaces. External providers must bound their
own allocations and honor the cooperative cancellation callback.

`SessionSnapshot::configuration()` borrows immutable `SessionConfiguration`:
the original constructor transport controls, native implementation, separate
boundary-workspace limit and step quota supplied to the validated owners. These
are actual constructor controls, including their original logical memory limits;
they do not reconstruct a future experiment recipe. Per-call requested forces,
timestep, viscosity and provider schedule remain separate intent. The accepted
receipt supplies the actual applied viscosity and actual pressure interval.

`SessionSnapshot::accepted_interval_start()` retains the exact pre-step carrier
time for every accepted Session path, including native-only configured calls
whose provider context is absent. It returns None at construction and preserves
the prior value after failed/cancelled attempts or stop. A recorder must use this
owner-provided value: subtracting dt from the recorded end can change its IEEE
bits after repeated nonbinary timesteps.

The viscous composition requires sealed full liquid fractions, represented
density equal to the carrier density, no phase source, and the existing
Newtonian stage. The inviscid path retains the existing passive represented-volume
domain. Both use the existing pressure candidate role; free-surface/column/boundary
provider compositions remain outside this seam. Existing legacy APIs and archive
versions retain their meanings.

Eight public contract tests exercise native viscous bit equivalence over multiple
steps; independently reconstructed nonzero post-viscosity RHS and actual dt;
accepted history; support/quality fallback; coefficient/history recovery;
cancellation/deadline/post-pressure rollback; bounded workspaces/domains; and
invalid-identity receipts. Their candidate fixture performs real native pressure
solves. These tests do not establish learned-model accuracy, measured-bundle
parity, a subprocess adapter connection, or durable typed-report persistence.

Actual CPU fixture contracts additionally connect this public Session seam to
the existing ModelProcess owner through a test-only driver. Independently
authored Python solves the nonzero two-cell sealed equation, checks accepted
lineage/named ports/residuals and supplies full-pressure or correction outputs.
The tests compare native accepted field bits, observe physical/context support
refusal before inference, quality fallback, real child failure, in-flight
cancellation/deadline and fresh-owner retry. They also challenge pre-dispatch
and blocked-writer cancellation. This remains analytic fixture evidence; the
production GUI/package bridge and durable process-failure receipts are separate.

The process owner offers cooperative polling in its existing20ms write/reply
waits, reusing the same cancellation flag and immediate-child slot. This is not
a response-time or hard-shutdown guarantee. A reply can win before another poll,
so the Session retains its later publication checkpoints. Cancellation terminates
that runtime; a retry must construct a fresh runtime owner.
