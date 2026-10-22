//! CV3-A-v1 continuum fields; all derived constants use represented primitives.
use super::super::{MU, RHO};
pub const DT: f64 = 1.0;
pub const LENGTH: f64 = 1.0;
pub const SPEED: f64 = 1.0;
pub const DEPTH: f64 = 1.0;
pub const STRESS: f64 = 1.0;
pub fn velocity([x, y]: [f64; 2]) -> [f64; 2] {
    let (x, y) = (x / LENGTH, y / LENGTH);
    [
        SPEED * (x * x + 2.0 * x * y),
        SPEED * (-2.0 * x * y - y * y),
    ]
}
pub fn gradient([x, y]: [f64; 2]) -> [[f64; 2]; 2] {
    let (x, y) = (x / LENGTH, y / LENGTH);
    let s = SPEED / LENGTH;
    [
        [2.0 * s * (x + y), 2.0 * s * x],
        [-2.0 * s * y, -2.0 * s * (x + y)],
    ]
}
pub fn pressure([x, y]: [f64; 2]) -> f64 {
    STRESS * (1.0 + x / LENGTH + 2.0 * y / LENGTH + x * y / (LENGTH * LENGTH))
}
pub fn stress(point: [f64; 2]) -> [f64; 3] {
    let g = gradient(point);
    let p = pressure(point);
    [
        -p + 2.0 * MU * g[0][0],
        MU * (g[0][1] + g[1][0]),
        -p + 2.0 * MU * g[1][1],
    ]
}
pub fn body(point: [f64; 2]) -> [f64; 2] {
    let u = velocity(point);
    let c = 2.0 * MU * SPEED / (LENGTH * LENGTH);
    [
        RHO * u[0] / DT - c + STRESS / LENGTH * (1.0 + point[1] / LENGTH),
        RHO * u[1] / DT + c + STRESS / LENGTH * (2.0 + point[0] / LENGTH),
    ]
}
/// The caller supplies the outward normal TIMES integration measure.
pub fn traction(point: [f64; 2], normal_measure: [f64; 2]) -> [f64; 2] {
    let [xx, xy, yy] = stress(point);
    let [nx, ny] = normal_measure;
    [xx * nx + xy * ny, xy * nx + yy * ny]
}
