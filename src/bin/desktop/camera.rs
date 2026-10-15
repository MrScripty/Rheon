//! Shared desktop orbit camera; navigation never changes numerical state.
use eframe::egui::{self, Pos2, Rect, Vec2};
type V3 = [f32; 3];
fn add(a: V3, b: V3) -> V3 {
    std::array::from_fn(|d| a[d] + b[d])
}
fn sub(a: V3, b: V3) -> V3 {
    std::array::from_fn(|d| a[d] - b[d])
}
fn mul(a: V3, s: f32) -> V3 {
    a.map(|v| v * s)
}
fn dot(a: V3, b: V3) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(a: V3) -> V3 {
    mul(a, 1.0 / dot(a, a).sqrt().max(1e-8))
}
#[derive(Clone, Debug, PartialEq)]
pub struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: V3,
    pub orthographic: bool,
}
impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.72,
            pitch: 0.42,
            distance: 2.35,
            target: [0.5; 3],
            orthographic: false,
        }
    }
}
impl OrbitCamera {
    pub(super) fn basis(&self) -> (V3, V3, V3, V3) {
        let eye = add(
            self.target,
            mul(
                [
                    self.yaw.sin() * self.pitch.cos(),
                    self.pitch.sin(),
                    self.yaw.cos() * self.pitch.cos(),
                ],
                self.distance,
            ),
        );
        let forward = normalize(sub(self.target, eye));
        let right = normalize(cross(forward, [0.0, 1.0, 0.0]));
        let up = cross(right, forward);
        (eye, forward, right, up)
    }
    pub fn orbit(&mut self, delta: Vec2) {
        self.yaw -= delta.x * 0.006;
        self.pitch = (self.pitch + delta.y * 0.006).clamp(-1.52, 1.52);
    }
    pub fn pan(&mut self, delta: Vec2, height: f32) {
        let (_, _, right, up) = self.basis();
        let scale = self.distance / height.max(1.0);
        self.target = add(
            self.target,
            add(mul(right, -delta.x * scale), mul(up, delta.y * scale)),
        )
        .map(|v| v.clamp(-3.0, 4.0));
    }
    pub fn zoom(&mut self, log_delta: f32) {
        self.distance = (self.distance * (-log_delta.clamp(-2.0, 2.0)).exp()).clamp(0.35, 12.0);
    }
    pub(super) fn project(&self, p: V3, rect: Rect) -> Option<(Pos2, f32)> {
        self.project_prepared(p, rect, self.basis())
    }
    pub(super) fn project_prepared(
        &self,
        p: V3,
        rect: Rect,
        basis: (V3, V3, V3, V3),
    ) -> Option<(Pos2, f32)> {
        let (eye, forward, right, up) = basis;
        let relative = sub(p, eye);
        let depth = dot(relative, forward);
        if depth <= 0.015 {
            return None;
        }
        let scale = rect.height() * 0.88
            / if self.orthographic {
                self.distance
            } else {
                depth
            };
        Some((
            rect.center() + egui::vec2(dot(relative, right) * scale, -dot(relative, up) * scale),
            depth,
        ))
    }
}
