# Canonical backend source receipt

2026-10-20. Desktop integration branch:
implementation/ui-scene-integration-20261020 in /workspace/Rheon-ui-scene.
HEAD remains7318023f04946171d7909119300af76e455c2926; no new commit/merge/push.
Original M2a, precision and timeline worktrees/checkpoints remain preserved.

The published backend branch was fetched by its explicit ref, expected tip
1d44c8429701f205f678b602040d4f62ea6457b3, tree
3a0f00235cd224bfae825bb3f09b4fa599bc46e9. Its core custody is byte-identical to
3e839cbf7c6aca56e8297ef740f7e48543de77b1, tree
c5e1f16a45a2244405610335dfeffdbc0123eca6. The tip adds only the prepared recording
integration test and plan, neither executed here. All13 custody-source seal entries
match both fetched Git objects and integrated local bytes. Backend's35 original
changed paths are exact. An independent reviewer checked those paths, all1372
entries of the earlier combined tree and all55 desktop files unchanged by transfer.

Integration sequence used fresh isolated branch at7318023, then exact preserved
M2a patch ->tree8740bf155b3db1f3c44e109277605b9411f459fe,
UI-5 patch ->tree3e1fd57146e2f1a02abe22a09ac317497f39d738,
T1 patch ->tree90bf811a5213d8fa021e9ff4323345f669fb1284,
then the pinned backend diff ->tree31033fb2c040c391603ed83ef31830958188c876
(before receipt/review documents and the final two prepared-test/doc files).
Every prerequisite tree and patch application was mechanically verified.
No cherry-pick/merge, speculative rebase or unrelated worktree mutation.

Locked offline desktop check after core integration passed in8.75s:
/tmp/rheon-ui-scene-backend-check.log. This is compilation only. No backend,
recording, numerical or model test was executed here. Upstream five adapter tests
apply only to its PRE-CUSTODY binary; current custody has pure codec/static
coverage. Do not claim post-custody runtime/SQLite/desktop acceptance.

The initial text-form backend patch produced EOF-whitespace notices for preserved
raw evidence logs during apply. Files remain byte-identical to fetched source;
the backend .gitattributes marks those sealed logs as binary. Nothing was trimmed
or cleaned. Current whitespace check must respect those attributes.

## Current UI progress and next boundary

UI-5 precision: five exact inspected UI-only probes passed before T1; settings
SHA099fa897a53feee4cb1de483655893f50e6c713c39e66c1a25c8f0549ee2c121 unchanged.
T1 compact timeline:47px versus106px empty idle; two exact UI-only probes and
check/Clippy/fmt pass. Focused speed buffer/track/report/native acceptance stays
open. Main SHA c0f7b12a2b75b95bb4c70da920b343da36658fc6c766b4f156bf51af88e57695
and timeline tests12bee2a70ab3b6cb40a73e183e0786d220ed516761c416bafa7ca097c64352fa
remain exact through backend integration.

The right scene inspector is contract/planning work only: no panel, shared picking,
outline, handles or force authoring UI is implemented yet. Domain source arrival
removes that source dependency. Next admit a bounded T2 write set under the existing
sole UI owner; do not claim backend arrival implements the user's workflow.

Parent confirms UI ownership of main Request/launch/run_material,
desktop/carrier_execution.rs dispatch, desktop/run_sets.rs and
run_sets/{configured.rs,source.rs,configured/archive.rs} frozen-intent/load/rerun.
Backend owns core recording/solver custody. No runtime/request/desktop archive
integration changes yet. Configure/session/provider/free-surface force paths
remain unsupported and must visibly refuse rather than silently approximate.
Preserve recorded identity/provenance and use immutable run snapshots for glyphs.
Native scene/JPEG validation remains unavailable locally; no run/cancel workload
is part of the idle visual handoff.
