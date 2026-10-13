//! Small live transport consumer of the public session API. It does not replace
//! the separate carrier-emitter or particle backends.
use super::{engineering::EngineeringView, export_ui::ExportFile, identity};
use eframe::egui;
use rheon::*;
pub struct TransportView {
    session: SimulationSession,
    pub view: EngineeringView,
    requested_dt: f64,
    flow: f32,
    playing: bool,
    status: String,
    export: ExportFile,
}
impl TransportView {
    pub fn new() -> Result<Self, String> {
        let grid = GridGeometry::new([8, 4, 2], [0.125, 0.25, 0.375], [-0.5, 0.25, 1.0])
            .map_err(|e| e.to_string())?;
        let mut fraction = vec![0.0; grid.cell_len()];
        for z in 0..2 {
            for y in 0..4 {
                for x in 2..4 {
                    fraction[grid.cell_index([x, y, z]).unwrap()] = 0.5;
                }
            }
        }
        let config = LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 1000.0,
                memory_limit: 16 * 1024 * 1024,
                pressure: PressureSettings {
                    relative_residual: 1e-10,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-7,
                    max_iterations: 2000,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 1.0,
            },
            represented_density: 800.0,
            volume_stamp: VolumeStamp { id: 1, version: 0 },
            carrier_id: 2,
        };
        let session = SimulationSession::new(
            identity::new_identity("transport"),
            grid,
            config,
            PressureImplementation::JacobiPcgV1,
            fraction,
            16 * 1024 * 1024,
            512,
        )
        .map_err(|e| e.to_string())?;
        let view = EngineeringView::from_session(&session.snapshot());
        Ok(Self {
            session,
            view,
            requested_dt: 0.0625,
            flow: 0.25,
            playing: false,
            status: String::new(),
            export: ExportFile::new(".generated/transport.vti"),
        })
    }
    fn step(&mut self) -> Result<(), String> {
        let boundary = BoxFluxStepBoundary {
            flux: PrescribedBoxFlux::new(
                BoxFluxStamp {
                    id: 3,
                    version: self.session.snapshot().frame() as u64,
                },
                [[-self.flow, self.flow], [0.0; 2], [0.0; 2]],
            )
            .map_err(|e| e.to_string())?,
            tracer: BoxFluxTracerPolicy::ClampedAppearance,
        };
        let inputs = LiquidStepInputs {
            requested_dt: self.requested_dt,
            smoke_source: None,
            forces: &[],
            source: None,
            inlet: LiquidInlet::new(VolumeStamp { id: 4, version: 0 }, [[0.0; 2]; 3])
                .map_err(|e| e.to_string())?,
            volume: LiquidVolumeSettings {
                max_outward_courant: 1.0,
                actual_divergence_limit: 1e-5,
            },
        };
        let report = self
            .session
            .step(inputs, boundary, None, |_| false)
            .map_err(|e| e.to_string())?;
        self.view.update_session(&self.session.snapshot());
        self.status = format!("Accepted dt {:.6} s", report.carrier.step.dt);
        Ok(())
    }
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.playing {
                    "Pause session"
                } else {
                    "Run session"
                })
                .clicked()
            {
                self.playing = !self.playing;
            }
            let advance = ui.button("Advance one step").clicked();
            if (advance || self.playing)
                && let Err(e) = self.step()
            {
                self.status = e;
                self.playing = false;
            }
            if ui.button("Reset session").clicked() {
                match Self::new() {
                    Ok(next) => *self = next,
                    Err(e) => self.status = e,
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::Slider::new(&mut self.requested_dt, 0.001..=0.125).text("requested dt (s)"),
            );
            ui.add(egui::Slider::new(&mut self.flow, 0.0..=0.5).text("inlet/outlet (m/s)"));
        });
        ui.small("Separate 8×4×2 full-box transport session. Each update advances one accepted API step; no real-time or SPH coupling claim.");
        if !self.status.is_empty() {
            ui.small(&self.status);
        }
        ui.collapsing("VTK grid export", |ui| {
            let snapshot = self.session.snapshot();
            self.export.show(ui, "Save current VTK (.vti)", |out| {
                write_session_vti(out, &snapshot)
            });
        });
        self.view.controls(ui);
        if self.playing {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn live_view_and_export_follow_api_publication_and_reset_identity() {
        let mut view = TransportView::new().unwrap();
        assert_eq!(view.session.snapshot().stage(), SnapshotStage::Constructor);
        let run = view.session.snapshot().identity().run_id;
        let initial = view.session.snapshot().state().liquid.fraction.to_vec();
        view.step().unwrap();
        view.step().unwrap();
        let snapshot = view.session.snapshot();
        assert_eq!(snapshot.frame(), 2);
        assert_eq!(snapshot.stage(), SnapshotStage::Accepted);
        assert!(snapshot.state().carrier.time > 0.0);
        assert_ne!(initial, snapshot.state().liquid.fraction);
        let mut vtk = Vec::new();
        write_session_vti(&mut vtk, &snapshot).unwrap();
        assert!(
            String::from_utf8(vtk)
                .unwrap()
                .contains("interval_pressure_pa")
        );
        let reset = TransportView::new().unwrap();
        assert_ne!(reset.session.snapshot().identity().run_id, run);
    }
}
