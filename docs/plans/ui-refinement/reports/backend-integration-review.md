# Independent backend transfer/integration review

2026-10-20. Read-only source review of the force backend overlay in
`/workspace/Rheon-ui-scene`, branch `implementation/ui-scene-integration-20261020`,
HEAD `7318023f04946171d7909119300af76e455c2926`. No tests, numerical/model workloads,
native UI, SQLite operations, compilation, publication or Git mutations were
executed by this reviewer. Only this review report was authored. Repository and
workspace `AGENTS.md`/local skill instructions were sought; none were present at
the inspected paths.

## Disposition

No blocking source-transfer or integration-disjointness finding. The canonical
backend is present unchanged, with its API available to the desktop feature.
This admits subsequent independently reviewed UI authoring work against those
types; it does **not** accept desktop force dispatch, frozen-intent migration,
recording integration, solver parity or scientific behavior. None of those
desktop paths is implemented by this overlay.

## Exact identities and independently checked seals

| Item | Verified identity |
| --- | --- |
| Supplied backend revision | `3e839cbf7c6aca56e8297ef740f7e48543de77b1` |
| Backend committed tree | `c5e1f16a45a2244405610335dfeffdbc0123eca6` |
| Integrated pre-new-UI checkpoint tree | `31033fb2c040c391603ed83ef31830958188c876` |
| Custody source manifest SHA-256 | `d873dd98d5d59d1f1aac2221cb5f1f30ae574bd6dff91c95d95d82a0c927a9ca` |
| Preserved desktop main SHA-256 | `c0f7b12a2b75b95bb4c70da920b343da36658fc6c766b4f156bf51af88e57695` |
| Preserved UI5 settings SHA-256 | `099fa897a53feee4cb1de483655893f50e6c713c39e66c1a25c8f0549ee2c121` |
| Preserved T1 tests SHA-256 | `12bee2a70ab3b6cb40a73e183e0786d220ed516761c416bafa7ca097c64352fa` |

The 13 entries in `force-authoring/evidence/custody-source.sha256` were hashed
directly from the supplied revision's Git blobs and compared to the manifest;
each working file also equals its committed blob. The manifest itself equals
the committed manifest. All 35 paths changed between the backend's `fd7c0f0`
base and supplied revision equal their supplied Git objects locally, including
the force documentation and evidence. All 1,372 entries in the integrated
checkpoint tree equal their current local file bytes. These checks read files
and objects; they did not stage untracked files or rewrite the tree.

The manifest is a source identity receipt, not an authentication or execution
certificate. In particular, importing the backend's whole branch relative to
UI HEAD would have reverted unrelated UI changes because its base predates
them. The actual checked overlay avoids that problem: the backend change set
contains no `src/bin` paths; all 55 current desktop files equal the preserved
UI5+T1 worktree. `Cargo.lock`, session/material-session/provider owners and both
cancellation-supervisor files also remain byte-identical to UI HEAD. Inspector,
desktop run-set custody and launch dispatch were not replaced by this transfer.

## Canonical API and guard inspection

`src/lib.rs` exports the single canonical `SceneObjectId`, `ForceScene`,
`ForceObject`, transforms, kinds, support, time, snapshot and validation error;
`ForceUnits` remains the shared type from `forces.rs`. `Cargo.toml` adds only
the optional `force-serde` feature. Authoring derives are enabled by either
that feature or `desktop`; desktop already enables the Serde dependencies.
Recording remains native-only under its existing target gate. There is no
parallel desktop force-domain implementation or premature desktop call site.

`force_field.rs` validates version, object/name bounds, nonzero and force-list
unique IDs, finite properties, quaternion tolerance, support and activation
before deep-copying a scene into the private immutable `Arc`-backed snapshot.
The snapshot exposes a shared scene reference and no mutable/Serde bypass.
Cross-model/force uniqueness and allocation remain UI responsibilities; the
backend deliberately validates only its force subset. JSON deserialization is
not validation, and a future UI importer must bound its input before parsing.

The transferred public accepted-pressure and particle-control stamps retain
actual frozen fields. The inspected MAC authored path assigns its force stamp
after commit; candidate preparation/reset invalidates pressure stamps. Particle
authored controls are replaced on success, retained on rejection, and cleared
by a successful legacy step. This source inspection does not establish runtime
acceptance of these paths on the integrated tree.

`recording.rs` re-exports the bounded codec, run binding/association checks and
carrier/material publication wrappers. `force_fields.rs` checks the 256 KiB
input bound before decode, validates schema and objects, and rejects trailing
bytes. It binds the complete scene digest through a versioned configuration
suffix. `require_fields` permits attaching metadata at construction, but an
accepted publication must already contain matching actual custody; it cannot
simply bless the caller's current draft. Writer association validation precedes
the transaction, and reader validation follows the existing publication digest
check. The material standalone decoder's absent run association is documented;
use the reader or explicit association check when a run header is available.

Legacy SQL version and digest implementation are unchanged in the inspected
diff. This is source preservation plus upstream fixture evidence, not a claim
that arbitrary historical archives have been exercised. Missing metadata means
legacy/absent declaration, not proven zero external force.

## Evidence boundaries and next admission

The supplied check log `/tmp/rheon-ui-scene-backend-check.log` reports a successful
locked desktop check in 8.75 seconds. It was read, not rerun. Upstream documents
report seven pure codec tests and static checks after custody changes. Its five
bounded adapter runtime results belong exclusively to pre-custody binary
`0042e68eb0de3f63e3d4ef98506a88f417a4f965f63c41b64890f4ee9e9003cd`.
The preserved pre-custody seal differs from this custody manifest, including
force-field, simulation and particle source hashes. Those runtime results
cannot be promoted to the newer custody source or this desktop integration.

No post-custody solver, SQLite, desktop dispatch, native interaction or archive
rerun was executed for this review. Existing T1 focused-editor/track/report and
native/accessibility gaps remain open; the source transfer does not resolve them.
Core recording ownership remains with backend; Request/freeze/launch,
carrier/material dispatch and versioned desktop run-set intent remain with UI.
Nonempty scenes must be visibly refused on a workflow until its exact owner
path and custody are qualified; session/configured/provider/transport support
must not be inferred from direct adapter exports.

The copied scene contract/plan still includes historical statements that exact
source transfer is pending and the baseline timeline has a 106px floor. This
receipt supersedes transfer-pending claims only. Canonical progress records
should distinguish verified source arrival from still-pending runtime custody
and retain the T1 partial acceptance boundary. No production repair is required
by this integration review.
