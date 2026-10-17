//! Bounded external observations, independent of numerical workers and Run custody.
use super::camera::OrbitCamera;
use eframe::egui;
use serde::Deserialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

const FRAME_BYTES: u64 = 2 * 1024 * 1024;
const ACTIVITY_BYTES: u64 = 64 * 1024;
const MAX_CELLS: usize = 65_536;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Forcing {
    regime: String,
    amplitude_m_s2: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Preview {
    schema_version: u32,
    session: String,
    sequence: u64,
    state: String,
    phase: String,
    label: String,
    frame: usize,
    time_s: f64,
    counts_xyz: [usize; 3],
    extents_m: [f64; 3],
    tracer: Option<Vec<f32>>,
    pressure: Option<Vec<f64>>,
    pressure_label: Option<String>,
    forcing: Option<Forcing>,
    source: Option<serde_json::Value>,
    cell_size_m: Option<f64>,
    dt_s: Option<f64>,
    density_kg_m3: Option<f64>,
}

fn bounded_text(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit && !value.chars().any(char::is_control)
}
fn state(value: &str) -> bool {
    ["idle", "running", "completed", "failed", "cancelled"].contains(&value)
}
impl Preview {
    fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || !state(&self.state)
            || !bounded_text(&self.session, 200)
            || !bounded_text(&self.phase, 200)
            || !bounded_text(&self.label, 200)
            || self.sequence >= (1 << 53)
            || !self.time_s.is_finite()
            || self.time_s < 0.0
            || self.counts_xyz.iter().any(|n| !(2..=128).contains(n))
            || self
                .extents_m
                .iter()
                .any(|n| !n.is_finite() || *n <= 0.0 || *n > 100.0)
        {
            return Err("Invalid external preview identity, state or geometry".into());
        }
        let cells = self.counts_xyz.iter().product::<usize>();
        if cells > MAX_CELLS
            || self
                .tracer
                .as_ref()
                .is_some_and(|v| v.len() != cells || v.iter().any(|n| !n.is_finite()))
            || self
                .pressure
                .as_ref()
                .is_some_and(|v| v.len() != cells || v.iter().any(|n| !n.is_finite()))
            || (self.tracer.is_none() && self.pressure.is_none())
            || (self.pressure.is_some()
                && !self
                    .pressure_label
                    .as_ref()
                    .is_some_and(|s| bounded_text(s, 200)))
        {
            return Err(
                "Preview fields must be finite, exact-shaped and within 65536 cells".into(),
            );
        }
        if self.forcing.as_ref().is_some_and(|f| {
            !bounded_text(&f.regime, 120) || !f.amplitude_m_s2.is_finite() || f.amplitude_m_s2 < 0.0
        }) {
            return Err("Invalid declared forcing metadata".into());
        }
        for value in [self.cell_size_m, self.dt_s, self.density_kg_m3]
            .into_iter()
            .flatten()
        {
            if !value.is_finite() || value <= 0.0 {
                return Err("Invalid declared physical metadata".into());
            }
        }
        if self
            .cell_size_m
            .is_some_and(|h| self.spacing().iter().any(|s| (s - h).abs() > h * 1e-12))
        {
            return Err("Declared cubic spacing disagrees with extents/counts".into());
        }
        if self
            .source
            .as_ref()
            .is_some_and(|v| !v.is_object() || v.to_string().len() > 8192)
        {
            return Err("Source provenance must be a bounded object".into());
        }
        Ok(())
    }
    fn spacing(&self) -> [f64; 3] {
        std::array::from_fn(|d| self.extents_m[d] / self.counts_xyz[d] as f64)
    }
    fn cubic(&self) -> bool {
        let h = self.spacing();
        h.iter().all(|s| (s - h[0]).abs() <= h[0] * 1e-12)
    }
    fn display(&self, fraction: [f64; 3]) -> [f32; 3] {
        let maximum = self.extents_m.into_iter().fold(0.0_f64, f64::max);
        std::array::from_fn(|d| (0.5 + (fraction[d] - 0.5) * self.extents_m[d] / maximum) as f32)
    }
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x + self.counts_xyz[0] * (y + self.counts_xyz[1] * z)
    }
    fn payload_bytes(&self) -> usize {
        self.tracer
            .as_ref()
            .map_or(0, |v| v.len() * std::mem::size_of::<f32>())
            + self
                .pressure
                .as_ref()
                .map_or(0, |v| v.len() * std::mem::size_of::<f64>())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Progress {
    completed: u64,
    #[serde(deserialize_with = "required_total")]
    total: Option<u64>,
}
fn required_total<'de, D: serde::Deserializer<'de>>(input: D) -> Result<Option<u64>, D::Error> {
    Option::<u64>::deserialize(input)
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Activity {
    schema_version: u32,
    producer: String,
    sequence: u64,
    updated_at: String,
    state: String,
    phase: String,
    message: String,
    progress: Progress,
}
impl Activity {
    fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || !state(&self.state)
            || self.sequence >= (1 << 53)
            || !bounded_text(&self.producer, 120)
            || !bounded_text(&self.phase, 120)
            || !bounded_text(&self.message, 1000)
            || !utc_timestamp(&self.updated_at)
            || self.progress.completed >= (1 << 53)
            || self
                .progress
                .total
                .is_some_and(|t| t < self.progress.completed || t >= (1 << 53))
        {
            return Err("Invalid current activity observation".into());
        }
        Ok(())
    }
}
fn utc_timestamp(value: &str) -> bool {
    let Some(stamp) = value
        .strip_suffix('Z')
        .or_else(|| value.strip_suffix("+00:00"))
    else {
        return false;
    };
    if !stamp.is_ascii()
        || stamp.len() < 19
        || &stamp[4..5] != "-"
        || &stamp[7..8] != "-"
        || &stamp[10..11] != "T"
        || &stamp[13..14] != ":"
        || &stamp[16..17] != ":"
        || (stamp.len() > 19
            && (!stamp[19..].starts_with('.')
                || stamp.len() == 20
                || !stamp[20..].bytes().all(|c| c.is_ascii_digit())))
    {
        return false;
    }
    let numbers = [(0, 4), (5, 7), (8, 10), (11, 13), (14, 16), (17, 19)].map(|(a, b)| {
        stamp[a..b]
            .bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| stamp[a..b].parse::<u32>().ok())
            .flatten()
    });
    let [Some(y), Some(m), Some(d), Some(h), Some(min), Some(s)] = numbers else {
        return false;
    };
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1..=12 => 31,
        _ => 0,
    };
    (1..=9999).contains(&y)
        && d > 0
        && d <= days
        && h < 24
        && min < 60
        && s < 60
        && value.len() <= 80
}

#[derive(Debug, PartialEq, Eq)]
struct Stamp {
    bytes: u64,
    modified: SystemTime,
    #[cfg(unix)]
    identity: (u64, u64),
}
fn stamp(metadata: &std::fs::Metadata) -> Stamp {
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    Stamp {
        bytes: metadata.len(),
        modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        #[cfg(unix)]
        identity: (metadata.dev(), metadata.ino()),
    }
}
fn read_changed<T: serde::de::DeserializeOwned>(
    path: &Path,
    previous: Option<&Stamp>,
    limit: u64,
) -> Result<Option<(T, Stamp)>, String> {
    let metadata = path.symlink_metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(format!(
            "Publication must be an ordinary file of at most {limit} bytes"
        ));
    }
    let before = stamp(&metadata);
    if previous == Some(&before) {
        return Ok(None);
    }
    #[cfg(unix)]
    let file = std::fs::File::from(
        rustix::fs::open(
            path,
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::NONBLOCK,
            rustix::fs::Mode::empty(),
        )
        .map_err(|e| e.to_string())?,
    );
    #[cfg(not(unix))]
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file()
        || stamp(&file.metadata().map_err(|e| e.to_string())?) != before
    {
        return Err("Publication changed while opening; retrying".into());
    }
    let mut raw = Vec::new();
    (&file)
        .take(limit + 1)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() as u64 != before.bytes
        || stamp(&file.metadata().map_err(|e| e.to_string())?) != before
    {
        return Err("Publication changed or exceeded its bound; retrying".into());
    }
    Ok(Some((
        serde_json::from_slice(&raw).map_err(|e| e.to_string())?,
        before,
    )))
}

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Pressure,
    Tracer,
}
pub struct LiveProject {
    path: PathBuf,
    activity_path: PathBuf,
    preview: Option<Preview>,
    activity: Option<Activity>,
    frame_stamp: Option<Stamp>,
    activity_stamp: Option<Stamp>,
    next_poll: Instant,
    camera: OrbitCamera,
    slice: usize,
    field: Field,
    feedback: String,
    activity_feedback: String,
    last_frame_change: Instant,
}
impl LiveProject {
    pub fn new(path: PathBuf, activity_path: Option<PathBuf>) -> Self {
        let activity_path = activity_path.unwrap_or_else(|| path.with_file_name("activity.json"));
        Self {
            path,
            activity_path,
            preview: None,
            activity: None,
            frame_stamp: None,
            activity_stamp: None,
            next_poll: Instant::now(),
            camera: Default::default(),
            slice: 0,
            field: Field::Pressure,
            feedback: String::new(),
            activity_feedback: String::new(),
            last_frame_change: Instant::now(),
        }
    }
    fn poll(&mut self) {
        match read_changed::<Preview>(&self.path, self.frame_stamp.as_ref(), FRAME_BYTES) {
            Ok(Some((frame, stamp))) => match frame.validate() {
                Ok(()) => {
                    let accept = self.preview.as_ref().is_none_or(|old| {
                        old.session != frame.session || frame.sequence > old.sequence
                    });
                    if accept {
                        self.preview = Some(frame);
                        self.last_frame_change = Instant::now();
                    }
                    self.frame_stamp = Some(stamp);
                    self.feedback = if accept {
                        String::new()
                    } else {
                        "Ignored repeated/out-of-order frame publication".into()
                    };
                }
                Err(error) => self.feedback = error,
            },
            Ok(None) => {}
            Err(error) => self.feedback = error,
        }
        match read_changed::<Activity>(
            &self.activity_path,
            self.activity_stamp.as_ref(),
            ACTIVITY_BYTES,
        ) {
            Ok(Some((activity, stamp))) => match activity.validate() {
                Ok(()) => {
                    self.activity = Some(activity);
                    self.activity_stamp = Some(stamp);
                    self.activity_feedback.clear();
                }
                Err(error) => self.activity_feedback = error,
            },
            Ok(None) => {}
            Err(error) => self.activity_feedback = error,
        }
    }
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        if Instant::now() >= self.next_poll {
            self.poll();
            self.next_poll = Instant::now() + Duration::from_millis(250);
        }
        let mut stay = true;
        egui::TopBottomPanel::top("live-project-observation").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| { ui.heading("Live experiment"); if ui.button("Back to simulation").clicked() { stay = false; } });
            ui.small("External observation · no Run custody, scientific acceptance or solver execution is established here");
            ui.collapsing("Publication paths", |ui| { ui.monospace(self.path.display().to_string()); ui.monospace(self.activity_path.display().to_string()); });
            if let Some(activity) = &self.activity {
                ui.label(format!("{} · {} · {}", activity.producer, activity.phase, activity.state));
                ui.label(&activity.message);
                ui.small(format!("Progress {} / {} · publication {} · {}", activity.progress.completed,
                    activity.progress.total.map_or_else(|| "unknown".into(), |n| n.to_string()), activity.sequence, activity.updated_at));
            }
            if !self.activity_feedback.is_empty() { ui.colored_label(egui::Color32::YELLOW, format!("Activity unavailable: {}", self.activity_feedback)); }
            if !self.feedback.is_empty() { ui.colored_label(egui::Color32::YELLOW, format!("Frame unavailable; last valid view retained: {}", self.feedback)); }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(frame) = &self.preview else { ui.label("Waiting for a bounded frame publication. The producer remains separately managed."); return; };
            ui.label(format!("Displayed frame: {} · {} · {} · frame {} · {:.6} s", frame.label, frame.phase, frame.state, frame.frame, frame.time_s));
            ui.small(format!("{:?} cells · extents {:?} m · spacing {:?} m · zero origin", frame.counts_xyz, frame.extents_m, frame.spacing()));
            if frame.dt_s.is_some() || frame.density_kg_m3.is_some() {
                ui.small(format!("Declared dt {:?} s · density {:?} kg/m³", frame.dt_s, frame.density_kg_m3));
            }
            if !frame.cubic() { ui.colored_label(egui::Color32::YELLOW, "Noncubic source geometry; external inspection does not qualify it for the cubic-cell pressure model"); }
            match &frame.forcing {
                Some(force) => { ui.label(format!("Declared forcing: {} · amplitude {:.6e} m/s² · measured vector field not supplied", force.regime, force.amplitude_m_s2)); }
                None if frame.source.as_ref().is_some_and(|s| s.get("forcing_parameters").is_some()) => {
                    ui.label("Declared source forcing parameters available below; actual force field not captured");
                }
                None => { ui.label("Force metadata not supplied"); }
            }
            if frame.tracer.is_none() { ui.label("Tracer not supplied by this publication"); }
            if let Some(label) = &frame.pressure_label { ui.label(label); }
            if frame.state == "running" && self.last_frame_change.elapsed() > Duration::from_secs(5) {
                ui.colored_label(egui::Color32::YELLOW, "No new frame for 5 seconds; current activity and displayed frame are separate observations");
            }
            if let Some(source) = &frame.source {
                ui.collapsing("Declared source provenance", |ui| { ui.monospace(serde_json::to_string_pretty(source).unwrap_or_default()); });
            }
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.field, Field::Pressure, "Pressure voxels (Pa)");
                ui.selectable_value(&mut self.field, Field::Tracer, "Tracer voxels");
                self.slice = self.slice.min(frame.counts_xyz[2] - 1);
                ui.add(egui::Slider::new(&mut self.slice, 0..=frame.counts_xyz[2] - 1).text("Z cell"));
                if ui.button("Frame view").clicked() { self.camera = OrbitCamera::default(); }
            });
            ui.small(format!("Latest frame only · {:.3} MiB field payload · source history is not loaded", frame.payload_bytes() as f64 / 1048576.0));
            ui.columns(2, |columns| {
                columns[0].heading("3D domain and tracer cells");
                show_domain(&mut columns[0], frame, &mut self.camera, self.slice);
                columns[1].heading("XY voxel field");
                show_slice(&mut columns[1], frame, self.field, self.slice);
            });
        });
        ctx.request_repaint_after(Duration::from_millis(250));
        stay
    }
}

fn color(value: f64, maximum: f64) -> egui::Color32 {
    let t = if maximum > 0.0 {
        (value / maximum).clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let end = if t < 0.0 {
        [45.0, 105.0, 225.0]
    } else {
        [225.0, 65.0, 65.0]
    };
    let neutral = [230.0, 235.0, 240.0];
    let [r, g, b] =
        std::array::from_fn(|i| (neutral[i] + t.abs() * (end[i] - neutral[i])).round() as u8);
    egui::Color32::from_rgb(r, g, b)
}
fn show_domain(ui: &mut egui::Ui, frame: &Preview, camera: &mut OrbitCamera, slice: usize) {
    ui.small("Drag to orbit · Shift-drag to pan · wheel to zoom");
    let (response, painter) = ui.allocate_painter(
        ui.available_size().max(egui::vec2(1.0, 180.0)),
        egui::Sense::drag(),
    );
    if response.dragged() {
        let delta = ui.input(|i| i.pointer.delta());
        if ui.input(|i| i.modifiers.shift) {
            camera.pan(delta, response.rect.height());
        } else {
            camera.orbit(delta);
        }
    }
    if response.hovered() {
        camera.zoom(ui.input(|i| i.smooth_scroll_delta.y) * 0.003);
    }
    if let Some(tracer) = &frame.tracer {
        let maximum = tracer
            .iter()
            .fold(0.0_f32, |m, v| m.max(v.abs()))
            .max(1e-30);
        let mut points = Vec::new();
        for (i, &value) in tracer.iter().enumerate() {
            if value.abs() <= maximum * 0.01 {
                continue;
            }
            let cell = [
                i % frame.counts_xyz[0],
                i / frame.counts_xyz[0] % frame.counts_xyz[1],
                i / (frame.counts_xyz[0] * frame.counts_xyz[1]),
            ];
            let p = frame.display(std::array::from_fn(|d| {
                (cell[d] as f64 + 0.5) / frame.counts_xyz[d] as f64
            }));
            if let Some((pos, depth)) = camera.project(p, response.rect) {
                points.push((depth, pos, value));
            }
        }
        points.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, pos, value) in points {
            painter.circle_filled(pos, 2.5, color(f64::from(value), f64::from(maximum)));
        }
    }
    for corner in 0..8 {
        for axis in 0..3 {
            if corner & (1 << axis) != 0 {
                continue;
            }
            let a = frame.display(std::array::from_fn(|d| f64::from((corner >> d) & 1)));
            let b = frame.display(std::array::from_fn(|d| {
                f64::from(((corner | (1 << axis)) >> d) & 1)
            }));
            if let (Some((a, _)), Some((b, _))) = (
                camera.project(a, response.rect),
                camera.project(b, response.rect),
            ) {
                painter.line_segment([a, b], egui::Stroke::new(1.0, egui::Color32::GRAY));
            }
        }
    }
    let z = (slice as f64 + 0.5) / frame.counts_xyz[2] as f64;
    let plane = [[0.0, 0.0, z], [1.0, 0.0, z], [1.0, 1.0, z], [0.0, 1.0, z]]
        .into_iter()
        .map(|p| {
            camera
                .project(frame.display(p), response.rect)
                .map(|(p, _)| p)
        })
        .collect::<Option<Vec<_>>>();
    if let Some(plane) = plane {
        painter.add(egui::Shape::closed_line(
            plane,
            egui::Stroke::new(2.0, egui::Color32::GOLD),
        ));
    }
    painter.text(
        response.rect.left_top() + egui::vec2(6.0, 6.0),
        egui::Align2::LEFT_TOP,
        format!(
            "XY plane · Z = {:.6} m",
            (slice as f64 + 0.5) * frame.spacing()[2]
        ),
        egui::FontId::proportional(12.0),
        egui::Color32::GOLD,
    );
}
fn show_slice(ui: &mut egui::Ui, frame: &Preview, field: Field, slice: usize) {
    let value = |i| match field {
        Field::Pressure => frame.pressure.as_ref().map(|v| v[i]),
        Field::Tracer => frame.tracer.as_ref().map(|v| f64::from(v[i])),
    };
    if value(0).is_none() {
        ui.label("Selected field not supplied; no substitute field is inferred");
        return;
    }
    let cells = frame.counts_xyz.iter().product::<usize>();
    let maximum = (0..cells).fold(0.0_f64, |m, i| m.max(value(i).unwrap().abs()));
    let unit = if field == Field::Pressure {
        "Pa"
    } else {
        "source tracer units"
    };
    ui.small(format!(
        "X right · Y up · −{maximum:.3e} … +{maximum:.3e} {unit}; symmetric frame scale"
    ));
    ui.horizontal_wrapped(|ui| {
        ui.colored_label(color(-maximum, maximum), format!("−{maximum:.3e}"));
        ui.colored_label(color(0.0, maximum), "0");
        ui.colored_label(color(maximum, maximum), format!("+{maximum:.3e} {unit}"));
    });
    let (response, painter) = ui.allocate_painter(
        ui.available_size().max(egui::vec2(1.0, 180.0)),
        egui::Sense::hover(),
    );
    let available = response.rect.shrink(16.0);
    let scale = (available.width() as f64 / frame.extents_m[0])
        .min(available.height() as f64 / frame.extents_m[1]);
    let plot = egui::Rect::from_center_size(
        available.center(),
        egui::vec2(
            (frame.extents_m[0] * scale) as f32,
            (frame.extents_m[1] * scale) as f32,
        ),
    );
    let size = egui::vec2(
        plot.width() / frame.counts_xyz[0] as f32,
        plot.height() / frame.counts_xyz[1] as f32,
    );
    for y in 0..frame.counts_xyz[1] {
        for x in 0..frame.counts_xyz[0] {
            let rect = egui::Rect::from_min_size(
                egui::pos2(
                    plot.left() + x as f32 * size.x,
                    plot.bottom() - (y + 1) as f32 * size.y,
                ),
                size,
            );
            painter.rect_filled(
                rect,
                0.0,
                color(value(frame.index(x, y, slice)).unwrap(), maximum),
            );
        }
    }
    if let Some(pos) = response.hover_pos().filter(|p| plot.contains(*p)) {
        let x = (((pos.x - plot.left()) / size.x) as usize).min(frame.counts_xyz[0] - 1);
        let y = (((plot.bottom() - pos.y) / size.y) as usize).min(frame.counts_xyz[1] - 1);
        response.on_hover_text(format!(
            "Cell [{x}, {y}, {slice}] · {:.12e} {unit}",
            value(frame.index(x, y, slice)).unwrap()
        ));
    }
}

#[cfg(test)]
#[path = "live_project/tests.rs"]
mod tests;
