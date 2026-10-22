# Independent M3 desktop authoring review

Review date: 2026-10-22. Reviewer: separate agent `/root/voxel_implementation_review`, distinct from the implementer. This completes the first desktop source/input-fixture findings round against the isolated M3 candidate after M2 commit `d0e92b7343599703978b660d3c057cf8602cc98f`. The companion [core review](m3-core-independent-review.md) records resolved core R1/R2. Its disposition does not accept desktop composition.

**Current disposition: desktop R1–R3 are resolved by the targeted rereview below.** No further required source correction was found in that targeted scope. The initial findings remain as review history. M3 is not accepted: its real native idle-edit gate remains blocked by the missing display environment, and M4 must remain gated. No new simulation capability, saved recipe or native Water/Air run is accepted here.

This is static source, contract and test-design review. I ran no test, build, simulation, benchmark or native executable. I read the implementer's startup log but did not reproduce it. Only this report and the previously requested core rereview report were written by this reviewer; source, tests, plan and ledger were not edited. No GitHub or Library action was attempted.

## Complete required findings round

### R1 — Medium: a release anywhere in the batch can release ownership of a later held press

Locations: `InitialContentsUi::canvas`, `initial_contents_ui.rs:366–410,496–500`, and `hidden:73–88`.

The cancellation latch is cleared using a frame-wide `released = events.any(primary release)`. This loses ordering when the frame also contains a later primary press. A concrete sequence is: begin an owned stroke; deliver an invalid pointer move, its release, and another primary press in one event batch, ending with the primary button down. The invalid move cancels the candidate and sets the latch. The end-of-loop branch sees the release anywhere in the batch, clears the latch and breaks before processing the new press. On the next frame the tool has neither a gesture nor a latch although the button is held. It can return `false`, allowing the viewport's primary drag navigation to receive that unowned tail. Starting a frame already latched and receiving release→press produces the same premature release. `hidden` also uses the same unordered release predicate.

Required correction: retain cancellation ownership until the discarded held interaction has actually drained. Use ordered ownership-aware terminal handling, or a conservative release-and-final-button-up condition when discarding the rest of the batch. Keep the terminal frame consumed. Apply the same convention to hidden-page drainage and pre-loop cancellation/shortcut paths; an unrelated earlier release must not unlock panel Undo or a later held tail. Do not replay a cancelled prefix or silently create a new command from the ignored press.

Add real egui fixtures for cancel→release→new press in one batch and already-latched release→new press, including a subsequent held move and final release. Require unchanged committed cells/revision/history, no candidate, ownership retained through the held tail, and no viewport camera navigation until drainage. Include the hidden-page equivalent. The existing coalesced successful press→move→release fixture does not exercise these cases. Owner: desktop gesture adapter; targeted rereview required.

### R2 — Medium: numeric view shortcuts bypass text-field keyboard ownership

Location: `Viewport::input`, `viewport.rs:601–671`, especially Num1/Num3/Num7/Num5.

The viewport snapshots `keyboard_owned = response.has_focus() || !ctx.wants_keyboard_input()` and applies it to Home/F, but each numeric view shortcut ignores it. With a brush Radius or Center text edit focused and the pointer over the viewport, typing 1, 3, 7 or 5 can also reset camera axes or toggle orthographic mode. The new text-local undo router does not protect these separate key consumers. A numeric authoring route must not move the editing view merely because its field contains those digits.

Required correction: apply the same keyboard ownership rule to all view keyboard shortcuts, preserving their behavior when the viewport actually owns keyboard input. Add a fixture with a real focused text/numeric editor, pointer over the real viewport input path, and these numeric key events; camera state and contents must remain unchanged by view shortcuts. Include a positive viewport-owned shortcut case. The existing TextEdit fixture only calls `authoring_history::action` and therefore cannot catch this path. Owner: viewport input adapter; targeted rereview required.

### R3 — Low: numeric brush controls lack associated names and their keyboard route is bypassed by the fixture

Locations: Radius `initial_contents_ui.rs:159–162`; in-plane Centers `239–243`; `owned_pointer_stroke_is_one_command_and_keyboard_dab_is_identical` in the desktop fixtures.

The visible Radius and Center labels are separate from their DragValue responses, with no `labelled_by` association. Rheon's existing named quantity/name controls already use that association. This leaves the new numeric inputs without the explicit control names required by this plan's accessibility route. Separately, the test described as keyboard dab equivalence assigns the private keyboard coordinates and calls private `apply_dab` directly. It verifies the shared data operation, but not reaching/editing those controls or activating Apply dab through keyboard input.

Required correction: associate Radius and both dynamic axis/units labels with their actual input responses using the existing egui pattern. Add a bounded fixture that renders the real controls, exercises their keyboard edit/activation path, and requires the same exact command/cells as the corresponding pointer dab. Verify the input names/units through the existing accessibility output mechanism where available. This needs no new accessibility framework. Actual Tab order, legibility and scaling in the native window remain a separate pending observation. Owner: brush panel; targeted source/fixture rereview required.

### Confirmed correction: force-to-contents handoff retains the old owner

Location: `Viewport::show_scene`, contents branch `viewport.rs:251–269`, and existing desktop fixture coverage.

The latest inspected source now drains a busy `scene_handles` owner with editing disabled before processing Initial contents, and suppresses contents acquisition and navigation when that owner consumes the frame. This corrects the earlier branch that returned before the force owner's cancellation/drain path. I am not retaining that old source defect as an outstanding finding.

At completion of this round I inspected the newly present `viewport::tests::selecting_contents_cancels_previous_force_gesture_before_brush_or_camera`. Unlike the eight isolated contents fixtures, it constructs the canonical Scene and real Viewport, presses a force handle, requires both the Scene edit and handle owner to be active, selects Initial contents, sends a held move and release, then requires cancellation/drainage, no contents candidate/revision and an unchanged camera. The implementer reports nine viewport fixtures passed, including this one. I did not rerun them. This addresses the ordinary composed branch that the earlier isolated fixture list did not cover, and there is no outstanding R4 correction.

The fixture does not yet compare an actually changed force preview against the exact original force or acquire a later fresh contents stroke; those would strengthen the boundary if it changes again. The unusual coalesced terminal ordering remains R1's separate concern. Do not introduce a second force history or broaden this into a new force-tool project. No further source defect is asserted for the ordinary terminal handoff currently inspected.

## Confirmed source boundaries

**One canonical selection and history authority remain.** Initial contents receives a SceneObjectId from the existing Scene allocator and selection is admitted only in Draft. It is not a second force object or a second scene tree. Core editor history alone owns reversible cell changes; desktop `authoring_history` routes named Undo/Redo/Cancel and yields to a focused TextEdit. The panel's Undo cancels a live candidate first, while the cancelled-tail latch blocks subsequent panel commands. Core no-op/cancel preserves redo; a nonempty commit invalidates it. R1/R2 concern composition around those correct core rules.

**The ordinary pointer transaction is atomic.** A valid unmodified primary press owns a frozen brush, camera, canvas rectangle and physical side. Moves call the core swept support predicate; the terminal release must itself pick validly before commit. A missed terminal hit discards the whole candidate. Selection, worker/editability, window/response focus, pointer disappearance, camera/rectangle/side change and hidden-page transitions cancel the private preview rather than publish its last prefix. The source now includes response-focus cancellation and hidden-page routing in the actual App transitions. The first Undo/Escape cancels a live stroke. R1 is the remaining exceptional event-order hole, not a finding that normal terminal commits mutate incrementally.

**Picking is physical, independent of visual surface extraction.** The camera-to-core conversion uses the existing normalized scene camera and physical domain side. Perspective and orthographic origins/rays differ appropriately. The core normalizes the ray; the caller converts its camera-depth near threshold to metric distance using direction magnitude divided by forward rate. It intersects the selected cell-center plane and rejects out-of-domain/parallel/behind/near cases. Rendering clips slice polygons at the camera near depth before projection, and the preview has a slice circle or three volume great circles. Neither mesh appearance, a render threshold nor an SDF supplies authoritative occupancy. Exact changed cells come from the core predicate, not the preview lines. Actual visual usability of inside-volume, near-plane and anisotropic slices is pending native observation; source inspection does not establish it.

**The displayed semantics remain bounded.** Water, Air and Vacuum have explicit text labels as well as colors. Add explains that it fills Vacuum and occupied cells need Replace. Remove does not restore Air. Fill domain with Air is an explicit undoable action. The panel reports per-material committed cell/volume/mass totals, Vacuum count and revision, history/eviction information and the exact candidate changed-cell count. Active-candidate Validate is disabled. Numeric state is checked by the core before publication. A sketch in a slice is binary authoring; there is no native claim of mixed-cell authoring, thermal state or arbitrary compressible gas.

**Running/retained custody and launch gating are respected at this stage.** App supplies editable contents only in Draft with no worker, and Scene controls additionally exclude existing force editing. Retained contexts cannot select the Draft contents ID. `launch_readiness` refuses a live contents gesture/latch and refuses legacy launch while Initial contents is selected; the selected-contents global caption explains why. Simulate contents stays explicitly disabled pending native voxel execution and saved initial input. Hidden-page cancellation occurs before alternate-page early returns. These boundaries prevent an idle brush from being advertised as a working two-material run. M4 still has to prove durable input, immutable run binding and worker/recorder composition; none is inferred here.

## Executed-evidence limits and native blocker

The implementer reports eight final real-egui contents fixtures, eight core contracts, eight Scene regressions, six force-handle regressions and nine viewport fixtures passed, plus formatting and strict desktop all-target Clippy. I inspected their source, not rerun the workloads. The contents harness submits real egui press/move/release events to the contents canvas; it does not itself traverse every Viewport/Scene composition or render the controls for its private `apply_dab` comparison. The separate new viewport fixture supplies the ordinary force-handoff coverage described above. R1–R3 identify the remaining target-bound checks without changing existing assertions or tolerances.

I read `.generated/voxel-material/m3-native-startup.log`. The implementer's actual native startup failed at Winit event-loop creation with: `neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.` The implementer also reports no available Xvfb/weston and an unsuccessful configured-snapshot apt metadata read (proxy 403); this reviewer did not repeat or bypass it. Exact blocked action: the required real native idle authoring workflow for M3, on this environment's desktop binary. Status: environment-blocked, not passed, not an uncertain write or automatic-approval denial. Native rendering, keyboard layout/Tab order, supported display scaling, inside-volume usability and idle paint→undo/redo remain unobserved. Do not mark M3 accepted or start gated M4 on the strength of compilation/headless fixtures alone.

No new numerical cohort, performance claim, deployment, shared-main integration or external publication is authorized by this report. The existing denied integration-coordination comment and uncertain Library uploads were untouched. No reviewer action was denied, pending or uncertain.

## Reviewed identities

The viewport identity below includes the implementer's force-owner drain and formatting visible at completion of this round. These hashes identify source reviewed, not executed native qualification.

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/initial_contents_ui.rs` | `d2b8db98515b1f2475b99c122b3dfc1a6c825d00994029be8c62132bf9fa0384` |
| `src/bin/desktop/initial_contents_ui_tests.rs` | `43f54b7ff839b8d68416fde5dcc1151f7224603ee846e8440e44af494ef8ad00` |
| `src/bin/desktop/authoring_history.rs` | `06e25629a4d4d48e42699a886484c1cd1c31c72475e1b8bf6fc048412b4a0b29` |
| `src/bin/desktop/scene_ui.rs` | `33c17f5632f8ee8e26f3d0c72310f4f3371ca3db198599de8bf3c10c1689ac79` |
| `src/bin/desktop/viewport.rs` | `fe5ae804c8b1c8c5516bca31d54a2aaa145fd9d4ce33c7d5472a98acd43a02a8` |
| `src/bin/rheon-desktop.rs` | `12e151358fcc5d40c2efad0594d4460e7dd693d53e968aa9ffaea9060ba57e0e` |
| `reports/m3-admission.md` | `39e76f6df971cdf102cbab2e79d3866f76b66f637c6d79d2b43991bba79c6ede` |

Core identities and corrected history/ray findings are recorded in the companion report. Complete findings are returned for implementer-owned serial correction, focused execution and targeted independent rereview.

## Correction verification — targeted desktop rereview

On 2026-10-22 the same separate reviewer inspected the implementer's frozen R1–R3 corrections and their focused fixtures. Only this report was updated. I executed no tests/builds/native app and made no external calls. I read the implementer's retained logs `m3-ui-keyboard-corrected2.log` (10 contents fixtures passed) and `m3-viewport-round-corrected.log` (9 viewport fixtures passed), alongside the current source. Those are implementer execution results, not independently repeated native qualification.

**R1 — resolved.** Every discarded-batch release path in `canvas` now preserves the latch when the final primary-button state is down. This includes the entry latch, pre-loop cancellation, shortcut return and cancelled in-loop prefix. Thus an earlier release no longer unlocks a later ignored held press. A successful terminal commit also assigns that final held state to the latch before breaking, so it commits only its owned stroke and drains an ignored subsequent press. `hidden` captures a held tail even without a contents gesture and clears it only for release plus final button-up. The force-consumed viewport branch calls `contents.hidden` before its disabled contents canvas, retaining the discarded tail after the old force owner terminates. Core candidate, command and history ownership are unchanged.

The new `cancelled_batches_keep_ownership_through_release_then_new_held_press` fixture covers three real-egui streams: invalid-move cancellation followed by release/new press in one batch; an existing Escape latch followed by that batch; and a hidden-page batch. Each requires an inactive candidate, a still-busy latch, panel Undo suppression, consumption of the later held move, unchanged exact cells/revision/history capacity, and drainage on the final release. The strengthened composed force-to-contents viewport fixture also delivers release/new press, held movement and final release, checks exact committed forces and camera preservation, and then requires a fresh brush stroke to commit. This closes the earlier missing composition case without adding another force/history owner. It does not constitute an exhaustive proof over arbitrary operating-system pointer delivery.

**R2 — resolved.** Num1/3/5/7 now use the same `keyboard_owned` guard as Home/F. The existing real Viewport/TextEdit fixture focuses the text editor while the pointer is over the viewport and requires an unchanged camera for all four numeric keys. It then surrenders text focus and verifies the viewport-owned Num5 shortcut still toggles orthographic projection. This directly targets the key consumer that the earlier shortcut-router-only fixture bypassed.

**R3 — resolved.** Radius and both dynamically named in-plane Center responses now use `labelled_by` with their visible axis/unit labels. The new panel fixture renders the actual controls, opens Details, resolves each accessibility label association to a SpinButton/TextInput, Tabs to each target, submits keyboard edits for radius 0.2 m and Center X/Y 0.4/0.6 m, and verifies the resulting numeric values. It then Tabs to the actual Apply dab button and activates it with Space. Exact committed cell IDs and the one-command revision are compared with a pointer dab at the same physical point/radius; it does not call private `apply_dab` to stand in for activation.

The first version of that new fixture failed while locating label nodes with `Node.label`. The implementer retained the failed log and changed the helper to read `Node.value`. I inspected the pinned egui 0.33.3 `Response::fill_accesskit_node_from_widget_info`: it explicitly writes Label-role text with `set_value`, while other widget labels use `set_label`. The helper correction follows the dependency's actual output contract; the name association, role, Tab reachability, typed numeric values, actual button activation and exact cell/revision assertions remain present. This is not a weakened expected product result or relaxed numeric tolerance.

The earlier force-handoff observation is now strengthened as described above; there is no outstanding R4. No further required source correction was found in this targeted scope. Core review disposition remains separate. Native rendering/legibility, physical keyboard layout, display scaling and the real idle authoring workflow remain unobserved. The Winit missing-display blocker and M3/M4 gate stated above are unchanged; headless Tab/Space evidence must not be relabelled as successful native interaction.

### Corrected desktop identities

These six source identities plus the unchanged M3 admission identify the targeted rereview candidate. Initial identities above remain historical. After the 10+9 focused execution, strict Clippy required replacing the viewport fixture's default-then-camera assignment with an equivalent struct initializer. I inspected that final initializer; the camera values, default remaining fields, fixture behavior and assertions are unchanged. The final viewport identity below includes this cleanup, while the cited 10+9 logs precede it. Final strict desktop checks were still running when the implementer reported the cleanup; this review does not predeclare their outcome.

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/initial_contents_ui.rs` | `c95fae88aff4ecfc00e46fd2896d57f64e0db81b2d94f5dd6eb22b9ac7c40d18` |
| `src/bin/desktop/initial_contents_ui_tests.rs` | `9fecda06d74e4a34e08bdebb27ed25b2fb9d15c279f1f3b3d561a4d96e989c1f` |
| `src/bin/desktop/authoring_history.rs` | `06e25629a4d4d48e42699a886484c1cd1c31c72475e1b8bf6fc048412b4a0b29` |
| `src/bin/desktop/scene_ui.rs` | `33c17f5632f8ee8e26f3d0c72310f4f3371ca3db198599de8bf3c10c1689ac79` |
| `src/bin/desktop/viewport.rs` | `34e7c9e072a7e11572945a82a57186563da411bb4c39a2713202c59c5527c874` |
| `src/bin/rheon-desktop.rs` | `12e151358fcc5d40c2efad0594d4460e7dd693d53e968aa9ffaea9060ba57e0e` |
| `reports/m3-admission.md` | `39e76f6df971cdf102cbab2e79d3866f76b66f637c6d79d2b43991bba79c6ede` |

No reviewer action was denied, pending or uncertain. The only remaining blocker identified by this targeted review is the previously recorded required native workflow environment; source-review resolution does not remove that gate.
