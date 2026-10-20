use super::*;

#[test]
fn c1b2_projection_profile_refusal_clears_cached_g1_evidence_and_gesture() {
    use rheon::projection_step::{ProjectionIntent, ProjectionStepWorkspace};
    use rheon::recording::provider::runtime::RuntimeTerminationCause;
    use rheon::recording::{RecordingLimits, StepRecorder, projection};
    use rheon::*;
    let output = crate::configured_worker_tests::Output::new();
    let (old_attempt, old_session) = observed_fixture(false);
    let old_path = output.0.join("old-g1.sqlite");
    let limits = RecordingLimits::default();
    let run =
        rheon::recording::provider::configured_run(&old_session.snapshot(), "retained G1 source")
            .unwrap();
    let mut writer = StepRecorder::create(&old_path, run, limits).unwrap();
    writer
        .append(
            &rheon::recording::workflow::configured_publication_with_workflow(
                &old_session.snapshot(),
                0,
                limits.max_record_bytes,
                None,
            )
            .unwrap(),
        )
        .unwrap();
    writer
        .finish("completed", "prior G1 source", Some(1))
        .unwrap();
    drop(writer);

    for terminal_only in [false, true] {
        let cfg = *old_session.snapshot().configuration();
        let grid = old_session.snapshot().grid().clone();
        let mut owner = SimulationSession::new_with_projection(
            old_session.snapshot().identity().clone(),
            grid.clone(),
            cfg.transport,
            cfg.implementation,
            vec![1.0; grid.cell_len()],
            cfg.boundary_workspace_limit,
            cfg.max_steps,
            ProjectionIntent::BuiltinNativeV1,
        )
        .unwrap();
        let mut workspace = ProjectionStepWorkspace::new(
            &grid,
            cfg.transport.carrier.density,
            4 * 1024 * 1024,
            cfg.implementation,
        )
        .unwrap();
        let path = output.0.join(format!("projection-{terminal_only}.sqlite"));
        let run =
            projection::projection_run(&owner.snapshot(), "new typed projection source").unwrap();
        let mut writer = StepRecorder::create(&path, run, limits).unwrap();
        if !terminal_only {
            writer
                .append(
                    &projection::projection_publication(
                        &owner.snapshot(),
                        0,
                        limits.max_record_bytes,
                    )
                    .unwrap(),
                )
                .unwrap();
            let accepted = owner
                .step_with_native_projection(
                    LiquidStepInputs {
                        requested_dt: 0.001,
                        smoke_source: None,
                        forces: &[],
                        source: None,
                        inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3])
                            .unwrap(),
                        volume: LiquidVolumeSettings {
                            max_outward_courant: 1.0,
                            actual_divergence_limit: 1e-5,
                        },
                    },
                    &mut workspace,
                    None,
                    |_| false,
                )
                .unwrap();
            writer
                .append(
                    &projection::projection_publication(
                        &owner.snapshot(),
                        1,
                        limits.max_record_bytes,
                    )
                    .unwrap(),
                )
                .unwrap();
            projection::ProjectionTermination::capture(
                &owner.snapshot(),
                RuntimeTerminationCause::Completed,
                "accepted projection",
                Some(&accepted.attempt),
            )
            .unwrap()
            .finish(&mut writer)
            .unwrap();
        } else {
            projection::ProjectionTermination::capture(
                &owner.snapshot(),
                RuntimeTerminationCause::Cancelled,
                "cancelled before any numerical attempt",
                None,
            )
            .unwrap()
            .finish(&mut writer)
            .unwrap();
        }
        drop(writer);
        drop(workspace);
        drop(owner);
        let inspector = Inspector::default();
        inspector.reload(&old_path, limits).unwrap();
        inspector.select_timestep(0).unwrap();
        let mut h = Harness::new();
        h.view.select(Some(&inspector));
        assert!(h.view.attempt.is_some());
        h.view.evidence = true;
        h.view.pan_drag = Some((egui::Pos2::ZERO, egui::Vec2::ZERO));
        let refusal = inspector.reload(&path, limits).unwrap_err();
        assert!(refusal.contains("typed projection reader"));
        assert!(inspector.workflow_selection().unwrap().is_none());
        assert!(inspector.workflow_identity().is_none());
        h.view.select(Some(&inspector));
        assert!(h.view.attempt.is_none() && h.view.source.is_none());
        assert!(!h.view.evidence && h.view.pan_drag.is_none());
        assert!(!h.view.message.contains("legacy"));
        assert!(old_attempt.publication.is_some());
        // The namespace refusal also covers unsupported future versions, without
        // changing historical raw-profile inspection for unrelated namespaces.
        let future = output
            .0
            .join(format!("future-projection-{terminal_only}.sqlite"));
        let source = rheon::recording::RecordingReader::open(&path, limits).unwrap();
        let mut future_run = source.run().clone();
        future_run.solver = "rheon-native-projection-session-v2".into();
        drop(source);
        let mut future_writer = StepRecorder::create(&future, future_run, limits).unwrap();
        future_writer
            .finish("cancelled", "unsupported future profile fixture", Some(0))
            .unwrap();
        drop(future_writer);
        assert!(
            inspector
                .reload(&future, limits)
                .unwrap_err()
                .contains("typed projection reader")
        );
    }
}

fn fixture() -> Arc<WorkflowAttempt> {
    observed_fixture(false).0
}
fn observed_fixture(cancel_before: bool) -> (Arc<WorkflowAttempt>, rheon::SimulationSession) {
    use rheon::*;
    const CAP: usize = 4 * 1024 * 1024;
    let grid = GridGeometry::new([2; 3], [0.25; 3], [0.0; 3]).unwrap();
    let mut session = SimulationSession::new(
        SessionIdentity {
            run_id: 42,
            source_revision: "native-workflow-interaction-fixture".into(),
        },
        grid.clone(),
        LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 2.0,
                memory_limit: CAP,
                pressure: PressureSettings {
                    relative_residual: 1e-10,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-8,
                    max_iterations: 100,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 0.5,
            },
            represented_density: 2.0,
            volume_stamp: VolumeStamp { id: 1, version: 0 },
            carrier_id: 2,
        },
        PressureImplementation::JacobiPcgV1,
        vec![1.0; 8],
        CAP,
        3,
    )
    .unwrap();
    let mut workspace =
        ProviderStepWorkspace::new(&grid, 2.0, CAP, PressureImplementation::JacobiPcgV1).unwrap();
    let mut viscosity = ViscosityWorkspace::new(grid, CAP).unwrap();
    let forces = [BodyForce {
        value: [0.125, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    let mut evidence = None;
    let result = session.step_with_configured_pressure_observed(
        LiquidStepInputs {
            requested_dt: 0.0625,
            smoke_source: None,
            forces: &forces,
            inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
            source: None,
            volume: LiquidVolumeSettings {
                max_outward_courant: 0.5,
                actual_divergence_limit: 1e-5,
            },
        },
        &mut workspace,
        Some((&mut viscosity, 0.0)),
        None,
        None,
        |_| cancel_before,
        7,
        &mut evidence,
    );
    assert_eq!(result.is_err(), cancel_before);
    (evidence.unwrap(), session)
}

struct Harness {
    context: egui::Context,
    view: WorkflowInspector,
    time: f64,
}
impl Harness {
    fn new() -> Self {
        let mut view = WorkflowInspector::default();
        view.set_attempt(fixture());
        view.open();
        Self {
            context: egui::Context::default(),
            view,
            time: 0.0,
        }
    }
    fn frame(&mut self, events: Vec<egui::Event>) -> egui::FullOutput {
        self.frame_on(events, egui::vec2(1500.0, 1200.0))
    }
    fn frame_on(&mut self, events: Vec<egui::Event>, size: egui::Vec2) -> egui::FullOutput {
        self.time += 0.016;
        self.context.run(
            egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ctx| self.view.show(ctx),
        )
    }
    fn click(&mut self, id: &str) -> egui::FullOutput {
        self.frame(vec![]);
        self.frame(vec![]);
        let pos = self
            .view
            .controls
            .get(id)
            .unwrap_or_else(|| panic!("missing {id}"))
            .center();
        self.at(pos)
    }
    fn at(&mut self, pos: egui::Pos2) -> egui::FullOutput {
        self.frame(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ]);
        self.frame(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }])
    }
    fn key(&mut self, key: egui::Key) -> egui::FullOutput {
        self.frame(vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }]);
        self.frame(vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Default::default(),
        }]);
        self.frame(vec![]);
        self.frame(vec![])
    }
}
fn contains(output: &egui::FullOutput, text: &str) -> bool {
    output.shapes.iter().any(
        |shape| matches!(&shape.shape,egui::Shape::Text(t) if t.galley.job.text.contains(text)),
    )
}
#[test]
fn evidence_overflow_scrolls_to_lower_hashes_and_keeps_close_on_screen() {
    use crate::configured_worker_tests::{Output, app, finish, launch, package, set};
    use rheon::solver_provider::configured::ContextProviderIdentity;
    let output = Output::new();
    let provider = package(false, "bad_once");
    let mut application = app(&output);
    let root = output.0.join("evidence-overflow");
    launch(&mut application, set(&provider), &root);
    let revision = finish(&mut application);
    let member = revision
        .members
        .iter()
        .find(|member| member.member_id == "configured")
        .unwrap();
    let inspector = Inspector::default();
    inspector
        .reload(
            &root.join(&member.attempt.as_ref().unwrap().archive_relative_path),
            rheon::recording::RecordingLimits::default(),
        )
        .unwrap();
    inspector.select_timestep(2).unwrap();
    let retained = inspector
        .workflow_selection()
        .unwrap()
        .unwrap()
        .attempt
        .unwrap();
    let mut h = Harness::new();
    let mut attempt = (*retained).clone();
    // Presentation boundary test: lengthen bounded identity text on a real
    // configured fallback receipt; numerical and execution facts stay intact.
    attempt.pressure.as_mut().unwrap().binding = Some(ContextProviderIdentity {
        id: "bounded provider identity ".repeat(35),
        version: "1".into(),
        manifest_sha256: [41; 32],
        checkpoint_sha256: Some([42; 32]),
        training_run_id: Some("retained training identity ".repeat(30)),
        runtime: "retained runtime ".repeat(50),
        source_revision: "retained source ".repeat(50),
    });
    attempt.validate().unwrap();
    h.view.set_attempt(Arc::new(attempt));
    h.view.evidence = true;
    let size = egui::vec2(1000.0, 600.0);
    for _ in 0..3 {
        h.frame_on(vec![], size);
    }
    let close = h.view.controls["close-evidence"];
    assert!(
        egui::Rect::from_min_size(egui::Pos2::ZERO, size).contains_rect(close),
        "footer must remain reachable on a small native viewport: {close:?}"
    );
    let scroll = h.view.controls["evidence-scroll"];
    assert!(scroll.height() <= 440.0);
    let visible_rhs = |out: &egui::FullOutput| {
        out.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains("Input RHS SHA-256") && shape.clip_rect.intersect(scroll).intersect(egui::Rect::from_min_size(text.pos,text.galley.size())).is_positive())
    })
    };
    assert!(
        !visible_rhs(&h.frame_on(vec![], size)),
        "lower RHS begins outside the bounded evidence viewport"
    );
    h.frame_on(
        vec![
            egui::Event::PointerMoved(scroll.center()),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -10000.0),
                modifiers: Default::default(),
            },
        ],
        size,
    );
    let mut lower = false;
    for _ in 0..5 {
        lower |= visible_rhs(&h.frame_on(vec![], size));
    }
    assert!(
        lower,
        "real wheel scrolling must reach lower retained RHS evidence"
    );
    let pos = h.view.controls["close-evidence"].center();
    h.frame_on(
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
        ],
        size,
    );
    h.frame_on(
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
        size,
    );
    assert!(!h.view.evidence);
    assert!(h.view.open);
}

#[test]
fn pointer_keyboard_focus_evidence_close_and_reopen_use_real_egui_events() {
    let mut h = Harness::new();
    h.frame(vec![]);
    h.frame(vec![]);
    h.click("node-1");
    assert_eq!(h.view.selected, 1);
    let out = h.key(egui::Key::ArrowRight);
    assert_eq!(h.view.selected, 2);
    assert!(contains(&out, "Requested dt"));
    assert!(contains(&out, "6.250000000e-2 s"));
    h.key(egui::Key::End);
    assert_eq!(
        h.view.attempt.as_ref().unwrap().descriptor.nodes()[h.view.selected],
        NodeKind::Publication
    );
    for _ in 0..4 {
        h.key(egui::Key::ArrowLeft);
    }
    assert_eq!(
        h.view.attempt.as_ref().unwrap().descriptor.nodes()[h.view.selected],
        NodeKind::NativeFallback
    );
    h.key(egui::Key::Home);
    assert_eq!(h.view.selected, 0);
    h.click("evidence");
    assert!(h.view.evidence);
    assert!(contains(&h.frame(vec![]), "Input RHS SHA-256"));
    h.click("close");
    assert!(!h.view.open);
    h.view.open();
    h.frame(vec![]);
    assert!(h.view.open);
    assert_eq!(h.view.attempt.as_ref().unwrap().attempt_id, 7);
    h.click("node-0");
    h.key(egui::Key::Escape);
    assert!(!h.view.open);
    assert!(!h.view.evidence);
}
#[test]
fn missing_and_invalid_sources_clear_the_previous_graph_and_no_node_status_is_inferred() {
    let mut h = Harness::new();
    assert_eq!(
        status(
            h.view.attempt.as_ref().unwrap(),
            NodeKind::PressureCandidate
        ),
        "execution unavailable"
    );
    for _ in 0..3 {
        h.frame(vec![]);
    }
    let canvas = h.view.controls["canvas"];
    let background = egui::pos2(canvas.right() - 25.0, canvas.top() + 25.0);
    h.frame(vec![
        egui::Event::PointerMoved(background),
        egui::Event::PointerButton {
            pos: background,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
    ]);
    assert!(h.view.pan_drag.is_some());
    h.view.select(None);
    assert!(
        h.view.pan_drag.is_none(),
        "source invalidation drops an in-progress background gesture"
    );
    assert!(h.view.attempt.is_none());
    assert!(h.view.source.is_none());
    h.frame(vec![]);
    assert!(contains(&h.frame(vec![]), "no exact recorded publication"));
    let mut bad = (*fixture()).clone();
    bad.descriptor_sha256[0] ^= 1;
    h.view.set_attempt(Arc::new(bad));
    assert!(h.view.attempt.is_none());
    assert!(
        h.view
            .message
            .contains("Invalid workflow execution evidence: workflow descriptor digest")
    );
}
#[test]
fn actual_worker_archive_projection_survives_provider_removal_and_cached_paint_needs_no_source() {
    use crate::configured_worker_tests::{Output, app, finish, launch, package, set};
    let output = Output::new();
    let provider = package(false, "bad_once");
    let mut application = app(&output);
    let root = output.0.join("workflow");
    launch(&mut application, set(&provider), &root);
    let revision = finish(&mut application);
    let member = revision
        .members
        .iter()
        .find(|m| m.member_id == "configured")
        .unwrap();
    let attempt = member.attempt.as_ref().unwrap();
    let archive = root.join(&attempt.archive_relative_path);
    drop(application);
    drop(provider);
    let inspector = Inspector::default();
    inspector
        .reload(&archive, rheon::recording::RecordingLimits::default())
        .unwrap();
    inspector.select_timestep(2).unwrap();
    let mut h = Harness::new();
    h.view.select(Some(&inspector));
    let retained = h
        .view
        .attempt
        .clone()
        .expect("actual accepted workflow receipt");
    assert_eq!(retained.publication, Some(2));
    assert!(retained.pressure.as_ref().unwrap().rejection.is_some());
    assert!(
        retained
            .nodes
            .iter()
            .any(|n| n.node == NodeKind::NativeFallback && n.status == NodeStatus::Selected)
    );
    drop(inspector);
    std::fs::rename(&archive, archive.with_extension("retained-outside-source")).unwrap();
    h.frame(vec![]);
    let out = h.frame(vec![]);
    assert!(contains(&out, "Actual pressure provider:"));
    assert_eq!(
        h.view.attempt.as_ref().unwrap().descriptor_sha256,
        retained.descriptor_sha256
    );
}

/// This prepares retained inputs only. Native qualification still operates the
/// ordinary application through its real display, pointer and keyboard paths.
#[test]
#[ignore = "explicit native smoke fixture preparation; requires exclusive temporary root"]
fn prepare_native_workflow_fixture() {
    use crate::configured_worker_tests::{Output, app, finish, launch, package, set};
    use sha2::{Digest, Sha256};
    let root = std::path::PathBuf::from(
        std::env::var_os("RHEON_WORKFLOW_NATIVE_ROOT").expect("explicit temporary output root"),
    );
    assert!(root.starts_with(std::env::temp_dir()));
    std::fs::create_dir(&root).unwrap();
    let mut provider = package(false, "bad_once");
    let retained_provider = root.join("provider");
    std::fs::rename(&provider.root, &retained_provider).unwrap();
    provider.root = retained_provider;
    provider.manifest = provider.root.join("adapter.json");
    let code_path = provider.root.join("adapter.py");
    let code = std::fs::read_to_string(&code_path).unwrap().replace(
        "    result = solve(a, rhs)",
        "    if (root / 'slow').exists(): time.sleep(0.4)\n    result = solve(a, rhs)",
    );
    std::fs::write(code_path, &code).unwrap();
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&provider.manifest).unwrap()).unwrap();
    manifest["files"]["adapter.py"]["bytes"] = serde_json::json!(code.len());
    manifest["files"]["adapter.py"]["sha256"] =
        serde_json::json!(format!("{:x}", Sha256::digest(code.as_bytes())));
    std::fs::write(&provider.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let mut revision = set(&provider);
    for member in &mut revision.members {
        member.spec.constructor.max_steps = 64;
        member.spec.future.requested_steps = 64;
        member.spec.future.recording.max_records = 65;
        member.spec = crate::run_sets::configured::FrozenConfiguredSpec::new(
            member.spec.constructor.clone(),
            member.spec.future.clone(),
        )
        .unwrap();
    }
    revision.validate().unwrap();
    let output = Output::new();
    let mut application = app(&output);
    launch(&mut application, revision, &root.join("set"));
    let completed = finish(&mut application);
    std::fs::write(root.join("manifest-path.txt"), &application.run_sets.path).unwrap();
    assert_eq!(completed.members.len(), 2);
    drop(application);
    std::fs::remove_file(provider.root.join("first-request-observed")).unwrap();
    std::fs::write(
        provider.root.join("slow"),
        b"bounded native interaction delay",
    )
    .unwrap();
    // The external native procedure owns this retained disposable root and its
    // eventual cleanup; the provider must remain available for the live rerun.
    std::mem::forget(provider);
    println!("NATIVE_WORKFLOW_FIXTURE_READY {}", root.display());
}

#[test]
fn interval_replacement_preserves_node_focus_pan_zoom_and_text_loop_equivalence() {
    let mut h = Harness::new();
    h.frame(vec![]);
    h.frame(vec![]);
    h.click("node-2");
    h.view.zoom = 1.1;
    h.view.pan = egui::vec2(-30.0, -30.0);
    let focused = h.context.memory(|memory| memory.focused());
    let mut next = (*fixture()).clone();
    next.attempt_id = 8;
    next.accepted_generation = 1;
    next.attempted_generation = 2;
    next.publication = Some(2);
    next.interval_start = 0.0625;
    h.view.set_attempt(Arc::new(next));
    h.frame(vec![]);
    assert_eq!(h.view.selected, 2);
    assert_eq!(h.view.zoom, 1.1);
    assert_eq!(h.view.pan, egui::vec2(-30.0, -30.0));
    assert_eq!(h.context.memory(|memory| memory.focused()), focused);
    h.key(egui::Key::ArrowRight);
    assert_eq!(h.view.selected, 3);
    h.view.pan = egui::vec2(-30.0, -280.0);
    let output = h.frame(vec![]);
    assert!(contains(&output, "pressure iteration loop"));
    h.click("reset");
    assert_eq!(h.view.zoom, 0.85);
    assert_eq!(h.view.pan, egui::Vec2::ZERO);
    h.click("node-2");
    h.key(egui::Key::ArrowRight);
    assert_eq!(h.view.selected, 3);
}

#[test]
fn existing_carrier_and_material_archives_remain_selectable_with_workflow_unavailable() {
    use rheon::*;
    let root = crate::configured_worker_tests::Output::new();
    let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
    let carrier = Simulation::new(
        grid.clone(),
        SimulationConfig {
            density: 2.0,
            memory_limit: 4 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-10,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 100,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 0.5,
        },
    )
    .unwrap();
    let material = ParticleFluidSimulation::new(
        grid,
        ParticleFluidConfig {
            fill: DomainFill::EmittedFluid,
            samples_per_axis: 4,
            max_particles: 64,
            ..Default::default()
        },
    )
    .unwrap();
    for (name, is_material) in [("carrier", false), ("material", true)] {
        let inspector = Inspector::default();
        let path = root.0.join(format!("{name}.sqlite"));
        let writer = if is_material {
            inspector
                .begin(
                    &material,
                    &path,
                    rheon::recording::RecordingLimits::default(),
                )
                .unwrap()
        } else {
            inspector
                .begin_carrier(
                    &carrier,
                    &path,
                    rheon::recording::RecordingLimits::default(),
                )
                .unwrap()
        };
        drop(writer);
        inspector.select_timestep(0).unwrap();
        let mut view = WorkflowInspector::default();
        view.select(Some(&inspector));
        assert!(view.attempt.is_none());
        assert!(view.source.is_some());
        assert!(view.message.contains("legacy archive"));
    }
}

#[test]
#[ignore = "release-only frozen G1 capture plus cached native projection protocol"]
fn paired_capture_and_cached_projection_overhead_protocol() {
    use rheon::*;
    use std::time::Instant;
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "run optimized release protocol"
    );
    const CAP: usize = 16 * 1024 * 1024;
    let make = |run_id| {
        let grid = GridGeometry::new([16; 3], [1.0 / 16.0; 3], [0.0; 3]).unwrap();
        let session = SimulationSession::new(
            SessionIdentity {
                run_id,
                source_revision: "g1-fixed-overhead-fixture".into(),
            },
            grid.clone(),
            LiquidTransportConfig {
                carrier: SimulationConfig {
                    density: 998.2,
                    memory_limit: CAP,
                    pressure: PressureSettings {
                        relative_residual: 1e-10,
                        absolute_residual: 1e-12,
                        divergence_limit: 1e-8,
                        max_iterations: 2000,
                    },
                    actual_divergence_limit: 1e-5,
                    max_courant: 1.0,
                },
                represented_density: 998.2,
                volume_stamp: VolumeStamp { id: 1, version: 0 },
                carrier_id: 2,
            },
            PressureImplementation::JacobiPcgV1,
            vec![0.5; 16usize.pow(3)],
            CAP,
            1000,
        )
        .unwrap();
        let workspace =
            ProviderStepWorkspace::new(&grid, 998.2, CAP, PressureImplementation::JacobiPcgV1)
                .unwrap();
        (session, workspace)
    };
    let (mut baseline, mut bw) = make(100);
    let (mut observed, mut ow) = make(101);
    let force = [BodyForce {
        value: [0.001, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    let inputs = || LiquidStepInputs {
        requested_dt: 0.001,
        smoke_source: None,
        forces: &force,
        source: None,
        inlet: LiquidInlet::new(VolumeStamp { id: 3, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    };
    let state_bits = |session: &SimulationSession| {
        let snapshot = session.snapshot();
        let state = snapshot.state();
        let mut bits = vec![
            state.carrier.generation,
            state.carrier.time.to_bits(),
            state.liquid.time.to_bits(),
            state.liquid.density.to_bits(),
            state.carrier_stamp.id,
            state.carrier_stamp.version,
            state.liquid.stamp.id,
            state.liquid.stamp.version,
        ];
        for field in [
            state.carrier.x,
            state.carrier.y,
            state.carrier.z,
            state.carrier.tracer,
        ] {
            bits.extend(field.iter().map(|v| v.to_bits() as u64));
        }
        for field in [state.pressure, state.liquid.fraction] {
            bits.extend(field.iter().map(|v| v.to_bits()));
        }
        bits
    };
    let mut view = WorkflowInspector::default();
    let mut pairs = Vec::new();
    for interval in 0..35u64 {
        let mut ns = [0u128; 2];
        for mode in if interval % 2 == 0 { [0, 1] } else { [1, 0] } {
            let start = Instant::now();
            if mode == 0 {
                baseline
                    .step_with_pressure_provider(inputs(), &mut bw, None, None, |_| false)
                    .unwrap();
            } else {
                let mut evidence = None;
                observed
                    .step_with_configured_pressure_observed(
                        inputs(),
                        &mut ow,
                        None,
                        None,
                        None,
                        |_| false,
                        interval + 1,
                        &mut evidence,
                    )
                    .unwrap();
                view.set_attempt(evidence.unwrap());
                assert!(view.attempt.is_some());
            }
            ns[mode] = start.elapsed().as_nanos();
        }
        assert_eq!(state_bits(&baseline), state_bits(&observed));
        if interval >= 5 {
            pairs.push(ns);
        }
    }
    let mut ratios: Vec<_> = pairs
        .iter()
        .map(|pair| pair[1] as f64 / pair[0] as f64 - 1.0)
        .collect();
    ratios.sort_by(f64::total_cmp);
    let median = (ratios[14] + ratios[15]) * 0.5;
    println!("G1_CAPTURE_AND_CACHED_PROJECTION_PAIRED_NS={pairs:?}");
    println!("G1_MEDIAN_CAPTURE_AND_CACHED_PROJECTION_OVERHEAD={median:.8}");
    println!(
        "SCOPE=actual capture plus cached native WorkflowInspector::set_attempt; excludes disk I/O and rasterization"
    );
    assert!(
        median <= 0.05,
        "fixed <=5% capture+projection gate failed; retain paired evidence and investigate"
    );
}

#[test]
fn ordered_text_focus_uses_stable_node_ids_and_canvas_controls_accept_pointer_events() {
    let mut h = Harness::new();
    h.frame(vec![]);
    h.frame(vec![]);
    let zoom = h.view.controls["zoom"];
    h.at(egui::pos2(zoom.left() + 25.0, zoom.center().y));
    assert_ne!(h.view.zoom, 0.85);
    let rect = h.view.controls["canvas"];
    let start = rect.left_bottom() + egui::vec2(18.0, -18.0);
    let end = start + egui::vec2(25.0, -70.0);
    h.frame(vec![
        egui::Event::PointerMoved(start),
        egui::Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
    ]);
    h.frame(vec![egui::Event::PointerMoved(end)]);
    h.frame(vec![egui::Event::PointerButton {
        pos: end,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    }]);
    assert_ne!(h.view.pan, egui::Vec2::ZERO);
    h.click("reset");
    assert_eq!(h.view.pan, egui::Vec2::ZERO);
    assert_eq!(h.view.zoom, 0.85);
    // The same restoration transition used by keyboard navigation scrolls the
    // complete text list into view before actual pointer activation.
    h.view.focus_text = true;
    h.view.restore_focus = true;
    h.view.selected = 2;
    h.frame(vec![]);
    h.frame(vec![]);
    h.frame(vec![]);
    h.click("text-2");
    assert!(h.view.focus_text);
    h.key(egui::Key::ArrowRight);
    assert_eq!(h.view.selected, 3);
    h.frame(vec![]);
    h.key(egui::Key::End);
    assert_eq!(
        h.view.attempt.as_ref().unwrap().descriptor.nodes()[h.view.selected],
        NodeKind::Publication
    );
    assert!(h.view.focus_text);
}

#[test]
fn verified_comparison_sides_load_exact_workflows_and_pair_or_source_changes_clear_them() {
    use crate::{
        comparison_archive::{SavedRecipe, Side},
        configured_worker_tests::{Output, app, finish, launch, package, set},
        run_sets::source::SavedRunSet,
        saved_comparison::*,
    };
    use rheon::recording::comparison::{FieldRole, PressurePolicy, Recipe};
    let output = Output::new();
    let provider = package(false, "bad_once");
    let mut application = app(&output);
    let root = output.0.join("comparison-workflow");
    launch(&mut application, set(&provider), &root);
    let revision = finish(&mut application);
    drop(application);
    drop(provider);
    let recipe = SavedRecipe::capture_source(
        &root,
        SavedRunSet::Configured(revision.clone()),
        "native",
        "configured",
        &[2, 3],
        &[2, 3],
        Recipe {
            field: FieldRole::IntervalPressure,
            time_tolerance: 0.0,
            pressure: PressurePolicy::Canonical,
        },
    )
    .unwrap();
    recipe.save_new(&root.join("comparison.json")).unwrap();
    let mut page = SavedComparisonPage::default();
    page.set_location(root.to_string_lossy(), "ignored-manifest.json");
    page.open_saved_recipe();
    page.compute();
    assert_eq!(
        page.presentation(),
        Presentation::Verified,
        "{}",
        page.message()
    );
    let mut view = WorkflowInspector::default();
    let mut run_ids = Vec::new();
    for side in [Side::Reference, Side::Candidate] {
        view.terminal_request = true;
        view.load_comparison(page.load_selected_workflow(side));
        assert!(!view.terminal_request);
        assert!(view.comparison_context);
        let key = page.workflow_key(side).unwrap();
        view.validate_comparison(Some(&key));
        let attempt = view.attempt.as_ref().unwrap();
        assert_eq!(attempt.publication, Some(2));
        run_ids.push(attempt.run_id);
    }
    assert_ne!(run_ids[0], run_ids[1]);
    let ctx = egui::Context::default();
    let mut frame = |events| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1800.0, 2600.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| page.show(ui));
            },
        )
    };
    frame(vec![]);
    frame(vec![]);
    let rect = page.control_rect("matched-pair").unwrap();
    let pos = egui::pos2(rect.left() + 95.0, rect.center().y);
    let mut frame = |events| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1800.0, 2600.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| page.show(ui));
            },
        )
    };
    frame(vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
    ]);
    frame(vec![egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    }]);
    frame(vec![egui::Event::Key {
        key: egui::Key::ArrowRight,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }]);
    assert_eq!(page.selected_pair().unwrap().pair.candidate.ordinal, 3);
    let key = page.workflow_key(Side::Candidate);
    view.validate_comparison(key.as_ref());
    assert!(view.attempt.is_none());
    assert!(!view.evidence);
    view.load_comparison(page.load_selected_workflow(Side::Candidate));
    assert_eq!(view.attempt.as_ref().unwrap().publication, Some(3));
    page.set_location(root.join("new-namespace").to_string_lossy(), "missing.json");
    view.validate_comparison(page.workflow_key(Side::Candidate).as_ref());
    assert!(view.attempt.is_none());
    page.set_location(root.to_string_lossy(), "ignored.json");
    page.open_saved_recipe();
    page.compute();
    let archive = root.join(
        &revision.members[1]
            .attempt
            .as_ref()
            .unwrap()
            .archive_relative_path,
    );
    std::fs::rename(&archive, archive.with_extension("unavailable")).unwrap();
    view.load_comparison(page.load_selected_workflow(Side::Candidate));
    assert!(view.attempt.is_none());
    assert!(view.message.contains("Invalid or unavailable"));
}

#[test]
fn generation_zero_cancelled_terminal_loads_via_native_action_without_an_accepted_row() {
    use rheon::recording::{
        RecordingLimits, StepRecorder,
        provider::{configured_run, runtime::RuntimeTerminationCause},
        workflow::WorkflowTermination,
    };
    let output = crate::configured_worker_tests::Output::new();
    let path = output.0.join("zero.sqlite");
    let (attempt, session) = observed_fixture(true);
    let run = configured_run(
        &session.snapshot(),
        "actual generation-zero cancelled workflow fixture",
    )
    .unwrap();
    let mut writer = StepRecorder::create(&path, run, RecordingLimits::default()).unwrap();
    WorkflowTermination::capture(
        &session.snapshot(),
        RuntimeTerminationCause::Cancelled,
        "actual cancellation before numerical publication",
        None,
        Some(&attempt),
    )
    .unwrap()
    .finish(&mut writer)
    .unwrap();
    assert_eq!(writer.recorded_count(), 0);
    drop(writer);
    drop(session);
    let inspector = Inspector::default();
    inspector.reload(&path, RecordingLimits::default()).unwrap();
    let mut h = Harness::new();
    h.view.select(Some(&inspector));
    h.frame(vec![]);
    h.frame(vec![]);
    h.click("terminal");
    assert!(h.view.terminal_request);
    h.view.select(Some(&inspector));
    let selected = h.view.attempt.as_ref().unwrap();
    assert_eq!(selected.accepted_generation, 0);
    assert_eq!(selected.attempted_generation, 1);
    assert!(selected.publication.is_none());
    assert_eq!(selected.outcome, rheon::workflow::AttemptOutcome::Cancelled);
    h.frame(vec![]);
    assert!(contains(&h.frame(vec![]), "Cancelled"));
    h.view.select(Some(&inspector));
    assert!(h.view.terminal_mode);
    assert!(h.view.attempt.is_some());
}

#[test]
fn cold_sized_end_and_coalesced_pointer_key_keep_selection_after_idle_frames() {
    let size = egui::vec2(1600.0, 1000.0);
    let render = |h: &mut Harness, events| {
        h.time += 0.016;
        h.context.run(
            egui::RawInput {
                time: Some(h.time),
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::TopBottomPanel::top("archive-navigation-fixture")
                    .exact_height(134.0)
                    .show(ctx, |ui| {
                        ui.label("Exact recorded ordinal 2");
                    });
                h.view.show(ctx);
            },
        )
    };
    for coalesced in [false, true] {
        let mut h = Harness::new();
        for _ in 0..5 {
            render(&mut h, vec![]);
        }
        render(
            &mut h,
            vec![egui::Event::PointerMoved(egui::pos2(87.0, 990.0))],
        );
        let pan_before = h.view.pan;
        let pos = h.view.controls["node-1"].center();
        render(
            &mut h,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Default::default(),
                },
            ],
        );
        assert_eq!(
            h.view.pan, pan_before,
            "node pointer press must not start background pan"
        );
        let release = egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        };
        let end = egui::Event::Key {
            key: egui::Key::End,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        };
        if coalesced {
            render(&mut h, vec![release, end]);
        } else {
            render(&mut h, vec![release]);
            for _ in 0..2 {
                render(&mut h, vec![]);
            }
            assert_eq!(h.view.selected, 1);
            render(&mut h, vec![end]);
        }
        render(
            &mut h,
            vec![egui::Event::Key {
                key: egui::Key::End,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Default::default(),
            }],
        );
        for _ in 0..6 {
            render(&mut h, vec![]);
        }
        assert_eq!(
            h.view.attempt.as_ref().unwrap().descriptor.nodes()[h.view.selected],
            NodeKind::Publication,
            "coalesced={coalesced}"
        );
        for _ in 0..4 {
            for pressed in [true, false] {
                render(
                    &mut h,
                    vec![egui::Event::Key {
                        key: egui::Key::ArrowLeft,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers: Default::default(),
                    }],
                );
            }
            for _ in 0..3 {
                render(&mut h, vec![]);
            }
        }
        assert_eq!(
            h.view.attempt.as_ref().unwrap().descriptor.nodes()[h.view.selected],
            NodeKind::NativeFallback,
            "coalesced={coalesced}"
        );
    }
}

#[test]
fn only_background_press_owns_pan_and_prepress_motion_is_excluded() {
    let mut h = Harness::new();
    for _ in 0..4 {
        h.frame(vec![]);
    }
    let button = |pos, pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    let outside = egui::pos2(5.0, 1190.0);
    h.frame(vec![egui::Event::PointerMoved(outside)]);
    let node = h.view.controls["node-1"].center();
    let before = h.view.pan;
    h.frame(vec![egui::Event::PointerMoved(node), button(node, true)]);
    assert_eq!(
        h.view.pan, before,
        "moving onto a node before its press is not a background gesture"
    );
    let moved = node + egui::vec2(40.0, 30.0);
    h.frame(vec![egui::Event::PointerMoved(moved)]);
    assert_eq!(
        h.view.pan, before,
        "a node-origin held motion cannot steal background panning"
    );
    h.frame(vec![button(moved, false)]);
    let canvas = h.view.controls["canvas"];
    let background = egui::pos2(canvas.right() - 25.0, canvas.top() + 25.0);
    h.frame(vec![egui::Event::PointerMoved(outside)]);
    h.frame(vec![
        egui::Event::PointerMoved(background),
        button(background, true),
    ]);
    assert_eq!(
        h.view.pan, before,
        "background press ignores preceding outside-pointer relocation"
    );
    let delta = egui::vec2(-40.0, 30.0);
    h.frame(vec![egui::Event::PointerMoved(background + delta)]);
    assert_eq!(
        h.view.pan,
        before + delta,
        "held background movement still pans by its actual displacement"
    );
    h.frame(vec![button(background + delta, false)]);
    h.frame(vec![egui::Event::PointerMoved(outside)]);
    assert_eq!(
        h.view.pan,
        before + delta,
        "released drag has no continuing ownership"
    );
    h.view.open();
    assert!(h.view.pan_drag.is_none());
    assert_eq!(
        h.view.pan,
        before + delta,
        "reopening preserves the view without a stale drag"
    );
}
