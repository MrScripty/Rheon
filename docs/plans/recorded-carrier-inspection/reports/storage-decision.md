# Whole-publication storage decision, 2026-10-14

Decision owner: recording task. Retain schema-v1 SQLite indexed metadata and one raw
binary BLOB per field per publication for the admitted bounded CPU desktop slice.
No storage architecture, compression, quantization or precision change. Carrier raw
f32 faces/tracer and f64 interval pressure reuse the generic adapter contract.

Measured release binary, Rust 1.92, Linux 6.18.44 x86_64 glibc2.41, workspace overlay
filesystem; virtual/backing device durability and hardware not established. One
constructor plus 32 accepted synthetic publications per grid, four f32 face/cell
arrays plus one f64 cell-pressure array on accepted steps. Finite varying deterministic
mantissas, not a physics run or compression dataset. FULL synchronous/DELETE journal,
1 MiB SQLite cache target. Separate write/read processes; OS cache not purged, so
reads are fresh-process/warm-OS-cache evidence, not cold-device latency. Full selected
step hashing is included in selection; independently checking every raw bit is timed
outside selection. Opening quick_check and full audit are reported separately.

| Grid | Raw accepted bytes | DB 33 publications | Encoding median/p95 ms | Append median/p95 ms | Verified select median/p95 ms |
|---|---:|---:|---:|---:|---:|
|32³|798720|26214400|0.095/0.134|2.637/2.944|1.037/1.075|
|64³|6340608|207446016|2.103/2.333|13.299/17.017|7.963/8.629|

32³ open quick_check7.409 ms, audit39.156 ms;64³ open45.149 ms,audit305.340 ms.
Linux child rusage maximum RSS:write/read 32³8448/8448 KiB,64³14612/13460 KiB.
Includes process/library/SQLite/input/independent-check buffers; excludes simulation,
GUI, decoded selected carrier and caller-retained records. Child rusage may include
pre-exec inherited process high-water memory; do not treat this as hard RSS cap or
exact allocator attribution. Memory remains bounded by current whole-record and
SQLite limits; selected UI may hold both raw and decoded arrays. 32 samples are a
bounded host observation, no production throughput/real-time/performance acceptance.

A repeat after adding direct Linux `/proc/self/status` VmHWM resolves the inherited
child high-water ambiguity. File/payload bytes were identical. Observed:

|Grid|Encoding median/p95 ms|Append median/p95 ms|Verified select median/p95 ms|Binary VmHWM write/read KiB|
|---|---:|---:|---:|---:|
|32³|0.157/0.185|2.872/3.229|1.104/1.331|5432/5404|
|64³|2.111/4.277|15.184/30.337|8.539/13.091|14612/13328|

Repeat open/audit 32³ 7.728/39.889 ms;64³ 44.072/312.839 ms. Direct binary VmHWM is
measurable process resident high-water on this Linux host, not a portable hard limit.
32³ external child rusage stayed8448 KiB while binary VmHWM~5.3 MiB, confirming the
inherited pre-exec floor. Repeated latency variation reinforces the qualified host
observation; no fixed throughput or hard real-time budget follows. Repeat logs under
`.generated/recording-storage-probe/repeat`; source prints VmHWM when available.

Reproduce:build `cargo build --release --locked --no-default-features --example
recording_storage_probe`;run binary `write NEW.sqlite 32 32`, then separate process
`read NEW.sqlite 32 32`;repeat64. External Linux wait4/rusage measures process peak;
logs/databases under ignored `.generated/recording-storage-probe`. Every read compared
all raw field bytes against independent deterministic values. No numerical artifacts
committed. Source probe is retained for repeatability, never called qualification.

Alternatives:CSV is an explicit human export, unsuitable as canonical precision/
association/transaction storage. Per-cell SQL rows multiply row/index overhead and
are rejected. A bespoke data-file/SQLite-index pair adds two-resource atomic publication
and crash reconciliation without a current spatial-query requirement. HDF5 or a
chunked/compressed array container can serve larger spatial/time-series workloads,
but needs independently qualified chunking, durability, integrity/versioning and
compression bounds. No unmeasured speed/compression superiority is asserted.

Consequences:whole-step indexed reads and atomic step publication fit observed CPU
slice; individual field/spatial range/time-series reads remain unsupported. Main DB
limits exclude journals and filesystem overhead. Default1 GiB cannot retain1025 records
at 64³(~6.05 GiB raw), so keep-all/stop discloses quota failure/unrecorded tail; user may
increase capacity explicitly. 64 MiB payload cap permits128³(~48.19 MiB) arrays but
full solver/UI memory and disk limits still apply. No guarantee that a requested run
fits its configured disk cap. Compression/chunked reads trigger a separate design
and versioned consumer compatibility decision, never silent adaptation.
