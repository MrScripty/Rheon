# Accepted-publication recording, version 1

The native `rheon::recording` API retains numerical publications after their producer
ends. `StepRecorder` commits one `RecordedStep` with its fields in one SQLite
transaction. `RecordingReader::select(ordinal)` and `select_generation(generation)`
read the exact indexed publication. Missing or corrupt records return an error.
Neither API consults the current simulation or substitutes a newer publication.
`verify()` checks the entire committed sequence in one read transaction using one bounded step at a time. Pause the producer for long audits: its commits can otherwise reach the bounded SQLite busy timeout.
This is inspection/interchange data, **not a restartable integrator checkpoint**.

The integrated desktop producers are the existing material/SPH preview (including
accepted collision response) and the fixed sealed-box native carrier using either
Jacobi PCG v1 or symmetric Gauss-Seidel PCG v1. Coverage is the exact inventory below,
not all numerical fields in all Rheon solvers. Other session/lab/provider/surface/
volume/viscosity/browser producers are not connected. The generic named-field archive
also accepts native f32/f64/u64/u8 fields from independent hosts.

Canonical storage is **SQLite with indexed metadata and contiguous binary field
BLOBs, one row per field per step**; no per-cell SQL rows, compression or quantization.
CSV is an explicit sample/particle export. Selection loads and hashes a complete
bounded publication; efficient spatial subsets/field-only verified reads/time-series
scans are not implemented. Append is synchronous FULL/DELETE-journal I/O. See the
[measured storage decision](../plans/recorded-carrier-inspection/reports/storage-decision.md)
for32³/64³ file size, latency, process RSS and host/cache qualification.

## Material field authority

`material_publication` reads `ParticleFluidSimulation`'s canonical accepted slices;
`material_snapshot_publication` reads the existing immutable `MaterialSnapshot`.
The adapters never recompute density, pressure, acceleration or particle state.

| Field | Representation / units | Association / timing |
|---|---|---|
| position | binary64 XYZ, m | append-only particle index within run; accepted publication |
| velocity | binary64 XYZ, m/s | same index; accepted publication |
| mass | binary64, kg | same index; accepted publication |
| temperature | binary64, K | same index; accepted publication |
| material | u8, 0 emitted / 1 surrounding | same index; accepted publication |
| density | binary64, kg/m³ | same index; post-emission/pre-integration evaluation |
| pressure | binary64, Pa | same evaluation; liquid positive compression / gas absolute EOS; no common gauge |
| acceleration | binary64 XYZ, m/s² | same evaluation; gravity/pair forces; excludes subsequent wall response |

`RecordedStep.time` is simulated publication time, `dt` is the actual accepted
increment, and each diagnostic field stores its earlier simulated evaluation time.
These are not wall-clock timestamps. Constructor generation zero has no `dt` or
diagnostic fields. The canonical material decoder rejects constructor diagnostics and accepted material
publications missing any diagnostic; generic named-field records may remain sparse.
Missing fields remain absent. Evaluation positions, individual
pair forces, wall impulses, solver scratch and complete emission/integrator state
are not retained. Inspector force is explicitly **derived** mass × acceleration;
its totals are derived raw totals, not conservation residuals.

The archive binds run ID, caller-declared source revision, solver/API/control
version identity, actual grid counts/origin/spacing, opaque exact Rust Debug
configuration/control/report text and caller-declared build/provenance text.
Rust's finite float Debug representation preserves numeric values, but this text is
opaque provenance, not a stable executable configuration codec. Optional collision
geometry has a content SHA256 of vertex bits and triangle indices, policy and
counts; mesh source bytes are not embedded. Inspection and existing VTK export need
no mesh lookup. Desktop source/build/toolchain identities are explicitly unspecified
unless supplied by an external host; process-local run ID is not globally unique.
No reproducibility or authenticity claim follows from a declared revision/hash.

## Fixed-box carrier field authority

`carrier_run` binds actual Simulation config/grid/method; `carrier_publication` borrows
accepted state and a success-stamped interval pressure view. Admitted methods are
`Simulation::step` (optional SmokeSource) and `step_with_emitter`, with both current
PressureImplementation variants. Actual requested dt, source/emitter inputs, accepted
report and actual previous-time bits are copied only after acceptance. No array or
physics algorithm/allocation inventory changes; the extra stamp is small metadata.

| Field | Native bits / units | Association / stage |
|---|---|---|
|velocity_x|f32, m/s|(Nx+1,Ny,Nz), offset(0,½,½), x-fast; constructor/accepted|
|velocity_y|f32, m/s|(Nx,Ny+1,Nz), offset(½,0,½); constructor/accepted|
|velocity_z|f32, m/s|(Nx,Ny,Nz+1), offset(½,½,0); constructor/accepted|
|tracer|f32, dimensionless1|(Nx,Ny,Nz), offset(½,½,½); appearance concentration[0,1]|
|interval_pressure|f64, Pa|same cell centers; accepted projection interval; cell0 gauge=0|

Lab Cartesian XYZ positions are origin+(indices+offset)×spacing in m. Constructor
has exactly 4 fields and no pressure. Accepted has exactly 5, with publication time/dt;
pressure evaluation_time means **actual interval start**, interval end is publication
time, and stage explicitly identifies interval projection pressure. It is not endpoint,
absolute/free-surface pressure or temperature/buoyancy. Actual controls/report Debug
text remains opaque provenance. The decoder binds all dimensions to recorded geometry,
checks inventory/types/units/associations/stages/tracer range/pressure gauge/interval.

Any later prepare/reset/commit attempt invalidates pressure inspection, including a
rejected/cancelled attempt which preserves accepted velocity/tracer. Capture immediately
after an admitted success, before another attempt. Generation>0 without a success
stamp is refused, never emitted as partial canonical carrier state. Other Simulation
APIs do not receive this stamp. Reset increments generation while zeroing time; create
a fresh Simulation and run instead of treating reset as constructor zero. Rejected
candidates add no recording; gaps/duplicate acceptance remain refused by StepRecorder.
Pressure residuals/Krylov scratch, masks/fraction, other domains/backends, complete
emitter/integrator state and restart capability are not retained.

## Desktop live and cold selection

Every accepted material/carrier step commits before its matching appearance preview.
Carrier comparisons use independent archives/readers/identities, caps **per track**;
changing active track does not overwrite another track's selected data. Settings expose
directory/database/record caps and keep-all/stop policy for both modes. 64³ full-pressure
1025 records require about6.05 GiB raw and do not fit the default 1 GiB database cap.
A configured run may stop at capacity; no automatic downsampling, eviction or loss
of numerical precision is used to fit it.

Open numerical archive→local SQLite path→Open recorded run is available when no
worker is active. Uses the displayed recording import caps (maximum 10001 publications),
starts at recorded ordinal 0, and supports previous/next/slider selection, raw named-field
sample inspection/CSV, canonical material VTP or carrier VTI export, and explicit full
integrity audit. Unknown generic solvers expose their actual named fields/sample CSV;
no canonical material/carrier export is fabricated. Field selection persists **by name**;
absent fields/indices stay unavailable. Typed scalar values use round-trip Display;
CSV includes exact selected generation/publication/evaluation times, units, association,
stage/type/dimensions/components. Particle IDs remain run-scoped append-only indices.

Cold opening uses archive geometry/provenance and no live simulation. It provides
numerical inspection, not a reconstructed appearance scene/emitter/collision mesh.
Back to simulation explicitly returns to the retained live timeline. Failed open,
explicit selection, count/status read or audit clears raw/decoded caches and export
controls; same-ordinal explicit selection revalidates. Repainting alone reuses the
owned previously verified publication until selection/reload/audit changes it; it does
not continuously poll externally modified bytes. Opening quick_check and full audit
are synchronous; audit may block a producer, so pause it first. Native-window graphics
remain unqualified in the display-less environment; headless egui controls are tested.

## Publication, failure and retention

Use one `StepRecorder` on a fresh caller-owned local path. Existing files are refused.
The caller must protect the path and its ancestors from concurrent replacement.
Untrusted/shared directory mutation and network filesystems are outside this slice.
No symlink archive import, writer resume, multiwriter operation, migration, eviction,
background writer, automatic expiry or deletion is supported. SQLite's local
filesystem locking/flush assumptions apply. The adapter uses DELETE rollback
journals and FULL synchronous commits; it never deletes a hot journal as recovery.
See [SQLite atomic commit](https://www.sqlite.org/atomiccommit.html).

Append only after numerical acceptance. Rejected steps, stability retries and
cancelled candidates do not acquire an ordinal. Adjacent committed generations and
actual increments are checked; gaps are rejected. The first record may be constructor
zero or accepted generation one; beginning after earlier accepted steps is refused.
The desktop commits each accepted step before publishing its matching visual preview.
Cancellation during physics publishes nothing; cancellation after acceptance allows
that step's synchronous recording to finish before terminal cancellation. Recording
is not preemptible and has no hard realtime promise.

A recording error stops the worker. It cannot undo an already accepted numerical
step. The terminal status reports the last accepted generation and the recorded
prefix, including a possible unrecorded accepted tail. Append errors poison further
append admission so an uncertain commit is never retried on guessed state. `recorded_count()` is acknowledged commits, not proof of a failed append’s outcome. A fresh
reader determines which transactions committed. A terminal attempt closes append admission before reconciliation or writing and
cannot be retried on the same writer. Terminal-status failure has an uncertain
outcome, is itself reported, and requires fresh-reader reconciliation; an `active`
status on that reader means unknown/incomplete. `active` on cold reopen means incomplete:
a crash may have occurred between numerical acceptance and recording. Do not infer
that every accepted step was recorded merely because the last indexed step is valid.
No destructor marks a dropped run complete. SQLite rolls back an interrupted partial
transaction on reopen; committed-prefix integrity is checked independently.

`finish("completed", ..., Some(last_generation))` requires the supplied accepted
tail to match the writer's acknowledged committed tail and requires an unpoisoned
writer. `failed` and `cancelled` may declare an unrecorded tail, but every terminal
declaration must cover the durable recorded generation; a lower or absent declared
tail is rejected when records exist and is independently rejected by full audit. An explicit successful finish and complete `verify()` are needed
for a completed-archive claim. Error classification preserves I/O/SQLite causes,
invalid input, unsupported version/type, limit, missing selection and integrity.
SHA256 detects accidental metadata/data modification; it does not authenticate an
archive against an author who can replace both values and hashes.

## Resource settings

`RecordingLimits` stores `max_database_bytes`, `max_record_bytes`, `max_records`.
Retention is `keep_all_stop`: committed steps remain until explicitly removed by
the owner. Record quota includes the constructor when supplied. The desktop exposes
directory, database cap and record cap; its record quota is requested steps + one.
Reset/new run uses a fresh filename and preserves earlier files on disk.

The database cap is floored to 4096-byte pages and enforced using SQLite
[`max_page_count`](https://www.sqlite.org/pragma.html#pragma_max_page_count).
At least 64 KiB is required. FULL returns an error and rolls back the step. The
main database cap excludes the active rollback journal, filesystem metadata and
page/sector overhead. A transaction may temporarily require roughly one additional
database's worth of journal space; budget approximately twice the database cap plus
filesystem overhead. This is an explicit database-size limit, not a hard total-volume
quota. There is no unbounded WAL, history queue or frame index in memory.

The per-record limit caps field blobs plus field/control text, checked before
material encoding; the material adapter conservatively includes 2048 descriptor
bytes. It accepts at most 64 MiB per record, 32 fields, 16 components per field and
16 KiB per metadata text. Selected readers preflight summed blob/text size before
allocating field data; SQLite also has a row-length limit (record cap + 64 KiB
metadata margin), a 1 MiB page-cache target per connection, no mmap and file-backed
temporary storage. Cache targets are not hard allocator caps. One encoded record, one raw and one decoded selected publication per supported
comparison track (desktop admits at most two distinct pressure methods), the retained
cold inspector, bounded glyph cache, SQLite copies, allocator/connection overhead and
the live simulation may coexist. Returning from cold inspection may retain its selected
raw/decoded publication until another archive replaces it. Payload limits are per
publication/connection, not the sum of these bounded caches. Caller-owned
archives/Vecs are outside this limit. Memory does not grow with recorded step count
unless the caller deliberately retains returned steps. No whole-process/RSS,
performance, power-loss-on-faulty-storage or hostile-file parsing cap is claimed.

## Runnable CPU example and separate-process reload

Use the pinned Rust 1.92 toolchain, a fresh owned directory, and actual source identity:

```sh
mkdir -p .generated/recording-demo
cargo run --locked --no-default-features --example recorded_material -- \
  record .generated/recording-demo/run.sqlite SOURCE_REVISION
cargo run --locked --no-default-features --example recorded_material -- \
  inspect .generated/recording-demo/run.sqlite 1 .generated/recording-demo/step-1.vtp
```

The record process writes constructor + five accepted CPU steps. The inspect process
verifies and selects ordinal one after the producer has exited, then calls the
existing `write_particles_vtp` contract. `.vtp` bytes and CSV in the inspector use
selected publication/evaluation times. CSV includes run ID and particle ID; an ID
absent at an older step stays absent rather than selecting a different particle. Desktop selection does not change when the
worker records another step; Follow live explicitly selects the matching latest
preview. The cold desktop view and public API/examples provide indexed reload and
selected export after the producer exits.

## Generic SQLite consumer contract

Version is `PRAGMA user_version = 1`; unknown versions/types/stages are rejected.
No old archive version exists and no migration/deletion fallback is supplied.
The schema is owned by `src/recording.rs`; all authoritative tables are:

- `run`: sole id=1 header, actual provenance/geometry, limits, retention and digest.
- `lifecycle`: sole id=1, active/completed/cancelled/failed, detail, optional accepted
  generation and digest. It is independent of step atomicity.
- `steps`: indexed integer ordinal, unique generation BLOB, time/dt BLOB, stage
  (0 constructor / 1 accepted), controls, field count, payload bytes and digest.
- `fields`: (ordinal, field_index) primary key, unique names per step, units,
  association, stage, optional evaluation-time BLOB, dimensions, components,
  scalar_type and data BLOB. No omitted-field placeholder rows.

U64 counters and f64 times are eight little-endian bytes, preserving full u64
range and IEEE754 bits. Geometry is three u64 counts + three f64 spacings + three
f64 origins (72 bytes). Dimensions are three LE u64 values (24 bytes). Field data
is LE IEEE754 f64/f32 or LE u64/u8; tuples are x-fast and components are interleaved.
Dimensions × components × scalar width must equal blob length with checked
arithmetic. Zero tuples are supported. Floating fields must be finite. Association
strings identify sampling layout; generic producers must supply physical stage,
units and associations rather than deriving them from names.

Hash framing is LE u64 byte-length prefixes for every text and field-data length,
explicit presence u8 + LE bits for optional times. Run SHA256 covers run_id,
revision/solver/configuration/provenance in that order, geometry, three limits and
retention. Step SHA256 covers run digest, ordinal, generation, time, optional dt,
u8 stage, controls, field count, then each field in index order: name/units/
association/stage/scalar_type, optional evaluation time, dimensions, components,
blob length and blob. Lifecycle SHA256 covers run digest, outcome/detail, optional
generation presence + LE u64. Integrity is always checked on Rust selection;
external consumers must implement these checks or call the Rust reader first.

A direct independent consumer can enumerate names and inspect exact bits without
Rheon-specific rendering code (this small snippet assumes a previously verified file):

```python
import sqlite3, struct
with sqlite3.connect(".generated/recording-demo/run.sqlite") as db:
    assert db.execute("PRAGMA user_version").fetchone()[0] == 1
    time_bits = db.execute("SELECT time FROM steps WHERE ordinal=1").fetchone()[0]
    print("selected publication seconds", struct.unpack("<d", time_bits)[0])
    for name, kind, units, association, data in db.execute(
        "SELECT name,scalar_type,units,association,data FROM fields "
        "WHERE ordinal=1 ORDER BY field_index"
    ):
        print(name, kind, units, association, len(data))
```

The legacy `MaterialSnapshot`, session dense3D and VTK formats remain independently
versioned. This archive is not a completed dense3D dataset or a replacement VTK format.

The small carrier CPU example records a non-cubic grid, constructor plus three
accepted smoke-source steps. In separate processes:

```sh
cargo run --locked --no-default-features --example recorded_carrier -- \
  record NEW.sqlite SOURCE_REVISION
cargo run --locked --no-default-features --example recorded_carrier -- \
  inspect RUN.sqlite 1 NEW.vti
```

Every named field is inventoried, the full committed prefix is audited, ordinal 1 is
selected exactly, and carrier VTI retains native face/tracer/pressure precision and
interval associations without inventing represented volume.


## Archive-kind isolation

Native accepted version1 archives retain SQLite application_id0. Accepted readers
reject any nonzero application_id as unsupported archive kind before interpreting
accepted tables, and recheck this on later operations. Existing native writer
schema/publication semantics are unchanged. Typed diagnostic attempts use separate
RHDP application identity/tables and [diagnostic-recording.md](diagnostic-recording.md);
that reader cannot promote requests, candidates or host observations to accepted
publications. Archive hashes/kind markers prove structural consistency, not source
authenticity or scientific correctness. Cold diagnostic reads execute no model.
