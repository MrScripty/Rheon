# Independent narrow/scaled timeline source review

## Admission and scope

Source review in `/workspace/Rheon-ui-authoring`, HEAD `5eab3ac4b6d4005ae77b145b3d970b9692debd4c`. The parent relayed native observation at 900×650/150% that the track selector and time captions clip. This reviewer did not observe that native session. Initial-frame native stalls leave picking/handles unverified; this review performs no duplicate native work and makes no claim about those interactions.

Final disposition: suitable for a separate bounded layout-repair checkpoint. The reproduced pure clipping failure is repaired in the focused Context probes, with no material source blocker. Actual native repair verification remains unverified. No runtime/recording qualification follows from this layout result.

Admitted exact name: `timeline_ui_tests::timeline_narrow_scaled_track_and_time_captions_stay_inside_clip`, after list-only enumeration. Complete helper inspection found only local Context/style/AccessKit setup, App default, literal inert Request and PreviewFrame, direct `timeline_ui` measurement/real painting, shape/clip assertions and ordinary in-memory drop. No App update, history creation/capture/view, selected-frame/Inspector path, Worker/start/poll, numerical/model constructor or step, SQLite, output fixture or cleanup is reached. App has no worker/history; its Drop does not join work. The reviewer performed no build, test or native execution and edited only this report.

The probe uses logical sizes 900/scale by 650/scale at 1 and 1.5 pixels per point. The initial source reserved only the left 280 px panel; review identified the missing collapsed Scene reservation. The owner added an inert right 64 px panel, without calling SceneUi, and a panel-bottom assertion against the actual logical height before exact execution. This produces the narrow production reservation: at 150%, 600 logical window width minus 280 and 64 leaves 256 for the timeline. Existing full-shape/clip assertions are retained; banner/screenshot/native rendering is outside this pure probe.

An initial run using `Context::set_pixels_per_point` failed before establishing production clipping: the scheduled zoom can overwrite the first RawInput screen rectangle using the previous default 10000 px content rect. Pinned `ContextImpl::begin_pass` confirms that overwrite. The owner preserved that failed attempt, then changed the probe to set ROOT `ViewportInfo.native_pixels_per_point` on RawInput without scheduling zoom. This literal metadata assignment keeps the helper inert and is admitted under the same exact name. Its resulting cold logical geometry, rather than the artificial scheduled-zoom rectangle, must be used for deciding before/after evidence.

## Deciding baseline and reviewed repair

No production repair existed at initial admission. Read-only receipt of `/tmp/rheon-ui-timeline-scaled-baseline.log` establishes the corrected probe fails on actual Jacobi text clipping: glyph bounds x498.0…573.3 exceed the clip's x536 right edge in the 600×433.3 logical state. The run names only the admitted exact test, with zero passes, one failure and 197 filtered out. This is the deciding production baseline, separate from the preserved scheduled-zoom setup failure.

The pinned egui 0.33.3 ComboBox builds an internal horizontal UI: a selected button cannot move itself to the parent wrapped row. This is the same mechanism already addressed by the density-unit row. The reviewed repair precomputes the complete selected galley against full-row width minus button/icon decorations, explicitly ends the row when the remainder cannot hold it, and sets the matching button width. `.width` alone would only be a minimum. The existing `timeline-track` salt, full discoverable/accessibility text, popup content and playback mutations are unchanged. Galley preparation and `end_row` introduce no widgets, focus state or alternate owner.

The new `timeline_caption` prewraps each complete caption against full row width using the original Body font; the wrapped parent can then move that widget to the next row. This avoids deriving a galley from a tiny remainder. Time units, numerical/report context, copy and font size are preserved. Both invisible measurement and interactive painting call the same contents/helper; hidden sizing AccessKit subtree and single state commit remain. No global App replay or layout-specific playback state is added. WidgetText galley text remains available to label/ComboBox accessibility metadata.

After a repair, re-read the exact production/test diff and preserve cold/warm 1×/1.5× containment evidence. Track popup, focused/invalid speed text and report rows remain separate states; prior wider-window results do not establish the new 256 px layout. Native verification is still required for actual scale/font/GPU behavior and pointer/keyboard usability. Runtime-force preparation remains a separate milestone with its production launch gate; this clipping work does not qualify numerical/archive behavior.

## Evidence status

The corrected deciding failure and production repair source were reviewed. The earlier `/tmp/rheon-ui-timeline-scaled-after1.log` records one exact repaired pass, with 197 filtered out, for the initial Jacobi-only probe. The owner then strengthened the same probe to loop both method identifiers at both scales and set the corresponding literal frame method; that source change was re-inspected and admitted without adding a runtime helper. The final result below covers that strengthened source. No screenshot or native interaction is claimed.

Complete helper paths for these four existing timeline regressions were re-inspected and re-admitted for individual exact execution after enumeration:

- `timeline_ui_tests::timeline_empty_idle_uses_content_height_after_sidebar`
- `timeline_ui_tests::timeline_controls_retain_transitions_and_physical_time`
- `timeline_ui_tests::timeline_speed_edit_focus_bounds_validation_and_keyboard`
- `timeline_ui_tests::timeline_track_selection_and_report_rows_fit_cold_and_changed_states`

They call only local Context, direct timeline helper, App default, literal Request/frame/report values, pointer/key input and shape/AccessKit checks. No History/Inspector/Worker/Viewport/native/model/solver/filesystem fixture or cleanup path is reached; App drop has no worker. Supplied report structs are synthetic presentation values, not workload results.

Final reviewed production source SHA-256: `src/bin/rheon-desktop.rs` = `a704b5dbb9978449187346f4b6eb8f85a9769c96a3f8ddfe85a5dd98f2766282`; strengthened test source `src/bin/desktop/timeline_ui_tests.rs` = `606b0e95edba4a813221af776e7b12f776adcf9645de302ef827aa595111694b`. No source change occurred between the final gate receipt and these hash reads.

Read-only final receipt: `/tmp/rheon-ui-timeline-scaled-list5.log` enumerates 198 tests and zero benchmarks. `final1.log` names the strengthened narrow test; `final2.log` through `final5.log` name the four regressions in the order above. Each records exactly one pass, zero failures, 197 filtered out. The narrow test checks both method labels at 1×/1.5×, cold/warm paints, logical panel-bottom/clip containment and height below 180 logical pixels with unchanged track/no worker/history. The wider regressions cover idle, playback transitions, bounded speed editing/invalid input and supplied report/track popup states. They do not establish every combination of narrow report stress, long configured names, focused editor and popup geometry.

Locked desktop check log records success in 1.15 s and strict production Clippy in 2.82 s. The final fmt log is empty; the owner explicitly confirmed session 63287 completed with exit 0. Reviewer read-only `git diff --check` passed, also confirmed by the owner. No executable, test or build was run by the reviewer.

Final transfer receipt: `evidence/timeline-scaled-20261021/manifest.json`, SHA-256 `1bbfbc8deeb2c40f149af2d9b073640ac74734bc75647ab44efca54ecd50e0cb`, was independently verified against both current source seals and all seventeen copied/raw log pairs by SHA-256 and size. The packet preserves the setup failure, true production baseline, intermediate single-track pass, final five passes, enumeration and static gates. It marks native verification and numerical execution false. No native repair result or unrelated runtime preparation is included in this acceptance.
