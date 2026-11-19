# Independent M3 authoring-core review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. Reviewed the new `src/initial_contents.rs`, additive library exposure, M3 admission, governing authoring contract and six newly written headless contract fixtures. Baseline is the isolated M2 commit `d0e92b7343599703978b660d3c057cf8602cc98f`. Native adapter integration, file encoding and saved-run execution are outside this core review.

This is static source/contract/test-design review. I executed no tests, builds, simulations, native app or benchmark and made no external calls. No M3 numerical or graphical acceptance is inferred from source inspection or from tests merely being present. Only this report was written by the reviewer.

**Current disposition:** the targeted rereview below resolves core R1/R2. No further required core correction was found in that scope. The initial findings remain as review history; native/UI composition is reviewed separately and is not accepted by this core disposition.

## Complete required findings round

### R1 — Medium: history metadata admission happens before its eviction population is known

Location in first candidate: `ContentsEditor::publish`, `src/initial_contents.rs:533–554`.

Replacement metadata is reserved for `cursor+1` commands, including all currently applied commands. The code then rejects `command.payload()+metadata > history_limit` before calculating which old commands to evict. Eviction later subtracts only old delta payload, while retaining the metadata capacity for the larger pre-eviction population. Consequently a new command that fits by itself can be refused because it is charged metadata for commands that should have been evicted. The same oversized temporary metadata can unnecessarily exhaust the candidate-preparation cap.

A bounded counterexample follows directly from the existing test's platform-independent cap calibration. Measure the cost of one whole-domain Air-fill command on a 4³ Vacuum document and use that as the history cap. In a fresh Vacuum editor with that cap, make three separate single-cell Water additions at different centers. Their small sparse deltas can fit. A subsequent whole-domain Air fill changes all cells and has the same dense before/after payload as the measured fitting command. It should succeed after evicting the applied small commands, retaining one undoable fill. Instead, the present early check charges four command slots plus the dense payload and rejects before eviction. No allocation failure or corrupted state is needed.

Required correction: determine the retained applied-command suffix and its metadata allocation jointly, before destructive changes. Preflight/allocate only the metadata population that will survive with the new command, recheck actual capacities against both preparation and history caps, and continue to move existing payloads without cloning them. All failures must remain before redo invalidation, old-command eviction and document publication. Do not fix this by raising either cap or discarding history before allocation can fail. Add the reverse-size regression above (several sparse commands followed by a fitting dense fill), requiring successful whole-command eviction, exact undo of the fill, the correct retained undo floor and no budget excess. Retain the existing oversized-command/candidate refusal checks.

Owner: authoring-core implementer. Disposition: required correction before history-cap behavior is treated as accepted.

### R2 — Low: ray direction normalization and near-distance units are unspecified

Location: `ContentsBrush::pick`, first-candidate `src/initial_contents.rs:259–288`.

The public API accepts any finite direction but compares the raw slice-axis component with absolute machine epsilon and the resulting parameter t with `near`. Scaling a direction therefore changes both the near test and the apparent parallel test. For the same origin and geometric ray, direction (0,0,1) and (0,0,1000) can disagree when a hit beyond a near plane in metres produces t below the same numeric `near` for the scaled vector. A tiny rescaling can be rejected as parallel. Neither a unit-direction requirement nor a non-metric parameter convention is documented or validated, so the upcoming camera/brush boundary cannot establish the physical clipping contract from this API alone.

Required correction: define and enforce the ray convention before the native caller is composed. Either normalize a checked finite nonzero input direction and interpret near in world metres, or validate/document a unit-direction precondition and have the camera supply that representation. State any deliberate near-parallel angular refusal independently of arbitrary vector magnitude. Add a small scale/near-distance fixture consistent with the selected convention, along with the existing parallel/inside/half-open cases. This needs no new picking framework or surface-mesh dependency.

Owner: authoring geometry boundary / native adapter implementer. Disposition: small required contract clarification before camera integration.

## Confirmed authoring and history semantics

**The operation table matches the approved model.** Add changes only Vacuum; Replace changes occupied Water/Air to the selected occupied material; Remove leaves Vacuum regardless of the selected material. Repeated support visits are idempotent for the frozen brush, so coalescing within a stroke produces one final before/after value per changed cell. Explicit `fill_air` replaces the entire domain as one command and does not establish automatic ambient refill. The tests enumerate original/selected material and all three operations, compare exact IDs, and exercise undo/redo of changed cases.

**Committed, preview and frozen data have separate ownership.** The editor owns one private committed document and an optional dense private candidate. Segment updates affect only the candidate. Freeze rejects an active stroke and returns owned copied cells, fractions, geometry and captured material values. Air maps exactly to F=0 and retains positive gas density; Water maps exactly to F=1. Vacuum validation returns its count and first cell with an actionable fill explanation. A later edit cannot alias the frozen vectors. This is the binary-input core boundary; M4 must still compose complete numerical/configuration/capability admission before launch.

**Stroke cancellation/no-op and redo behavior are correct in the inspected paths.** Begin validates and allocates before installing a candidate; an invalid point or stale base drops the candidate rather than committing its last valid prefix. Commit takes the candidate and returns false for an exact no-op before changing revision/history/redo. Explicit cancel and the first Undo during a live stroke discard only the candidate. A successful nonempty publication invalidates the redo suffix; camera/selection/save routing is not implemented here and must not invent a second history owner later. Redo during a live stroke returns Busy. The tests inspect committed versus preview bytes, freeze Busy, no-op and invalid-point redo preservation, committed redo invalidation and monotonic undo/redo revision increments.

**Commands restore values rather than rerun brush geometry.** Sparse deltas use ordered unique indices gathered from the final scan with exact before/after IDs. Dense deltas copy complete before/after documents when that nominal payload is cheaper. Verification checks document identity and the expected content projection before applying; private cells/geometry/palette prevent an external mutation from silently bypassing that proof. Command base/result revisions retain transaction provenance while the document's current revision remains monotonic on undo/redo. Loading must construct a new editor/history, as admitted; this core has no in-place geometry or palette mutation.

**The memory owners are explicit, with R1's selection flaw outstanding.** Dense candidate and preparation have a separate cap from retained history. Command payload uses actual Vec capacities and history includes command metadata capacity. Allocation failure and every checked admission occur before the current implementation drains old commands or publishes the document. Therefore R1 currently causes a refusal, not an untracked non-undoable edit or premature eviction. The repair must preserve that strong failure-before-mutation ordering. The 4–64 axis-count domain bounds the cell-payload products; reported byte counts mean managed capacity, not allocator overhead or measured process RSS. Immutable freeze/input and the later native/simulation/recorder owners remain distinct charges.

## Picking and swept-support reasoning

Brush radius is a positive finite normal world-space length bounded by the finite physical domain diagonal. Axis and layer are checked before indexing. Endpoints must lie in the half-open domain and exactly on the frozen slice plane; neither invalid endpoints nor exact outer maximum are clamped into a cell. Layer mode filters one axis layer; Volume mode uses full 3D cell-center distance. A zero-length segment is a dab. A nonzero segment evaluates hard round support against its closest point, so a long pointer segment does not leave stamp-spacing gaps. No render threshold, SDF, pressure state or simulation array participates.

The geometry fixture uses anisotropic spacing/nonzero origin and an independently specified axis-aligned capsule, then compares one long stroke with 17 collinear partitions in both depth modes. This meaningfully checks the intended fixed-radius sweep behavior. “Exact support” here must be read as the declared hard cell-center predicate evaluated in f64, not an exact-real-arithmetic proof of every radius tie or arbitrary input reparameterization. Additional native camera, preview and keyboard equivalence are not established by this fixture. R2 specifically closes the presently unspecified ray parameter/near-distance convention.

Core `pick` already rejects finite-check failures, behind/near intersections, parallel rays and outside hits; a camera starting inside the domain is supported when the selected plane lies forward. Native viewport changes, pointer/focus loss, camera changes, selection changes, held-button tails and invalid terminal releases remain the adapter's pending obligations. `segment`'s correct explicit invalid-point cancellation does not itself prove that the adapter calls it or cancels on every missing hit.

## Evidence limits and next review boundary

The six headless fixtures were inspected as written, not executed by this reviewer. Their operation-table, cancellation/no-op, simple sweep, frozen-input, capacity and picking oracles address the core boundary. R1 needs the additional sparse→dense capacity regression; R2 needs a declared ray convention and its focused fixture. Existing generic-error assertions are not being relabelled as native interaction qualification.

No save/load codec, crash-recovery journal, scene/force selection, text-undo shortcut route, keyboard numeric dab, egui pointer stream, retained-result custody or native window was reviewed or accepted here. The report admits dense candidate storage as a bounded implementation refinement while retaining one core history authority; it does not require a general sculpting platform, sparse-tree engine or second desktop undo implementation. The documented missing graphics environment remains an evidence gap, not a passed idle-edit workflow.

**Initial disposition:** R1 and R2 require serial correction and targeted rereview before treating this core as ready for camera/history integration. Root owns focused execution and preserves any initial results separately. No source or test was changed by the reviewer, and no numerical/native/performance acceptance or main integration is authorized by this report.

### Initial reviewed identities

| Path | SHA-256 |
| --- | --- |
| `src/initial_contents.rs` | `9bf44af0c5a7d02379e684639ce03655c7e78c95a3f6e7fcc85918001e3b9824` |
| `src/lib.rs` | `8da618ce427027c4841f485211d5fed1f5c9452919049ef2eb31bcde48c6ecb8` |
| `tests/initial_contents_contract.rs` | `4090d63948b8a7a4dca077901ba289109f34eb9ef31f8f00e535974fb65f6171` |
| `reports/m3-admission.md` | `39e76f6df971cdf102cbab2e79d3866f76b66f637c6d79d2b43991bba79c6ede` |

These identify the first source/admission candidate, before the requested corrections. Only this report was written by the reviewer. No reviewer action was denied, pending or uncertain.

## Correction verification — targeted core rereview

On 2026-10-22 the same separate reviewer inspected the corrected retained-history selection, ray convention and their two added fixtures. Root reports eight core fixtures passed; I inspected their code without rerunning them. This is a source/contract/test-design disposition, not real native interaction acceptance.

**R1 — resolved.** `publish` now selects a retained applied-command suffix jointly with its replacement metadata requirement. It checks both history and candidate-preparation estimates, reserves only that suffix plus the new command, and rechecks actual capacity. If an actual capacity does not fit, it discards that still-private allocation and considers a shorter suffix; no old history is drained during this selection. The eventual command move, redo invalidation, document/revision update and whole-command eviction still follow every fallible allocation/admission and checked eviction counter. This closes the false refusal for a fitting dense command without changing either cap or evicting history before a potentially failing preparation.

The new `dense_command_evicts_small_commands_and_their_metadata_jointly` fixture obtains the cap from one actually fitting dense fill, admits three separate single-cell additions in a fresh editor, and then requires a whole-domain fill to fit after evicting all three. It checks the exact previous content on Undo, the retained undo floor, Redo and the reported capacity bound. It directly reaches the previously faulty reverse-size history composition without hardcoding Rust struct padding.

**R2 — resolved.** `pick` now normalizes finite nonzero directions internally using a largest-component scaling before its norm, so extreme finite vector magnitudes do not create norm overflow or alter near units. Near is explicitly distance along the ray in world metres; the near-parallel threshold is applied to the normalized direction. The new scale fixture compares directions of magnitude 1e-200, 1 and 1e200, requires the same hit and near-plane refusal, and checks zero-direction rejection. The native adapter is inspected separately to ensure its camera-depth threshold is converted to that metric distance.

No further required correction was found within this targeted core scope. Eight core fixtures and this rereview do not qualify egui controls, camera composition, scene/force routing, native graphics or persistence. The initial review's native and M4 gate limits remain explicit.

### Corrected core identities

| Path | SHA-256 |
| --- | --- |
| `src/initial_contents.rs` | `28aae15ca5fb64dd69457a7e91767000f9d96399878663bd838f151b9bc21532` |
| `src/lib.rs` | `8da618ce427027c4841f485211d5fed1f5c9452919049ef2eb31bcde48c6ecb8` |
| `tests/initial_contents_contract.rs` | `2951202c259328b3f52de09316eadfd150a5515f1a8496e5b8914dc5216d4cc3` |
| `reports/m3-admission.md` | `39e76f6df971cdf102cbab2e79d3866f76b66f637c6d79d2b43991bba79c6ede` |

Only this report was changed by the reviewer during core rereview. No test/build/numerical/native workload or external action was performed.
