# Native recording comparison, analysis v1

`src/recording/comparison.rs` is an additive read-only analysis module. It is
compiled through the isolated `recording_comparison_contract` integration test
until the integration owner admits its `recording::comparison` export. It owns
neither accepted publication, recording storage, runtime execution nor a worker.
The canonical implementation operation is M3 of
`docs/plans/configurable-solver-components/plan.md` at PR84 `f45cd91`, admitted
by the parent/user for these four new files. This does not accept A5's GUI or
model-runtime workflow claims.

## Source selection and identity

`Selection::read(reader, ordinal)` obtains one exact immutable owned publication
through the real `RecordingReader`, which verifies its stored hash and metadata.
`identity()` retains run ID, declared source revision, ordinal, generation and an
analysis-v1 SHA256 digest. `digest()` returns that digest; `run_digest()` binds
run metadata. The module-level `run_digest(reader.run())` also supports archives
with no selected publication. `run()` and `publication()` expose immutable
source facts.
Selections remain bounded by the reader's recording limits. A caller building
selection arrays owns the aggregate memory and count budget; alignment is
quadratic in those caller-selected counts and holds no additional field cache.

The run digest is SHA256 of domain bytes `rheon-native-run-v1\0`, run ID as
little-endian u64, source revision/solver/configuration/provenance as
length-prefixed UTF-8, counts as three LE u64 and spacing/origin as six LE f64.
Lengths use LE u64. The selection digest is SHA256 of domain bytes
`rheon-native-selection-v1\0`, the 32-byte run digest, ordinal/generation as LE
u64, time as LE f64, optional dt, stage byte (constructor=0, accepted=1), controls
text, field count and every field in recorded order. Fields encode
name/units/association/stage/scalar type text, optional evaluation time, dimensions
(three LE u64), components, byte length and raw bytes. An optional value encodes
presence byte 0/1 followed by LE f64 only when present. All text is prefixed by
its UTF-8 byte length. Exact native f32/f64 bits are retained, including signed
zero. These are analysis identities, independent of SQLite page layout, archive
limits, lifecycle, record count and subsequent append/finish. They are not file
hashes, authentication or proof of declared provenance.

Completed archive-file hashes and application run-set membership are separate
owners. The module does not silently select latest data, derive a completed
outcome from the last record, mutate archives or execute model code.

## Supported fields and compatibility

Only `rheon-carrier-full-box-v1` records with a recognized native pressure method
are supported. Unknown profiles return `UnsupportedProfile`; they are not
interpreted by guessing field names or parsing configuration/controls Debug
text. Native field schemas must match the existing carrier producer exactly:

| Role | Native support | Units/stage |
| --- | --- | --- |
| Tracer | all cell centers, f32 | 1; constructor or accepted endpoint |
| Velocity X/Y/Z | each full native face lattice, f32 | m/s; constructor or accepted endpoint |
| Interval pressure | all cell centers, f64 | Pa; accepted projection interval, cell0 gauge |
| Velocity divergence | all native cells from three stored face arrays | 1/s; same endpoint |

Compare requires identical counts and bit-identical spacing/origin, the same
publication stage, equal actual physical endpoint times within explicit finite
nonnegative absolute tolerance (seconds), and exact field units, association,
dimensions, scalar kind, components and stage. Pressure additionally requires
matching actual interval starts within tolerance and a valid stored interval
start/end/dt relation. Same ordinal or generation alone never aligns runs.
Native geometry and stage are authoritative; camera, quantization and display
pixels are absent from analysis. There is no spatial/time remapping,
interpolation, particle ID pairing, periodic-grid assumption or derived pressure
residual/Courant claim.

`align(reference, candidate, recipe)` matches physical time/stage one-to-one.
Multiple candidates within tolerance or reuse of one candidate for two
reference selections return `AmbiguousAlignment`; there is no nearest-match or
input-order tie breaking. The result binds analysis version 1 and the complete
recipe, and reports paired
identities plus unmatched identities on both sides. A temporal pair with
incompatible geometry or an absent/invalid field remains a paired unavailable
outcome. For a malformed/missing pressure interval field, authoritative
publication time−dt is used only to retain the temporal pair; field validation
still returns `FieldSchema`/`MissingField`. Valid actual unequal intervals remain
unmatched. Thus temporal coverage and usable numerical support are distinguishable.
All source values are validated finite f32/f64 through the recording owner;
nonfinite intermediate arithmetic returns `ArithmeticRange`.

## Reductions and gauge

Errors use the declared convention candidate − reference. Metrics include
Euclidean L2, native-volume-weighted RMSE, Linf, support count and relative L2. Uniform Cartesian cell/face
sample volumes have equal weights, so weighted RMSE is
`sqrt(sum(error²)/support_count)`. The full face lattice uses equal sample
weights; no half-volume boundary quadrature is claimed. L2 is the Euclidean
sample norm, not its volume integral. Relative L2 is absent precisely when the
reference norm is zero; absolute errors remain available. Empty support is
unavailable; no hidden epsilon or zero-valued missing result is used.

Divergence at (i,j,k) is the sum of native upper-minus-lower face velocities over
actual spacing on each axis. It uses stored f32 face values promoted to f64,
x-fast indexing, unequal counts and anisotropic spacing. This measures the
recorded velocity field; it does not infer residual from pressure or reproduce
numerical acceptance.

`PressurePolicy::Canonical` requires each selected pressure's actual first cell
to equal zero. `RemoveMeanOffsetDiagnostic` subtracts mean(candidate − reference)
from the candidate before reduction and common color bounds. Its report retains
the removed offset in Pa, each original canonical-gauge status and a diagnostic
label. It does not repair or promote a source publication; canonical-gauge
failure remains visible. Direct pressure metrics require matching canonical
gauge. Common color bounds are the finite union of both compared arrays after
any declared transform; constant fields legitimately produce min=max.

## Native presentation data

`compared_values(reference, candidate, recipe)` synchronously materializes a
single immutable selected pair. It returns `ComparedValues` with owned promoted
f64 reference/candidate arrays, the exact `PairComparison` summary and identities,
native `NativeGeometry` (counts XYZ, spacing/origin in metres), and `NativeSampling`.
The sampling descriptor contains association, actual scalar lattice dimensions,
and sample offsets in cells. Cartesian XYZ and x-fast ordering are preserved.
No arrays or descriptors are serialized by this API.

| Association | Sample dimensions | Offset in cells |
| --- | --- | --- |
| Cell center (tracer, pressure, divergence) | `[nx, ny, nz]` | `[0.5, 0.5, 0.5]` |
| Native X face | `[nx+1, ny, nz]` | `[0, 0.5, 0.5]` |
| Native Y face | `[nx, ny+1, nz]` | `[0.5, 0, 0.5]` |
| Native Z face | `[nx, ny, nz+1]` | `[0.5, 0.5, 0]` |

The physical sample point is `origin_m + (sample_index + offset_cells)*spacing_m`
componentwise. Face arrays are never cell-centered, averaged, interpolated or
remapped. A renderer that supports only cell centers must report its limitation
for a face association; it cannot relabel faces as cells or fabricate a Frame or
Scene. The metadata enables an honest native sample projection, not a new
renderer or physical-operator contract.

The function first calls the existing `compare`, then reuses the existing native
`values` extraction. For pressure it subtracts only the existing summary's exact
`GaugeReport.removed_offset_pa` from the candidate array. It does not recompute a
mean, reduction, divergence stencil or color scale through another algorithm.
Direct values preserve native f32-to-f64 promotion and raw f64 bits; divergence
continues to use the established stored-face calculation. Unavailable pairs
return `ComparedValuesError::Unavailable` with their original reason, and an
invalid recipe returns `ComparedValuesError::Recipe`. The persisted unavailable
variants, codec and analysis-version semantics are unchanged.

This deliberately repeats bounded extraction after validation to avoid changing
the established numerical reduction. Source recording limits bound one selected
record; they are not a peak RSS limit. Two source records, promoted arrays and
transient divergence/reduction buffers can coexist. There is no new field cache,
persistent numerical store or worker, and callers own returned-array lifetimes.

The cold page's `load_selected_fields` is an explicit synchronous consumer. It
returns no data and performs no source I/O without a current verified matched
pair. On invocation it reopens M2's exact terminal artifact pins, selects and
checks both publication identities, calls `compared_values` and checks its summary
against the currently verified pair. Missing/corrupt/changed sources clear old
verified presentation and refresh source availability snapshots. A valid but
unavailable field pair returns its typed unavailable reason while retaining its
verified analysis status. It never turns missing data into arrays or acceptance.
The existing `selected_pair` accessor remains cheap. A future linked renderer may
cache transient returned data by both exact pair identities and full recipe;
materialization is not a per-frame getter and does not itself register a view.

Pointwise tests independently decode real Jacobi/SGS SQLite field bytes and check
all six role arrays. Direct scalar outputs are compared bit-for-bit after IEEE
promotion; the independent coordinate-dictionary/`fsum` oracle bounds the final
three-term divergence summation as described below. Separate analytical fixtures
check signed zero/subnormal f32 values, declared pressure-offset transforms and
constant divergence. The existing metric oracle is unchanged. Mutation fixtures
retain the original typed stage/time/geometry/schema/gauge/arithmetic failures.
The cold consumer tests verify no data/source access while Unverified, actual
native X-face sampling after verification, and invalidation on missing or changed
whole-file sources; unavailable constructor pressure remains verified-but-absent.

## Evidence and limits

The Rust contracts use real `Simulation` captures, `carrier_publication`,
`StepRecorder` and cold `RecordingReader` paths. Some tests mutate captured
fields as labeled analytical fixtures; they do not claim those synthetic
fields passed physics acceptance. Native face fixtures establish constant
analytical divergence on a non-cubic anisotropic grid. Tracer fixtures establish
L2/RMSE/Linf and exact zero-denominator behavior. An independent Python standard
library oracle reads raw SQLite blobs, unpacks native f32/f64 bits, uses
coordinate dictionaries for face derivatives and `math.fsum` for reductions,
and independently encodes the selected-publication fingerprint. Its command is
invoked by the Rust test, and oracle failure fails that test. This proves the
sampled native analysis contracts, not comprehensive external conformance.
A separate unmodified native source test records three Jacobi and two SGS accepted
intervals with common geometry, dt and nonzero smoke forcing, closes the producers,
and cold-compares two matched intervals plus one unmatched Jacobi tail. Its first
pair checks exact identities and all six role metrics with the Python oracle.

For divergence only, the oracle accounts for the difference between Rust's two
sequential adds and Python's `math.fsum` of identical per-axis derivatives. The
per-cell absolute bound is `4*epsilon*sum(abs(derivative)) + 4*min_subnormal`.
It propagates the bound to L2 by a Euclidean norm, RMSE by division by sqrt(n),
and Linf by the maximum bound; candidate and reference bounds add for errors.
Relative L2 is checked against `[max(E−error_bound,0)/(N+reference_bound),
(E+error_bound)/(N−reference_bound)]`. If the reference norm interval reaches
zero, relative-oracle evidence is explicitly unavailable while absolute metrics
and exact hashes are still checked. All bounds and results must remain finite;
overflow cannot produce a vacuous pass. Other roles retain the fixed tight
reduction-rounding tolerance. This bounded oracle qualified the real native
pair's near-cancelled divergence without claiming bit-identical summation.

Accepted-v1 records contain no typed attempted-interval count, provider fallback
outcome, warmup receipt, timing sample/scope, RHS/operator or diagnostic-attempt
identity. Warmup/fallback/cost/first-rejection/attempted-horizon comparisons and
host true residual are therefore unavailable in this module. Opaque Debug
provenance is not parsed. Rejected diagnostic records cannot be read as accepted
publications here; supporting them requires the recording owner's separate
versioned typed path. Offset-aligned pressure metrics are explicitly diagnostic
analysis of selected accepted-v1 source bytes, never diagnostic-step acceptance.
No speed, accuracy, GUI, CPU model runtime or completed run-set workflow claim is
made by these tests.

Verification commands (Rust 1.92.0):

```sh
cargo test --locked --no-default-features --test recording_comparison_contract
cargo test --locked --test recording_comparison_contract
cargo clippy --locked --no-default-features --test recording_comparison_contract -- -D warnings
rustfmt --check --edition 2024 src/recording/comparison.rs tests/recording_comparison_contract.rs
```

Observed verification on Linux CPU with Rust 1.92.0 (2026-10-15): the focused
comparison contract passed 15 tests in both no-default and default feature
configurations; the Python oracle passed tracer, interval pressure, divergence
and all three native velocity roles. Existing no-default recording contracts
passed 15 tests and carrier contracts passed 4 tests. Focused Clippy with warnings
denied, Rust formatting and Python compilation passed. These observations cover
this isolated module and the stated storage regressions; shared export wiring,
run-set/desktop integration and native interaction remain integration-owner work.

Presentation-seam verification on Linux CPU, Rust 1.92.0 (2026-10-15): 18 native
comparison tests pass with no default features and with desktop enabled; 9 cold
page interaction/consumer tests pass with desktop enabled. Focused Clippy with
warnings denied and formatting pass. These tests establish native data custody
and the explicit materialization consumer, not linked rendering, shared desktop
registration or model/runtime acceptance.
