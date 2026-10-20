# Workerless retained/timeline presentation harness contract

Authorized by parent after exact6127 native idle validation. Separate from T4
production handles and from archive reconstruction. No existing DB is opened or
reinterpreted as a running App scene. Sole UI source owner; reviewer owns report.

Proposed test-only file desktop/native_presentation_harness.rs, root module under
cfg(test) only. No production CLI flag or default launch. One ordinary exact pure construction/Context probe. A separate standalone
`cargo rustc --locked --offline --features desktop --bin rheon-desktop -- --cfg test`
build WITHOUT --test uses the cfg(test) main-thread wrapper; normal cfg(not(test))
production main is unchanged and has no harness CLI/env switch. Native execution remains with existing validator. Native window900×650 or
1250×820, existing100/150% scale, Close control and cooperative self-close after300seconds
requesting repaint at100ms while idle. External validator retains normal process
supervision; wrapper cannot create worker/child/solver/model/recorder or file output.

Frozen legal Request: size2,steps1,methods[JacobiPcgV1,SymmetricGaussSeidelPcgV1],
threads1,side1m,dt0.25s,preview Edge2,history budget16MiB (planning limit, not allocated
physical solver storage); no material/emitter/model/configured/saved intent,
observer/child/gates/receipt/prior run IDs. Recording directory only a nonempty
unused sentinel string; nominal valid limits65536database/16384record bytes,2records.
Validate metadata only (GridGeometry/BufferPlan arithmetic, no Simulation creation).
History uses existing in-memory bounded LiveHistory::with_resolution and publish;
2tracks×2ordered literal PreviewFrames, edge2 and8densitybytes each,step[0,1],
time[0,0.25]s; report/particle_report/material_color None,seconds0 (no timing claim).
Different fixed display densities distinguish frame and track. No capture helpers.

Use App's existing selected_frame/scene_timeline_ui, not App.update/sidebar/poll/
start/run/rerun or saved/archive views. Default Inspector has no reader; selected
frame selection may report absent archive but must reach no I/O. Wrapper labels
“Synthetic presentation data · no worker or physics” persistently. Scene context
switches pause hidden playback and preserve retained index using the same owner
logic. Advance play_time only while retained/playing, using bounded stable_dt;
track popup and physical timestamps come from the frozen Request/history.

Center may use the existing default Appearance renderer with UI disabled throughout
so no numerical lab mode/control/camera action can be entered. It paints the literal
2³preview only; no workload/evaluator is reachable. This disabled presentation
canvas is explicitly synthetic and does not validate normal viewport gestures,
physics, recording, cancellation, retained force provenance or runtime/archive
parity. Right Scene may select Draft/Retained, hide/reopen; no model exists. Tests
and native operator must not invoke any production run action (none is composed).

Evidence: independently inspect complete construction/render/helper path, list-only
then one individually exact pure UI test. Compile the standalone test-configuration native entry without
executing it here; immediately copy/seal a unique artifact before another build; source/patch seals and exact invocation handed to validator.
Actual native Play/Pause/speed/track/slider/Draft↔Retained tests and quality85JPEGs
must identify synthetic presentation state and source revision. Never describe
these as simulated scientific frames, worker cancellation or archive verification.

Layout/evidence admission: initialize follow_live=false,track/index0 despite last
publication track1; match dark8×7 style and280px sidebar reservation except open
narrow Scene. Re-fetch selected frame after timeline controls as production does.
Keep supplied timestamps and unmeasured calculation caption annotation visible.
No persistence hooks; persist_window=false. Deadline starts once and is cooperative,
not a hard containment claim; native validator supervises process/window closure.
The default native window1250×820 can be manually resized; no size env switch.
