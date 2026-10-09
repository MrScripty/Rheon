# Bounded dense3D accepted-state sequence for Tuldok, v1

Base: public MrScripty/Rheon main 9cd4587a54befa61bdfddc8e35014bd3c34f02fb.
No physics, controller, scientific-law, frozen verifier, or comparison changes.

## Architecture proposed for independent review before implementation

A std-only Rust module writes one accepted borrowed LiquidTransportView at a time,
without copying field arrays or materializing a JSON frame. A bounded sequence
wrapper owns LiquidTransportSimulation and its Write sink, writes the constructor
frame at construction, and calls the existing step_with_box_flux. It writes only
after Ok(report). Any step error or output error poisons the wrapper permanently.
On I/O failure after commit, the accepted physics state remains advanced; no rollback
or rerun. Read-only state remains inspectable. Caller specifies cell/frame caps;
fixture is at most 512 cells and 9 frames. Per-field finite/shape checks precede
writing. No new Cargo dependencies.

A separate fixed fixture example (no comparison loop) creates a fresh directory,
uses create_new for frames.jsonl, and runs exactly eight requested 0.0625 s steps.
On success it flushes/syncs frames and prints one JSON metadata object to stdout.
A Python stdlib orchestrator builds/runs only this example with core-only features,
records git commit/dirty status, source and executable SHA256, Cargo.lock SHA256,
rustc/cargo version, and the exact command/config. It validates successful output,
hashes frames, writes/fsyncs a temporary manifest, and publishes run.json with a
no-overwrite hard link as the final action. No run.json exists on partial exports.
No resume, overwrite, automatic retry, dataset upload, or model activity.

## Proposed wire contract (shared with independent importer implementer)

run.json schema = "rheon.dense3d.accepted-sequence", version = 1,
complete = true, frame_count = 9, frames_file = "frames.jsonl",
frames_sha256 = lowercase SHA256, frames_bytes = byte count.
geometry = {counts:[16,8,4], origin_m:[0,0,0], spacing_m:[0.0625,0.125,0.25],
axis_order:["x","y","z"], flattening:"i+nx*(j+ny*k)",
cell_offset:[0.5,0.5,0.5], face_offsets:{x:[0,0.5,0.5],y:[0.5,0,0.5],z:[0.5,0.5,0]},
field_shapes:{velocity_x:[17,8,4],velocity_y:[16,9,4],velocity_z:[16,8,5],
tracer:[16,8,4],fraction:[16,8,4],pressure:[16,8,4]}}.
field_types = {velocity_x:"f32",velocity_y:"f32",velocity_z:"f32",tracer:"f32",fraction:"f64",pressure:"f64"}.
units = {velocity:"m/s",tracer:"dimensionless appearance",fraction:"dimensionless",pressure:"Pa",
time:"s",divergence:"1/s",pressure_residual:"m^3/s^2",volume:"m^3",mass:"kg"}.
pressure_semantics = "last accepted interval projection; constructor zero; not independently evolved endpoint".
config = {carrier_density_kg_m3:1000,represented_density_kg_m3:800,requested_dt_s:0.0625,
steps:8,pressure_implementation:"jacobi-pcg-v1",initial_fraction:0.5,
initial_slab_i:[4,8],initial_velocity_m_s:[0,0,0],outward_speed_m_s:[[-0.25,0.25],[0,0],[0,0]],
inlet_fraction:[[0,0],[0,0],[0,0]],carrier_id:"43",liquid_id:"41",initial_liquid_version:"0",
boundary_id:"47",boundary_version:"0",inlet_id:"53",inlet_version:"0",
memory_limit_bytes:16777216,workspace_memory_limit_bytes:16777216,
pressure_relative_residual:1e-12,pressure_absolute_residual:1e-12,pressure_divergence_limit:1e-9,
pressure_max_iterations:10000,actual_divergence_limit:1e-5,max_courant:1,max_outward_courant:1}.
provenance = {base_commit,source_commit,source_dirty,source_sha256:{relative-path:digest},
executable_sha256,toolchain:{rustc,cargo},command:[...],build_command:[...]}; limitations array explicitly
states fixed all-fluid constant-density carrier, represented fraction donor transport,
no free-surface pressure/reconstruction/two-phase inertia/material calibration,
no multidirectional accuracy/performance/training qualification, tracer appearance only.

Every JSONL line = {frame:0..8,time_s:actual accepted clock,dt_s:0 for constructor else actual dt,
carrier_stamp:{id:"43",version:"0".."8"},liquid_stamp:{id:"41",version:"0".."8"},
fields:{velocity_x:[...],velocity_y:[...],velocity_z:[...],tracer:[...],fraction:[...],pressure:[...]},
diagnostics:null for constructor else {pressure_iterations,pressure_residual_m3_s2,
carrier_divergence_s_inv,liquid_divergence_s_inv,volume_before_m3,volume_after_m3,
mass_after_kg,inward_m3,outward_m3,source_m3,balance_m3,rounding_budget_m3}}.
All stamps are canonical decimal u64 strings, all floating values finite and
round-trip to their declared native type. Use at least 9 significant decimal digits
for f32, 17 for f64. Constructor frame uses accepted zero velocities/tracer/pressure.
Frame pressure is held interval output, not independently evolved end state.

## Validation proposal

Rust tiny fixture tests compare all parsed frame fields against exact accepted view
bits, plus cancellation/invalid input/frame cap/nonfinite/shape/I/O failure controls.
Python importer checks bounded file sizes, strict JSON (duplicate keys/constants rejected),
complete manifest/hash, all axes/shapes/native-float finiteness, stamp continuity,
actual clock/dt and constructor. Independently recompute all-frame divergence, volume,
mass, every-cell existing binomial donor oracle (courant .25), first interval pressure,
and final centroid .5 m with frozen tolerances (fraction 2e-10, pressure 1e-5 Pa,
volume 1e-12 m3, mass 1e-9 kg, centroid 1e-10 m). Corruption tests mutate valid fixture
and rehash where needed: malformed/truncated/nonfinite/wrong axis/stamp/hash controls.
Only one actual 16x8x4 eight-step pilot. Do not execute all comparison cases.

## Independent review resolution before implementation

Frame/cell/byte caps are checked before advancing physics. Exhaustion poisons the
sequence without an additional accepted step. A failing write may leave an incomplete
last JSONL line; this is a partial export, rejected without run.json. Flush/sync and
manifest-finalization failures also prohibit completion, with no retries. The importer
bounds run.json (64 KiB), frames (2 MiB), each line (256 KiB), and exact fixture scalar
counts before accepting values; requires final newline and exactly nine frames.
Source hashes include every src/*.rs (recursively), exporter example, importer,
runner, Cargo.toml, Cargo.lock and rust-toolchain.toml. A clean local candidate commit
is recorded for the pilot; dirty contents are explicitly recorded when allowed.
The final manifest is validated before its atomic no-overwrite publication, and the
parent output directory is fsynced. Crash durability remains subject to filesystem
semantics; a published manifest can be rejected using hashes if files are modified.

Final implementation review clarified that the capped example uses an unbuffered
file sink, preventing BufWriter Drop from retrying output after a failure. Provenance
also records the exact core-only Cargo build command in build_command.
