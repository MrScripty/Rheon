//! One pinned native training-scene timeline, using Rheon's shared voxel renderer.
use super::{camera::OrbitCamera, voxel_view};
use eframe::egui::{self, Color32, Rect, Stroke};
use serde::Deserialize;
use std::{collections::BTreeMap, io::Read, path::Path, sync::Arc, time::Duration};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Force {
    center_fraction: [f64; 3],
    direction: [f64; 3],
    acceleration_m_s2: f64,
    radius_fraction: f64,
    swirl_fraction: f64,
    fluctuation_fraction: f64,
    frequency_hz: f64,
    motion_fraction: f64,
    phase: f64,
    spatial_modes: [usize; 3],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {
    index: usize,
    time_s: f64,
    force_time_s: f64,
    tracer: Vec<f64>,
    pressure: Vec<f64>,
    velocity: Vec<[f64; 3]>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Provenance {
    binary_sha256: String,
    capture_manifest_sha256: String,
    retained_frame_parity: BTreeMap<String, bool>,
    native_steps_qualified: usize,
    tracer_max_abs_change: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Session {
    schema: String,
    trajectory_id: String,
    counts_xyz: [usize; 3],
    cell_size_m: f64,
    density_kg_m3: f64,
    dynamic_viscosity_pa_s: f64,
    dt_s: f64,
    force: Force,
    frames: Vec<Frame>,
    provenance: Provenance,
}
fn finite(v: f64) -> bool {
    v.is_finite()
}
fn hex(v: &str) -> bool {
    v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit())
}
impl Session {
    fn load(path: &Path) -> Result<Self, String> {
        let info = path.symlink_metadata().map_err(|e| e.to_string())?;
        if !info.is_file() || info.len() > 32 * 1024 * 1024 {
            return Err("Choose a regular timeline file of at most 32 MiB".into());
        }
        let mut raw = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(32 * 1024 * 1024 + 1)
            .read_to_end(&mut raw)
            .map_err(|e| e.to_string())?;
        if raw.len() > 32 * 1024 * 1024 {
            return Err("Timeline exceeds input budget".into());
        }
        let s: Self = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        s.validate()?;
        Ok(s)
    }
    fn validate(&self) -> Result<(), String> {
        let cells = self
            .counts_xyz
            .iter()
            .try_fold(1_usize, |v, n| v.checked_mul(*n))
            .ok_or("Cell budget overflow")?;
        if self.schema != "rheon-water-training-timeline-v1"
            || self.trajectory_id.is_empty()
            || self.trajectory_id.len() > 160
            || self.trajectory_id.chars().any(char::is_control)
            || !self.counts_xyz.iter().all(|n| (5..=128).contains(n))
            || cells > 65536
            || !finite(self.cell_size_m)
            || !(1.0 / 128.0..=1.0).contains(&self.cell_size_m)
            || self.dt_s != 1.0 / 60.0
            || self.density_kg_m3 != 1000.0
            || self.dynamic_viscosity_pa_s != 0.001
            || self.frames.is_empty()
            || self.frames.len() > 128
        {
            return Err("Unsupported native water timeline contract".into());
        }
        let f = &self.force;
        if !f
            .center_fraction
            .iter()
            .all(|v| finite(*v) && (0.0..=1.0).contains(v))
            || !f.direction.iter().all(|v| finite(*v))
            || (f.direction.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() > 1e-10
            || !finite(f.acceleration_m_s2)
            || !(0.0..=4.0).contains(&f.acceleration_m_s2)
            || !finite(f.radius_fraction)
            || !(0.1..=0.5).contains(&f.radius_fraction)
            || !finite(f.swirl_fraction)
            || f.swirl_fraction.abs() > 1.0
            || !finite(f.fluctuation_fraction)
            || !(0.0..=0.8).contains(&f.fluctuation_fraction)
            || !finite(f.frequency_hz)
            || !(0.0..=4.0).contains(&f.frequency_hz)
            || !finite(f.motion_fraction)
            || !(0.0..=0.12).contains(&f.motion_fraction)
            || !finite(f.phase)
            || !(0.0..=std::f64::consts::TAU).contains(&f.phase)
            || !f.spatial_modes.iter().all(|v| (1..=32).contains(v))
        {
            return Err("Invalid recorded native force".into());
        }
        for (i, frame) in self.frames.iter().enumerate() {
            if frame.index >= self.provenance.native_steps_qualified
                || (i > 0 && frame.index <= self.frames[i - 1].index)
                || (frame.time_s - (frame.index + 1) as f64 * self.dt_s).abs() > 1e-12
                || (frame.force_time_s - frame.index as f64 * self.dt_s).abs() > 1e-12
                || frame.tracer.len() != cells
                || frame.pressure.len() != cells
                || frame.velocity.len() != cells
                || !frame.tracer.iter().all(|v| finite(*v))
                || !frame.pressure.iter().all(|v| finite(*v))
                || !frame.velocity.iter().flatten().all(|v| finite(*v))
                || frame.pressure[0] != 0.0
            {
                return Err("Incomplete or nonfinite same-scene timeline".into());
            }
        }
        let p = &self.provenance;
        if !hex(&p.binary_sha256)
            || !hex(&p.capture_manifest_sha256)
            || !(64..=600).contains(&p.native_steps_qualified)
            || self.frames.last().unwrap().index + 1 != p.native_steps_qualified
            || ![15, 63]
                .iter()
                .all(|index| self.frames.iter().any(|f| f.index == *index))
            || p.retained_frame_parity.len() != 2
            || p.retained_frame_parity.get("15") != Some(&true)
            || p.retained_frame_parity.get("63") != Some(&true)
            || !finite(p.tracer_max_abs_change)
            || p.tracer_max_abs_change < 0.0
        {
            return Err("Measured native replay provenance required".into());
        }
        Ok(())
    }
    fn frame_at_time(&self, time: f64) -> usize {
        self.frames
            .partition_point(|f| f.time_s <= time)
            .saturating_sub(1)
    }
    fn extents(&self) -> [f64; 3] {
        self.counts_xyz.map(|n| n as f64 * self.cell_size_m)
    }
    fn center(&self, time: f64) -> [f64; 3] {
        let e = self.extents();
        std::array::from_fn(|d| {
            e[d] * (self.force.center_fraction[d]
                + self.force.motion_fraction
                    * (std::f64::consts::TAU * self.force.frequency_hz * time
                        + self.force.phase
                        + d as f64)
                        .sin())
        })
    }
    fn radius(&self) -> f64 {
        self.force.radius_fraction * self.extents().into_iter().fold(f64::INFINITY, f64::min)
    }
    fn world(&self, position: [f64; 3]) -> [f32; 3] {
        let e = self.extents();
        let scale = e.into_iter().fold(0.0_f64, f64::max);
        std::array::from_fn(|d| (0.5 + (position[d] - e[d] * 0.5) / scale) as f32)
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Field {
    WaterVelocity,
    Tracer,
    Pressure,
}
fn speed(v: [f64; 3]) -> f64 {
    v.iter().map(|v| v * v).sum::<f64>().sqrt()
}
pub struct TrainingViewer {
    path: String,
    presets: Vec<String>,
    session: Option<Arc<Session>>,
    error: String,
    camera: OrbitCamera,
    selected: usize,
    playing: bool,
    looping: bool,
    speed: f64,
    clock: f64,
    field: Field,
    scales: [f64; 3],
    alpha: u8,
    force: bool,
    arrows: bool,
    cache: Option<(usize, u8, Field)>,
    faces: Vec<voxel_view::Face>,
    mesh: Option<(OrbitCamera, Rect, Arc<egui::Mesh>)>,
}
impl TrainingViewer {
    pub fn new(path: String) -> Self {
        let mut app = Self {
            presets: vec![path.clone()],
            path,
            session: None,
            error: String::new(),
            camera: Default::default(),
            selected: 0,
            playing: true,
            looping: true,
            speed: 1.0,
            clock: 0.0,
            field: Field::WaterVelocity,
            scales: [0.0; 3],
            alpha: 8,
            force: true,
            arrows: true,
            cache: None,
            faces: Vec::new(),
            mesh: None,
        };
        app.load();
        app
    }
    pub fn with_presets(paths: Vec<String>) -> Self {
        let mut app = Self::new(paths[0].clone());
        app.presets = paths;
        app
    }
    fn load(&mut self) {
        self.session = None;
        self.cache = None;
        self.mesh = None;
        self.faces.clear();
        self.selected = 0;
        self.clock = 0.0;
        match Session::load(Path::new(&self.path)) {
            Ok(s) => {
                self.scales = [
                    s.frames
                        .iter()
                        .flat_map(|f| f.velocity.iter())
                        .map(|v| speed(*v))
                        .fold(0.0_f64, f64::max),
                    s.frames
                        .iter()
                        .flat_map(|f| f.tracer.iter())
                        .copied()
                        .fold(0.0_f64, f64::max),
                    s.frames
                        .iter()
                        .flat_map(|f| f.pressure.iter())
                        .map(|v| v.abs())
                        .fold(0.0_f64, f64::max),
                ];
                self.session = Some(Arc::new(s));
                self.error.clear();
            }
            Err(e) => {
                self.error = e;
                self.playing = false;
            }
        }
    }
    fn line(&self, painter: &egui::Painter, rect: Rect, a: [f32; 3], b: [f32; 3], stroke: Stroke) {
        if let (Some((a, _)), Some((b, _))) =
            (self.camera.project(a, rect), self.camera.project(b, rect))
        {
            painter.line_segment([a, b], stroke);
        }
    }
    fn arrow(&self, painter: &egui::Painter, rect: Rect, a: [f32; 3], b: [f32; 3], color: Color32) {
        if let (Some((a, _)), Some((b, _))) =
            (self.camera.project(a, rect), self.camera.project(b, rect))
        {
            painter.arrow(a, b - a, Stroke::new(1.8, color));
        }
    }
}
impl TrainingViewer {
    fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("training-viewer-status").show(ctx, |ui| {
            ui.heading("Rheon · native training simulation");
            let before = self.path.clone();
            egui::ComboBox::from_id_salt("native-training-scene")
                .selected_text(self.session.as_ref().map_or("Choose simulation", |s| s.trajectory_id.as_str()))
                .show_ui(ui, |ui| {
                    for path in &self.presets {
                        let label = Path::new(path).parent().and_then(Path::file_name)
                            .and_then(|s| s.to_str()).unwrap_or(path);
                        ui.selectable_value(&mut self.path, path.clone(), label);
                    }
                });
            if self.path != before {
                self.load();
            }
            ui.collapsing("Load a recorded simulation", |ui| {
                ui.text_edit_singleline(&mut self.path);
                if ui.button("Load timeline").clicked() {
                    self.load();
                }
            });
            if !self.error.is_empty() {
                ui.colored_label(Color32::LIGHT_RED, &self.error);
            }
            if let Some(s) = &self.session {
                ui.label(format!("Pinned scene {} · {} recorded states / {} native steps · producer reports original saved fields reproduced exactly",
                    s.trajectory_id, s.frames.len(), s.provenance.native_steps_qualified));
                ui.small("Recorded native simulation. Training uses saved states through native step 64. Later preview steps are observation only.");
            }
        });
        let Some(s) = self.session.clone() else {
            ctx.request_repaint_after(Duration::from_millis(100));
            return;
        };
        if self.playing {
            self.clock += self.speed * f64::from(ctx.input(|i| i.stable_dt.min(0.1)));
            let duration = s.frames.last().unwrap().time_s;
            if self.looping {
                self.clock = self.clock.rem_euclid(duration);
            } else if self.clock >= duration {
                self.clock = duration;
                self.playing = false;
            }
            self.selected = s.frame_at_time(self.clock);
        }
        egui::TopBottomPanel::bottom("training-viewer-time").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(if self.playing { "Pause" } else { "Play" }).clicked() {
                    self.playing = !self.playing;
                    self.clock = s.frames[self.selected].time_s;
                }
                ui.checkbox(&mut self.looping, "Loop same simulation");
                ui.add(egui::Slider::new(&mut self.speed, 0.05..=2.0).text("Playback speed"));
            });
            if ui.add(egui::Slider::new(&mut self.selected, 0..=s.frames.len()-1).text("Recorded state")).changed() {
                self.playing = false;
                self.clock = s.frames[self.selected].time_s;
            }
            ui.label(format!("Native step {} / {} · simulated time {:.6} s · fixed domain {:?} cells / {:?} m · cubic h = {:.5} m",
                s.frames[self.selected].index+1, s.provenance.native_steps_qualified,
                s.frames[self.selected].time_s, s.counts_xyz, s.extents(), s.cell_size_m));
        });
        let f = &s.force;
        let frame = &s.frames[self.selected];
        egui::SidePanel::left("training-viewer-inspector").resizable(true).default_width(330.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Water voxels");
            ui.selectable_value(&mut self.field, Field::WaterVelocity, "Water · velocity magnitude");
            ui.selectable_value(&mut self.field, Field::Tracer, "Passive tracer concentration");
            ui.selectable_value(&mut self.field, Field::Pressure, "Pressure magnitude |p|");
            ui.add(egui::Slider::new(&mut self.alpha, 1..=255).text("Voxel opacity"));
            ui.checkbox(&mut self.arrows, "Fluid velocity arrows");
            ui.checkbox(&mut self.force, "Show force widget");
            ui.small("Water: density 1000 kg/m³ · viscosity 0.001 Pa·s.");
            ui.small("The sealed domain is full of water. Tracer is a passive marker, not the amount of water.");
            ui.separator();
            ui.heading("Recorded force · read only");
            ui.label("Shape: spherical Gaussian with smooth taper; applied only inside the domain.");
            ui.label(format!("Current center: {:.4?} m", s.center(frame.force_time_s)));
            ui.label(format!("Core radius: {:.4} m", s.radius()));
            ui.label(format!("Support radius: {:.4} m (4 × core)", 4.0*s.radius()));
            ui.label(format!("Base direction: {:.4?}", f.direction));
            ui.label(format!("Nominal acceleration: {:.5} m/s²", f.acceleration_m_s2));
            let pulse = 0.65+0.35*(std::f64::consts::TAU*f.frequency_hz*frame.force_time_s+f.phase).sin();
            ui.label(format!("Current pulse: {:.1}%", 100.0*pulse));
            ui.label(format!("Current base acceleration: {:.5} m/s²", f.acceleration_m_s2*pulse));
            ui.label(format!("Swirl: {:+.4}", f.swirl_fraction));
            ui.label(format!("Fluctuation: {:.2}%", 100.0*f.fluctuation_fraction));
            ui.label(format!("Frequency: {:.4} Hz", f.frequency_hz));
            ui.label(format!("Center motion: {:.2}% of domain", 100.0*f.motion_fraction));
            ui.label(format!("Spatial modes XYZ: {:?}", f.spatial_modes));
            ui.small("Orange sphere: core radius. Orange arrow: base direction; swirl and fluctuating components also act. Cyan arrows: actual corrected water velocity.");
            ui.separator();
            ui.label(format!("Tracer max change over this timeline: {:.6}", s.provenance.tracer_max_abs_change));
            ui.small("Fluctuation/swirl describe prescribed forcing, not certified resolved turbulence.");
            if ui.button("Frame view").clicked() {
                self.camera = Default::default();
            }
            ui.checkbox(&mut self.camera.orthographic, "Orthographic");
            ui.small("Drag: orbit · Shift-drag: pan · wheel: zoom");
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let maximum = self.scales[match self.field {
                Field::WaterVelocity => 0,
                Field::Tracer => 1,
                Field::Pressure => 2,
            }];
            let units = match self.field {
                Field::WaterVelocity => "m/s",
                Field::Tracer => "dimensionless concentration",
                Field::Pressure => "Pa",
            };
            ui.label(format!(
                "Fixed timeline scale: {}0 … {:.6e} {units}",
                "", maximum
            ));
            let (legend, _) = ui.allocate_exact_size(egui::vec2(260.0, 16.0), egui::Sense::hover());
            for segment in 0..64 {
                let width = legend.width() / 64.0;
                let swatch = Rect::from_min_size(
                    legend.min + egui::vec2(segment as f32 * width, 0.0),
                    egui::vec2(width + 0.5, legend.height()),
                );
                ui.painter().rect_filled(
                    swatch,
                    0.0,
                    voxel_view::water_color((segment * 255 / 63) as u8, 1.0, None, 255),
                );
            }
            let (rect, response) = ui.allocate_exact_size(
                ui.available_size().max(egui::vec2(1.0, 1.0)),
                egui::Sense::click_and_drag(),
            );
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, Color32::from_rgb(14, 20, 29));
            ui.input(|i| {
                if response.dragged() {
                    let delta = i.pointer.delta();
                    if i.modifiers.shift {
                        self.camera.pan(delta, rect.height());
                    } else {
                        self.camera.orbit(delta);
                    }
                }
                if response.hovered() {
                    self.camera.zoom(i.smooth_scroll_delta.y * 0.003);
                }
            });
            let key = (self.selected, self.alpha, self.field);
            if self.cache != Some(key) {
                let density = if self.field == Field::Tracer {
                    frame
                        .tracer
                        .iter()
                        .map(|v| {
                            if maximum > 0.0 {
                                (v / maximum * 255.0).clamp(0.0, 255.0) as u8
                            } else {
                                0
                            }
                        })
                        .collect()
                } else {
                    vec![255; frame.pressure.len()]
                };
                self.faces = voxel_view::faces(
                    s.counts_xyz,
                    s.extents().map(|v| v as f32),
                    &density,
                    2,
                    self.alpha == 255,
                    |index, shade, _| {
                        let value = match self.field {
                            Field::WaterVelocity => speed(frame.velocity[index]),
                            Field::Tracer => frame.tracer[index],
                            Field::Pressure => frame.pressure[index].abs(),
                        };
                        let t = if maximum > 0.0 {
                            (value / maximum * 255.0).clamp(0.0, 255.0) as u8
                        } else {
                            0
                        };
                        voxel_view::water_color(t, shade, None, self.alpha)
                    },
                );
                self.cache = Some(key);
                self.mesh = None;
            }
            if self
                .mesh
                .as_ref()
                .is_none_or(|(camera, area, _)| *camera != self.camera || *area != rect)
            {
                self.mesh = Some((
                    self.camera.clone(),
                    rect,
                    Arc::new(voxel_view::mesh(self.faces.clone(), &self.camera, rect)),
                ));
            }
            if let Some((_, _, mesh)) = &self.mesh {
                painter.add(egui::Shape::mesh(Arc::clone(mesh)));
            }
            let e = s.extents();
            let corners = std::array::from_fn::<_, 8, _>(|i| {
                s.world(std::array::from_fn(|d| {
                    if i & (1 << d) == 0 { 0.0 } else { e[d] }
                }))
            });
            for i in 0..8 {
                for d in 0..3 {
                    let j = i ^ (1 << d);
                    if i < j {
                        self.line(
                            &painter,
                            rect,
                            corners[i],
                            corners[j],
                            Stroke::new(1.0, Color32::from_gray(150)),
                        );
                    }
                }
            }
            if self.arrows {
                let stride = s.counts_xyz.map(|n| (n / 4).max(1));
                for z in (0..s.counts_xyz[2]).step_by(stride[2]) {
                    for y in (0..s.counts_xyz[1]).step_by(stride[1]) {
                        for x in (0..s.counts_xyz[0]).step_by(stride[0]) {
                            let index = x + s.counts_xyz[0] * (y + s.counts_xyz[1] * z);
                            let p = [x, y, z].map(|n| (n as f64 + 0.5) * s.cell_size_m);
                            let velocity = frame.velocity[index];
                            let velocity_max = self.scales[0];
                            let scale = 0.2 * e.into_iter().fold(f64::INFINITY, f64::min)
                                / velocity_max.max(1e-12);
                            self.arrow(
                                &painter,
                                rect,
                                s.world(p),
                                s.world(std::array::from_fn(|d| p[d] + velocity[d] * scale)),
                                Color32::from_rgb(65, 235, 240),
                            );
                        }
                    }
                }
            }
            if self.force {
                let center = s.center(frame.force_time_s);
                let radius = s.radius();
                let color = Color32::from_rgb(255, 168, 40);
                for normal in 0..3 {
                    let a = (normal + 1) % 3;
                    let b = (normal + 2) % 3;
                    for i in 0..64 {
                        let circle = |step: usize| {
                            let angle = std::f64::consts::TAU * step as f64 / 64.0;
                            let mut p = center;
                            p[a] += radius * angle.cos();
                            p[b] += radius * angle.sin();
                            s.world(p)
                        };
                        self.line(
                            &painter,
                            rect,
                            circle(i),
                            circle(i + 1),
                            Stroke::new(2.0, color),
                        );
                    }
                }
                self.arrow(
                    &painter,
                    rect,
                    s.world(center),
                    s.world(std::array::from_fn(|d| {
                        center[d] + f.direction[d] * radius * 1.8
                    })),
                    color,
                );
                if let Some((p, _)) = self.camera.project(s.world(center), rect) {
                    painter.circle_filled(p, 4.0, color);
                    painter.text(
                        p + egui::vec2(8.0, -15.0),
                        egui::Align2::LEFT_BOTTOM,
                        "Applied force",
                        egui::FontId::proportional(15.0),
                        color,
                    );
                }
            }
        });
        ctx.request_repaint_after(Duration::from_millis(33));
    }
}

impl eframe::App for TrainingViewer {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.show(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn timeline(stride: usize, steps: usize) -> Session {
        let frames: Vec<_> = (0..steps)
            .filter(|i| (i + 1) % stride == 0 || *i == 15 || *i == 63)
            .map(|i| {
                json!({"index":i,"time_s":(i+1) as f64/60.0,
                "force_time_s":i as f64/60.0,"tracer":vec![0.1;125],
                "pressure":vec![0.0;125],"velocity":vec![[0.0;3];125]})
            })
            .collect();
        serde_json::from_value(json!({
            "schema":"rheon-water-training-timeline-v1","trajectory_id":"water-train-fixture",
            "counts_xyz":[5,5,5],"cell_size_m":0.125,"density_kg_m3":1000.0,
            "dynamic_viscosity_pa_s":0.001,"dt_s":1.0/60.0,
            "force":{"center_fraction":[0.5,0.5,0.5],"direction":[1.0,0.0,0.0],
                "acceleration_m_s2":1.0,"radius_fraction":0.2,"swirl_fraction":0.5,
                "fluctuation_fraction":0.2,"frequency_hz":1.0,"motion_fraction":0.1,
                "phase":0.0,"spatial_modes":[1,2,3]},"frames":frames,
            "provenance":{"binary_sha256":"a".repeat(64),"capture_manifest_sha256":"b".repeat(64),
                "retained_frame_parity":{"15":true,"63":true},"native_steps_qualified":steps,
                "tracer_max_abs_change":0.01}
        }))
        .unwrap()
    }

    #[test]
    fn playback_uses_physical_time_for_both_dense_and_decimated_timelines() {
        for (stride, steps) in [(1, 64), (4, 256)] {
            let s = timeline(stride, steps);
            s.validate().unwrap();
            for (i, f) in s.frames.iter().enumerate() {
                assert_eq!(s.frame_at_time(f.time_s), i);
            }
            assert_eq!(s.frame_at_time(0.0), 0);
            assert_eq!(s.frame_at_time(999.0), s.frames.len() - 1);
        }
    }

    #[test]
    fn reset_or_missing_original_reference_cannot_be_a_continuous_timeline() {
        let mut s = timeline(4, 256);
        s.frames[20].index = 0;
        assert!(s.validate().is_err());
        let mut s = timeline(4, 256);
        s.frames.retain(|f| f.index != 63);
        assert!(s.validate().is_err());
        let mut s = timeline(4, 256);
        s.provenance.native_steps_qualified = 300;
        assert!(s.validate().is_err());
    }

    #[test]
    fn real_egui_frame_selection_moves_tracer_voxels_instead_of_recentering_them() {
        let mut s = timeline(1, 64);
        for (i, frame) in s.frames.iter_mut().enumerate() {
            frame.tracer.fill(0.0);
            // Move by one voxel in X during this fixed-domain simulation.
            frame.tracer[62 + usize::from(i == 63)] = 1.0;
        }
        let mut app = TrainingViewer::new("/nonexistent/rheon-timeline.json".into());
        app.session = Some(Arc::new(s));
        app.scales = [0.0, 1.0, 0.0];
        app.field = Field::Tracer;
        app.playing = false;
        let ctx = egui::Context::default();
        let mut centers = Vec::new();
        for selected in [0, 63] {
            app.selected = selected;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1200.0, 900.0),
                    )),
                    ..Default::default()
                },
                |ctx| app.show(ctx),
            );
            assert!(!output.shapes.is_empty());
            assert_eq!(app.faces.len(), 6);
            centers.push(
                app.faces
                    .iter()
                    .flat_map(|f| f.points)
                    .map(|p| p[0])
                    .sum::<f32>()
                    / 24.0,
            );
            assert!(app.mesh.is_some());
        }
        assert!((centers[1] - centers[0] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn invalid_reload_clears_the_previously_loaded_scene() {
        let mut app = TrainingViewer::new("/nonexistent/rheon-timeline.json".into());
        app.session = Some(Arc::new(timeline(1, 64)));
        app.faces.push(voxel_view::Face {
            points: [[0.0; 3]; 4],
            color: Color32::WHITE,
        });
        app.load();
        assert!(app.session.is_none());
        assert!(app.faces.is_empty());
        assert!(!app.error.is_empty());
    }
}
