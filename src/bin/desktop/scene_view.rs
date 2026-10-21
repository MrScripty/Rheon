//! Visual authoring geometry only: no field evaluation, solver or model loading.
use super::{camera::OrbitCamera, scene_ui::SceneUi};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
use rheon::{ForceKind, ForceObject, ForceSupport, SceneObjectId};
use std::collections::BTreeMap;
const NEAR: f32 = 0.0151;
const PICK_RADIUS: f32 = 7.0;
const MODEL_TRIANGLE_LIMIT: usize = 12_000;
// Display-only coordinates are bounded to prevent projection/interpolation
// overflow. Refuse explicitly; never clamp or rewrite physical authoring values.
type V3 = [f32; 3];
#[derive(Clone, Copy)]
pub struct MeshRef<'a> {
    pub vertices: &'a [[f64; 3]],
    pub triangles: &'a [[usize; 3]],
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Point {
    pub pos: Pos2,
    pub depth: f32,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Line {
    pub a: Point,
    pub b: Point,
}
struct Object {
    id: SceneObjectId,
    name: String,
    enabled: bool,
    anchor: Option<Point>,
    lines: Vec<Line>,
}
struct Model {
    id: SceneObjectId,
    triangles: Vec<[Point; 3]>,
    outline: Vec<Line>,
}
struct Projection {
    objects: Vec<Object>,
    model: Option<Model>,
    warning: Option<&'static str>,
    pick_incomplete: bool,
    orthographic: bool,
}
#[derive(Clone, Copy, Debug)]
struct Hit {
    id: SceneObjectId,
    distance: f32,
    depth: f32,
}
fn dot(a: V3, b: V3) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn sub(a: V3, b: V3) -> V3 {
    std::array::from_fn(|d| a[d] - b[d])
}
fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn lerp(a: V3, b: V3, t: f32) -> V3 {
    std::array::from_fn(|d| a[d] + (b[d] - a[d]) * t)
}
pub(super) fn normalized(p: [f64; 3], side: f64) -> Option<V3> {
    let p = p.map(|v| v / side);
    (side.is_finite() && side > 0.0 && p.iter().all(|v| v.is_finite() && v.abs() <= 1.0e12))
        .then(|| p.map(|v| v as f32))
}
fn project(camera: &OrbitCamera, p: V3, rect: Rect) -> Option<Point> {
    let (pos, depth) = camera.project(p, rect)?;
    (pos.is_finite() && depth.is_finite() && depth >= NEAR - 0.00001)
        .then_some(Point { pos, depth })
}
fn depth(camera: &OrbitCamera, p: V3) -> f32 {
    let (eye, forward, _, _) = camera.basis();
    dot(sub(p, eye), forward)
}
fn interpolate_depth(a: f32, b: f32, t: f64, ortho: bool) -> f32 {
    if ortho {
        (f64::from(a) + (f64::from(b) - f64::from(a)) * t) as f32
    } else {
        (1.0 / ((1.0 - t) / f64::from(a) + t / f64::from(b))) as f32
    }
}
/// Clip before painting as well as picking; finite but enormous offscreen points
/// must not reach the tessellator. Depth follows screen interpolation correctly.
fn clip_line(line: Line, rect: Rect, ortho: bool) -> Option<Line> {
    let a = [f64::from(line.a.pos.x), f64::from(line.a.pos.y)];
    let b = [f64::from(line.b.pos.x), f64::from(line.b.pos.y)];
    if !a.into_iter().chain(b).all(f64::is_finite) {
        return None;
    }
    let delta = [b[0] - a[0], b[1] - a[1]];
    let mut lower: f64 = 0.0;
    let mut upper: f64 = 1.0;
    for (p, q) in [
        (-delta[0], a[0] - f64::from(rect.left())),
        (delta[0], f64::from(rect.right()) - a[0]),
        (-delta[1], a[1] - f64::from(rect.top())),
        (delta[1], f64::from(rect.bottom()) - a[1]),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                lower = lower.max(t);
            } else {
                upper = upper.min(t);
            }
            if lower > upper {
                return None;
            }
        }
    }
    let point = |t: f64| Point {
        pos: egui::pos2((a[0] + delta[0] * t) as f32, (a[1] + delta[1] * t) as f32),
        depth: interpolate_depth(line.a.depth, line.b.depth, t, ortho),
    };
    Some(Line {
        a: point(lower),
        b: point(upper),
    })
}
pub(super) fn segment(camera: &OrbitCamera, rect: Rect, mut a: V3, mut b: V3) -> Option<Line> {
    let da = depth(camera, a);
    let db = depth(camera, b);
    if !da.is_finite() || !db.is_finite() || (da < NEAR && db < NEAR) {
        return None;
    }
    if da < NEAR {
        a = lerp(a, b, (NEAR - da) / (db - da));
    } else if db < NEAR {
        b = lerp(a, b, (NEAR - da) / (db - da));
    }
    clip_line(
        Line {
            a: project(camera, a, rect)?,
            b: project(camera, b, rect)?,
        },
        rect,
        camera.orthographic,
    )
}
fn clipped_triangle(camera: &OrbitCamera, rect: Rect, triangle: [V3; 3]) -> Vec<[Point; 3]> {
    let mut polygon = Vec::with_capacity(4);
    for i in 0..3 {
        let a = triangle[i];
        let b = triangle[(i + 1) % 3];
        let da = depth(camera, a);
        let db = depth(camera, b);
        if !da.is_finite() || !db.is_finite() {
            return vec![];
        }
        if da >= NEAR {
            polygon.push(a);
        }
        if (da >= NEAR) != (db >= NEAR) {
            polygon.push(lerp(a, b, (NEAR - da) / (db - da)));
        }
    }
    let projected: Option<Vec<_>> = polygon
        .into_iter()
        .map(|p| project(camera, p, rect))
        .collect();
    let Some(p) = projected else {
        return vec![];
    };
    (1..p.len().saturating_sub(1))
        .map(|i| [p[0], p[i], p[i + 1]])
        .collect()
}
pub(super) fn line_hit(line: Line, p: Pos2, ortho: bool) -> Option<(f32, f32)> {
    let vector = line.b.pos - line.a.pos;
    let length = vector.length_sq();
    let t = if length > 0.0001 {
        ((p - line.a.pos).dot(vector) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let distance = p.distance(line.a.pos + vector * t);
    (distance <= PICK_RADIUS).then(|| {
        (
            distance,
            interpolate_depth(line.a.depth, line.b.depth, f64::from(t), ortho),
        )
    })
}
fn triangle_hit(triangle: [Point; 3], p: Pos2, ortho: bool) -> Option<f32> {
    let [a, b, c] = triangle;
    let cross = |u: egui::Vec2, v: egui::Vec2| {
        f64::from(u.x) * f64::from(v.y) - f64::from(u.y) * f64::from(v.x)
    };
    let area = cross(b.pos - a.pos, c.pos - a.pos);
    if !area.is_finite() || area.abs() < 0.01 {
        return None;
    }
    let u = cross(b.pos - p, c.pos - p) / area;
    let v = cross(c.pos - p, a.pos - p) / area;
    let w = 1.0 - u - v;
    if [u, v, w].iter().any(|v| *v < -1e-6 || *v > 1.0 + 1e-6) {
        return None;
    }
    let z = if ortho {
        u * f64::from(a.depth) + v * f64::from(b.depth) + w * f64::from(c.depth)
    } else {
        1.0 / (u / f64::from(a.depth) + v / f64::from(b.depth) + w / f64::from(c.depth))
    };
    (z.is_finite() && z > 0.0).then_some(z as f32)
}
pub(super) fn rotate([x, y, z, w]: [f64; 4], [a, b, c]: [f64; 3]) -> [f64; 3] {
    [
        (1.0 - 2.0 * (y * y + z * z)) * a + 2.0 * (x * y - z * w) * b + 2.0 * (x * z + y * w) * c,
        2.0 * (x * y + z * w) * a + (1.0 - 2.0 * (x * x + z * z)) * b + 2.0 * (y * z - x * w) * c,
        2.0 * (x * z - y * w) * a + 2.0 * (y * z + x * w) * b + (1.0 - 2.0 * (x * x + y * y)) * c,
    ]
}
fn world(object: &ForceObject, local: [f64; 3]) -> [f64; 3] {
    let p = rotate(object.transform.rotation, local);
    std::array::from_fn(|d| p[d] + object.transform.translation[d])
}
fn force_lines(object: &ForceObject, side: f64) -> Vec<([f64; 3], [f64; 3])> {
    let mut lines = Vec::with_capacity(128);
    let size = side * 0.055;
    let mut push = |a, b| lines.push((world(object, a), world(object, b)));
    match object.support {
        ForceSupport::Global => {}
        ForceSupport::Sphere { radius, .. } => {
            for axis in 0..3 {
                for i in 0..32 {
                    let p = |i: usize| {
                        let angle = i as f64 * std::f64::consts::TAU / 32.0;
                        let mut p = [0.0; 3];
                        p[(axis + 1) % 3] = radius * angle.cos();
                        p[(axis + 2) % 3] = radius * angle.sin();
                        p
                    };
                    push(p(i), p(i + 1));
                }
            }
        }
        ForceSupport::Box { half_extent, .. } => {
            for axis in 0..3 {
                for a in [-1.0, 1.0] {
                    for b in [-1.0, 1.0] {
                        let mut p = half_extent;
                        p[axis] = -half_extent[axis];
                        p[(axis + 1) % 3] *= a;
                        p[(axis + 2) % 3] *= b;
                        let mut q = p;
                        q[axis] = half_extent[axis];
                        push(p, q);
                    }
                }
            }
        }
    }
    // A fixed authoring glyph, deliberately unrelated to force magnitude/density.
    match object.kind {
        ForceKind::Directional { vector } => {
            let max = vector.iter().fold(0.0_f64, |a, v| a.max(v.abs()));
            if max > 0.0 {
                let v = vector.map(|v| v / max);
                let norm = v.iter().map(|v| v * v).sum::<f64>().sqrt();
                let end = v.map(|v| v / norm * size * 2.0);
                push([0.0; 3], end);
                let axis = if v[1].abs() < 0.9 {
                    [0.0, 1.0, 0.0]
                } else {
                    [1.0, 0.0, 0.0]
                };
                let tangent = [
                    v[1] * axis[2] - v[2] * axis[1],
                    v[2] * axis[0] - v[0] * axis[2],
                    v[0] * axis[1] - v[1] * axis[0],
                ];
                for sign in [-1.0, 1.0] {
                    push(
                        end,
                        std::array::from_fn(|d| end[d] * 0.75 + tangent[d] * size * 0.25 * sign),
                    );
                }
            }
        }
        ForceKind::Radial { strength } if strength != 0.0 => {
            for d in 0..3 {
                for sign in [-1.0, 1.0] {
                    let mut end = [0.0; 3];
                    end[d] = size * sign;
                    let (start, tip) = if strength > 0.0 {
                        ([0.0; 3], end)
                    } else {
                        (end, [0.0; 3])
                    };
                    push(start, tip);
                    for wing in [-1.0, 1.0] {
                        let mut tail = tip;
                        tail[d] += (start[d] - tip[d]) * 0.25;
                        tail[(d + 1) % 3] = size * 0.16 * wing;
                        push(tip, tail);
                    }
                }
            }
        }
        ForceKind::Vortex { strength } if strength != 0.0 => {
            let sign = if strength < 0.0 { -1.0 } else { 1.0 };
            let p = |i: usize| {
                let a = i as f64 * std::f64::consts::TAU / 24.0;
                [size * a.cos(), 0.0, -size * a.sin() * sign]
            };
            for i in 0..24 {
                push(p(i), p(i + 1));
            }
            push(p(0), [size * 0.7, 0.0, size * 0.3 * sign]);
        }
        _ => {}
    }
    lines
}
impl Projection {
    fn build(
        state: &SceneUi,
        camera: &OrbitCamera,
        rect: Rect,
        side: f64,
        mesh: Option<MeshRef<'_>>,
    ) -> Self {
        let mut result = Self {
            objects: vec![],
            model: None,
            warning: None,
            pick_incomplete: false,
            orthographic: camera.orthographic,
        };
        let (eye, forward, right, up) = camera.basis();
        if !rect.is_finite()
            || rect.width() <= 0.0
            || rect.height() <= 0.0
            || rect.width() > 1e6
            || rect.height() > 1e6
            || !camera.distance.is_finite()
            || camera.distance < 1e-6
            || !eye
                .into_iter()
                .chain(forward)
                .chain(right)
                .chain(up)
                .all(f32::is_finite)
            || [forward, right, up]
                .iter()
                .any(|v| (dot(*v, *v) - 1.0).abs() > 0.01)
        {
            result.warning = Some("Scene selection unavailable · invalid camera or canvas");
            result.pick_incomplete = true;
            return result;
        }
        let display_scene = state.display_scene();
        {
            if display_scene.freeze().is_err() {
                result.warning = Some("Invalid force draft · fix properties to preview");
                result.pick_incomplete = true;
            } else {
                for object in &display_scene.objects {
                    let anchor_position = normalized(object.transform.translation, side);
                    let mut unavailable = anchor_position.is_none();
                    let anchor = anchor_position.and_then(|p| project(camera, p, rect));
                    let lines = force_lines(object, side)
                        .into_iter()
                        .filter_map(|(a, b)| match (normalized(a, side), normalized(b, side)) {
                            (Some(a), Some(b)) => segment(camera, rect, a, b),
                            _ => {
                                unavailable = true;
                                None
                            }
                        })
                        .collect();
                    if unavailable {
                        result.pick_incomplete = true;
                        result.warning =
                            Some("Force preview out of display range · use properties");
                    }
                    result.objects.push(Object {
                        id: object.id,
                        name: object.name.clone(),
                        enabled: object.enabled,
                        anchor,
                        lines,
                    });
                }
            }
        }
        if let (Some(id), Some(mesh)) = (state.model_id(), mesh) {
            if mesh.triangles.len() > MODEL_TRIANGLE_LIMIT {
                result.warning = Some("Model selection unavailable above 12,000 triangles");
                result.pick_incomplete = true;
                return result;
            }
            let mut model = Model {
                id,
                triangles: vec![],
                outline: vec![],
            };
            // Weld only exactly equal positions (including signed zero), never nearby
            // physical vertices. Opposite-vertex screen side is winding independent.
            type Key = [u64; 3];
            let key = |p: [f64; 3]| p.map(|v| if v == 0.0 { 0 } else { v.to_bits() });
            let mut edges: BTreeMap<(Key, Key), (u8, usize, V3, V3)> = BTreeMap::new();
            let (eye, forward, _, _) = camera.basis();
            let mut bad = false;
            for triangle in mesh.triangles {
                let raw: Option<Vec<_>> = triangle
                    .iter()
                    .map(|&i| mesh.vertices.get(i).copied())
                    .collect();
                let Some(raw) = raw else {
                    bad = true;
                    continue;
                };
                let points: Option<Vec<_>> = raw.iter().map(|&p| normalized(p, side)).collect();
                let Some(p) = points else {
                    bad = true;
                    continue;
                };
                let normal = cross(sub(p[1], p[0]), sub(p[2], p[0]));
                let area = dot(normal, normal);
                if !area.is_finite() {
                    bad = true;
                    continue;
                }
                if area == 0.0 {
                    continue;
                }
                model
                    .triangles
                    .extend(clipped_triangle(camera, rect, [p[0], p[1], p[2]]));
                for i in 0..3 {
                    let j = (i + 1) % 3;
                    let k = (i + 2) % 3;
                    let (ka, kb, a, b) = if key(raw[i]) < key(raw[j]) {
                        (key(raw[i]), key(raw[j]), p[i], p[j])
                    } else {
                        (key(raw[j]), key(raw[i]), p[j], p[i])
                    };
                    let view = if camera.orthographic {
                        forward.map(|v| -v)
                    } else {
                        sub(eye, a)
                    };
                    let side = dot(cross(sub(b, a), sub(p[k], a)), view);
                    let flag = if side > 0.0 {
                        1
                    } else if side < 0.0 {
                        2
                    } else {
                        0
                    };
                    let edge = edges.entry((ka, kb)).or_insert((0, 0, a, b));
                    edge.0 |= flag;
                    edge.1 += 1;
                }
            }
            let nonmanifold = edges.values().any(|e| e.1 > 2);
            if bad {
                result.pick_incomplete = true;
                result.warning = Some("Model selection incomplete · invalid display geometry");
            }
            if nonmanifold {
                result.warning = Some("Model outline unavailable · non-manifold edges");
            } else {
                for (_, (sides, count, a, b)) in edges {
                    if (count == 1 || sides != 3)
                        && let Some(line) = segment(camera, rect, a, b)
                    {
                        model.outline.push(line);
                    }
                }
            }
            result.model = Some(model);
        }
        if state.model_id().is_some() && mesh.is_none() {
            result.warning = Some("Model selection unavailable · geometry absent");
            result.pick_incomplete = true;
        }
        result
    }
    fn pick(&self, p: Pos2, rect: Rect) -> Result<Option<SceneObjectId>, ()> {
        if !p.is_finite() || !rect.contains(p) {
            return Err(());
        }
        // Glyphs/support wires are x-ray overlays; their stroke hit takes priority
        // over underlying model surfaces, matching the visible rendering order.
        let mut hits = vec![];
        for object in &self.objects {
            if let Some(anchor) = object.anchor.filter(|p| rect.contains(p.pos)) {
                let distance = anchor.pos.distance(p);
                if distance <= PICK_RADIUS {
                    hits.push(Hit {
                        id: object.id,
                        distance,
                        depth: anchor.depth,
                    });
                }
            }
            for &line in &object.lines {
                if let Some((distance, depth)) = line_hit(line, p, self.orthographic) {
                    hits.push(Hit {
                        id: object.id,
                        distance,
                        depth,
                    });
                }
            }
        }
        hits.sort_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then(a.depth.total_cmp(&b.depth))
                .then(a.id.0.cmp(&b.id.0))
        });
        if let Some(hit) = hits.first() {
            return Ok(Some(hit.id));
        }
        let hit = self.model.as_ref().and_then(|model| {
            model
                .triangles
                .iter()
                .filter_map(|&t| triangle_hit(t, p, self.orthographic))
                .min_by(f32::total_cmp)
                .map(|_| model.id)
        });
        if hit.is_none() && self.pick_incomplete {
            Err(())
        } else {
            Ok(hit)
        }
    }
}
pub fn interact(
    ui: &egui::Ui,
    response: &egui::Response,
    camera: &OrbitCamera,
    side: f64,
    mesh: Option<MeshRef<'_>>,
    state: &mut SceneUi,
) {
    if response.clicked_by(egui::PointerButton::Primary)
        && ui.input(|i| i.modifiers.is_none())
        && let Some(p) = response.interact_pointer_pos()
    {
        let projection = Projection::build(state, camera, response.rect, side, mesh);
        if let Ok(hit) = projection.pick(p, response.rect) {
            state.select(hit);
            if hit.is_some() {
                state.open = true;
            }
        }
        response.request_focus();
    }
}
pub struct PaintArea {
    pub rect: Rect,
    pub warning_top: f32,
}
impl From<Rect> for PaintArea {
    fn from(rect: Rect) -> Self {
        Self {
            rect,
            warning_top: rect.top() + 12.0,
        }
    }
}
pub fn paint(
    ui: &egui::Ui,
    area: PaintArea,
    camera: &OrbitCamera,
    side: f64,
    mesh: Option<MeshRef<'_>>,
    state: &SceneUi,
) {
    let rect = area.rect;
    let projection = Projection::build(state, camera, rect, side, mesh);
    ui.interact(rect,ui.id().with("scene-marker-help"),egui::Sense::hover()).on_hover_text("Click a force marker or model to select. Force markers are x-ray authoring aids; glyph size does not represent magnitude. Zero strengths use a neutral anchor. Support wires use metres. Edit properties in Scene.");
    let painter = ui.painter_at(rect);
    let selected = state.selected();
    if let Some(warning) = projection.warning {
        let galley = painter.layout(
            warning.into(),
            egui::FontId::proportional(12.0),
            Color32::YELLOW,
            (rect.width() - 24.0).max(1.0),
        );
        painter.galley(
            egui::pos2(rect.left() + 12.0, area.warning_top),
            galley,
            Color32::YELLOW,
        );
    }
    for object in &projection.objects {
        let is_selected = selected == Some(object.id);
        let color = if is_selected {
            Color32::from_rgb(230, 245, 255)
        } else if object.enabled {
            Color32::from_rgb(90, 180, 215)
        } else {
            Color32::from_rgb(115, 125, 135)
        };
        for line in &object.lines {
            painter.line_segment(
                [line.a.pos, line.b.pos],
                Stroke::new(if is_selected { 1.35 } else { 0.8 }, color),
            );
        }
        if let Some(anchor) = object.anchor.filter(|p| rect.contains(p.pos)) {
            painter.circle_stroke(
                anchor.pos,
                if is_selected { 4.0 } else { 3.0 },
                Stroke::new(1.0, color),
            );
            if is_selected {
                let mut job = egui::text::LayoutJob::simple(
                    object.name.clone(),
                    egui::FontId::proportional(12.0),
                    color,
                    (rect.width() - 20.0).clamp(1.0, 180.0),
                );
                job.wrap.max_rows = 1;
                job.wrap.break_anywhere = true;
                let galley = painter.layout_job(job);
                let origin = egui::pos2(
                    (anchor.pos.x + 10.0).clamp(
                        rect.left(),
                        (rect.right() - galley.size().x).max(rect.left()),
                    ),
                    (anchor.pos.y - 22.0).clamp(
                        rect.top(),
                        (rect.bottom() - galley.size().y).max(rect.top()),
                    ),
                );
                painter.galley(origin, galley, color);
            }
        }
    }
    if let Some(model) = &projection.model
        && selected == Some(model.id)
    {
        for line in &model.outline {
            painter.line_segment(
                [line.a.pos, line.b.pos],
                Stroke::new(1.35, Color32::from_rgb(230, 245, 255)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rheon::{ForceScene, ForceTime, ForceTransform, ForceUnits};
    fn force(id: u64, kind: ForceKind) -> ForceObject {
        ForceObject {
            id: SceneObjectId(id),
            name: format!("Force {id}"),
            enabled: true,
            transform: ForceTransform {
                translation: [0.5; 3],
                ..Default::default()
            },
            units: ForceUnits::Acceleration,
            kind,
            support: ForceSupport::Global,
            time: ForceTime::default(),
        }
    }
    fn camera() -> OrbitCamera {
        OrbitCamera {
            yaw: 0.0,
            pitch: 0.0,
            orthographic: true,
            ..Default::default()
        }
    }
    fn rect() -> Rect {
        Rect::from_min_size(egui::pos2(20.0, 40.0), egui::vec2(600.0, 400.0))
    }
    fn state(objects: Vec<ForceObject>, model: bool) -> SceneUi {
        SceneUi::geometry_fixture(
            ForceScene {
                version: 1,
                objects,
            },
            model,
        )
    }
    #[test]
    fn geometry_depth_near_clip_and_degenerate_hits() {
        let r = rect();
        let p = |x, y, z| Point {
            pos: egui::pos2(x, y),
            depth: z,
        };
        let t = [p(0.0, 0.0, 1.0), p(100.0, 0.0, 2.0), p(0.0, 100.0, 4.0)];
        assert!((triangle_hit(t, egui::pos2(25.0, 25.0), true).unwrap() - 2.0).abs() < 1e-6);
        assert!(
            (triangle_hit(t, egui::pos2(25.0, 25.0), false).unwrap() - 1.0 / 0.6875).abs() < 1e-6
        );
        assert!(triangle_hit([t[0]; 3], egui::Pos2::ZERO, false).is_none());
        assert!(triangle_hit(t, egui::pos2(90.0, 90.0), false).is_none());
        let line = Line {
            a: p(-1000.0, 100.0, 1.0),
            b: p(1000.0, 100.0, 4.0),
        };
        let clipped = clip_line(line, r, false).unwrap();
        assert_eq!(clipped.a.pos.x, r.left());
        assert_eq!(clipped.b.pos.x, r.right());
        assert!(clipped.a.depth > 1.0 && clipped.b.depth < 4.0);
        assert!(line_hit(Line { a: t[0], b: t[0] }, egui::Pos2::ZERO, false).is_some());
        let mut c = camera();
        c.orthographic = false;
        let (eye, forward, right, _) = c.basis();
        let at = |z: f32, x: f32| std::array::from_fn(|d| eye[d] + forward[d] * z + right[d] * x);
        let near = segment(&c, r, at(-1.0, 0.0), at(1.0, 0.0)).unwrap();
        assert!(near.a.depth >= NEAR - 1e-5);
        assert!(segment(&c, r, at(-2.0, 0.0), at(-1.0, 0.0)).is_none());
        assert!(!clipped_triangle(&c, r, [at(-1.0, 0.0), at(1.0, 0.1), at(1.0, -0.1)]).is_empty());
        assert!(normalized([f64::MAX; 3], 1.0).is_none());
        assert!(normalized([0.0; 3], 0.0).is_none());
    }
    #[test]
    fn geometry_two_sided_outline_welds_exact_seams_and_refuses_nonmanifold() {
        let s = state(vec![], true);
        let c = camera();
        let r = rect();
        let v = [
            [0.2, 0.2, 0.5],
            [0.8, 0.2, 0.5],
            [0.8, 0.8, 0.5],
            [0.2, 0.8, 0.5],
        ];
        // Two-sided input may reverse either triangle. The shared diagonal is not an outline.
        for triangles in [[[0, 1, 2], [0, 2, 3]], [[2, 1, 0], [0, 2, 3]]] {
            let projection = Projection::build(
                &s,
                &c,
                r,
                1.0,
                Some(MeshRef {
                    vertices: &v,
                    triangles: &triangles,
                }),
            );
            assert_eq!(projection.model.as_ref().unwrap().outline.len(), 4);
            assert!(projection.warning.is_none());
            assert_eq!(projection.pick(r.center(), r), Ok(s.model_id()));
        }
        let duplicate = [v[0], v[1], v[2], v[0], v[2], v[3]];
        let t = [[0, 1, 2], [5, 4, 3]];
        assert_eq!(
            Projection::build(
                &s,
                &c,
                r,
                1.0,
                Some(MeshRef {
                    vertices: &duplicate,
                    triangles: &t
                })
            )
            .model
            .unwrap()
            .outline
            .len(),
            4
        );
        let disconnected = [[0, 1, 2], [3, 4, 5]];
        let dv = [
            v[0],
            v[1],
            v[2],
            [0.1, 0.1, 0.2],
            [0.15, 0.1, 0.2],
            [0.1, 0.15, 0.2],
        ];
        assert_eq!(
            Projection::build(
                &s,
                &c,
                r,
                1.0,
                Some(MeshRef {
                    vertices: &dv,
                    triangles: &disconnected
                })
            )
            .model
            .unwrap()
            .outline
            .len(),
            6
        );
        let t = [[0, 1, 2], [0, 1, 3], [1, 0, 2]];
        let projection = Projection::build(
            &s,
            &c,
            r,
            1.0,
            Some(MeshRef {
                vertices: &v,
                triangles: &t,
            }),
        );
        assert!(projection.warning.unwrap().contains("non-manifold"));
        assert!(projection.model.as_ref().unwrap().outline.is_empty());
        assert_eq!(projection.pick(r.left_top(), r), Ok(None));
    }
    #[test]
    fn geometry_force_glyphs_preserve_zero_direction_sign_and_support() {
        for kind in [
            ForceKind::Directional { vector: [0.0; 3] },
            ForceKind::Radial { strength: 0.0 },
            ForceKind::Vortex { strength: 0.0 },
        ] {
            assert!(force_lines(&force(1, kind), 1.0).is_empty());
        }
        let mut f = force(
            1,
            ForceKind::Directional {
                vector: [1e300, 0.0, 0.0],
            },
        );
        let a = std::f64::consts::FRAC_PI_4;
        f.transform.rotation = [0.0, 0.0, a.sin(), a.cos()];
        let line = force_lines(&f, 1.0)[0];
        assert!((line.1[0] - 0.5).abs() < 1e-12);
        assert!((line.1[1] - 0.61).abs() < 1e-12);
        for sign in [-1.0, 1.0] {
            let lines = force_lines(&force(1, ForceKind::Radial { strength: sign }), 1.0);
            if sign > 0.0 {
                assert_eq!(lines[0].0, [0.5; 3]);
            } else {
                assert_eq!(lines[0].1, [0.5; 3]);
            }
            let lines = force_lines(&force(1, ForceKind::Vortex { strength: sign }), 1.0);
            assert!((lines[0].1[2] - 0.5) * sign < 0.0);
        }
        f.kind = ForceKind::Radial { strength: 0.0 };
        f.transform = ForceTransform::default();
        f.support = ForceSupport::Box {
            half_extent: [0.2, 0.3, 0.4],
            edge_fraction: 0.0,
        };
        let lines = force_lines(&f, 20.0);
        assert_eq!(lines.len(), 12);
        assert_eq!(lines[0].0, [-0.2, -0.3, -0.4]);
        assert_eq!(lines[0].1, [0.2, -0.3, -0.4]);
    }
    #[test]
    fn scene_pick_identity_overlap_retained_and_unavailable_preview() {
        let a = force(2, ForceKind::Directional { vector: [0.0; 3] });
        let mut s = state(vec![a.clone(), force(1, a.kind)], false);
        let c = camera();
        let r = rect();
        let projection = Projection::build(&s, &c, r, 1.0, None);
        let hit = projection.pick(r.center(), r).unwrap();
        assert_eq!(hit, Some(SceneObjectId(1)));
        s.select(hit);
        assert_eq!(s.selected(), hit);
        assert_eq!(projection.pick(r.left_top(), r), Ok(None));
        let mut disabled = a.clone();
        disabled.enabled = false;
        let p = Projection::build(&state(vec![disabled], false), &c, r, 1.0, None);
        assert!(!p.objects[0].enabled);
        assert_eq!(p.pick(r.center(), r), Ok(Some(SceneObjectId(2))));
        let mut near = force(7, a.kind);
        near.transform.translation[2] = 0.8;
        let p = Projection::build(&state(vec![a.clone(), near], false), &c, r, 1.0, None);
        assert_eq!(p.pick(r.center(), r), Ok(Some(SceneObjectId(7))));
        let m = state(vec![], true);
        let vertices = [[0.2, 0.2, 0.5], [0.8, 0.2, 0.5], [0.2, 0.8, 0.5]];
        let triangles = [[0, 1, 2]];
        let p = Projection::build(
            &m,
            &c,
            r,
            1.0,
            Some(MeshRef {
                vertices: &vertices,
                triangles: &triangles,
            }),
        );
        let outside = c.project([0.7, 0.7, 0.5], r).unwrap().0;
        assert_eq!(p.pick(outside, r), Ok(None));
        let invisible = Projection {
            objects: vec![Object {
                id: SceneObjectId(9),
                name: "Offscreen".into(),
                enabled: true,
                anchor: Some(Point {
                    pos: egui::pos2(r.left() - 1.0, 100.0),
                    depth: 1.0,
                }),
                lines: vec![],
            }],
            model: None,
            warning: None,
            pick_incomplete: false,
            orthographic: true,
        };
        assert_eq!(
            invisible.pick(egui::pos2(r.left() + 1.0, 100.0), r),
            Ok(None)
        );
        let mut invalid_camera = c.clone();
        invalid_camera.yaw = f32::NAN;
        let p = Projection::build(&s, &invalid_camera, r, 1.0, None);
        assert!(p.pick_incomplete);
        assert!(p.pick(r.center(), r).is_err());
        let mut invalid = a.clone();
        invalid.support = ForceSupport::Sphere {
            radius: -1.0,
            edge_fraction: 0.0,
        };
        let mut invalid = state(vec![invalid], false);
        invalid.select(Some(SceneObjectId(2)));
        let p = Projection::build(&invalid, &c, r, 1.0, None);
        assert!(p.warning.is_some());
        assert!(p.pick(r.center(), r).is_err());
        assert_eq!(invalid.selected(), Some(SceneObjectId(2)));
        let mut large = a.clone();
        large.transform.translation = [1e37, 0.5, 0.5];
        let p = Projection::build(&state(vec![large], false), &c, r, 1.0, None);
        assert!(p.pick_incomplete);
        assert!(p.warning.is_some());
        let mut far = a.clone();
        far.transform.translation = [f64::MAX; 3];
        let p = Projection::build(&state(vec![far], false), &c, r, 1.0, None);
        assert!(p.warning.is_some());
        assert!(p.pick(r.center(), r).is_err());
        s.begin_run(false);
        let p = Projection::build(&s, &c, r, 1.0, None);
        assert!(p.objects.is_empty());
        assert_eq!(p.pick(r.center(), r), Ok(None));
        assert_eq!(s.selected(), None);
        let model = state(vec![], true);
        let t = vec![[0, 1, 2]; MODEL_TRIANGLE_LIMIT + 1];
        let p = Projection::build(
            &model,
            &c,
            r,
            1.0,
            Some(MeshRef {
                vertices: &[],
                triangles: &t,
            }),
        );
        assert!(p.warning.unwrap().contains("12,000"));
        assert!(p.pick(r.center(), r).is_err());
    }
    #[test]
    fn scene_pointer_selects_same_identity_ignores_drag_and_modified_click() {
        let ctx = egui::Context::default();
        let mut s = state(vec![force(1, ForceKind::Radial { strength: 0.0 })], false);
        let c = camera();
        let frame = |events: Vec<egui::Event>, modifiers: egui::Modifiers, s: &mut SceneUi| {
            let mut response_rect = Rect::NOTHING;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(700.0, 500.0),
                    )),
                    events,
                    modifiers,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let (r, response) = ui.allocate_exact_size(
                            ui.available_size(),
                            egui::Sense::click_and_drag(),
                        );
                        response_rect = r;
                        interact(ui, &response, &c, 1.0, None, s);
                        paint(ui, r.into(), &c, 1.0, None, s);
                    });
                },
            );
            response_rect
        };
        let r = frame(vec![], egui::Modifiers::NONE, &mut s);
        let p = r.center();
        let button = |pos, pressed, modifiers| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers,
        };
        for modifiers in [egui::Modifiers::SHIFT, egui::Modifiers::NONE] {
            frame(
                vec![egui::Event::PointerMoved(p), button(p, true, modifiers)],
                modifiers,
                &mut s,
            );
            frame(vec![button(p, false, modifiers)], modifiers, &mut s);
            assert_eq!(
                s.selected(),
                if modifiers.is_none() {
                    Some(SceneObjectId(1))
                } else {
                    None
                }
            );
        }
        let end = r.left_top() + egui::vec2(30.0, 30.0);
        frame(
            vec![button(p, true, egui::Modifiers::NONE)],
            egui::Modifiers::NONE,
            &mut s,
        );
        frame(
            vec![egui::Event::PointerMoved(end)],
            egui::Modifiers::NONE,
            &mut s,
        );
        frame(
            vec![button(end, false, egui::Modifiers::NONE)],
            egui::Modifiers::NONE,
            &mut s,
        );
        assert_eq!(s.selected(), Some(SceneObjectId(1)));
        frame(
            vec![button(end, true, egui::Modifiers::NONE)],
            egui::Modifiers::NONE,
            &mut s,
        );
        frame(
            vec![button(end, false, egui::Modifiers::NONE)],
            egui::Modifiers::NONE,
            &mut s,
        );
        assert_eq!(s.selected(), None);
    }
}
