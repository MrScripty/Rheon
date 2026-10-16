# Domain density and independent preview resolution

Application controls offer physical sampling up to **1000 cells/m**, native
counts up to **10000 per axis**, and independent preview counts up to **10000
per axis**. Counts, spacing, extent and total allocation are separate quantities.
`GridGeometry::new(counts, spacing_m, origin_m)` retains its general explicit
spacing contract. `GridGeometry::at_cell_density` supplies cubic spacing
`1 / density` without allocating fields. The exported limits describe application
settings, not unlimited dense, sparse or out-of-core capacity.

| Surface | Geometry contract | Other limits |
| --- | --- | --- |
| Native CLI | `--size N` is cells/axis; default 64 in a 1 m cube. `--cells-per-metre D` sets spacing `1/D` and side `N/D` metres. | N 1–10000, D 1–1000; default 64 MiB retained native arrays; at most 10000 steps. Unit-box count mode checks actual density. |
| Desktop carrier | Separate count or density mode; fixed physical extent; density resolves counts with `ceil(side × D)`. | New realized density at most 1000/m; 10000 cells/axis; 192 MiB native arrays per track. |
| Desktop particles | Physical samples are independent of preview; density resolves particle samples. | Original 4–64 samples/side and particle/allocation/stability bounds. |
| Paired diagnostic Scene | XYZ counts and spacing `1/D`; extents are counts times spacing. | D 1–1000; 5–128 cells/axis, 65536 total cells, 512 steps, 96 MiB numeric history, fixed 1/60 s; model support is additional. |
| Preview | Legacy adaptive, exact density 1–1000/m, or exact edge 1–10000. | Exact realized density at most 1000/m; 16–4096 MiB history (default 96 MiB); every track, constructor and step counted. |
| Appearance | Scan and exposed surface mesh of stored preview. | At most 512³ voxel visits and 262144 faces; explicit unavailable result above either limit. |

Defaults remain unchanged. Covering rounding can increase realized density:
1000/m over 0.0015 m needs 2 cells and would resolve 1333.33/m, so it is refused
with actual/request context. It is never silently floored or clamped. Over
0.016 m it resolves 16 cells with 1 mm spacing. One floating-point ulp at the
cap accommodates decimal SI representation, such as 9 mm; no numerical
acceptance tolerance changes. Domain edits preserve the requested density and
recompute counts before launch. Configured carrier counts resolve independently
of the particle-mode switch.

Exact preview edge also checks physical density. The legacy adaptive mode keeps
its 512-axis ceiling and reports any budget reduction. Exact requests either fit
or fail before worker/output allocation. Finer carrier previews conservatively
replicate source appearance; overlapping nondivisible bins avoid holes and retain
narrow features. Preview never adds physical samples or changes numerical fields,
pressure calls, recording, CSV/VTK, training inputs or stored geometry.

Immutable native/configured specifications retain counts, spacing and digests,
including historical geometry outside the new edit cap. Cold opening and
supported exact rerun preserve that geometry rather than reinterpreting it
through view controls. Recorder schemas, Dense3d fixtures and model declarations
remain unchanged. Dynamic adapters must declare actual spacing, counts, material,
step and context. Legacy v2 remains limited to 128/m and refuses higher density
inertly, before a child starts. Selection does not qualify a model.

`rheon --preflight` reports checked cells, staggered faces, retained array payload,
selected budget and pressure cell-iteration budget without allocating fields or
outputs. Oversized payload returns nonzero. Comparison `--preflight` reports
geometry/history and, when a deployment is supplied, performs its inert support
check. Neither predicts solver success, elapsed time, RSS or model accuracy.
CLI completion schema 2 adds `cell_size_m` and `domain_side_m`. The existing
unit-box comparison qualifier accepts historical absent geometry; any new explicit
geometry declaration must be complete and match its unit-box workload before
residual conversion.

At 10000³ there are one trillion cells. One u8 preview alone needs one trillion
bytes (about 931 GiB), before other frames; native arrays need about 80 TB.
Geometry and payload planning allocate no cell fields. The axis setting is
representable while ordinary budgets refuse that cubic allocation. By contrast,
a 16³ grid at 1000/m occupies a 16 mm cube and has only 4096 cells.

The sidebar reports history payload, scan visits and worst-case six-face surface
payload, including two face buffers and two projected meshes. Rendering refuses
excessive scans/faces explicitly, preserving stored resolution. Allocations are
fallible. Allocator, GPU, egui, recording, thread and model memory are outside
those payload estimates. Native thumbnails have a separate 1 MiB pixel budget,
removing the old 128² thumbnail failure for an admitted 129³ carrier.

Pressure and stability gates retain their existing meaning. A forced smoke run
at 1000/m rejected with `Pressure(IncompatibleRhs)` after forcing stopped; its
accepted prefix remains evidence of failure. Continuously forced CLI acceptance
and zero-flow cold round trips have separate, bounded claims. The
[measured A1 blocker](../plans/configurable-solver-components/reports/measured-a1-owner-inputs.md)
remains unchanged. No training, Tuldok/surrogate-lab change, tolerance relaxation
or checkpoint replacement is included.

```sh
cargo run --locked --bin rheon -- --size 16 --cells-per-metre 1000 --preflight
cargo run --locked --bin rheon -- --size 10000 --cells-per-metre 1000 --preflight
cargo run --locked --features desktop --bin rheon-ai-compare -- --counts 16,16,16 --cells-per-metre 1000 --preflight
```
