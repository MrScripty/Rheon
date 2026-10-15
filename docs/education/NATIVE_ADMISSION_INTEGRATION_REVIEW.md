# Recording and periodic S1 source admission review

This is a bounded source admission decision for **attempting** fresh native and
book qualification. It is not a numerical receipt, S1 acceptance, native-window
review, restart contract, or authorization to merge an unqualified component.

The candidate is the isolated integration branch at
`21d4122c3d0edb2ca5c3ce089175c70035d737a3`, descended from main
`06eb5daee9e77e2b0843a5ee0a4105f6950e9b1b`. Trial merges retain recording PR69
`b1acf2e514b88b528cf43e873035a598a1080ae1`, carrier PR71
`ecfc741af733f909c79170530080a98fdc5c78eb`, validation PR73
`27193c77b173a7437494584438847fedc8ecf552`, and native S1 PR74
`5a049ed604b99817a428d10bbc0140732d3421d6` as ancestors. The reviewed prospective
canonical plan at `fc2270a85e38080d9943048aaf689d99545d3715` stays byte-identical
to current main. No component or main ref is changed by trial composition.
Parent subsequently delivered final PR74 head
`a0efbc1aaf67bb65a724de028d4a773270977875`. Its two documentation changes were
reviewed and trial-merged at `c29cb2a`; the entire native source closure is
unchanged from the above candidate. S1 A/C independent evidence is now supplied,
with checkpoint disposition still owned by the parent and canonical planning owner.

## Complete admission closure and ownership

The previous runtime epoch contains 73 native/build/independent-qualification
paths. This candidate has 78: five added, nine changed, none removed. All other
64 paths, including grid geometry, aligned strain, pressure/SGS numerics, the
aligned native example, rational oracle and qualification runner, retain their
runtime-admitted bytes. Exact path-set comparison covers every `src/` file,
`.cargo/` input, Rust toolchain file and existing qualification input; it does
not project away unused modules to bypass complete crate admission.

| Added/changed inputs | Review basis and bounded claim |
| --- | --- |
| Cargo.toml/Cargo.lock | Source-identical to reviewed recording PR71. Pinned native-only rusqlite 0.37.0 (MIT) and sha2 0.10.9 (MIT/Apache-2.0); SQLite is explicitly bundled. Existing package identities remain, with locked transitive additions. No S1 dependency added. SHA256 detects accidental archive corruption; it does not authenticate hostile rewriting. |
| src/recording.rs, recording/material.rs, recording/carrier.rs | Unchanged PR69/71 implementations and independent lifecycle review. Bounded accepted-publication capture, transactional committed prefix, indexed field decoding and strict identities/units/association/stage. Poison/uncertain terminal writes require cold reconciliation; no completion retry. Keep-all then stop at explicit limits; missing accepted tails disclosed. No universal field coverage or integrator restart claim. |
| src/simulation.rs | Unchanged PR71 success-stamped interval pressure accessor and actual controls/configuration. Scratch is invalidated before attempts and published only after acceptance. Solver arithmetic is unchanged. Thermodynamic pressure from S1 remains independent of this gauge-fixed projection pressure. |
| src/vtk_export.rs | Unchanged additive PR71 carrier contract-v1 VTI writer. Native f32 faces/tracer and f64 accepted interval pressure preserve association/stage/time/gauge; missing fields remain absent. Derived cell velocities are labelled derived. Existing material/session contracts retained. |
| src/bin/desktop/archive_view.rs and changed desktop history/inspector/particle_view/main | Unchanged PR69/71 persistence and selection integration. Cold selection and CSV/VTK use selected numerical publication, never latest-state substitution. Observed audit/read failures invalidate all cached display/export capability. Explicit selections perform I/O; immutable repaint reuse is separate. Headless egui tests do not establish native-window rendering. |
| src/compressible.rs | Unchanged independently reviewed PR74 bounded periodic first-order ideal-gas LLF/Euler owner. Five conservative cell-average components, unique periodic faces, atomic acceptance/retry/cancellation and compensated budgets. No link to incompressible pressure/session/recording owners, GPU or S2 scope. Final owner A/C evidence and independent prospective B/D/E/F/G evidence remain subject-pinned, separate from combined publication qualification. |
| src/lib.rs | Sole composition conflict, resolved additively: retain native-only recording module and all S1 exports. Prior exports unchanged. No conditional solver selection, fallback method, shared EOS or changed caller route. |

The validation tool and native field producer remain byte-identical to their
reviewed component subjects. Their original prospective 16/24/32 receipt remains
evidence for its exact source; it is not relabelled as integration-source evidence.
Original 8/12/20 failure receipts remain failed. Independent review and focused
tests cover component source behavior; clean committed fresh qualification must
cover this actual composed crate and all build inputs.

Independent GPT-6.1 Sol High recording reviewer inspected this composition and
the epoch protocol read-only. It confirmed numerical source separation and the
unchanged qualification kernel, and found the standalone validation lock stale.
The isolated integration repair adds 22 required native identities, preserving
all 49 prior identities/checksums. Precise alignment of newly resolved cc/smallvec
retains the reviewed root graph's versions; all additions now match root lock
identities. The [license addendum](../plans/recording-compressible-integration/reports/dependency-licenses.json)
records local registry MIT/Zlib texts and bundled SQLite source identity. Root
and browser locks preserve previous identities/checksums with 13/21 additions.
No new dependency is admitted through floating-version resolution. Locked
metadata and build checks remain required on the repaired standalone harness.

## Explicit new epoch and preserved receipts

Preserve the historical inventory bytes at SHA256
`84c036618b32de66623d8ded24ae8cf95194457e36b9a130087a9c2bf3bd3f6a` and the runtime
inventory bytes at SHA256
`6ada2133501aab449ed7ee73f802d8a4593db50c857f0a8f8ce2af4507721d6d`.
The new separately named integration version-three inventory pins this reviewed
composition ancestry and all 78 current paths. It binds both predecessor digests
and this review record. A present malformed/stale new epoch refuses admission;
it cannot fall back to an earlier epoch. Fresh native receipts must bind all
three inventory files, this review record, every native digest, and their actual
committed blobs. There is no automatic inventory generation in any build or CI.

The immutable primary tree and ancestry checks stay in force. Source identity is
an admission input, never numerical qualification. Require fresh clean committed
native/rational execution, real-packet rejection tests normally and under `-O`,
locked native/browser builds, and full hosted book/runtime/publication checks.
Lean proof bytes are unchanged; no new kernel claim follows from these additions.
Native window inspection is pending until a supported display is available.
Execution receipts and final review outcomes belong in the integration plan's
evidence report, separate from this fixed admission decision.

## Configurable diagnostic pressure M1 admission, 2026-10-15

The previous 78-path v3 review above remains historical evidence. This explicit
review refreshes that same integration epoch after PR84 main `c85dfee` and selected
comparison imports `20b451d`/`d448563`. The independent GPT-6.1 Sol High reviewer
confirmed the complete current closure: 90 paths, 12 added, 6 changed, 72 unchanged,
none removed. The unchanged v3 schema/base ancestry and v1/v2 predecessor hashes
remain authoritative. Prior qualification receipts are not evidence for this tree.

Added paths are `desktop/ai_comparison/{adapter,cpu_contract_tests,engine,mod,
model_process,record,scene,view,worker}.rs`, `desktop/camera.rs`,
`src/bin/rheon-ai-compare.rs` and `src/solver_provider/context.rs` (desktop paths
under `src/bin`). Changed paths are Cargo.toml/lock, desktop/README.md,
desktop/viewport.rs, src/bin/rheon-desktop.rs and src/solver_provider.rs.

The selected original comparison imports retain the existing sealed-water native
operator and add an independent diagnostic trajectory/UI, shared camera and owned
worker/process. They do not modify Session, Simulation, accepted recording or
aligned-strain numerical kernels. The new public context is additive and checks
borrowed physical/lineage coherence; it is not upstream composition acceptance.
Manifest/CPU protocol uses closed bounded trusted-local contracts and existing
process ownership, now with queued input writes inside prediction deadlines.
Generic frame-v3 does not invent a trained checkpoint; legacy frame-v2 remains.
Source review closed history/domain, legacy-reader and finished-worker UI findings.
Focused actual stdlib CPU tests and desktop Clippy passed before admission; they
are not trained Torch/model, native-window or descendant-cleanup qualification.

Cargo adds the comparison binary and pinned optional serde 1.0.229 / serde_json
1.0.154 (float_roundtrip), activated only by desktop; no new crate identity or version is
introduced into the lock (only the root package dependency list changes).
Viewport camera movement was extracted into the shared owner with existing tests;
unrelated training/voxel work is excluded. All conservative selector paths,
including test-only desktop modules, are inventoried without projecting an assumed
transitive numerical subset. Independent review found no material admission issue.

This authorizes attempting fresh qualification only. The reviewed bytes are bound
by the refreshed inventory and this record's digest, committed together. Require
clean-source native/rational execution, packet rejection tests normally/under -O
and hosted checks on this exact source. Existing missing trained deployment and
native window cannot be qualified by source admission. Future M2/M3 source wiring
requires its own explicit reviewed inventory refresh and fresh receipt.
