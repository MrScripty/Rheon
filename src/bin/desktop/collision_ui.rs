//! Draft import settings and transactional immutable collision admission.
use eframe::egui;
use rheon::{CollisionMesh, MeshPolicy};
use std::{path::Path, sync::Arc};

pub struct CollisionControls {
    pub path: String,
    pub scale: f64,
    pub translation: [f64; 3],
    pub solid: bool,
    pub mesh: Option<Arc<CollisionMesh>>,
    pub status: String,
}
impl Default for CollisionControls {
    fn default() -> Self {
        Self {
            path: String::new(),
            scale: 1.0,
            translation: [0.0; 3],
            solid: true,
            mesh: None,
            status: "No collision mesh".into(),
        }
    }
}
pub fn validate_bounds(mesh: &CollisionMesh, side: f64) -> Result<(), String> {
    let (lo, hi) = mesh.bounds();
    if (0..3).any(|d| lo[d] < 0.0 || hi[d] > side) {
        return Err(format!(
            "Mesh bounds {lo:?} … {hi:?} m must fit inside [0, {side}] m on every axis"
        ));
    }
    Ok(())
}
impl CollisionControls {
    pub fn apply(&mut self, side: f64) -> Result<(), String> {
        let imported = rheon::mesh_import::import_mesh(
            Path::new(&self.path),
            rheon::mesh_import::MeshPlacement {
                scale: self.scale,
                translation: self.translation,
            },
        )
        .map_err(|e| e.to_string())?;
        if imported.triangles.len() > 12_000 {
            return Err(
                "Native viewer limit is 12,000 triangles; simplify this mesh before importing"
                    .into(),
            );
        }
        let mesh = CollisionMesh::new(
            imported.vertices,
            imported.triangles,
            if self.solid {
                MeshPolicy::ConvexSolid
            } else {
                MeshPolicy::TwoSidedSurface
            },
        )
        .map_err(|e| e.to_string())?;
        validate_bounds(&mesh, side)?;
        self.status = format!(
            "{} triangles · {:.2} MiB · bounds {:?}\n{}",
            mesh.triangles().len(),
            mesh.allocated_bytes() as f64 / 1048576.0,
            mesh.bounds(),
            imported.diagnostics.join("\n")
        );
        self.mesh = Some(Arc::new(mesh));
        Ok(())
    }
    /// True means accepted geometry changed; caller clears the old timeline.
    pub fn show(&mut self, ui: &mut egui::Ui, side: f64, material_mode: bool) -> bool {
        let mut changed = false;
        egui::CollapsingHeader::new("Collision mesh (OBJ / glTF / GLB)").default_open(true).show(ui, |ui| {
            ui.label("Local model path");
            ui.text_edit_singleline(&mut self.path);
            ui.horizontal(|ui| {
                ui.label("Metres per source unit");
                ui.add(egui::DragValue::new(&mut self.scale).speed(0.01));
            });
            for (d, label) in ["Translation X", "Translation Y", "Translation Z"].iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(*label);
                    ui.add(egui::DragValue::new(&mut self.translation[d]).speed(0.01).suffix(" m"));
                });
            }
            ui.radio_value(&mut self.solid, true, "Validated convex watertight solid");
            ui.radio_value(&mut self.solid, false, "Two-sided surface barrier (no solid interior)");
            ui.small("Static particle-center collisions only; preview voxels can straddle the boundary. No calibrated wall pressure/shear or moving-body coupling. glTF units are metres; OBJ units must be specified above.");
            if !material_mode { ui.colored_label(egui::Color32::YELLOW, "Collision meshes require liquid / gas material simulation."); }
            ui.horizontal(|ui| {
                if ui.add_enabled(material_mode, egui::Button::new("Import / apply mesh")).clicked() {
                    match self.apply(side) {
                        Ok(()) => changed = true,
                        Err(e) => self.status = format!("Import rejected; previous mesh retained: {e}"),
                    }
                }
                if ui.add_enabled(self.mesh.is_some(), egui::Button::new("Remove mesh")).clicked() {
                    self.mesh = None;
                    self.status = "Mesh removed".into();
                    changed = true;
                }
            });
            ui.small(&self.status);
            if let Some(mesh) = &self.mesh {
                if let Err(e) = validate_bounds(mesh, side) { ui.colored_label(egui::Color32::YELLOW, e); }
                ui.small("Amber wireframe: accepted collision mesh, shown through fluid. Input edits apply only after Import / apply mesh. Applying or removing clears recorded history.");
            }
        });
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_import_reject_remove_and_repeat_are_transactional() {
        let path =
            std::env::temp_dir().join(format!("rheon-native-collision-{}.obj", std::process::id()));
        std::fs::write(
            &path,
            "v 0.2 0.3 0.2\nv 0.8 0.3 0.2\nv 0.2 0.3 0.8\nf 1 2 3\n",
        )
        .unwrap();
        let mut controls = CollisionControls {
            path: path.to_string_lossy().into_owned(),
            solid: false,
            ..Default::default()
        };
        let ctx = egui::Context::default();
        ctx.style_mut(|s| s.animation_time = 0.0);
        let render = |controls: &mut CollisionControls, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 1000.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        controls.show(ui, 1.0, true);
                    });
                },
            )
        };
        let click = |controls: &mut CollisionControls, label: &str| {
            render(controls, vec![]);
            let out = render(controls, vec![]);
            let pos = out
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == label => {
                        Some(t.pos + t.galley.size() * 0.5)
                    }
                    _ => None,
                })
                .unwrap();
            for pressed in [true, false] {
                render(
                    controls,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::default(),
                        },
                    ],
                );
            }
        };
        click(&mut controls, "Import / apply mesh");
        let accepted = controls.mesh.clone().unwrap();
        controls.scale = 10.0;
        click(&mut controls, "Import / apply mesh");
        assert!(controls.status.contains("previous mesh retained"));
        assert!(Arc::ptr_eq(controls.mesh.as_ref().unwrap(), &accepted));
        controls.scale = 1.0;
        controls.solid = true;
        click(&mut controls, "Import / apply mesh");
        assert!(Arc::ptr_eq(controls.mesh.as_ref().unwrap(), &accepted));
        controls.solid = false;
        click(&mut controls, "Remove mesh");
        assert!(controls.mesh.is_none());
        click(&mut controls, "Import / apply mesh");
        assert_eq!(
            controls.mesh.as_ref().unwrap().triangles(),
            accepted.triangles()
        );
        std::fs::remove_file(path).unwrap();
    }
}
