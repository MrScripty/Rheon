# Independent one-step force Worker results review

## Disposition

Accept the two recorded outcomes as narrowly scoped real-Worker dispatch/effect/custody and cold-readback evidence for the sealed cases below. Both were separately authorized and executed once serially by the owner; each raw log records one exact pass. No retry, tuning or additional workload is supplied or requested by this review. The production force gate remains closed. Native interaction and broader runtime/scientific qualification remain incomplete.

The reviewer read logs, receipts, source assertions and manifests, and hashed artifact bytes. No executable/test was run or rerun; no SQLite database was opened or queried; no numerical/model/native workload or artifact cleanup occurred. Only this report was written after the runtime clean-tree preflight had completed.

## Custody and independent checks

Execution revision: `3b3fccea3657d711897130643a478adb48b5edd0`. The preparation source manifest SHA-256 is `603a0edf75b255b6ba217aaa6e72632a5e3ab16eb8997fe0013fa7a1175fbafc`; all 278 entries were independently rehashed against the current source. Test module SHA-256 remains `da753230b81df85227ab348c39ea4ac97e2ff093cf2da09ff9ac5dce225d0584`. The complete helper/assertion closure was reviewed before execution and re-read against the recorded results.

The preserved executable `/workspace/scratch/rheon-ui-force-worker-execution-20261021/sealed-runtime-tests-3b3fcce.bin` is 480,456,920 bytes and independently hashes to `af2da1539521e10407ac73b11cd58494118aa93a3f99ed529b11697ed85727b5`, matching both preflight/disposition receipts. Each captured supervisor copy matches the separately reviewed `b7f13a3bdee65a6d4345ddbc0a0eaa7ddef7d64e4224633d63e16f8db1046444` script. Its post-reap signaling defect was repaired before execution; no signal action occurred in either case.

The post-exit index `/workspace/scratch/rheon-ui-force-worker-execution-20261021/post-exit-artifact-index.json`, SHA-256 `4b9dc8a8188671cb13ce67a1585a589f012ae8cf7e92a7918b067d59caea8a85`, was independently checked against all fourteen indexed DATA/capture artifacts by byte size and SHA-256. Both raw logs and their receipts agree on exact name, one pass/zero failures/200 filtered out, pinned source/binary/supervisor, exit zero, no timeout, no signals and no group/PID survivor observation. The two recorded child PIDs were absent on independent readback. This is not a guarantee against PID reuse or a general process-supervisor certification.

The preparation manifest's earlier `new_test_bodies_executed=false` describes its historical compile/list checkpoint. These separate execution receipts now supply the two authorized results without rewriting that historical evidence.

## Recorded cases and resources

| Case | Exact ignored test suffix | Supervisor elapsed, including teardown | Observed child CPU, user+system | Final DATA | Captured combined log |
| --- | --- | ---: | ---: | ---: | ---: |
| Native | `native_force_dispatch_records_and_cold_binds_v2` | 0.064929 s | 0.028593 s | 5 files; 39,527 bytes | 1,296 bytes |
| Material | `material_force_dispatch_records_immutable_scene` | 0.032914 s | 0.020462 s | 1 file; 49,152 bytes | 567 bytes |

Both exact names are under `force_runtime_integration_tests`. Their commands use `--exact --ignored --test-threads=1 --nocapture`, never a broad filter. Raw evidence is under `/workspace/scratch/rheon-ui-force-worker-execution-20261021/{native,material}/{DATA,capture}`. Native ran before material; the preserved per-case roots and source-reviewed one-shot supervisor refuse duplicate use of those destinations.

Applied child limits recorded by the reviewed supervisor are a 30 s timeout deadline, 20 s CPU, 512 MiB address space, 1 MiB per-file/combined-log limit and zero core size, with TERM→1 s grace→KILL teardown available. Neither case reached a timeout or required teardown signals. CPU values are the single child interval's RUSAGE_CHILDREN deltas after git preflight. Individual peak memory was not measured; 512 MiB is a limit, not consumption. DATA sizes are final retained logical bytes, not peak disk blocks, cumulative I/O or SQLite temporary/anonymous-file use. SQLite transient peak remains unknown; the success-only eight-file/8 MiB inventory assertion is not an aggregate filesystem quota. Existing protocol unlinking of journals/staging is not test cleanup and does not preserve historical transient files.

## What the passing assertions establish

Both cases passed through actual Request→Worker::start→owned Worker/Rayon execution, with threads=1, join before result assertions, progress 1, one result row and two ordered in-memory previews. They used the same frozen enabled version-1 Global Directional acceleration document (`[0.25,0,0]` m/s²), canonical backend adapter and recorder/publication association. The temporary cfg(test) gate bypass did not bypass the remaining eligibility, budget, snapshot or binding validation. No App/UI start route or production gate was exercised.

**Native:** one JacobiPcgV1 track, 2³ cells, density 2 kg/m³, no source/emitter/model, one requested 1e-6 s interval and pressure iteration cap 100. The source asserts zero constructor velocities, absent constructor interval pressure, accepted interval starting at zero, eight finite pressure values within the predeclared ±0.05 Pa guard around `(index%2)×0.25 Pa`, force-qualified controls and accepted dt exactly equal to requested dt. The pass supports this coarse nonzero dispatch symptom; actual pressure values were not printed, independently extracted or scientifically re-evaluated here.

The executed cold path required the finished immutable member to open Ready, verified finalized pin/run/spec/declaration association, completed status/count 2/generation 1 and exact force bytes on constructor and accepted records. Preview/result timestamps match the accepted record. Mutating a separate draft was followed by archive rereads requiring original force bytes. The native prepared-rerun clone preserved the stored v2 intent and cleared the attempt without starting a second Worker; it retains old output/receipt paths and is not execution-ready or evidence of production rerun/numerical equivalence.

**Material:** a 1 m domain with 2³ preview/grid cells, 4³=64 initially stationary emitted-fluid water particles, no emitter/collision, gravity `[0,-1,0]` m/s² and one public step (canonical internal preparation bound 16). The source asserts 64 initial/accepted particles; finite positive actual dt≤requested dt; finite positive mass and net X momentum change within the predeclared 50–150% of initial mass×0.25 m/s²×actual dt. This is a coarse dispatch-effect guard, not SPH accuracy/force-equivalence validation. The pass also checks cold header configuration/geometry, both exact force documents, completed/count/generation, actual accepted timing/diagnostics and preview/result association.

The material Request clone is retained typed intent with its force rebound from cold metadata, followed by archive rereads after mutating an independent draft. Material has no serialized FrozenSpec/cold-rerun API: this does not reconstruct all controls from archive text, validate production cold material rerun or execute a second step/Worker.

## Retained artifacts and provenance limitation

Native final DATA contains the four expected immutable JSON manifests and one 32,768-byte SQLite archive, SHA-256 `ea8ccde4ec9a94a218443dc2570212f37cd7439c6af73e56d934f73fd4d13279`. Material retains one 49,152-byte archive, SHA-256 `17ace71c0ed82ea905b11b90fdb1ff0413afddb5f700ad4a84a249abb26c14d5`. No unexpected final data name or sidecar was recorded. Hash verification was byte-only; this reviewer did not independently perform the databases' SQL/decoder checks. Those checks are evidence from the passing sealed assertions.

The native finished manifest's expected source revision is `rheon-desktop-saved-carrier/0.1.0; source=build revision unspecified; tracked-material=unspecified`. Thus archive/header association matches the producer's declared string, but the archive is not self-identifying as revision 3b3. The external clean-source/binary/preflight/index seals supply the reviewed source custody. Do not describe this as embedded authenticated revision provenance or silently rewrite the archive; no rerun is authorized to change it.

## Remaining acceptance boundaries

These results cover one enabled Global Directional acceleration field, one native Jacobi track and one default-water material track. They do not exercise other field kinds/units/supports/time windows, emitter/collision combinations, authored-empty metadata, mixed saved members, configured/provider/session/free-surface paths, failure/unrecorded tails, timeout/kill behavior, interrupted Cancel/join, legacy equivalence, general field accuracy or scientific validation. Prior backend or pure codec evidence must remain separately identified.

Native force launch, actual progress/Cancel, retained versus draft gestures, picking/handles, keyboard/accessibility and cold material rerun remain unqualified. A future typed eligibility draft/pure-test review may be prepared separately; these passes do not authorize enabling the production gate, removing the test bypass, another workload or further native execution. Preserve the exact artifacts and prior evidence without cleanup or tuning.
