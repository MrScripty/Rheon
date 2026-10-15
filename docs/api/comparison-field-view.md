# Linked native field view

`src/bin/desktop/comparison_field_view.rs` is a bounded application component for
inspecting one exact matched native publication pair. It uses the public
`recording::comparison::compared_values` contract and existing M2 run-set source
resolution. It creates no solver, numerical store, worker, persistence format,
`Frame`, or `Scene`.

## Calling contract

Construct `ComparisonFieldView::default()` once. After the saved comparison page
processes its controls, call `view.show(ui, page.selected_pair())` each frame.
`show` compares the root, embedded run-set digest, analysis version, member IDs,
recipe, and full selected pair with the cached key. A changed key, `None`
(including an unverified saved result), or unavailable pair clears both panes in
that frame. It does not search for a newer run or publication.

The **Load exact native pair** button calls `load(SelectedNativePair)` explicitly.
Applications may also invoke `load` on an explicit user action. This is synchronous
source verification and materialization, potentially including whole-archive I/O;
it must not run every paint. `SelectedNativePair` is caller-constructible, so its
name alone does not establish custody. Loading validates the saved recipe,
selected member/publication membership, terminal archive pins and exact selection
identities, then calls `compared_values` once. The resulting full comparison must
equal the supplied pair, including units, metrics, gauge report and common bounds.
Actual endpoint, dt, stage and pressure interval start are read from those same
two selections.

`clear`, `is_loaded`, `data`, `times`, `probe`, `set_slice`, and `set_probe` expose
the cached state and shared controls. `data` borrows the authoritative projection;
its arrays and summary are never changed by painting. Failed loads and invalid
slice/probe changes clear the view's previous arrays. Errors distinguish source
resolution, unavailable native comparisons, summary mismatch, invalid input,
limits, and unrepresentable canvas geometry. A load error clears this view only;
the owning comparison page separately controls its metrics and member summaries.

The cached data represents the **last explicitly verified and loaded** source
pair. There is no background file monitoring. Hiding or modifying a file after
loading does not cause paint to read it. A subsequent explicit load rechecks the
pins and identities and fails closed if the source changed or disappeared. Root
integration must process page controls before passing the current selection to
`show`; showing an old caller-owned selection cannot convey a newer page state.
Protected immutable source paths retain the existing M2 custody assumptions.

## Native samples and shared coordinates

Both panes share slice normal, integer slice index, integer probe, and exactly
one finite color range from the verified comparison. Clicking either canvas sets
the same native probe in both. Positive horizontal/vertical physical coordinates
point right/up; labels identify the current axes. The probe reports x-fast native
index, physical XYZ in metres, association, both exact promoted sample values and
source units. Each pane also displays these facts and its actual publication time.
Pressure displays the exact canonical/mean-offset diagnostic gauge report and
removed candidate offset; the view performs no mean calculation or gauge change.

| Field | Native sample dimensions | Offset from origin, in cell spacings |
| --- | --- | --- |
| Tracer, interval pressure, velocity divergence | `[nx, ny, nz]` | `[0.5, 0.5, 0.5]` |
| Velocity X | `[nx+1, ny, nz]` | `[0, 0.5, 0.5]` |
| Velocity Y | `[nx, ny+1, nz]` | `[0.5, 0, 0.5]` |
| Velocity Z | `[nx, ny, nz+1]` | `[0.5, 0.5, 0]` |

The view consumes the projection's association, dimensions, offsets and physical
geometry directly. A sample's coordinate is
`origin[d] + (index[d] + offset_cells[d]) * spacing[d]`. Display tiles are glyphs
for native sample locations, with pitch proportional to native physical spacing.
Their outer edges are not a claim about cell volumes or domain boundaries. This
also supports a one-sample axis without inventing a spatial interval between
samples. Native faces remain faces: no face averaging, remapping, interpolation,
cross-run particle pairing, zero substitution or alternate reduction occurs.

Finite values use the authoritative common bounds in both panes. A constant
range is explicitly labelled and painted neutral grey. Normalization handles a
finite minimum/maximum whose subtraction overflows; nonfinite inputs or an
unrepresentable panel/aspect ratio produce an explicit canvas error.

## Work limits

Before opening either SQLite source, loading checks both frozen native geometries,
selected role dimensions and terminal pinned publication counts. It checks both
actual archive file lengths before hashing or opening either reader.

| Limit | Bound |
| --- | --- |
| Samples in either selected scalar field | 1,048,576 |
| Two selected native publication payload estimates | 32 MiB |
| Paired promoted f64 arrays | 16 MiB |
| Pinned publications per source | 4,096 |
| Actual file size per source | 256 MiB |
| Native samples painted per pane/slice | 16,384 |
| Canvas side | 4,096 pixels |

The native payload estimate conservatively includes all staggered f32 faces,
f32 tracer and f64 interval pressure for each selected publication. Actual
projection array products, lengths, finite values, common bounds and physical
sample extents are checked again before retaining the cache. At least one slice
normal must fit the draw-work budget; choosing an oversized plane clears the
cache with an explicit limit error. Canvas work remains bounded for each paint.

These are selected payload, retained scalar-array, source-file and drawing-work
limits, **not an RSS or total I/O guarantee**. Existing reader verification may
visit the archive's other publications and temporarily retain native records;
`compared_values` performs its existing validated extraction and reduction work
before returning the paired arrays. No second numerical cache is persisted.

## Evidence and integration scope

`tests/comparison_field_view_contract.rs` path-includes the application component
against the actual public comparison API. It produces and closes native Jacobi
and symmetric Gauss-Seidel solver archives with `StepRecorder`, pins them through
M2, and computes real saved comparisons. Tests cover all six roles, exact native
samples and gauge transformation, association/physical coordinates, same-source
times, shared egui slice/probe input, single-sample axes, bounded thin-grid slices,
missing/changed sources, stale selections, forged summary units/geometry,
nonfinite bounds, preflight limits, and source-free cached painting. Separate
canvas tests check finite/constant/overflow normalization and geometry rejection.

This component does not register itself in the desktop root or modify an existing
renderer. Its independently tested source and documentation do not establish
whole-application GUI integration, model runtime, or scientific acceptance.
