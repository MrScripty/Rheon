# Native recording comparison, analysis v1

`src/recording/comparison.rs` is an additive read-only analysis module. It is
exported as `rheon::recording::comparison`; integration tests use that actual
public API. It owns
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

Recognized `rheon-carrier-full-box-v1` native methods and the explicit
`rheon-configured-session-v1` profile are supported. Configured selections must
pass the full typed `RecordedConfiguredSession::decode`, including RCP3
association; see [configured comparison](configured-recording-comparison.md).
Unknown profiles return `UnsupportedProfile`; they are not interpreted by
guessing field names or parsing configuration/controls Debug text. Native field
schemas must match the existing carrier producer exactly:

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
outcome. For a malformed/missing legacy native pressure interval field,
authoritative publication time−dt is used only to retain the temporal pair;
field validation still returns `FieldSchema`/`MissingField`. Configured intervals
use the stored owner-supplied start only; a missing start is not reconstructed.
Valid actual unequal intervals remain unmatched. Thus temporal coverage and
usable numerical support are distinguishable.
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

The public `native_samples(selection, role)` inspection hook returns untransformed
x-fast native samples after profile/schema and finiteness checks. Dimensions are
run counts, adding one along the selected native face axis; divergence uses the
stored staggered faces. It does not align sources or apply pressure gauge offsets.
Callers own aggregate memory and rendering budgets. Finite input faces can overflow
derived divergence on valid extreme spacing; this returns ArithmeticRange rather
than exposing nonfinite values to a renderer. A synthetic native-bit fixture
checks this separately from physical qualification.

The independent Python verifier now uses explicit checked failures throughout;
identity and metric negatives fail in normal Python, `-O` and PYTHONOPTIMIZE=1.
Fresh isolated-target source tests passed 16 comparison contracts, including the
public sample/schema/overflow boundaries, and 12 saved comparison contracts.


### Exact native presentation values

`compared_values(reference, candidate, recipe)` first obtains the existing exact
PairComparison. Available output carries unaveraged x-fast native samples, typed
cell/face sampling dimensions/offsets, physical geometry and the exact comparison.
Only its explicitly declared pressure offset is applied to candidate presentation
values; raw native_samples remains untransformed. Derived nonfinite values return
ArithmeticRange. This is transient derived presentation data, not accepted fields
or another numerical store. Caller recording/selection budgets apply; promoted
arrays and transient extraction do not constitute a process RSS bound.

The cached page's explicit `load_selected_fields` reopens both exact archive pins
and selected identities, then requires the returned comparison equal its verified
pair. Unverified/no selected pair returns None without archive reads. Source failure
clears verification; scientific unavailability can retain honest verified analysis
without arrays. Linked views must clear prior arrays on every Err/None and bind
both selection identities plus the full recipe when caching. Native views/physical
qualification remain separate acceptance gates. Twenty numerical and nine page
contracts passed after serial selection; fresh source qualification is separate.
