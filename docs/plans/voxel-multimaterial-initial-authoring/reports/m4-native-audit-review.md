# Independent targeted review: native artifact audit helper

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`. Production base `0059ca035901bc6ca662c4540ec345b31070571e`. Reviewed new `src/bin/desktop/voxel_material_native_audit.rs`, the frozen metric protocol and the actual called manifest/recording/authoring reader owners. Only this report was written; no helper, build, test, native process, simulation or benchmark was executed by the reviewer.

**Disposition: one required helper-only correction before executing this audit under its stated unchanged-artifact contract.** The mathematical mask, exact cold publication comparison and absence of extra solver work are otherwise appropriate. This is helper source review, not native evidence acceptance. The helper is not part of the already-built non-test native binary.

## Complete required findings round

### R1 — medium: archive recovery can write before the helper's unchanged checks

`open_finished` calls `open_member`, which calls the existing `RecordingReader::open`. That reader deliberately uses `SQLITE_OPEN_READ_WRITE` through `recording.rs::connect` and explicitly allows hot-journal recovery before enabling `query_only`. Therefore the helper's unconditional read-only description is stronger than its actual call chain. An unclean recovery sidecar can cause an archive write even though the helper never invokes a producer. Existing `open_member` compares the pinned archive digest before/after its open, which detects a changed archive but cannot undo recovery that already happened. The helper's final `unchanged` vector currently contains only the supplied JSON/oracle/draft paths; it does not retain and recheck the SQLite files over the whole audit.

Required proportional correction is confined to the helper, without changing production readers or numerical owners: for each validated closed member resolve its already-admitted archive path, retain its bounded whole-file digest before calling `open_member`, require that digest equals its existing pin, and refuse recovery-related sidecars (`-journal`, `-wal`, `-shm`) before opening. State the existing protected-path/no-concurrent-writer assumption explicitly. After all selections and comparisons, rehash every opened archive and require its original pinned digest unchanged. This makes the helper non-writing on the admitted clean closed input contract and detects mutation across the entire audit; it does not turn the existing SQLite connection into an OS read-only descriptor or provide a general hostile-filesystem race guarantee. Do not recover, delete sidecars, repair, repin or overwrite anything during this audit. A sidecar/digest refusal should leave the gate pending for inspection.

This finding concerns the audit's input-preservation claim. It does not identify a solver, archive-format or native UI defect and does not require reopening those accepted components.

## Other reviewed semantics

The helper constructs no App, Worker, VoxelMaterialSimulation or numerical constructor and calls no step. `initial_editor` and draft restoration create authoring documents only; validation is nonnumerical preflight. `open_member` performs the existing cold typed publication audit and pin verification, not a solver rerun. No producer, publication, finalization, pin, replacement or child process is requested by this helper. Registration must remain cfg(test), ignored and explicitly invoked only with observed artifact paths. Compile/execution evidence is pending and belongs to the implementer.

The independent mask uses explicit x-fast Cartesian cell centers and Euclidean sphere inclusion, without calling production brush, viewport or simulation code. The inspected frozen protocol describes the Water sphere at `[0.5,0.5,0.5625]` m with radius `0.35` m, then an occupied Air pocket at `[0.4375,0.4375,0.5625]` m with radius `0.07` m. Its expected 96 Water cells become 95 Water/417 occupied Air, with flat index283 Air and six listed Water neighbors. The helper independently recomputes these masks, checks their counts/IDs and compares loaded draft/captured constructor. This is appropriate for the specified binary cell-center operation; it does not validate appearance-derived occupancy or a general volume-fraction brush. Freeze the protocol hash independently before observation and retain its chronology; a helper cannot prove that a JSON was predeclared merely from its `kind` string.

The manifest lineage checks verify existing intent→started→finished digests, one frozen member, closed pins and a rerun parent bound to the original finished manifest. Distinct run IDs plus equal frozen input digest and exact raw field bytes/timing establish deterministic recorded replay for these supplied archives. Controls/provenance diagnostics are intentionally not compared as deterministic numerical bytes. `RecordedField` compares its raw data and metadata; explicit `to_bits` checks cover optional field evaluation time and publication time/dt. Expected differences in attempt identity are preserved.

Typed decode and the existing cold reader enforce field inventory, f64 shapes/MAC walls and metadata binding. The helper adds canonical constructor F/zero velocity, accepted clock/pressure-start continuity, normal bounded dt, pressure gauge, finite/bounded F and captured material bits. Phase volumes use independent sums of F and occupied gas `1-F` over the specified unit-volume grid; mass uses captured densities. The `1e-8` relative drift ceiling plus independent reduction allowance is consistent with the previously admitted step-contract ceiling. These are observations of recorded fields under an existing ceiling, not reconstruction of internal cumulative budgets or new scientific qualification. Raw Debug controls are printed as declarations and never parsed as a validation oracle.

The completed baseline/rerun require constructor plus eight accepted publications and distinct identities. Optional Cancel must be genuinely cancelled with accepted generation below requested intervals; permitted one-interval unrecorded accepted tail follows the existing owner. Its supplied policy is checked against the protocol's same8³, at-most1024, dt0.001, zero-gravity cancellation schedule. If no Cancel path is supplied, the helper explicitly leaves cancellation unassessed; an overall test pass must not be reported as complete Cancel acceptance.

Artifacts alone cannot establish real pointer/keyboard action chronology, Save/alter/Load interaction, software renderer, window dimensions/zoom, a fresh OS process, an active Cancel click or its45-second observation bound. Those remain separate actual-native logs/captures/process evidence and independent review. The final “no scientific/performance validation” caption must be understood as no new scientific qualification: this helper does calculate and check observed conservation drift. It is not a performance cohort or another numerical run.

## Reviewed identities and status

| Input | SHA-256 |
| --- | --- |
| `src/bin/desktop/voxel_material_native_audit.rs` (pre-format reviewed candidate) | `0627b08de543f18b5ae1578f425db46a411c0e232b4b95275e7b925b28a97c08` |
| `/workspace/.voxel-display/m4-native-protocol-20261022.json` | `960d399b25749a76ec0eaa66c1631ef9910607d7f65fe83668816424e21433f8` |

Required next boundary: parent serial helper-only correction for R1, then targeted rereview/final formatted identity before deciding audit execution. No production or native-binary source correction is requested. No reviewer action was denied, pending or uncertain. Prior denied operations and uncertain Library uploads were untouched.

## Targeted R1 rereview: corrected helper

**R1 is resolved in the reviewed formatted source.** `open_finished` now resolves the member path through the existing admitted `archive_path`, refuses any `-journal`, `-wal` or `-shm` entry before opening SQLite (including dangling symlinks and any metadata result other than NotFound), computes its bounded whole-file digest and compares it to the already-published pin. The archive path/digest/policy cap are retained before `open_member`. After baseline/rerun/optional cancellation reads, all retained archives are rehashed and compared to their original digests. No sidecar is removed, no recovery or repair is requested, and no pin is made. Explicit comments state protected caller-owned paths, closed producers, no hostile-filesystem race guarantee and no OS read-only SQLite handle. This meets the requested clean-closed-artifact contract without changing the production reader.

The only registration change is `mod audit` beneath `voxel_material_native_tests.rs`, itself imported through the existing `#[cfg(test)]` native-tests module. The deciding test remains ignored and explicitly artifact-driven. The module is excluded from the ordinary non-test native binary; no production owner, numerical method or existing native binary identity is changed by this registration.

No further required correction was identified in this targeted round. The helper is suitable for its explicit post-observation audit when the required native artifacts exist and the stated path contract holds. This is static source disposition only: reviewer execution remains zero, and parent compilation/audit/native evidence is not predeclared. All evidence limits above continue to apply, including optional Cancel being unassessed when omitted.

Final formatted SHA-256:

| Owner | SHA-256 |
| --- | --- |
| `src/bin/desktop/voxel_material_native_audit.rs` | `67aa34e1dd66b5e4d4706b868353e0fc0af5443cdfa1c77bf36e85a6d81a9f98` |
| `src/bin/desktop/voxel_material_native_tests.rs` (test-only registration) | `cbc49eeb511fce88c380e421834a5e28b7d3e7a01f20eb9ce13ad9074ab53769` |

The frozen protocol SHA remains `960d399b25749a76ec0eaa66c1631ef9910607d7f65fe83668816424e21433f8`. Only this report was appended; no workload, source edit or external action occurred in rereview.
