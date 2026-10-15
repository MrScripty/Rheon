# Configured Session recording

`recording::provider` captures published `SimulationSession` values in the existing
SQLite v1 `StepRecorder` / `RecordingReader` envelope. Its distinct solver profile
is `rheon-configured-session-v1`. It neither changes the SQL schema nor makes a
native-carrier, material, or diagnostic record acquire new meaning. The native
`RecordedCarrier` decoder refuses this profile; the configured decoder refuses
other profiles. Unsupported profile or codec versions fail explicitly.

## Producer and reader

- `configured_run(&SessionSnapshot, provenance)` captures actual immutable
  constructor configuration and geometry in `RecordingRun`.
- `configured_publication(&SessionSnapshot, ordinal, limit)` copies one published
  constructor or fully accepted configured interval. Call after an accepted
  step, before attempting another append. Failed physics does not create a new
  publication; attempting to append the retained old generation is refused by
  the existing recorder's ordering contract.
- `RecordedConfiguredSession::decode(run, step)` returns one owned typed selection
  after validating its profile, configuration, descriptors, stamps, interval,
  provider receipt and accepted field domains.
- `verify_configured_recording(reader)` first invokes existing SQLite/digest/tail
  verification, then checks the typed rows and recomputes retained prior-pressure
  links. It reports checked and unavailable prior-link counts. The store's
  contiguous initial-prefix rules remain authoritative: an archive beginning at
  generation two is refused, not reported as a verified missing prefix.

The constructor has velocities, tracer and fraction, with no interval pressure or
provider receipt. An accepted publication adds actual interval pressure and the
owned configured-provider receipt. Face velocities are native staggered f32 m/s;
tracer is cell f32; fraction and pressure are cell f64. Every numerical byte is
little endian with explicit dimensions, x-fast order, association, units, stage
and evaluation time. Pressure uses the accepted interval's cell-zero gauge and
must not be interpreted as endpoint pressure. Fields are copied from the Session
snapshot, never candidate scratch or the separate native carrier success stamp.

The configuration is the numerical owner's `SessionConfiguration`: original
transport configuration, pressure implementation, boundary workspace cap and
Session quota. The snapshot getter retains the actual constructor arguments
before internal memory repartitioning. It does not reconstruct Debug text.
Per-call requested timesteps, forces, sources, inlet and provider schedules are
not retained by that constructor configuration. An application must freeze their
intent separately. This profile does not claim an executable simulation recipe,
authenticate declared source/package identities, or contain a restart checkpoint.
Absent initial rows, controls, runtime details or model artifacts stay absent.

## Owned receipt codec

Run configuration uses independently versioned `RSC1:` lowercase hexadecimal
binary metadata. Publication controls use `RCP1:`. The existing envelope hashes
and budgets both texts together with their fields. Integer and floating values
use little-endian fixed-width bytes; f64 values use their IEEE754 bits, without
JSON conversions or floating-point text formatting. Counts are u64 on disk with
checked host-size conversion. Optional values and booleans use explicit tags;
unknown codes, malformed/trailing bytes and invalid UTF8 are refused. Each publication also binds its actual Session identity, configuration and
geometry to the run header with a SHA256 digest; caller provenance is separate.
The source codec is the canonical field order. There is no alternate decoder or migration.

Strings are length-prefixed UTF8, bounded to 1000 bytes; the complete metadata
must fit the existing 16 KiB text bound and caller record budget. Package identity
also passes the configured provider's existing bounded identity validation.
Attempted role metadata must match the configured pressure role, shape, units
and stage. That stable role is distinct from the actual owned package binding.
A future role/provenance contract requires an explicit new supported profile.

The owned receipt preserves:

- Optional actual package binding, manifest/checkpoint pins, training ID, runtime
  and declared source revision.
- Optional actual host context: generation, exact interval/dt, native prefix,
  prior generation/time/pressure hash and observed RHS hash.
- Actual applied dynamic viscosity and `prediction_called`, meaning host trait
  method entry. It does not attest IPC dispatch or external execution.
- Closed native/configured selection and rejection codes, bounded rejection
  detail, attempted role metadata, rejected candidate diagnostics and accepted
  pressure diagnostics.
- Exact `Duration` seconds and nanoseconds for total pressure call, inference,
  qualification and fallback. Nanoseconds must be below one billion.

Rejected-candidate metrics preserve nonfinite IEEE values, including NaN payload
bits. A legitimate accepted native fallback can retain such diagnostics after
rejecting a finite but excessively large candidate. They are not converted to
null, zero, or accepted physics. Accepted pressure diagnostics and all accepted
field values must be finite; nonnegative metrics, fraction/tracer domains,
pressure gauge and descriptor relationships remain enforced. Candidate selection
requires actual prediction, binding/context and matching accepted diagnostics;
an accepted receipt cannot contain a cancellation rejection.

For retained consecutive contexts the verifier hashes the preceding accepted
pressure's exact little-endian f64 bytes and checks its generation, interval end,
composition and native-prefix history. A per-row decode alone does not verify
that cross-publication link. An RHS hash identifies an observed host request;
without pre-pressure RHS bytes this is not independent cold reconstruction of
that request or a new physical qualification. Existing numerical gates remain
the acceptance authority.

## Lifecycle and costs

Use `StepRecorder::create`, `append` and `finish` unchanged. Keep existing record,
database and payload caps, single-writer transactions, poisoning, terminal-tail
classification and no-overwrite behavior. Capture/append failures leave accepted
physics unchanged and must be reported as an unrecorded accepted tail. Do not
mark a truncated archive completed or roll back an accepted interval to make the
recording appear complete. Reader selection owns one bounded record, not a live
simulation or hidden history cache.

The pressure durations exclude Session publication, subsequent transport and
final callback work, caller preparation, model construction/startup, recording
and rendering. They are scoped observations, not full-step timing, RSS, an IPC
measurement or a speedup claim. Future runtime-owned failure/dispatch/digest facts
must be separately captured by that owner; this codec does not fabricate them.

## Consumer and evidence

```
cargo run --locked --no-default-features --example recorded_configured_provider -- record NEW.sqlite SOURCE_REVISION
cargo run --locked --no-default-features --example recorded_configured_provider -- inspect NEW.sqlite 2
cargo test --locked --no-default-features --test recording_provider_contract
```

The record example uses actual native pressure solves as its authored candidate
fixture. Inspect constructs only the reader/decoder, never a Session, provider,
package or model. Public tests execute separate producer and inspector processes,
compare native field/receipt bits and exercise semantic negatives, limits,
unrecorded tails and cancellation. Final validation results belong to the
source-bound PR evidence, not these command examples. This slice does not prove
the missing measured `aa82` package, external runtime parity, model accuracy,
hardware behavior, saved run-set integration or complete M2/M3 acceptance.
