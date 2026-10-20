# Force-aware accepted custody and archive metadata

This extends the core milestone without editing desktop/request definitions or
session/provider owners. Source review is independent; runtime tests below apply
only to the sealed pre-custody adapter binary. Current custody code has pure codec
coverage and static checks, not executed solver/SQLite/desktop integration.

## Accepted numerical publications

`Simulation::accepted_force_interval_pressure()` returns a separate
`AcceptedForceIntervalPressure<'_>` containing actual `values: &[f64]`,
`interval_start`, `report: StepReport`, `controls: CarrierStepControls`,
`fields: &ForceSnapshot`, and `work: Option<ForceReport>`.
The force step stamps it only after accepted commit, including for empty authored
documents. Any candidate attempt/reset/alternate publication invalidates both
MAC pressure stamps. `accepted_interval_pressure()` remains exclusively the
legacy view; its layout and legacy control formatting are unchanged.

`ParticleFluidSimulation::accepted_force_controls()` returns
`Option<&AcceptedParticleForceControls>` containing actual `interval_start`,
original `requested_dt`, `emitter`, accepted `report`, and `fields: ForceSnapshot`.
This stamp is assigned only after a successful authored step. Rejection retains
the previous accepted stamp and diagnostic arrays; a successful legacy call
clears it. Empty authored documents are retained for custody but skipped during
numerical evaluation, avoiding addition of +0 to signed-zero gravity.

Freeze still deep-copies draft data. ForceSnapshot now shares that immutable
owned document through Arc, so accepted custody clones do not duplicate names or
objects. Snapshot metadata remains outside numerical array budgets.

## Public recording API additions

All recording APIs remain native-only, as before. New functions are re-exported
from `rheon::recording`, and codec helpers also live in its `force_fields` module.

```rust
pub const FORCE_METADATA_VERSION: u32 = 1;
pub const MAX_FORCE_METADATA_BYTES: usize = 256 * 1024;

pub fn encode_force_metadata(fields: &ForceSnapshot)
    -> Result<Vec<u8>, RecordingError>;
pub fn decode_force_metadata(bytes: &[u8])
    -> Result<ForceSnapshot, RecordingError>;
pub fn recorded_force_fields(step: &RecordedStep)
    -> Result<Option<ForceSnapshot>, RecordingError>;
pub fn validate_run_force_fields(run: &RecordingRun, step: &RecordedStep)
    -> Result<(), RecordingError>;

impl RecordingRun {
    pub fn with_force_fields(self, fields: &ForceSnapshot)
        -> Result<Self, RecordingError>;
    pub fn force_fields_digest(&self)
        -> Result<Option<[u8; 32]>, RecordingError>;
}

pub fn carrier_publication_with_force_fields(
    sim: &Simulation, fields: &ForceSnapshot, ordinal: usize, limit: usize,
) -> Result<RecordedStep, RecordingError>;

pub fn material_publication_with_force_fields(
    sim: &ParticleFluidSimulation,
    report: Option<ParticleStepReport>,
    controls: Option<MaterialStepControls>,
    fields: &ForceSnapshot, ordinal: usize, limit: usize,
) -> Result<RecordedStep, RecordingError>;
```

`RecordedCarrier` and `RecordedMaterial` additionally expose
`pub force_fields: Option<ForceSnapshot>`. Absence denotes legacy metadata, not
proof that an arbitrary external producer applied no forces. RecordedMaterial's
standalone decode accepts a step without a run header; use RecordingReader or
explicit `validate_run_force_fields` for run association. RecordedCarrier::decode
has the run argument and checks it directly.

## Versioned representation and historical hashes

The SQLite envelope remains user_version=1. No existing run/step digest algorithm,
legacy solver identity, legacy numerical descriptor or legacy controls branch is
rewritten. Golden in-memory run/step fixtures verify the existing byte-level
hash contract. This does not claim that every historical archive has been tested.

Opt-in run configuration appends exactly a newline followed by:

```
external-force-metadata-v1-sha256=<64 lowercase hexadecimal digits>
```

The SHA-256 binds the complete versioned force document encoding. It is a
producer declaration, not authentication. Rebinding an already marked run is
rejected. The existing run digest includes configuration, binding this marker
without reinterpreting hashes of old records. Desktop FrozenSpec/intent schema
and digest migration are separate remaining owner work.

Every constructor/accepted publication for that bound run must carry the same
complete metadata field. Append checks association before starting a SQLite
transaction; selection checks it after existing publication digest verification.
Missing metadata, changed scenes or force fields attached to an unmarked run
are rejected. A worker cannot silently change a run's force intent between steps.

The field descriptor is:

```
name: external_force_scene_v1
units: 1
association: immutable_run_input; world_m; activation_at_interval_start_s
stage: external_force_document_v1
evaluation_time: None
scalar_type: u8
components: 1
dimensions: [byte_length,1,1]
```

Data has an 8-byte `RHEONFF\0` magic, u32 codec version, u32 ForceScene version,
and u32 object count, all little-endian. Each ordered object carries u64 ID,
u32 UTF-8 name byte count/name, enabled byte, 3 translation f64s, 4 quaternion
f64s, units tag, kind tag/payload, support tag/payload, start f64, end-present
byte and optional end f64. Floats preserve exact IEEE bits including signed zero.
Tags: units acceleration=0/density=1; kinds directional=0/radial=1/vortex=2;
support global=0/sphere=1/box=2. Kind payloads are vector3 or strength1; support
payloads are none, radius+edge, or half_extent3+edge. Boolean bytes are 0/1 only.

The codec rejects input above 256 KiB before parsing; checks 256 objects and
256-byte names before allocation; rejects malformed UTF-8, tags, booleans,
truncation/trailing bytes and invalid authoring documents; then freezes decoded
data. It is independent of optional Serde. The existing overall per-record
limits still apply to all fields, descriptors and controls together.

## Required host sequence (UI owner)

Keep the existing numerical and recorder owners:

1. Freeze the draft once and retain that snapshot with Request/history/run intent.
2. Build the existing carrier/material RecordingRun and call
   `.with_force_fields(&fields)` before `StepRecorder::create`.
3. Capture the constructor with the appropriate new
   `*_publication_with_force_fields` wrapper. This publishes the declared scene
   before a numerical step exists.
4. Step through the corresponding authored-force API. Use the same frozen scene.
5. Existing `carrier_publication` now recognizes the separate force pressure
   stamp. Existing `material_publication` recognizes actual force controls and
   checks supplied dt/emitter/report against those inputs before recording the
   actual snapshot. Alternatively use the new wrappers to also verify the host's
   expected scene. Append through the existing StepRecorder.
6. Load through the existing RecordingReader. Use typed decoded force_fields for
   historical glyphs. Rerun only from a properly versioned frozen run intent,
   not from this inspection archive or current drafts.

Old calls with old run metadata keep their old formatting/hash inputs. Empty
force workflows may keep using legacy calls. Do not bind a force run and then
omit its metadata, even for an empty document.

No src/bin files, MaterialSession, SimulationSession, configured-provider context,
free-surface/column/boundary owners, desktop FrozenSpec schema or request
structures are changed here. Those paths must not silently claim applied fields.
