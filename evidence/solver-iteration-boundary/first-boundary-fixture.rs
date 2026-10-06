#![allow(dead_code)]
use rheon::*;
pub const CAP: [[f64; 2]; 3] = [[0., 1.], [0.5, 1.25], [1., 1.]];
pub const XI: [f64; 12] = [
    0.25, 0.5, -0.125, 0.375, 0.625, -0.25, 0.1875, 0.4375, -0.0625, 0.5625, 0.3125, 0.125,
];
pub fn geometry() -> FittedHeightGeometry<'static> {
    FittedHeightGeometry {
        cap: &CAP,
        bottom_x: &[0., 0.5, 1.],
        extrusion_width: 1.,
        density: 3.,
        dynamic_viscosity: 0.05,
    }
}
pub fn owner(budget: usize, rest: bool) -> CoupledDiscreteFlow {
    let g = FittedHeightWorkspace::new(geometry(), Default::default(), |_| false).unwrap();
    let mut z = [0.; 34];
    if !rest {
        for n in g.nodes() {
            let e = g.velocity_embedding(n.periodic_index, 0).unwrap();
            if e.weights[0] == 1.
                && let Some(j) = e.columns[0]
            {
                z[j] = n.position[1];
            }
        }
    }
    z[22..].copy_from_slice(&if rest { [0.25; 12] } else { XI });
    let mut u = [[0.; 3]; 16];
    g.embed_velocity(&z, &mut u).unwrap();
    CoupledDiscreteFlow::new_forced_extruded(
        geometry(),
        &u,
        Default::default(),
        TranslatedViscousSettings {
            max_iterations: budget,
            ..Default::default()
        },
        131,
    )
    .unwrap()
}
pub fn force(reverse: bool) -> BodyForce {
    let sign = if reverse { -1. } else { 1. };
    BodyForce {
        value: [0.0625 * sign, -0.125 * sign, 0.03125 * sign],
        units: ForceUnits::Acceleration,
        region: None,
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Snapshot {
    velocity: [[u64; 3]; 16],
    positions: [[u64; 2]; 19],
    mass: [u64; 16],
    pressure: [u64; 16],
    time: u64,
    stamp: TranslatedViscousStamp,
    triangles: [[usize; 3]; 24],
    periodic_indices: [usize; 19],
}
pub fn snapshot(o: &CoupledDiscreteFlow) -> Snapshot {
    let s = o.state();
    Snapshot {
        velocity: std::array::from_fn(|i| s.velocity[i].map(f64::to_bits)),
        positions: std::array::from_fn(|i| s.geometry.nodes()[i].position.map(f64::to_bits)),
        mass: std::array::from_fn(|i| s.geometry.nodal_mass()[i].to_bits()),
        pressure: s.pressure_coefficients.map(f64::to_bits),
        time: s.time.to_bits(),
        stamp: s.stamp,
        triangles: std::array::from_fn(|i| s.geometry.triangles()[i].nodes),
        periodic_indices: std::array::from_fn(|i| s.geometry.nodes()[i].periodic_index),
    }
}
