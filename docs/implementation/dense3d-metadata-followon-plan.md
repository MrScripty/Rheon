# Accepted-control metadata follow-on: source map and pending contract

This is a preliminary implementation plan, not a proposed wire schema or a
qualification of training data. It is based on exact public producer commit
`fee7b4a139574f87b259796b1ba8698a41d31ac1`. The separate aligned-strain lab
and aligned Stokes experiment branches remain unchanged.

The required consumer input is Tuldok's
`docs/plans/sequence-field-controls/contract.md` from the confirmed archive
`Tuldok-native-sequence-inspector-29240e4-source-and-evidence.zip`. The current
supported Library materialization route resolved the archive but its byte
transfer failed (`library file transfer failed: download failed`). No readable
contract bytes arrived. Field names, version selection and admissible control
representations therefore remain pending; no replacement schema is inferred.

## Existing producer ownership

`src/dense3d_sequence.rs` owns both `LiquidTransportSimulation` and its counted
output sink. Construction accepts only the constructor state and writes frame
zero. `step_with_box_flux` receives the actual copied/borrowed controls, checks
frame and output byte budgets before physics, and calls the simulation once.
Only an `Ok` report is serialized. Rejection or cancellation emits no accepted
frame and permanently aborts the export. A writer error after successful physics
leaves the new accepted simulation state intact and poisons export; a partial
trailing line is possible. This behavior must remain explicit and tested.

`src/liquid_step.rs` defines `LiquidStepInputs`: requested timestep, optional
appearance smoke source, borrowed external body forces, inlet snapshot, optional
borrowed cell-volume source and volume acceptance settings. It prepares the
carrier and represented-volume candidates, then commits both only after its
final cancellation checkpoint. The returned report carries the actual accepted
dt/time, carrier generation, liquid/flow/inlet/source stamps and optional box
boundary report. Accepted dt may differ from requested dt; the exported record
must follow the accepted report rather than relabel a request as an outcome.

`src/box_flux_step.rs` defines copied end-of-step boundary requests. Advection
uses the previous velocity; projection applies the requested normal boundary
snapshot; appearance transport uses the resulting velocity with the selected
extension policy. `src/box_flux.rs` exposes the actual native-f32 outward-normal
speeds and stamp through `outward_speeds()` and `stamp()`. Lower/upper signs are
outward-normal conventions, not Cartesian component values.

`src/liquid_volume.rs` exposes copied inlet fractions/stamp. Liquid source rates
are native-f64 cell values in m^3/s, borrowed for the call; the type currently
has no public rate/stamp accessors. It must not be serialized as an empty source
or reconstructed from the aggregate source ledger. Depending on the exact
consumer contract, honest choices are exact read-only accessors or explicit
refusal of unsupported nonempty sources before advancing physics.

`src/forces.rs` exposes native-f64 vectors, acceleration versus force-density
units and optional half-open world-space regions. `src/simulation.rs` exposes
the optional appearance-source region, tracer rate and prescribed Y acceleration.
The interval controls can be observed at the sequence call boundary. They cannot
be inferred reliably from accepted fields or reconstructed from force-work
diagnostics.

## Existing output and provenance

The v1 frame has exactly `frame`, `time_s`, `dt_s`, `carrier_stamp`,
`liquid_stamp`, `fields` and `diagnostics`. Frame zero has zero dt and null
diagnostics. Velocity/tracer fields are native f32; fraction/pressure are native
f64. Pressure describes the last accepted interval projection, not an independently
evolved endpoint. IDs/versions are canonical decimal-u64 strings. Floats have
decimal spellings that round-trip their native types.

`tools/export_dense3d_sequence.py` constructs a fixed configuration manifest,
binds source commit/dirty flag, source hashes, binary hash, toolchain and exact
commands, independently validates output, and publishes completion last with
no overwrite. It currently runs one explicitly bounded pilot. The new work
must test the metadata without launching another pilot or held campaign.

`tools/import_dense3d_sequence.py` requires the exact v1 schema and key sets,
bounded files/lines, finite native values, hash/byte binding, stamp/time continuity
and pilot-specific semantic checks. Its old tests already provide authored
static fixtures. Adding fields under v1 would break the existing contract;
the follow-on must preserve old-reader behavior or explicitly refuse the new
version. The exact consumer input will determine the versioned representation.

Implementation provenance does not establish independent source-family identity.
The existing nine frames belong to one correlated pilot family. Naming frames
or intervals separately must not convert them into independent train/test groups.
Family grouping is an explicit provenance declaration with honest attribution;
it is not something the solver can infer from numerical fields.

## Disjoint patch sequence once the exact contract arrives

1. Preserve the provided contract bytes and immutable reference; compare its
   exact version, fields, types, units, continuity and unsupported-input rules
   against this source map. Record any unresolved conflicts before coding.
2. Implement the smallest explicit versioned export route around the existing
   accepted owner. Bind actual controls at the call boundary and actual accepted
   interval/stamps after successful publication. Frame zero has no interval.
   Preflight all metadata support/size checks before physics. Do not alter the
   physics solver or infer controls from outcomes.
3. Bind source-family declarations at manifest publication, retaining existing
   source/binary evidence and single-family limitations. Preserve old readers
   or require explicit version refusal; do not silently upgrade v1 data.
4. Use existing/static accepted fixtures for serialization, independent import
   and adversarial tests. Test requested versus accepted dt, native precision,
   changing interval controls, before/after stamps, constructor behavior,
   rejection/cancellation, unsupported controls, byte caps and post-commit output
   failure. Static authored fixtures remain labeled authored. No new campaign.
5. Independently review the exact frozen source and evidence before ordinary
   draft publication. Save durable checkpoints and a final source/evidence
   bundle. Parent schedules CodeRabbit; no merge, deployment or quality claim.

This plan intentionally chooses no consumer wire fields before the required
contract is readable. No simulation has been run for this follow-on.
