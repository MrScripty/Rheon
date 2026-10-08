//! Bounded actual mesh trajectory; no display-only integration or contact solve.
use rheon::*;
use std::{
    error::Error,
    io::{self, Read},
    str::{FromStr, SplitWhitespace},
};
fn token<T: FromStr>(s: &mut SplitWhitespace<'_>) -> Result<T, Box<dyn Error>> {
    s.next()
        .ok_or("missing token")?
        .parse()
        .map_err(|_| "bad token".into())
}
fn vector(s: &mut SplitWhitespace<'_>) -> Result<[f64; 3], Box<dyn Error>> {
    Ok([token(s)?, token(s)?, token(s)?])
}
fn frame(owner: &SphericalRigidMotion) {
    let s = owner.snapshot();
    print!(
        "{{\"time_s\":{},\"center_of_mass\":{:?},\"orientation\":{:?},\"velocity_m_s\":{:?},\"angular_velocity_rad_s\":{:?},\"generation\":{},\"surface_version\":{},\"vertices\":{:?}}}",
        s.time_s,
        s.body.center_of_mass,
        s.orientation,
        s.body.velocity_m_s,
        s.body.angular_velocity_rad_s,
        s.body.stamp.generation,
        s.body.surface.version,
        owner.world_surface().vertices()
    );
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut text = String::new();
    io::stdin().take(4097).read_to_string(&mut text)?;
    if text.len() > 4096 {
        return Err("input cap".into());
    }
    let mut input = text.split_whitespace();
    let mode = input.next().ok_or("mode")?;
    if !["free", "uniform", "moving", "pressure"].contains(&mode) {
        return Err("mode".into());
    }
    let steps: usize = token(&mut input)?;
    if steps > 64 {
        return Err("step cap".into());
    }
    let h: f64 = token(&mut input)?;
    let v = vector(&mut input)?;
    let w = vector(&mut input)?;
    if input.next().is_some() {
        return Err("trailing input".into());
    }
    let stamp = SurfaceStamp { id: 17, version: 4 };
    let surface = TriangleSurface::new(
        stamp,
        vec![
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ],
        vec![
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 5],
            [0, 5, 4],
            [3, 7, 6],
            [3, 6, 2],
            [0, 4, 7],
            [0, 7, 3],
            [1, 2, 6],
            [1, 6, 5],
        ],
        SurfaceSettings::default(),
    )?;
    let body = FrozenRigidBody::new(RigidSnapshot {
        stamp: RigidStamp {
            id: 7,
            generation: 2,
        },
        surface: stamp,
        center_of_mass: [0.; 3],
        mass_kg: 2.,
        inertia_kg_m2: [1.; 3],
        velocity_m_s: v,
        angular_velocity_rad_s: w,
    })?;
    let mut owner = SphericalRigidMotion::new(body, surface, 0., RigidMotionSettings::default())?;
    print!(
        "{{\"mode\":\"{}\",\"requested_dt_s\":{},\"mass_kg\":2,\"spherical_inertia_kg_m2\":1,\"triangles\":{:?},\"initial\":",
        mode,
        h,
        owner.world_surface().triangles()
    );
    frame(&owner);
    print!(",\"steps\":[");
    for step in 0..steps {
        let mut traction = [[[0.; 3]; 3]; 12];
        let mut pressure = [[0.; 3]; 12];
        if mode == "uniform" {
            traction = [[[0.2, -0.1, 0.05]; 3]; 12];
        }
        if mode == "moving" {
            for row in &mut traction[2..4] {
                *row = [[1.2, 0.3, 0.6]; 3];
            }
        }
        if mode == "pressure" {
            for (i, tri) in owner.world_surface().triangles().iter().enumerate() {
                for (j, &index) in tri.iter().enumerate() {
                    let p = owner.world_surface().vertices()[index];
                    pressure[i][j] = 2. + 0.25 * p[0] - 0.5 * p[1] + 0.75 * p[2];
                }
            }
        }
        let before = owner.snapshot();
        let loading = if mode == "pressure" {
            SurfaceLoading::Pressure(&pressure)
        } else {
            SurfaceLoading::Traction(&traction)
        };
        let r = owner.advance(
            before.body.stamp,
            before.body.surface,
            loading,
            h,
            |_, _| false,
        )?;
        if step > 0 {
            print!(",");
        }
        print!(
            "{{\"traction\":{:?},\"pressure\":{:?},\"force_n\":{:?},\"torque_n_m\":{:?},\"momentum_defect\":{:?},\"angular_momentum_defect\":{:?},\"kinetic_before_j\":{},\"kinetic_after_j\":{},\"impulse_work_j\":{},\"energy_defect_j\":{},\"represented_elapsed_s\":{},\"clock_defect_s\":{},\"rotation_increment_rad\":{},\"quaternion_norm_defect\":{},\"translation_defect_m\":{:?},\"peak_payload_bytes\":{},\"stored\":",
            traction,
            pressure,
            r.impulse.force_n,
            r.impulse.torque_n_m,
            r.impulse.momentum_defect,
            r.impulse.angular_momentum_defect,
            r.impulse.kinetic_before_j,
            r.impulse.kinetic_after_j,
            r.impulse.impulse_work_j,
            r.impulse.energy_defect_j,
            r.represented_elapsed_s,
            r.clock_defect_s,
            r.rotation_increment_rad,
            r.quaternion_norm_defect,
            r.translation_defect_m,
            r.peak_payload_bytes
        );
        frame(&owner);
        print!("}}");
    }
    println!("]}}");
    Ok(())
}
