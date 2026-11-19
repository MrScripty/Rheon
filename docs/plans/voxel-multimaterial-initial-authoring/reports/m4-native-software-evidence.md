# M4 actual native software-rendered workflow evidence

Observed on 2026-10-22 with the ordinary **non-test** eframe App at source commit `0059ca035901bc6ca662c4540ec345b31070571e`, not the cfg(test) presentation main. Separate source/procedure admission is in the native source/design reports. This report records observed integration; independent evidence disposition and whole-feature A1–A6 reconciliation remain separate. Normal `AVAILABLE=false` is unchanged; the temporary debug transport is not a release artifact.

## Candidate, environment and process custody

- Build: `cargo build --locked --features desktop --bin rheon-desktop`, pinned Rust1.92; source declaration `0059ca035901bc6ca662c4540ec345b31070571e`, tracked material SHA-256 `50b585746712a5984b7170aed0a6e4519dfa8fd9827003bea15f8afb9e6bb752`. Binary SHA-256 `265de7f1039ffaaddc97862c5ae00bffdc4a05279d2e342439f491ad58bac367`, preserved outside Git as `/workspace/.voxel-display/m4-rheon-desktop`. This is an observed build/declaration, not authentication or a reproducible-build certificate.
- Installed Xorg dummy :97 used a new private0600 Xauthority under the caller-owned0700 evidence directory, no TCP, no VT switch or automatic operator devices. Exact server command/PID is in `m4-display-process.json`; Xorg confirms `swrast`/`DRISWRAST` GLX. Eframe uses its existing Glow feature with explicit `LIBGL_ALWAYS_SOFTWARE=1`. This is actual native software GL, not physical GPU/vendor validation. No package/network metadata, operator desktop, sandbox/security relaxation or shared-memory workaround was used. `/dev/shm` had8.8GiB available at preflight.
- Both actual App processes used `--qualify-voxel-material /workspace/.voxel-display/m4-run-recordings`; no startup document or run was seeded. Window properties bind first PID75385 and fresh PID76304 to title `Rheon / Live 3D` and the preserved binary. XTest performed actual OS pointer/keyboard events; actions/timestamps are in `m4-native-actions.jsonl`. The test audit later creates no App/Worker/solver or step.
- First process exited0 after WM_DELETE_WINDOW. The second was separately launched from the same preserved binary, then also exited0 after the observed cancellation/Inspector workflow. Owned Xorg received SIGTERM only after App close and reports successful termination0. No native process remains required for the evidence.
- Xorg reports unavailable legacy physical-input modules/system D-Bus; XTest was available and actual events worked. Mesa's attempted default shader-cache creation met a read-only home and caching disabled itself; no cache access was retried or security changed. These logs remain in the evidence directory. This observation does not qualify unrelated desktop environments.

## Predeclared state and actual interactions

The independently calculated metric oracle `m4-native-protocol-20261022.json` was written before observation: SHA-256 `960d399b25749a76ec0eaa66c1631ef9910607d7f65fe83668816424e21433f8`. Actual controls created8³ cells at0.125m spacing in a1m closed domain. Fill Air produced512 occupied Air cells. In Volume mode, Replace Water with radius0.35m at `[0.5,0.5,0.5625]` m produced96Water/416Air. Replace Air with radius0.07m at `[0.4375,0.4375,0.5625]` produced95Water/417Air; cell283 is interior and its six direct neighbors remain Water. Actual Undo Contents returned96/416 and Redo Contents returned95/417. These dabs used the real panel's numeric center fields and Apply dab, with OS keyboard/pointer input. Freehand/gesture interruption evidence remains the separately accepted M3 observation, not invented here.

Actual Save draft created `m4-painted-draft.json` (SHA-256 `f6d370b12debbea45d9cdd538c65c09fbee5b5d9c2e1c812f16f26b002a293ef`). Fill Air visibly altered the draft to0Water/512Air; Load draft restored95/417 and fresh contents undo. The App then launched through the enabled global Simulate contents button with actual8 intervals, maximumdt0.001s and gravity `[0,-9.80665,0]` m/s². Constructor acknowledgment appeared, progress was observed and the run completed with9 publications, accepted generation8 and elapsed simulated0.008s. The Inspector showed both phase volumes/masses, the occupied Air pocket on interior Z layer4, numeric cell `[3,3,4]` and original f64 fraction inventory/pressure-stage text. The visible appearance is not an occupancy or solver-state oracle.

Returning to Draft and filling Air changed the live authoring counts to0/512 while the retained Inspector kept the previous accepted phase fields and probe. Hashes of original intent, started manifest, finished manifest and SQLite were captured before this edit and verified unchanged afterwards and after later runs. The original frozen input/constructor is also audited against the saved95/417 document.

After closing PID75385, fresh PID76304 had no draft/history. Through actual Saved run sets path entry/Open, it displayed the original closed9-publication member as Ready/completed. Rerun saved set from zero launched its original recipe through the real Worker, completed8 intervals and showed a new run ID with9 publications. No API seeded this fresh native process. The later audit compares every raw field byte, stage, generation, publication-time/dt bit and field-evaluation-time bit with the original; all match, while attempt IDs/archives differ and child ancestry points to the original finished revision.

One further admitted cancellation attempt loaded the same saved draft through the actual panel, selected1024 maximum intervals,dt0.001s and Zero gravity on the same8³ grid. The **panel** Simulate contents action launched it; capture shows live5/1024. Actual Cancel simulation was clicked; its immediate capture shows37/1024 with the active button. The next observed Inspector confirms `cancelled`,38 publications and accepted generation37, with no unrecorded accepted tail. Start→click/capture took1.416s; the terminal observation occurred15.664s after start, within the predeclared45s bound. This is lifecycle observation, not a speed benchmark. There was no second cancellation attempt or extra numerical cohort.

Actual1250×820 and900×650 windows were inspected at100% and150% application UI zoom (five built-in Ctrl+Plus increments of0.1; Ctrl+0 reset). At compact logical width the existing top run strip replaces the settings sidebar; Scene scroll positions expose brush/details/file/Validate/Simulate. Long quantities wrap and local path fields scroll horizontally. Captures show reachable file actions and concise controls in the inspected positions; not every control is simultaneously visible. This is application zoom, not monitor-DPI/accessibility certification. Settings/Inspector navigation and the qualification banner remain visibly distinct. One immediate resize capture has a transient unpainted strip; the subsequent settled capture is the deciding compact100% view.

## Typed closed-artifact audit

The separate agent's audit review found the existing SQLite reader permits recovery. Before execution, the helper was corrected to refuse recovery sidecars/errors/symlinks, verify original pinned database hashes before open, and recheck all databases after all selections under the caller-owned clean/no-writer contract. No sidecar repair, pin, publication or numerical step occurs. All producers/display were closed before audit. This is not an OS read-only descriptor or a hostile-filesystem race guarantee.

Explicit command: `cargo test --locked --features desktop --bin rheon-desktop voxel_material_setup::native_tests::audit::actual_native_artifacts_match_metric_draft_and_cold_replay -- --exact --ignored --nocapture --test-threads=1`, with paths in `m4-audit-inputs.json`. Result: **1 passed**,0 failed,0 ignored. Log: ignored `.generated/voxel-material/m4-native-artifact-audit.log`. It verified exact512 saved IDs/constructor F, captured Water/Air coefficient bits, original schedules, typed field/pressure clocks/gauge, closed pins, immutable ancestry, exact cold f64 replay and genuine cancelled prefix. All supplied JSON/draft/oracle and all three pinned SQLite hashes remained unchanged over the audit.

| Attempt | Outcome | Publications / accepted generation | Maximum independently summed phase-volume drift m³ | Maximum derived mass drift kg |
| --- | --- | --- | --- | --- |
| Native baseline | completed |9 /8 |`[1.38778e-16,8.88178e-16]` |`[1.42109e-13,1.11022e-15]` |
| Fresh native rerun | completed |9 /8 |identical |identical |
| Actual Cancel | cancelled |38 /37 |`[0,0]` |`[0,0]` |

Initial phase volumes are `[0.185546875,0.814453125]` m³ and masses `[185.212890625,0.9807064977009284]` kg; occupied gas is positive, not Vacuum. Drift comparisons use the existing step-contract ceiling plus independent reduction allowance; they do not reconstruct internal producer budgets or establish a new accuracy theorem. Raw recorded controls are printed as producer declarations, not parsed scientific validation. This bounded two-phase integration/replay check is not arbitrary-grid/long-horizon/material qualification, measured RSS, physical realism, performance or physical-GPU acceptance.

## Local artifacts and integrity

All below are under `/workspace/.voxel-display/`; generated pixels/databases/binary/authentication are outside Git. No Library write, upload, push, PR, merge or deployment occurred. Authentication contents are omitted. The native source remains at0059ca0; later audit-only cfg(test) files do not alter the qualified non-test binary.

| Local evidence | SHA-256 |
| --- | --- |
| `m4-build-identity.json` | `5ea439e576887718d60bfe46c86e8c2848175d926b6ebd32607a94d82bbbd4cc` |
| `m4-display-process.json` | `39708b696b99fa231e6acb6568a6a12b981bc970a66595ce661873f7f65c7aa5` |
| `m4-window-first.txt` | `3399fb3f2af555db46d7ed953cf7279e1dfea6e256d943e597554d73475bad70` |
| `m4-window-second.txt` | `30c337a80450617cede10e9cc4b198e1e5034b3bb578790eb03ed69a5f56bc2a` |
| `m4-native-actions.jsonl` | `8fe1e0c04dd85ad14b38683dec30c22855893eb8b44663251411b015661ea47f` |
| `m4-cancel-observation.json` | `edbf186d017b1fce36727656143577be906391e07fa431b493d3afa6820613da` |
| `m4-audit-inputs.json` | `89a0d3d4c8abb3948e30d1e3410d4ec6fb68d0e21d173004a5f1cc3132936368` |
| `m4-baseline-before-draft-edit.json` | `04361762a6bdff1ea60d764aead2818ef989af9f64017e9ff30bc05c698767bf` |
| `m4-app-first.log` | `580e65b8c5729582d2fdc93853e7bc2a52837b3c049ca16c697c0d3cd78a9a30` |
| `m4-app-second.log` | `580e65b8c5729582d2fdc93853e7bc2a52837b3c049ca16c697c0d3cd78a9a30` |
| `m4-xorg.log` | `13c9a3f9dd376f0b0d4a7843dfdc8300c2767a017485b082fa7b9b63f9637311` |
| `m4-native-input.py` | `743a7175da1ebe5a391f4eda23a274ea75e547f49b0dab0772e21f5bab377da9` |
| `m4-act.py` | `d6819b5b0235560b0b76ae971036e337ba70354cedb468f42859e8614bc7b7f8` |
| `m4-01-initial.png` | `e9f68dfd6dd66d95488eff172ea2e0fa806a1a63ca7b437422c06bad30b11a01` |
| `m4-02-created.png` | `bb455333a2b895c54edd1d2309ab7e4e7c5da6289ed984e98c8e03bb6ffd7e18` |
| `m4-03-brush-ready.png` | `95a7b44ea33ed1d141919a7678e0e9b60897315802f16ea937984510280ed30d` |
| `m4-04-water96.png` | `7af86903ca425195b6adfc6a5c21e3d3c7103b92779ef0d0e5566acb0b4af41a` |
| `m4-05-air-menu.png` | `e1fc9d74d2f5147c48e74b9547c14a788301f95a6ce5583fcbf1403d51af3097` |
| `m4-06-interior-air95.png` | `5ff82ad8a8360cf81439f1b8b31e8e4a8720d3e870dc7d0d02614a8a0d4373fa` |
| `m4-07-undo96.png` | `e50b048680254b7a30a52093347ed67dbedaadd46b388e3b0dcf76dec03fce0c` |
| `m4-08-redo95.png` | `f39844d65e5574b3ba9700671f9d01421e2f4f42bdcb3e7ef92e2f08cff6625d` |
| `m4-09-draft-file-controls.png` | `9c844ff153ac5178cc158f5f92adfeec8ffd2b1f3a865cb6dde85f8f73a83d65` |
| `m4-10-draft-saved.png` | `e9cac493815b5d77d9f440432be39783c15f1b2c865f93d59e6e59fa67a6c8c2` |
| `m4-11-altered-after-save.png` | `d3656d03550096dd1880396c280a89b9efd74ef05e8ddc3d94e83da997ab2d7d` |
| `m4-12-reloaded95.png` | `2f16f6c3bf77904003825f565427ba3b0fbaac16082c72c93817a8272dc63d0d` |
| `m4-13-run-entry.png` | `31443fe8b2b23b0396686f06b4e8b25b0f4b57f7ca1c3304a9a7af6c5f79095f` |
| `m4-14-complete-inspector.png` | `41657c609b5b0118968e4ec9bf2c4ea6b00a9ca8a7513efd3f58325da9486a06` |
| `m4-15-both-phase-slice.png` | `9c1db303302d0193c6753720940192039cacdbe7b006b70abecaad244e7a03d7` |
| `m4-16-interior-phase-and-raw.png` | `2194a12d0b3a6269f92bf0c199ffc393362e0b3130f29edf8dca9272a945e5be` |
| `m4-17-interior-pocket-probe.png` | `92bdf6ff13e2fc7c760ad8b55178567cee9b11adf6623f5454cd72597e66c176` |
| `m4-18-return-draft.png` | `b1f67bffd9180fe12816ab56914c322f27e1f78b9b670a40b4da7f14db1abf82` |
| `m4-19-draft-edit-retained-fields.png` | `e5015fa9de437c14e59977f406c00b1be17d102b013caee90a2c399686226eab` |
| `m4-20-compact100.png` | `8a3b2fd6bab7597cd5fd4547bbdcba5ff64a7ddf30fad955cd754d162ed8eb8c` |
| `m4-21-compact100-scrolled.png` | `6d0be7425829a69d789a95eaaf887f75c9212587b7a7348b438f07ea38cb94df` |
| `m4-22-compact150.png` | `55e833a79ee6050e283e0449ea85086a14293d1b656ca015092ad275a8cab1cb` |
| `m4-23-compact150-files.png` | `581c49a1a5c39aecf9debc7bb3a8957d9822a988f91e990a6e3f70cb5e8aba21` |
| `m4-24-wide150-top.png` | `21894e9414aebc3e604c82a58b2c0c78ce1c0a08b7205a0734c814a1f306ae53` |
| `m4-25-fresh-process.png` | `ce62f0ec8a6d3787973e08f077ab54513eda34fbdd066f06395c48337125385e` |
| `m4-26-open-saved-controls.png` | `5d77be89fe615ee9e18d03e46bc4467b655376f3f57d7c8846fdc90bbf428b16` |
| `m4-27-cold-saved-ready.png` | `cd19526e836167b149b58004845318ec808f615c6a13e89d05cc7da76d10e61e` |
| `m4-28-cold-rerun-entry.png` | `d856c18a3cd8533f948da862a664d1e864112bb0e4ab3395398b17a0f93e44c7` |
| `m4-29-cold-rerun-complete.png` | `88c1ea0aaf362cc878a7b7b1c40325aa4fc4ece5b1b2dd324c505f0f0bf0b2d0` |
| `m4-30-cancel-load-controls.png` | `f4266f36e2943598b9424176cee201cd8b8749f968b0ee1e8d098e8d2f4c0abf` |
| `m4-31-cancel-draft-loaded.png` | `80c365b41b1e97892cf5219dd06e056d48b39e16a12a4c7f90cd5ae84fdc8a16` |
| `m4-32-cancel-protocol-ready.png` | `1cdc38803d000eb9052b2849b83b346bbaaacd43cc869715f0a46fd99483ac6f` |
| `m4-33-cancel-observed-active.png` | `1863ea62a4513001a092a1272244e072f0abac50a4e02952b94fa1060337a3ac` |
| `m4-34-cancel-clicked.png` | `2d89621d7831bae9e077c4bc7f3976025b3f02a5f8aed39a99eda73fc75e1071` |
| `m4-35-cancel-closed-inspector.png` | `1ba6c889a13d8cbfb841692356996b11486eb2b276f6927bd2cea1a6feda63d3` |

Closed outputs (paths relative to the evidence directory):

| Output | SHA-256 |
| --- | --- |
| `m4-run-recordings/voxel-set-1792653644216796927-75385-0.voxelset/finished.json` | `6da0c7d3c84518d9acc4385c4f9a3b6ef9cc18819a3a058f686524a88a10ac65` |
| `m4-run-recordings/voxel-set-1792653644216796927-75385-0.voxelset/intent.json` | `fecd4178f858ad6a609ce63dd9ad649e897819dc4e348ef1cab5eac72314b65e` |
| `m4-run-recordings/voxel-set-1792653644216796927-75385-0.voxelset/run.sqlite` | `b24a87f866daab692e72f293d7e46bb6225a60f7ce61da20e5a23f0c3dbd223d` |
| `m4-run-recordings/voxel-set-1792653644216796927-75385-0.voxelset/started.json` | `e85a02d754b09175d1ef48832a05afe8d3330c70efd581f41857ce7e0e47c02e` |
| `m4-run-recordings/voxel-set-1792653981745308154-76304-0.voxelset/finished.json` | `cb2a48912c2842bee20cfafadd3f04d73537460f68afc4bf8f5f2f8dd8f49b9b` |
| `m4-run-recordings/voxel-set-1792653981745308154-76304-0.voxelset/intent.json` | `d5bb831e71e71f6998c3ce4476188aea4b0beef1e7d13b089b124ca032be48aa` |
| `m4-run-recordings/voxel-set-1792653981745308154-76304-0.voxelset/run.sqlite` | `0733120fa07ab385523a33cd1e265dd1399f201401db26b3bc001446a2dadf75` |
| `m4-run-recordings/voxel-set-1792653981745308154-76304-0.voxelset/started.json` | `7ffe97e1b0924162d19dcda4b5aad77e038999b7178a0bcd66972002b9929997` |
| `m4-run-recordings/voxel-set-1792654091867157602-76304-1.voxelset/finished.json` | `140a54ada667045eb8e1b6bfc033a3b72403a65e37e573e03233ce764fdb3007` |
| `m4-run-recordings/voxel-set-1792654091867157602-76304-1.voxelset/intent.json` | `6c2dea781df24eaba93a1c32ad85a04391187b0e972f40a0a39cb9d0a7f8272c` |
| `m4-run-recordings/voxel-set-1792654091867157602-76304-1.voxelset/run.sqlite` | `e1662f8f80a6d01c53992c16507e7bef8fdb3ee2329f33632321dff9ddb771f2` |
| `m4-run-recordings/voxel-set-1792654091867157602-76304-1.voxelset/started.json` | `8e8f716ee7a63b9af28d7ea8490aa1475906aa0c8173291f21c7b92c48e06e45` |

Ignored build/test logs, relative to the implementation worktree:

| Log | SHA-256 |
| --- | --- |
| `.generated/voxel-material/m4-native-build.log` | `00557abbcb51f2e1641f6a13455d6bd554b3495ea2da6c3044866e6eab39cb97` |
| `.generated/voxel-material/m4-native-audit-compile.log` | `732358e60e6f98ac7e1d98ecfca6950dca18a26dc3de89ba848f13fa255fd498` |
| `.generated/voxel-material/m4-native-artifact-audit.log` | `3bb0d63496bf58a8067202ae83dea8fe4f726ce373fe64f086d1131b9e11fc6d` |
| `.generated/voxel-material/m4-native-final-clippy.log` | `f55ab222f15dc6426633bdd1603b274d61e5092f5b66d71e1d2570bd3aa9af75` |

Next gate: separate actual-evidence review and A1–A6/M5 reconciliation. Normal execution and saved-voxel rerun remain unavailable outside the explicit debug qualification session. Temporary transport removal, reviewed normal enablement and final normal-artifact smoke remain required before whole-feature exposure. Main/integration hold and historical denied/uncertain-action restrictions remain in force.
