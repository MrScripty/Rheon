//! Bounded native bridge: explicit sphere collider, actual finite static mesh.
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
        "{{\"time_s\":{},\"center\":{:?},\"q\":{:?},\"velocity\":{:?},\"omega\":{:?},\"generation\":{},\"surface_version\":{},\"vertices\":{:?},\"peak_payload_bytes\":{}}}",
        s.time_s,
        s.body.center_of_mass,
        s.orientation,
        s.body.velocity_m_s,
        s.body.angular_velocity_rad_s,
        s.body.stamp.generation,
        s.body.surface.version,
        owner.world_surface().vertices(),
        owner.peak_payload_bytes()
    );
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut text = String::new();
    io::stdin().take(65537).read_to_string(&mut text)?;
    if text.len() > 65536 {
        return Err("input cap".into());
    }
    let mut input = text.split_whitespace();
    let radius: f64 = token(&mut input)?;
    let restitution: f64 = token(&mut input)?;
    let h: f64 = token(&mut input)?;
    let mass: f64 = token(&mut input)?;
    let inertia: f64 = token(&mut input)?;
    let center = vector(&mut input)?;
    let velocity = vector(&mut input)?;
    let omega = vector(&mut input)?;
    if ![radius, restitution, h, mass, inertia]
        .into_iter()
        .chain(center)
        .chain(velocity)
        .chain(omega)
        .all(f64::is_finite)
    {
        return Err("finite input required".into());
    }
    let nv: usize = token(&mut input)?;
    let nt: usize = token(&mut input)?;
    if nv > 192 || nt > 64 {
        return Err("static mesh cap".into());
    }
    let mut vertices = Vec::with_capacity(nv);
    for _ in 0..nv {
        vertices.push(vector(&mut input)?);
    }
    let mut triangles = Vec::with_capacity(nt);
    for _ in 0..nt {
        triangles.push([token(&mut input)?, token(&mut input)?, token(&mut input)?]);
    }
    if input.next().is_some() {
        return Err("trailing input".into());
    }
    let static_surface = TriangleSurface::new(
        SurfaceStamp { id: 80, version: 9 },
        vertices,
        triangles,
        SurfaceSettings::default(),
    )?;
    // The octahedron is a retained render/traction mesh, distinct from the
    // radius-R analytic collider. Its six vertices lie at half collider radius.
    let r = radius * 0.5;
    let reference = [
        [r, 0., 0.],
        [-r, 0., 0.],
        [0., r, 0.],
        [0., -r, 0.],
        [0., 0., r],
        [0., 0., -r],
    ];
    let p = reference
        .map(|x| [center[0] + x[0], center[1] + x[1], center[2] + x[2]])
        .to_vec();
    let stamp = SurfaceStamp { id: 17, version: 4 };
    let moving = TriangleSurface::new(
        stamp,
        p,
        vec![
            [0, 2, 4],
            [2, 1, 4],
            [1, 3, 4],
            [3, 0, 4],
            [2, 0, 5],
            [1, 2, 5],
            [3, 1, 5],
            [0, 3, 5],
        ],
        SurfaceSettings::default(),
    )?;
    let body = FrozenRigidBody::new(RigidSnapshot {
        stamp: RigidStamp {
            id: 7,
            generation: 2,
        },
        surface: stamp,
        center_of_mass: center,
        mass_kg: mass,
        inertia_kg_m2: [inertia; 3],
        velocity_m_s: velocity,
        angular_velocity_rad_s: omega,
    })?;
    let mut owner = SphericalRigidMotion::new(body, moving, 0., RigidMotionSettings::default())?;
    print!(
        "{{\"radius_m\":{},\"restitution\":{},\"requested_interval_s\":{},\"mass_kg\":{},\"inertia_kg_m2\":{},\"static_vertices\":{:?},\"static_triangles\":{:?},\"moving_triangles\":{:?},\"before\":",
        radius,
        restitution,
        h,
        mass,
        inertia,
        static_surface.vertices(),
        static_surface.triangles(),
        owner.world_surface().triangles()
    );
    frame(&owner);
    let before = owner.snapshot();
    let settings = SphereContactSettings::default();
    print!(
        ",\"collision_shape\":\"declared_sphere\",\"moving_mesh_role\":\"render_and_traction\",\"relative_tolerance\":{},\"max_gap_residual_m\":{},\"simultaneous_window_s\":{},\"static_triangle_limit\":{}",
        static_surface.relative_tolerance(),
        settings.max_gap_residual_m,
        settings.simultaneous_window_s,
        settings.static_triangle_limit
    );
    let result = owner.coast_static_sphere(
        before.body.stamp,
        before.body.surface,
        &static_surface,
        static_surface.stamp(),
        radius,
        restitution,
        h,
        settings,
        |_, _| false,
    );
    match result {
        Err(error) => {
            print!(",\"error\":\"{error:?}\",\"result\":null");
        }
        Ok(report) => {
            print!(
                ",\"error\":null,\"result\":{{\"unused_interval_s\":{},\"requested_event_dt_s\":{},\"represented_elapsed_s\":{},\"clock_defect_s\":{},\"rotation_increment_rad\":{},\"quaternion_norm_defect\":{},\"translation_defect_m\":{:?},\"hit\":",
                report.unused_interval_s,
                report.coast.requested_dt_s,
                report.coast.represented_elapsed_s,
                report.coast.clock_defect_s,
                report.coast.rotation_increment_rad,
                report.coast.quaternion_norm_defect,
                report.coast.translation_defect_m
            );
            if let Some(hit) = report.hit {
                print!(
                    "{{\"triangle\":{},\"feature\":\"{:?}\",\"parameter\":{},\"center\":{:?},\"point\":{:?},\"normal\":{:?},\"barycentric\":{:?},\"gap_residual_m\":{}}}",
                    hit.triangle,
                    hit.feature,
                    hit.parameter,
                    hit.center,
                    hit.point,
                    hit.normal,
                    hit.barycentric,
                    hit.gap_residual_m
                );
            } else {
                print!("null");
            }
            print!(",\"impact\":");
            if let Some(i) = report.impact {
                print!(
                    "{{\"normal\":{:?},\"normal_norm_defect\":{},\"gap_residual_m\":{},\"normal_velocity_before_m_s\":{},\"normal_velocity_after_m_s\":{},\"impulse_n_s\":{:?},\"momentum_defect\":{:?},\"restitution_defect_m_s\":{},\"kinetic_before_j\":{},\"kinetic_after_j\":{},\"predicted_energy_change_j\":{},\"energy_defect_j\":{}}}",
                    i.normal,
                    i.normal_norm_defect,
                    i.gap_residual_m,
                    i.normal_velocity_before_m_s,
                    i.normal_velocity_after_m_s,
                    i.impulse_n_s,
                    i.momentum_defect,
                    i.restitution_defect_m_s,
                    i.kinetic_before_j,
                    i.kinetic_after_j,
                    i.predicted_energy_change_j,
                    i.energy_defect_j
                );
            } else {
                print!("null");
            }
            print!("}}");
        }
    }
    print!(",\"after\":");
    frame(&owner);
    println!("}}");
    Ok(())
}
