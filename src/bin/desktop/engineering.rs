//! Bounded, independent static-obstacle projection lab. Never reads appearance
//! voxels or attaches its pressure correction to the particle/carrier timeline.
use super::viewport::OrbitCamera;
use eframe::egui::{self, Color32, Rect, Stroke};
use rheon::{
    Axis, GridGeometry, PressureSettings, StaticObstacleGeometry, StaticObstaclePressure,
    SurfaceSettings, SurfaceStamp, TriangleSurface,
};

const COUNTS: [u64; 3] = [12, 8, 6];
const SPACING: [f64; 3] = [0.125; 3];
const LO: [f64; 3] = [0.5, 0.25, 0.25];
const HI: [f64; 3] = [1.0, 0.75, 0.5];

struct SliceData {
    obstacle: Option<([f64; 3], [f64; 3])>,
    pressure_available: bool,
    fraction: Option<Vec<f64>>,
    heading: String,
    context: String,
    grid: GridGeometry,
    pressure: Vec<f64>,
    velocity: Vec<[f64; 3]>,
    valid: Vec<bool>,
    gauges: Vec<usize>,
    divergence: f64,
}
impl SliceData {
    fn solve() -> Result<Self, String> {
        let grid = GridGeometry::new(COUNTS, SPACING, [0.0; 3]).map_err(|e| e.to_string())?;
        let surface = TriangleSurface::new(
            SurfaceStamp {
                id: 4301,
                version: 1,
            },
            (0..8)
                .map(|c| std::array::from_fn(|d| if c & (1 << d) == 0 { LO[d] } else { HI[d] }))
                .collect(),
            vec![
                [0, 2, 3],
                [0, 3, 1],
                [4, 5, 7],
                [4, 7, 6],
                [0, 1, 5],
                [0, 5, 4],
                [2, 6, 7],
                [2, 7, 3],
                [0, 4, 6],
                [0, 6, 2],
                [1, 3, 7],
                [1, 7, 5],
            ],
            SurfaceSettings::default(),
        )
        .map_err(|e| e.to_string())?;
        let owner = StaticObstacleGeometry::new(grid.clone(), surface, 8_000_000, |_, _| false)
            .map_err(|e| e.to_string())?;
        let volume = SPACING.iter().product::<f64>();
        if owner
            .fluid_volumes()
            .iter()
            .any(|&v| v != 0.0 && v != volume)
        {
            return Err("Lab requires exactly aligned full-fluid or solid cells".into());
        }
        let mut faces: [Vec<f64>; 3] =
            std::array::from_fn(|d| vec![0.0; grid.face_len([Axis::X, Axis::Y, Axis::Z][d])]);
        // Prescribed initial pulse, projected by the real sealed obstacle operator.
        // No claim of a steady wake, no-slip viscosity, or an advancing flow solver.
        for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
            let shape = grid.face_counts(axis);
            for (i, value) in faces[d].iter_mut().enumerate() {
                let c = [
                    i % shape[0],
                    i / shape[0] % shape[1],
                    i / (shape[0] * shape[1]),
                ];
                if c[d] > 0 && c[d] < grid.counts()[d] && owner.open_areas(axis)[i] > 0.0 {
                    let p = grid.face_position(axis, c).unwrap();
                    *value = if d == 0 {
                        (std::f64::consts::PI * p[1] / 1.0).sin()
                    } else {
                        0.0
                    };
                }
            }
        }
        let mut solver = StaticObstaclePressure::new(&owner, 1.0, 8_000_000, |_, _| false)
            .map_err(|e| e.to_string())?;
        let [x, y, z] = &mut faces;
        let report = solver
            .project(
                [x, y, z],
                0.05,
                PressureSettings {
                    relative_residual: 1e-10,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-7,
                    max_iterations: 2000,
                },
                |_, _| false,
            )
            .map_err(|e| e.to_string())?;
        let velocity = reconstruct_velocity(&grid, &faces);
        Ok(Self {
            obstacle: Some((LO,HI)), pressure_available: true, fraction: None,
            heading: "Static obstacle / accepted projection • independent of live timeline".into(),
            context: "Sealed stationary walls; density 1 kg/m³; projection interval 0.05 s. No advancing wake or viscous wall loads.".into(),
            grid,
            pressure: solver.pressure().ok_or("No accepted pressure")?.to_vec(),
            velocity,
            valid: owner.fluid_volumes().iter().map(|v| *v > 0.0).collect(),
            gauges: solver.gauge_cells().to_vec(),
            divergence: report
                .corrected
                .ok_or("No accepted correction")?
                .actual_divergence_max,
        })
    }
    fn from_session(snapshot: &rheon::SessionSnapshot<'_>) -> Self {
        let grid = snapshot.grid().clone();
        let state = snapshot.state();
        let n = grid.counts();
        let velocity = (0..grid.cell_len())
            .map(|i| {
                let c = [i % n[0], i / n[0] % n[1], i / (n[0] * n[1])];
                std::array::from_fn(|d| {
                    let axis = [Axis::X, Axis::Y, Axis::Z][d];
                    let values = [state.carrier.x, state.carrier.y, state.carrier.z][d];
                    let mut hi = c;
                    hi[d] += 1;
                    0.5 * (f64::from(values[grid.face_index(axis, c).unwrap()])
                        + f64::from(values[grid.face_index(axis, hi).unwrap()]))
                })
            })
            .collect();
        let accepted = snapshot.stage() == rheon::SnapshotStage::Accepted;
        let context=snapshot.report().map_or("Constructor state: pressure unavailable until an accepted step.".into(), |r| format!("Interval pressure [{:.6}, {:.6}] s; gauge cell 0. Full-box transport only; no free-surface pressure or obstacle boundary.",state.carrier.time-r.carrier.step.dt,state.carrier.time));
        Self {
            valid: vec![true; grid.cell_len()],
            pressure: state.pressure.to_vec(),
            velocity,
            fraction: Some(state.liquid.fraction.to_vec()),
            grid,
            obstacle: None,
            pressure_available: accepted,
            gauges: vec![0],
            divergence: snapshot
                .report()
                .map_or(0.0, |r| r.carrier.step.actual_divergence_max),
            heading: format!(
                "Transport run {} · {:?} frame {} · {:.6} s",
                snapshot.identity().run_id,
                snapshot.stage(),
                snapshot.frame(),
                state.carrier.time
            ),
            context,
        }
    }
    fn field(&self, i: usize, pressure: bool, fraction: bool) -> Option<f64> {
        if fraction {
            self.fraction.as_ref().map(|v| v[i])
        } else if pressure && !self.pressure_available {
            None
        } else {
            self.value(i, pressure)
        }
    }
    fn field_range(&self, pressure: bool, fraction: bool) -> [f64; 2] {
        let mut r = [f64::INFINITY, f64::NEG_INFINITY];
        for i in 0..self.valid.len() {
            if let Some(v) = self.field(i, pressure, fraction) {
                r[0] = r[0].min(v);
                r[1] = r[1].max(v);
            }
        }
        r
    }
    fn value(&self, i: usize, pressure: bool) -> Option<f64> {
        self.valid[i].then(|| {
            if pressure {
                self.pressure[i]
            } else {
                self.velocity[i].iter().map(|v| v * v).sum::<f64>().sqrt()
            }
        })
    }
    #[cfg(test)]
    fn range(&self, pressure: bool) -> [f64; 2] {
        let mut range = [f64::INFINITY, f64::NEG_INFINITY];
        for i in 0..self.valid.len() {
            if let Some(v) = self.value(i, pressure) {
                range[0] = range[0].min(v);
                range[1] = range[1].max(v);
            }
        }
        range
    }
    fn coordinate(&self, i: usize) -> [usize; 3] {
        let n = self.grid.counts();
        [i % n[0], i / n[0] % n[1], i / (n[0] * n[1])]
    }
    // One common scale preserves physical aspect ratios (1.5 x 1.0 x 0.75 m).
    fn display(&self, p: [f64; 3]) -> [f32; 3] {
        let origin = self.grid.origin();
        let upper = self.grid.upper();
        let scale = (0..3).map(|d| upper[d] - origin[d]).fold(0.0_f64, f64::max);
        std::array::from_fn(|d| ((p[d] - (upper[d] + origin[d]) * 0.5) / scale + 0.5) as f32)
    }
}
// Valid for the aligned full-fluid cells in this lab. Not a cut-cell sampler.
fn reconstruct_velocity(grid: &GridGeometry, faces: &[Vec<f64>; 3]) -> Vec<[f64; 3]> {
    let n = grid.counts();
    (0..grid.cell_len())
        .map(|i| {
            let c = [i % n[0], i / n[0] % n[1], i / (n[0] * n[1])];
            std::array::from_fn(|d| {
                let axis = [Axis::X, Axis::Y, Axis::Z][d];
                let mut hi = c;
                hi[d] += 1;
                0.5 * (faces[d][grid.face_index(axis, c).unwrap()]
                    + faces[d][grid.face_index(axis, hi).unwrap()])
            })
        })
        .collect()
}
fn color(value: f64, range: [f64; 2]) -> Color32 {
    let t = if range[1] > range[0] {
        ((value - range[0]) / (range[1] - range[0])).clamp(0.0, 1.0)
    } else {
        0.5
    };
    // Sequential map; exact numeric range is always displayed beside it.
    Color32::from_rgb(
        (30.0 + 215.0 * t) as u8,
        (65.0 + 170.0 * t) as u8,
        (150.0 - 110.0 * t) as u8,
    )
}
pub struct EngineeringView {
    snapshot: Result<SliceData, String>,
    axis: usize,
    index: usize,
    pressure: bool,
    fraction: bool,
    arrows: bool,
    selected: Option<usize>,
}
impl Default for EngineeringView {
    fn default() -> Self {
        Self {
            snapshot: SliceData::solve(),
            axis: 2,
            index: 2,
            pressure: false,
            fraction: false,
            arrows: true,
            selected: None,
        }
    }
}
impl EngineeringView {
    pub fn from_session(snapshot: &rheon::SessionSnapshot<'_>) -> Self {
        Self {
            snapshot: Ok(SliceData::from_session(snapshot)),
            axis: 2,
            index: 0,
            pressure: false,
            fraction: true,
            arrows: true,
            selected: None,
        }
    }
    pub fn update_session(&mut self, snapshot: &rheon::SessionSnapshot<'_>) {
        self.snapshot = Ok(SliceData::from_session(snapshot));
        if snapshot.stage() == rheon::SnapshotStage::Constructor {
            self.pressure = false;
        }
    }
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        let counts = self
            .snapshot
            .as_ref()
            .map_or(COUNTS.map(|n| n as usize), |s| s.grid.counts());
        if let Ok(s) = &self.snapshot {
            ui.label(&s.heading);
        }

        ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_label(!self.pressure && !self.fraction, "Speed")
                .clicked()
            {
                self.pressure = false;
                self.fraction = false;
            }
            let pressure_available = self.snapshot.as_ref().is_ok_and(|s| s.pressure_available);
            if ui
                .add_enabled(
                    pressure_available,
                    egui::Button::selectable(self.pressure, "Pressure correction"),
                )
                .clicked()
            {
                self.pressure = true;
                self.fraction = false;
            }
            if self.snapshot.as_ref().is_ok_and(|s| s.fraction.is_some())
                && ui
                    .selectable_label(self.fraction, "Liquid fraction")
                    .clicked()
            {
                self.fraction = true;
                self.pressure = false;
            }

            for (d, label) in ["YZ", "XZ", "XY"].into_iter().enumerate() {
                if ui.selectable_value(&mut self.axis, d, label).changed() {
                    self.index = self.index.min(counts[d] - 1);
                }
            }
            ui.add(
                egui::Slider::new(&mut self.index, 0..=counts[self.axis] - 1).text("cell slice"),
            );
            ui.checkbox(&mut self.arrows, "Velocity arrows");
        });
        if let Ok(s) = &self.snapshot {
            ui.small(&s.context);
        }
        if let Ok(s) = &self.snapshot {
            let range = s.field_range(self.pressure, self.fraction);
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "{}: {:.6} … {:.6} {}",
                    if self.fraction {
                        "Liquid fraction"
                    } else if self.pressure {
                        "Gauge correction"
                    } else {
                        "Cell-reconstructed speed"
                    },
                    range[0],
                    range[1],
                    if self.fraction {
                        "dimensionless"
                    } else if self.pressure {
                        "Pa"
                    } else {
                        "m/s"
                    }
                ));
                let (r, _) = ui.allocate_exact_size(egui::vec2(100.0, 12.0), egui::Sense::hover());
                for i in 0..100 {
                    let t = i as f64 / 99.0;
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            r.min + egui::vec2(i as f32, 0.0),
                            egui::vec2(1.0, 12.0),
                        ),
                        0.0,
                        color(range[0] + t * (range[1] - range[0]), range),
                    );
                }
            });
            if s.pressure_available {
                ui.small(format!("Full-state range fixed across slices. p = 0 at cell {:?}; max flux divergence {:.2e} s⁻¹.",s.gauges,s.divergence));
            }
            ui.small("Vector overlay: displacement = velocity × 0.1 s; adjacent-face averages. Gray = solid. Click a cell to probe.");
            ui.small(format!(
                "Slice {} = {:.3} m (cell center)",
                ["X", "Y", "Z"][self.axis],
                s.grid.origin()[self.axis]
                    + (self.index as f64 + 0.5) * s.grid.spacing()[self.axis]
            ));
            if let Some(i) = self.selected {
                let c = s.coordinate(i);
                let p = s.grid.cell_position(c).unwrap();
                if s.valid[i] {
                    let pressure = if s.pressure_available {
                        format!("{:.6} Pa", s.pressure[i])
                    } else {
                        "unavailable".into()
                    };
                    ui.label(format!(
                        "Cell {c:?} at {p:.3?} m | u = {:.5?} m/s | p = {pressure}",
                        s.velocity[i]
                    ));
                    if let Some(f) = &s.fraction {
                        ui.small(format!("Native cell liquid fraction: {:.8}", f[i]));
                    }
                } else {
                    ui.label(format!("Cell {c:?}: solid — field unavailable"));
                }
            } else {
                ui.small("Probe: no cell selected");
            }
        } else if let Err(error) = &self.snapshot {
            ui.colored_label(Color32::LIGHT_RED, error);
        }
    }
    pub fn paint(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        camera: &OrbitCamera,
    ) {
        let Ok(s) = &self.snapshot else {
            return;
        };
        let painter = ui.painter_at(rect);
        let range = s.field_range(self.pressure, self.fraction);
        let mut cells = Vec::new();
        for i in 0..s.valid.len() {
            let c = s.coordinate(i);
            if c[self.axis] != self.index {
                continue;
            }
            let center = s.grid.cell_position(c).unwrap();
            let a = (self.axis + 1) % 3;
            let b = (self.axis + 2) % 3;
            let corners: [[f64; 3]; 4] = std::array::from_fn(|j| {
                let mut p = center;
                p[a] += if j == 1 || j == 2 {
                    s.grid.spacing()[a] * 0.5
                } else {
                    -s.grid.spacing()[a] * 0.5
                };
                p[b] += if j >= 2 {
                    s.grid.spacing()[b] * 0.5
                } else {
                    -s.grid.spacing()[b] * 0.5
                };
                p
            });
            let projected: Option<Vec<_>> = corners
                .iter()
                .map(|p| camera.project(s.display(*p), rect).map(|v| v.0))
                .collect();
            if let (Some(points), Some((_, depth))) =
                (projected, camera.project(s.display(center), rect))
            {
                cells.push((i, center, points, depth));
            }
        }
        cells.sort_by(|a, b| b.3.total_cmp(&a.3));
        let mut arrows = Vec::new();
        for (i, center, points, _) in cells {
            let fill = s
                .field(i, self.pressure, self.fraction)
                .map_or(Color32::from_rgb(70, 76, 86), |v| color(v, range));
            if response.clicked()
                && let Some(p) = response.interact_pointer_pos()
                && inside(&points, p)
            {
                self.selected = Some(i);
            }
            let stroke = if self.selected == Some(i) {
                Stroke::new(2.0, Color32::WHITE)
            } else {
                Stroke::new(0.5, Color32::from_black_alpha(80))
            };
            painter.add(egui::Shape::convex_polygon(points, fill, stroke));
            if self.arrows && s.valid[i] {
                let end = std::array::from_fn(|d| center[d] + 0.1 * s.velocity[i][d]);
                if let (Some((p, _)), Some((q, _))) = (
                    camera.project(s.display(center), rect),
                    camera.project(s.display(end), rect),
                ) {
                    arrows.push((p, q));
                }
            }
        }
        // Glyphs are an explicit overlay: opaque tiles must not erase arrow heads.
        for (p, q) in arrows {
            painter.arrow(p, q - p, Stroke::new(1.3, Color32::WHITE));
        }
        // Geometric context is explicitly a wire overlay, not opaque CAD shading.
        let mut boxes = vec![(
            s.grid.origin(),
            s.grid.upper(),
            Color32::from_rgb(100, 145, 185),
        )];
        if let Some((lo, hi)) = s.obstacle {
            boxes.push((lo, hi, Color32::from_rgb(255, 160, 65)));
        }
        for (lo, hi, col) in boxes {
            for i in 0..8 {
                for d in 0..3 {
                    let j = i ^ (1 << d);
                    if i < j {
                        let vertex = |k| {
                            std::array::from_fn(|a| if k & (1 << a) == 0 { lo[a] } else { hi[a] })
                        };
                        if let (Some((p, _)), Some((q, _))) = (
                            camera.project(s.display(vertex(i)), rect),
                            camera.project(s.display(vertex(j)), rect),
                        ) {
                            painter.line_segment([p, q], Stroke::new(1.2, col));
                        }
                    }
                }
            }
        }
        for (d, label, color) in [
            (0, "X", Color32::LIGHT_RED),
            (1, "Y", Color32::LIGHT_GREEN),
            (2, "Z", Color32::LIGHT_BLUE),
        ] {
            let mut end = s.grid.origin();
            end[d] = s.grid.upper()[d];
            if let Some((p, _)) = camera.project(s.display(end), rect) {
                painter.text(
                    p + egui::vec2(7.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(13.0),
                    color,
                );
            }
        }
        painter.text(
            rect.left_bottom() + egui::vec2(8.0, -8.0),
            egui::Align2::LEFT_BOTTOM,
            format!(
                "Domain {:.3?} m",
                std::array::from_fn::<_, 3, _>(|d| s.grid.upper()[d] - s.grid.origin()[d])
            ),
            egui::FontId::proportional(11.0),
            Color32::LIGHT_GRAY,
        );
    }
}
fn inside(points: &[egui::Pos2], p: egui::Pos2) -> bool {
    let mut positive = false;
    let mut negative = false;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
        positive |= cross > 0.001;
        negative |= cross < -0.001;
    }
    (positive || negative) && !(positive && negative)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn session_slice_matches_native_fields_and_refreshes_accepted_publications() {
        use rheon::*;
        let grid = GridGeometry::new([3, 2, 4], [0.25, 0.5, 0.125], [-1.25, 2.5, -0.75]).unwrap();
        let fraction = (0..24).map(|i| (i as f64 + 1.0) / 32.0).collect();
        let config = LiquidTransportConfig {
            carrier: SimulationConfig {
                density: 1000.0,
                memory_limit: 16 * 1024 * 1024,
                pressure: PressureSettings {
                    relative_residual: 1e-12,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-9,
                    max_iterations: 10000,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 1.0,
            },
            represented_density: 800.0,
            volume_stamp: VolumeStamp { id: 1, version: 0 },
            carrier_id: 2,
        };
        let mut session = SimulationSession::new(
            SessionIdentity {
                run_id: 99,
                source_revision: "slice-regression".into(),
            },
            grid,
            config,
            PressureImplementation::JacobiPcgV1,
            fraction,
            16 * 1024 * 1024,
            4,
        )
        .unwrap();
        let mut view = EngineeringView::from_session(&session.snapshot());
        view.selected = Some(23);
        let initial = view.snapshot.as_ref().unwrap();
        assert!(!initial.pressure_available);
        assert_eq!(initial.field(23, true, false), None);
        assert_eq!(initial.field(23, false, true), Some(24.0 / 32.0));
        assert_eq!(
            initial.grid.cell_position(initial.coordinate(23)).unwrap(),
            [-0.625, 3.25, -0.3125]
        );
        assert_eq!(initial.display([-0.875, 3.0, -0.5]), [0.5; 3]);
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| view.controls(ui));
        });
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::Shape::Text(t) if t.galley.job.text.contains("p = unavailable"))));
        let mut prior_velocity = view.snapshot.as_ref().unwrap().velocity.clone();
        let mut prior_fraction = view.snapshot.as_ref().unwrap().fraction.clone().unwrap();
        for (step, flow) in [(1, 0.125), (2, 0.375)] {
            let boundary = BoxFluxStepBoundary {
                flux: PrescribedBoxFlux::new(
                    BoxFluxStamp {
                        id: 3,
                        version: step,
                    },
                    [[-flow, flow], [0.0; 2], [0.0; 2]],
                )
                .unwrap(),
                tracer: BoxFluxTracerPolicy::ClampedAppearance,
            };
            let inputs = LiquidStepInputs {
                requested_dt: 0.03125,
                smoke_source: None,
                forces: &[],
                source: None,
                inlet: LiquidInlet::new(VolumeStamp { id: 4, version: 0 }, [[0.0; 2]; 3]).unwrap(),
                volume: LiquidVolumeSettings {
                    max_outward_courant: 1.0,
                    actual_divergence_limit: 1e-5,
                },
            };
            session.step(inputs, boundary, None, |_| false).unwrap();
            let native = session.snapshot();
            view.update_session(&native);
            let data = view.snapshot.as_ref().unwrap();
            let state = native.state();
            assert!(data.pressure_available);
            assert_eq!(view.selected, Some(23));
            assert!(data.obstacle.is_none());
            assert!(data.valid.iter().all(|v| *v));
            assert_eq!(data.gauges, [0]);
            assert_eq!(data.pressure, state.pressure);
            assert_eq!(data.fraction.as_deref().unwrap(), state.liquid.fraction);
            assert!(data.heading.contains(&format!("Accepted frame {step}")));
            assert_eq!(
                data.divergence,
                native.report().unwrap().carrier.step.actual_divergence_max
            );
            // Independent staggered flat indexing on the asymmetric 3x2x4 grid.
            for k in 0..4 {
                for j in 0..2 {
                    for i in 0..3 {
                        let cell = i + 3 * (j + 2 * k);
                        let x = i + 4 * (j + 2 * k);
                        let y = i + 3 * (j + 3 * k);
                        let z = i + 3 * (j + 2 * k);
                        let expected = [
                            (f64::from(state.carrier.x[x]) + f64::from(state.carrier.x[x + 1]))
                                * 0.5,
                            (f64::from(state.carrier.y[y]) + f64::from(state.carrier.y[y + 3]))
                                * 0.5,
                            (f64::from(state.carrier.z[z]) + f64::from(state.carrier.z[z + 6]))
                                * 0.5,
                        ];
                        assert_eq!(data.velocity[cell], expected);
                        assert_eq!(data.field(cell, true, false), Some(state.pressure[cell]));
                        assert_eq!(
                            data.field(cell, false, true),
                            Some(state.liquid.fraction[cell])
                        );
                        assert_eq!(
                            data.field(cell, false, false),
                            Some(expected.iter().map(|v| v * v).sum::<f64>().sqrt())
                        );
                    }
                }
            }
            assert_ne!(
                data.velocity, prior_velocity,
                "new accepted fields must replace prior display"
            );
            assert_ne!(data.fraction.as_ref().unwrap(), &prior_fraction);
            prior_velocity.clone_from(&data.velocity);
            prior_fraction.clone_from(data.fraction.as_ref().unwrap());
        }
    }

    #[test]
    fn real_obstacle_projection_is_accepted_and_masked() {
        let s = SliceData::solve().unwrap();
        assert!(s.divergence < 1e-7);
        assert_eq!(s.valid.iter().filter(|v| !**v).count(), 32);
        assert_eq!(s.gauges.len(), 1);
        for &i in &s.gauges {
            assert_eq!(s.pressure[i], 0.0);
        }
        assert!(s.range(false)[1] > 0.01);
        assert!(s.range(true)[1] > s.range(true)[0]);
        for i in 0..s.valid.len() {
            assert_eq!(s.value(i, true).is_some(), s.valid[i]);
        }
        let x = s.display([1.5, 0.0, 0.0]);
        let o = s.display([0.0; 3]);
        let y = s.display([0.0, 1.0, 0.0]);
        assert!(((x[0] - o[0]) / (y[1] - o[1]) - 1.5).abs() < 1e-6);
    }
    #[test]
    fn staggered_linear_velocity_reconstructs_at_physical_cell_centers() {
        let g = GridGeometry::new([3, 2, 4], [0.2, 0.3, 0.5], [-1.0, 2.0, 0.5]).unwrap();
        let faces = std::array::from_fn(|d| {
            let axis = [Axis::X, Axis::Y, Axis::Z][d];
            let n = g.face_counts(axis);
            (0..g.face_len(axis))
                .map(|i| {
                    let c = [i % n[0], i / n[0] % n[1], i / (n[0] * n[1])];
                    let p = g.face_position(axis, c).unwrap();
                    (d + 1) as f64 * p[d] + 0.75
                })
                .collect()
        });
        let v = reconstruct_velocity(&g, &faces);
        let c = [2, 1, 3];
        let p = g.cell_position(c).unwrap();
        assert_eq!(p, [-0.5, 2.45, 2.25]);
        let actual = v[g.cell_index(c).unwrap()];
        for d in 0..3 {
            assert!((actual[d] - ((d + 1) as f64 * p[d] + 0.75)).abs() < 1e-12);
        }
    }
    #[test]
    fn egui_controls_pick_masked_and_fluid_cells_and_keep_vectors_above_tiles() {
        let ctx = egui::Context::default();
        let mut view = EngineeringView::default();
        let camera = OrbitCamera::default();
        let mut plot = Rect::NOTHING;
        let render = |view: &mut EngineeringView, plot: &mut Rect, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(600.0, 650.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        view.controls(ui);
                        let (rect, response) = ui.allocate_exact_size(
                            ui.available_size(),
                            egui::Sense::click_and_drag(),
                        );
                        *plot = rect;
                        view.paint(ui, rect, &response, &camera);
                    });
                },
            )
        };
        render(&mut view, &mut plot, vec![]);
        render(&mut view, &mut plot, vec![]);
        for c in [[5, 3, 2], [2, 1, 2]] {
            let s = view.snapshot.as_ref().unwrap();
            let i = s.grid.cell_index(c).unwrap();
            let pos = camera
                .project(s.display(s.grid.cell_position(c).unwrap()), plot)
                .unwrap()
                .0;
            for pressed in [true, false] {
                render(
                    &mut view,
                    &mut plot,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ],
                );
            }
            assert_eq!(view.selected, Some(i));
        }
        let out = render(&mut view, &mut plot, vec![]);
        let last_tile = out
            .shapes
            .iter()
            .rposition(
                |s| matches!(&s.shape,egui::Shape::Path(p) if p.fill != Color32::TRANSPARENT),
            )
            .unwrap();
        let first_arrow=out.shapes.iter().position(|s| matches!(&s.shape,egui::Shape::LineSegment {stroke,..} if stroke.color == Color32::WHITE && stroke.width == 1.3)).unwrap();
        assert!(
            first_arrow > last_tile,
            "All vector glyphs must follow opaque tiles"
        );
        for label in ["YZ", "Pressure correction"] {
            let out = render(&mut view, &mut plot, vec![]);
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
                    &mut view,
                    &mut plot,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: Default::default(),
                        },
                    ],
                );
            }
        }
        assert_eq!(view.axis, 0);
        assert!(view.pressure);
        assert!(
            plot.width() > 500.0 && plot.height() > 200.0,
            "Controls must leave a usable view"
        );
    }
    #[test]
    fn probe_polygon_and_constant_legend_are_defined() {
        let points = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(2.0, 0.0),
            egui::pos2(2.0, 1.0),
            egui::pos2(0.0, 1.0),
        ];
        assert!(inside(&points, egui::pos2(1.0, 0.5)));
        assert!(!inside(&points, egui::pos2(3.0, 0.5)));
        assert_eq!(color(2.0, [2.0, 2.0]), color(3.0, [3.0, 3.0]));
    }
}
