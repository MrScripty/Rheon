//! Display-only full-resolution voxel fields, shared geometry and signed legends.
use super::engine::Frame;
use super::scene::Scene;
use crate::camera::OrbitCamera;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
pub fn color(value: f64, scale: f64) -> Color32 {
    let t = if scale > 0.0 {
        (value / scale).clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let zero = [247.0, 247.0, 247.0];
    let end = if t < 0.0 {
        [49.0, 102.0, 172.0]
    } else {
        [180.0, 4.0, 38.0]
    };
    let rgb =
        std::array::from_fn::<_, 3, _>(|i| (zero[i] + t.abs() * (end[i] - zero[i])).round() as u8);
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}
pub fn ranges(frame: &Frame) -> (f64, f64) {
    let pressure = frame
        .reference_pressure
        .iter()
        .chain(&frame.ai_pressure)
        .fold(0.0_f64, |a, b| a.max(b.abs()));
    let delta = frame
        .ai_pressure
        .iter()
        .zip(&frame.reference_pressure)
        .fold(0.0_f64, |a, (ai, reference)| a.max((ai - reference).abs()));
    (pressure, delta)
}
pub fn point(scene: &Scene, p: [usize; 3]) -> [f32; 3] {
    let extents = scene.extents();
    let scale = extents.into_iter().fold(0.0_f64, f64::max);
    std::array::from_fn(|d| (0.5 + (p[d] as f64 * scene.h() - extents[d] / 2.0) / scale) as f32)
}
#[derive(Clone, Copy)]
pub enum Field {
    Reference,
    Model,
    Difference,
}
fn field(frame: &Frame, index: usize, selected: Field) -> f64 {
    match selected {
        Field::Difference => frame.ai_pressure[index] - frame.reference_pressure[index],
        Field::Model => frame.ai_pressure[index],
        Field::Reference => frame.reference_pressure[index],
    }
}
pub struct Display<'a> {
    pub scene: &'a Scene,
    pub frame: &'a Frame,
    pub camera: &'a OrbitCamera,
    pub cut: usize,
    pub field: Field,
    pub scale: f64,
}
fn inside(point: Pos2, polygon: &[Pos2; 4]) -> bool {
    let mut sign = 0.0_f32;
    for i in 0..4 {
        let a = polygon[i];
        let b = polygon[(i + 1) % 4];
        let cross = (b.x - a.x) * (point.y - a.y) - (b.y - a.y) * (point.x - a.x);
        if cross.abs() < 1e-5 {
            continue;
        }
        if sign == 0.0 {
            sign = cross;
        } else if sign * cross < 0.0 {
            return false;
        }
    }
    true
}
pub fn legend(ui: &mut egui::Ui, scale: f64, label: &str) {
    ui.label(label);
    let width = ui.available_width().max(20.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 14.0), egui::Sense::hover());
    for i in 0..128 {
        let t = i as f64 / 127.0;
        ui.painter().rect_filled(
            Rect::from_min_size(
                rect.min + egui::vec2(width * i as f32 / 128.0, 0.0),
                egui::vec2(width / 128.0 + 0.5, 14.0),
            ),
            0.0,
            color(scale * (2.0 * t - 1.0), scale),
        );
    }
    ui.horizontal(|ui| {
        ui.label(format!("−{scale:.3e} Pa"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("+{scale:.3e} Pa"));
        });
    });
    let zero = rect.center_bottom() + egui::vec2(0.0, 3.0);
    ui.painter().text(
        zero,
        egui::Align2::CENTER_TOP,
        "0 Pa",
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
    ui.add_space(8.0);
}
struct Quad {
    points: [Pos2; 4],
    depth: f32,
    index: usize,
    color: Color32,
}
pub fn voxels(ui: &mut egui::Ui, display: Display<'_>, selected: &mut Option<usize>) {
    let Display {
        scene,
        frame,
        camera,
        cut,
        field: selected_field,
        scale,
    } = display;
    let size = egui::vec2(ui.available_width(), ui.available_height().max(180.0));
    let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
    let rect = response.rect;
    let counts = scene.counts;
    let visible = [counts[0], counts[1], cut.min(counts[2] - 1) + 1];
    let mut quads = Vec::new();
    for k in 0..visible[2] {
        for j in 0..visible[1] {
            for i in 0..visible[0] {
                let cell = [i, j, k];
                let index = i + counts[0] * (j + counts[1] * k);
                for axis in 0..3 {
                    for side in 0..2 {
                        if (side == 0 && cell[axis] != 0)
                            || (side == 1 && cell[axis] + 1 != visible[axis])
                        {
                            continue;
                        }
                        let a = (axis + 1) % 3;
                        let b = (axis + 2) % 3;
                        let corners = [[0, 0], [1, 0], [1, 1], [0, 1]].map(|ab| {
                            let mut p = cell;
                            p[axis] += side;
                            p[a] += ab[0];
                            p[b] += ab[1];
                            camera.project(point(scene, p), rect)
                        });
                        if corners.iter().any(Option::is_none) {
                            continue;
                        }
                        let corners = corners.map(Option::unwrap);
                        let points = corners.map(|v| v.0);
                        let depth = corners.iter().map(|v| v.1).sum::<f32>() / 4.0;
                        quads.push(Quad {
                            points,
                            depth,
                            index,
                            color: color(field(frame, index, selected_field), scale),
                        });
                    }
                }
            }
        }
    }
    quads.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    let mut picked = None;
    for quad in &quads {
        let stroke = if *selected == Some(quad.index) {
            Stroke::new(2.0, Color32::YELLOW)
        } else {
            Stroke::new(0.35, Color32::from_black_alpha(70))
        };
        painter.add(egui::Shape::convex_polygon(
            quad.points.to_vec(),
            quad.color,
            stroke,
        ));
        if response.clicked()
            && let Some(p) = response.interact_pointer_pos()
            && inside(p, &quad.points)
        {
            picked = Some(quad.index);
        }
    }
    if picked.is_some() {
        *selected = picked;
    }
    for corner in 0..8 {
        for d in 0..3 {
            if corner & (1 << d) == 0 {
                let a = std::array::from_fn(|axis| ((corner >> axis) & 1) * counts[axis]);
                let b =
                    std::array::from_fn(|axis| (((corner | (1 << d)) >> axis) & 1) * counts[axis]);
                if let (Some((a, _)), Some((b, _))) = (
                    camera.project(point(scene, a), rect),
                    camera.project(point(scene, b), rect),
                ) {
                    painter.line_segment([a, b], Stroke::new(0.8, Color32::GRAY));
                }
            }
        }
    }
    painter.text(
        rect.left_top() + Vec2::splat(8.0),
        egui::Align2::LEFT_TOP,
        format!(
            "Z cut: {}/{} · cubic h = {:.5} m",
            visible[2],
            counts[2],
            scene.h()
        ),
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn difference_is_signed_zero_centered_and_range_uses_all_cells() {
        let mut frame = Frame {
            phase: super::super::engine::FramePhase::AiPrediction,
            step: 0,
            time_s: 0.0,
            reference_pressure: vec![0.0, 1.0, -2.0],
            ai_pressure: vec![0.0, -1.0, 3.0],
            reference_tracer: vec![],
            ai_tracer: vec![],
            reference_quality: None,
            ai_quality: None,
            reference_seconds: 0.0,
            ai_seconds: 0.0,
        };
        assert_eq!(ranges(&frame), (3.0, 5.0));
        assert_eq!(field(&frame, 1, Field::Difference), -2.0);
        assert_eq!(field(&frame, 2, Field::Difference), 5.0);
        assert_eq!(color(0.0, 5.0), color(0.0, 0.0));
        assert_ne!(color(-1.0, 1.0), color(1.0, 1.0));
        frame.ai_pressure = frame.reference_pressure.clone();
        assert_eq!(ranges(&frame).1, 0.0);
    }
    #[test]
    fn rectangular_world_preserves_equal_edges_and_picking() {
        let scene = Scene {
            counts: [24, 16, 12],
            ..Default::default()
        };
        let origin = point(&scene, [0, 0, 0]);
        let edges = std::array::from_fn::<_, 3, _>(|d| {
            let mut p = [0; 3];
            p[d] = 1;
            let endpoint = point(&scene, p);
            (0..3)
                .map(|a| (endpoint[a] - origin[a]).powi(2))
                .sum::<f32>()
                .sqrt()
        });
        assert!((edges[0] - edges[1]).abs() < 1e-7 && (edges[1] - edges[2]).abs() < 1e-7);
        let polygon = [
            Pos2::new(0.0, 0.0),
            Pos2::new(1.0, 0.0),
            Pos2::new(1.0, 1.0),
            Pos2::new(0.0, 1.0),
        ];
        assert!(inside(Pos2::new(0.5, 0.5), &polygon));
        assert!(!inside(Pos2::new(1.5, 0.5), &polygon));
    }
}
