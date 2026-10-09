//! Actual retained force proposal and stationary held frames. No renderer physics.
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
fn frame(b: &SphericalRigidMotion) {
    let s = b.snapshot();
    print!(
        "{{\"time_s\":{:?},\"center\":{:?},\"q\":{:?},\"velocity\":{:?},\"omega\":{:?},\"generation\":{},\"surface_version\":{},\"vertices\":{:?}}}",
        s.time_s,
        s.body.center_of_mass,
        s.orientation,
        s.body.velocity_m_s,
        s.body.angular_velocity_rad_s,
        s.body.stamp.generation,
        s.body.surface.version,
        b.world_surface().vertices()
    );
}
fn mesh(
    s: &mut SplitWhitespace<'_>,
    stamp: SurfaceStamp,
) -> Result<TriangleSurface, Box<dyn Error>> {
    let nv: usize = token(s)?;
    let nt: usize = token(s)?;
    if nv > 192 || nt > 64 {
        return Err("mesh cap".into());
    }
    let mut vs = Vec::with_capacity(nv);
    for _ in 0..nv {
        vs.push(vector(s)?);
    }
    let mut ts = Vec::with_capacity(nt);
    for _ in 0..nt {
        ts.push([token(s)?, token(s)?, token(s)?]);
    }
    Ok(TriangleSurface::new(
        stamp,
        vs,
        ts,
        SurfaceSettings::default(),
    )?)
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut text = String::new();
    io::stdin().take(65537).read_to_string(&mut text)?;
    if text.len() > 65536 {
        return Err("input cap".into());
    }
    let mut s = text.split_whitespace();
    let radius: f64 = token(&mut s)?;
    let h: f64 = token(&mut s)?;
    let steps: usize = token(&mut s)?;
    if steps > 1024 {
        return Err("step cap".into());
    }
    let mass: f64 = token(&mut s)?;
    let inertia: f64 = token(&mut s)?;
    let time: f64 = token(&mut s)?;
    let center = vector(&mut s)?;
    let velocity = vector(&mut s)?;
    let omega = vector(&mut s)?;
    let point = vector(&mut s)?;
    let gravity = vector(&mut s)?;
    let fixed = mesh(&mut s, SurfaceStamp { id: 80, version: 9 })?;
    let moving = mesh(&mut s, SurfaceStamp { id: 17, version: 4 })?;
    let mut traction = Vec::with_capacity(moving.triangles().len());
    for _ in moving.triangles() {
        traction.push([vector(&mut s)?, vector(&mut s)?, vector(&mut s)?]);
    }
    if s.next().is_some() {
        return Err("trailing token".into());
    }
    let body = FrozenRigidBody::new(RigidSnapshot {
        stamp: RigidStamp {
            id: 7,
            generation: 2,
        },
        surface: moving.stamp(),
        center_of_mass: center,
        mass_kg: mass,
        inertia_kg_m2: [inertia; 3],
        velocity_m_s: velocity,
        angular_velocity_rad_s: omega,
    })?;
    let mut owner = SphericalRigidMotion::new(body, moving, time, RigidMotionSettings::default())?;
    print!("{{\"initial\":");
    frame(&owner);
    print!(",\"records\":[");
    let mut status = "Complete".to_string();
    for i in 0..steps.max(1) {
        let state = owner.snapshot().body;
        let request = SphereSupportRequest {
            expected_body: state.stamp,
            expected_moving: state.surface,
            expected_static: fixed.stamp(),
            radius_m: radius,
            contact_point_m: point,
            gravity_m_s2: gravity,
            equivalent_interval_s: h,
            settings: SphereContactSettings::default(),
        };
        let result = if steps == 0 {
            owner
                .stationary_single_face_support(
                    request,
                    &fixed,
                    SurfaceLoading::Traction(&traction),
                    |_, _| false,
                )
                .map(|f| (f, None))
        } else {
            owner
                .advance_stationary_single_face_support(
                    request,
                    &fixed,
                    SurfaceLoading::Traction(&traction),
                    |_, _| false,
                )
                .map(|r| (r.forces, Some(r.zero_load_motion)))
        };
        match result {
            Err(e) => {
                status = format!("{e:?}");
                break;
            }
            Ok((f, m)) => {
                if i > 0 {
                    print!(",");
                }
                print!(
                    "{{\"mesh_force\":{:?},\"mesh_torque\":{:?},\"gravity_force\":{:?},\"external_force\":{:?},\"support_force\":{:?},\"support_torque\":{:?},\"lever\":{:?},\"net_force\":{:?},\"net_torque\":{:?},\"normal\":{:?},\"normal_defect\":{:?},\"magnitude\":{:?},\"probe_point\":{:?},\"probe_defect\":{:?},\"mesh_impulse\":{:?},\"gravity_impulse\":{:?},\"support_impulse\":{:?},\"impulse_defect\":{:?},\"angular_impulse_defect\":{:?},\"external_power\":{:?},\"support_power\":{:?},\"external_work\":{:?},\"support_work\":{:?},\"requested_h\":{:?},\"actual_elapsed\":{:?},\"clock_defect\":{:?},\"proxy_force\":{:?},\"after\":",
                    f.external_mesh.force,
                    f.external_mesh.torque,
                    f.gravity_force_n,
                    f.total_external_force_n,
                    f.support_force_n,
                    f.support_torque_n_m,
                    f.contact_lever_m,
                    f.net_force_n,
                    f.net_torque_n_m,
                    f.normal_estimate,
                    f.normal_norm_defect,
                    f.support_magnitude_n,
                    f.probe.position,
                    f.probe_point_defect_m,
                    f.mesh_impulse_n_s,
                    f.gravity_impulse_n_s,
                    f.support_impulse_n_s,
                    f.linear_impulse_ledger_defect_n_s,
                    f.angular_impulse_ledger_defect_n_m_s,
                    f.external_power_w,
                    f.support_power_w,
                    f.external_work_j,
                    f.support_work_j,
                    h,
                    m.map_or(0., |r| r.represented_elapsed_s),
                    m.map_or(0., |r| r.clock_defect_s),
                    m.map_or([0.; 3], |r| r.impulse.force_n)
                );
                frame(&owner);
                print!("}}");
            }
        }
    }
    print!("],\"status\":{:?},\"final\":", status);
    frame(&owner);
    println!("}}");
    Ok(())
}
