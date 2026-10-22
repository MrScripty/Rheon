use super::*;
use rheon::{GridGeometry, PressureSettings, voxel_material::*};
#[test]
fn typed_voxel_selection_survives_later_evolution_and_failed_selection_clears_it() {
    let path = std::env::temp_dir().join(format!(
        "rheon-voxel-inspector-{}.sqlite",
        std::process::id()
    ));
    assert!(!path.exists());
    let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
    let mut fractions = [0.0; 64];
    fractions[21] = 1.0;
    let mut sim = VoxelMaterialSimulation::new(
        grid,
        MaterialPair::water_air(),
        &fractions,
        VoxelMaterialConfig {
            gravity: [0.0; 3],
            pressure: PressureSettings {
                relative_residual: 1e-11,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            memory_limit: 64 * 1024 * 1024,
        },
    )
    .unwrap();
    let run = rheon::recording::voxel_material_run(
        SessionIdentity {
            run_id: 41,
            source_revision: "inspector-test".into(),
        },
        &sim,
        [19; 32],
        "fixture",
    )
    .unwrap();
    let limits = RecordingLimits {
        max_database_bytes: 1024 * 1024,
        max_record_bytes: 128 * 1024,
        max_records: 2,
    };
    let mut writer = StepRecorder::create(&path, run, limits).unwrap();
    writer
        .append(
            &rheon::recording::voxel_material_publication(&sim, 0, limits.max_record_bytes)
                .unwrap(),
        )
        .unwrap();
    let inspector = Inspector::default();
    inspector.reload(&path, limits).unwrap();
    inspector.select_timestep(0).unwrap();
    inspector
        .with_voxel(|v| {
            assert_eq!(v.liquid_fraction[21], 1.0);
            assert_eq!(v.liquid_fraction.iter().sum::<f64>(), 1.0);
            assert!(v.metadata.materials.gas().density() > 0.0);
            assert_eq!(v.generation, 0);
        })
        .unwrap();
    sim.step(0.01, |_| false).unwrap();
    writer
        .append(
            &rheon::recording::voxel_material_publication(&sim, 1, limits.max_record_bytes)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(inspector.with_voxel(|v| v.generation), Some(0));
    let ctx = egui::Context::default();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| inspector.ui(ui));
    });
    assert!(!output.shapes.is_empty());
    assert!(inspector.select_timestep(9).is_err());
    assert!(inspector.with_voxel(|v| v.generation).is_none());
    inspector.select_timestep(1).unwrap();
    assert_eq!(inspector.with_voxel(|v| v.generation), Some(1));
    writer
        .finish("completed", "inspection fixture", Some(1))
        .unwrap();
    drop(writer);
    drop(inspector);
    std::fs::remove_file(path).unwrap();
}

fn panel(ctx: &egui::Context, inspector: &Inspector, events: Vec<egui::Event>) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(500.0, 1400.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| inspector.ui(ui));
        },
    )
}
fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn named(output: &egui::FullOutput, label: &str) -> egui::accesskit::NodeId {
    let tree = output.platform_output.accesskit_update.as_ref().unwrap();
    let label = tree
        .nodes
        .iter()
        .find(|(_, n)| n.value() == Some(label))
        .unwrap()
        .0;
    let (id, node) = tree
        .nodes
        .iter()
        .find(|(_, n)| n.labelled_by().contains(&label))
        .unwrap();
    assert!(matches!(
        node.role(),
        egui::accesskit::Role::SpinButton | egui::accesskit::Role::TextInput
    ));
    *id
}
#[test]
fn actual_named_keyboard_probe_and_rendered_mixed_phase_totals_use_captured_fields() {
    let path = super::tests::test_path("voxel-mixed");
    let grid = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
    let mut fractions = [0.0; 64];
    fractions[20] = 0.5;
    let pair = MaterialPair::new(
        MaterialProperties::new(3.0, 0.001).unwrap(),
        MaterialProperties::new(2.0, 0.002).unwrap(),
    );
    let sim = VoxelMaterialSimulation::new(
        grid,
        pair,
        &fractions,
        VoxelMaterialConfig {
            gravity: [0.0; 3],
            pressure: PressureSettings {
                relative_residual: 1e-11,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            memory_limit: 64 * 1024 * 1024,
        },
    )
    .unwrap();
    let run = rheon::recording::voxel_material_run(
        SessionIdentity {
            run_id: 42,
            source_revision: "mixed-inspector-test".into(),
        },
        &sim,
        [20; 32],
        "generic nonreference fixture",
    )
    .unwrap();
    let limits = RecordingLimits {
        max_database_bytes: 1024 * 1024,
        max_record_bytes: 128 * 1024,
        max_records: 1,
    };
    let mut writer = StepRecorder::create(&path, run, limits).unwrap();
    writer
        .append(
            &rheon::recording::voxel_material_publication(&sim, 0, limits.max_record_bytes)
                .unwrap(),
        )
        .unwrap();
    writer
        .finish("failed", "constructor-only display fixture", Some(0))
        .unwrap();
    drop(writer);
    let inspector = Inspector::default();
    inspector.reload(&path, limits).unwrap();
    inspector.select_timestep(0).unwrap();
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    ctx.style_mut(|s| s.animation_time = 0.0);
    let output = panel(&ctx, &inspector, vec![]);
    let ids = [named(&output, "Cell j"), named(&output, "Cell k")];
    for id in ids {
        let mut reached = false;
        for _ in 0..80 {
            if ctx
                .memory(|m| m.focused())
                .is_some_and(|focus| focus.value() == id.0)
            {
                reached = true;
                break;
            }
            panel(
                &ctx,
                &inspector,
                vec![
                    key(egui::Key::Tab, egui::Modifiers::NONE),
                    egui::Event::Key {
                        key: egui::Key::Tab,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        assert!(
            reached,
            "named numeric cell probe must be keyboard reachable"
        );
        panel(
            &ctx,
            &inspector,
            vec![key(
                egui::Key::A,
                egui::Modifiers {
                    ctrl: true,
                    command: true,
                    ..Default::default()
                },
            )],
        );
        panel(&ctx, &inspector, vec![egui::Event::Text("1".into())]);
        panel(
            &ctx,
            &inspector,
            vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
        );
    }
    let output = panel(&ctx, &inspector, vec![]);
    let text = output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) => Some(t.galley.text()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains(&format!(
            "Liquid: {:.9e} m³ · {:.9e} kg",
            0.0078125, 0.0234375
        )),
        "{text}"
    );
    assert!(
        text.contains(&format!(
            "Occupied gas: {:.9e} m³ · {:.9e} kg",
            0.9921875, 1.984375
        )),
        "{text}"
    );
    assert!(
        text.contains("Cell [0, 1, 1] · center [0.125, 0.375, 0.375] m"),
        "{text}"
    );
    assert!(
        text.contains("Liquid F=5.00000000000000000e-1 · Occupied gas F=5.00000000000000000e-1"),
        "{text}"
    );
    assert!(!text.contains("Water") && !text.contains("Air"));
    let s = inspector.0.lock().unwrap();
    assert_eq!(s.field_name, "liquid_fraction");
    assert_eq!(s.scalar_index, 20);
    let record = s.record.as_ref().unwrap();
    let field = record
        .fields
        .iter()
        .position(|f| f.name == "liquid_fraction")
        .unwrap();
    let csv = field_csv(42, record, field, s.scalar_index).unwrap();
    assert!(csv.contains(",\"liquid_fraction\",20,0.5,\"1\","), "{csv}");
    drop(s);
    drop(inspector);
    std::fs::remove_file(path).unwrap();
}
