# Tuldok dense3D accepted-state import contract, version 1

This local candidate adds a bounded exporter around the existing
`LiquidTransportSimulation`; numerical algorithms and scientific laws are unchanged.
The reviewed design and its lifecycle decisions are in
[dense3d-sequence-design.md](dense3d-sequence-design.md) and
[dense3d-sequence-review.md](dense3d-sequence-review.md).

## Produce and validate one pilot

With Rust/Cargo 1.92.0 on PATH, from the candidate checkout:

```sh
python3 tools/export_dense3d_sequence.py /absolute/path/to/FRESH_DIRECTORY
python3 tools/import_dense3d_sequence.py /absolute/path/to/FRESH_DIRECTORY
```

The producer builds only `dense3d_sequence` with `--locked --no-default-features`,
runs exactly one 16x8x4 case with eight requested 0.0625 s steps, and independently
validates it before publishing `run.json`. Python uses only the standard library;
Cargo dependencies are unchanged. No comparison campaign, uploads or training occur.
The build requests Cargo compiler-artifact JSON and selects the executable reported
for the exact local Rheon example/package and non-test dev build, including configured
targets. It never derives an executable from a target-directory layout. The recorded
build command includes `--message-format=json-render-diagnostics`; version 1 import
validation also accepts the original command, preserving existing pilot manifests.
A direct Rust example invocation writes frames but does not complete a dataset.
Output requires a new directory with an existing parent. Existing directories fail
before build/run. There is no overwrite, resume, automatic rerun or rollback.

## Completion and provenance

Import only directories containing a valid `run.json` with schema
`rheon.dense3d.accepted-sequence`, version `1`, `complete: true` and nine frames.
`frames_file` is exactly `frames.jsonl`. Verify its SHA256 and byte count before
parsing any frames. Missing, truncated, malformed or inconsistent completion
metadata rejects the sequence. A partial final JSONL line is possible on I/O failure;
absence of a completion manifest makes the entire run unusable for training.

The manifest is validated, written to a temporary file, fsynced, linked without
overwrite to `run.json`, then the directory is fsynced. Publication failures abort;
ordinary failure cleanup removes any completion name created by this attempt.
Crash durability depends on the filesystem. Import verifies file content even when
a manifest is present; hashes detect later modification, not source authenticity.

`provenance` records the requested public base commit, actual candidate source
commit, dirty status, SHA256 of every Rust source plus exporter, importer,
Cargo.toml/lock and rust-toolchain.toml, executable SHA256, rustc/cargo versions,
exact build command and actual executable invocation. A dirty source tree is
explicitly recorded; its content hashes must be retained with the candidate.

## Geometry, fields and interval meaning

Counts are `[16,8,4]`, box origin `[0,0,0]` m, spacing
`[0.0625,0.125,0.25]` m. The box is one metre along every axis.
Coordinate axes are named X/Y/Z; every field is flattened X fastest:
`i + nx*(j + ny*k)`, using each field's own shape. For a NumPy consumer, reshape
flat arrays to `(nz,ny,nx)` in C order; transpose only deliberately.
World positions are origin plus spacing times index plus the stated offset.

| Field | Shape `[nx,ny,nz]` | Offset in cell spacings | Native type | Units |
|---|---|---|---|---|
| velocity_x | `[17,8,4]` | `[0,0.5,0.5]` | f32 | m/s |
| velocity_y | `[16,9,4]` | `[0.5,0,0.5]` | f32 | m/s |
| velocity_z | `[16,8,5]` | `[0.5,0.5,0]` | f32 | m/s |
| tracer | `[16,8,4]` | `[0.5,0.5,0.5]` | f32 | dimensionless appearance |
| fraction | `[16,8,4]` | `[0.5,0.5,0.5]` | f64 | dimensionless represented occupancy |
| pressure | `[16,8,4]` | `[0.5,0.5,0.5]` | f64 | Pa |

JSON numbers preserve finite native float round trips. Parse velocity/tracer to f32
and fraction/pressure to f64; a decimal f32 need not equal the f64 widening of that
f32. IDs and versions are canonical decimal **strings** within the unsigned 64-bit
range. Never convert stamps through floating point.

Each line has exactly `frame`, `time_s`, `dt_s`, `carrier_stamp`, `liquid_stamp`,
`fields`, `diagnostics`. Frame 0 is the constructor state at time/dt zero, with null
diagnostics. Frames 1..8 are emitted once, only after each successful accepted
publication. Clocks and dt come from accepted state/report, rather than requested
values; this fixture accepts 0.0625 s and reaches 0.5 s. Carrier stamp id `43` and
liquid stamp id `41` have versions `"0"` through `"8"`.

Pressure is the **last accepted interval projection**, using that interval's
pressure problem. Constructor pressure is zero. It is held with the accepted view;
it is not an independently evolved endpoint field. Tracer is appearance guidance,
not the represented liquid fraction. Pressure residual uses m³/s²; divergence uses
1/s; volume and mass use m³ and kg.

## Fixed scenario and bounded acceptance

Carrier density is 1000 kg/m³; represented density is independently 800 kg/m³.
Initial velocity/tracer are zero. Initial fraction is 0.5 at `i=4..7` across all
J/K cells and zero elsewhere. Prescribed outward X boundary speeds are -0.25/+0.25
m/s (a +X through-flow); Y/Z speeds and inlet fraction are zero. No sources or forces
are applied. Pressure uses `jacobi-pcg-v1` and the existing liquid_step tolerances,
recorded fully in `config`. Owner/workspace limits are each 16 MiB. The exporter caps
512 cells, nine frames and 2 MiB encoded output; importer caps the manifest at 64 KiB,
the frames file at 2 MiB and each line at 256 KiB.

Cancellation, failed steps and cap exhaustion abort export before emitting a new
accepted frame. Caps are checked before an additional physics step. If output fails
after physics acceptance, physics remains advanced and export is permanently
poisoned; neither rollback nor rerun is attempted. Flush/sync failures also prevent
completion. Generic Write sinks must avoid their own retry-on-drop behavior; the
pilot uses an unbuffered file.

Independent validation checks all nine states: exact dimensions/stamps/clocks,
finite native fields, stored MAC divergence, represented volume/mass, carry-forward
volume ledger and every-cell binomial donor oracle from the frozen liquid-step
verification formula. It checks the first projection pressure and final centroid.
Existing tolerances are fraction 2e-10, first pressure 1e-5 Pa, volume 1e-12 m³,
mass 1e-9 kg, centroid 1e-10 m and actual divergence 1e-5 s⁻¹.

The final target is volume 0.125 m³, represented mass 100 kg and centroid X=0.5 m.
This is X-directed donor transport in a dense 3D box with a fixed all-fluid carrier.
There is no free-surface pressure, reconstructed surface, two-phase inertia,
material calibration, multidirectional accuracy, performance or training-quality
qualification. Import completion establishes this bounded data contract only.
