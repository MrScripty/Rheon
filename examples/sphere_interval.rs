//! Actual stored multi-impact sequence. No renderer-side simulation.
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
    let budget: usize = token(&mut input)?;
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
    let request = SphereIntervalRequest {
        expected_body: before.body.stamp,
        expected_moving: before.body.surface,
        expected_static: static_surface.stamp(),
        radius_m: radius,
        restitution,
        interval_s: h,
        settings,
        max_impacts: budget,
    };
    let mut records = [None; MAX_SPHERE_INTERVAL_IMPACTS + 1];
    print!(",\"impact_budget\":{budget},\"sequence\":[");
    let result = owner.advance_static_sphere_interval(request, &static_surface, &mut records, |_,_,_| false, |index, stored, segment| {
        if index != 0 { print!(","); }
        let c = segment.contact;
        print!("{{\"frame\":"); frame(stored);
        print!(",\"duration_s\":{},\"remaining_s\":{},\"clock_defect_s\":{},\"translation_defect_m\":{:?},\"rotation_increment_rad\":{},\"quaternion_norm_defect\":{},\"departure\":", c.hit.map_or(c.requested_interval_s, |h| h.requested_event_dt_s), c.unused_interval_s, c.coast.clock_defect_s, c.coast.translation_defect_m, c.coast.rotation_increment_rad, c.coast.quaternion_norm_defect);
        if let Some(d) = segment.departure {
            print!("{{\"triangle\":{},\"point\":{:?},\"kind\":\"{:?}\",\"from_generation\":{},\"from_surface_version\":{}}}", d.triangle, d.point, d.kind, d.from.body.stamp.generation, d.from.body.surface.version);
        } else { print!("null"); }
        print!(",\"hit\":");
        if let Some(hit) = c.hit {
            print!("{{\"triangle\":{},\"feature\":\"{:?}\",\"point\":{:?},\"normal\":{:?},\"center\":{:?},\"barycentric\":{:?},\"gap_residual_m\":{}}}",hit.triangle,hit.feature,hit.point,hit.normal,hit.center,hit.barycentric,hit.gap_residual_m);
        } else { print!("null"); }
        print!(",\"impact\":");
        if let Some(i) = c.impact {
            print!("{{\"normal\":{:?},\"normal_norm_defect\":{},\"gap_residual_m\":{},\"normal_velocity_before_m_s\":{},\"normal_velocity_after_m_s\":{},\"impulse_n_s\":{:?},\"momentum_defect\":{:?},\"restitution_defect_m_s\":{},\"kinetic_before_j\":{},\"kinetic_after_j\":{},\"predicted_energy_change_j\":{},\"energy_defect_j\":{}}}",i.normal,i.normal_norm_defect,i.gap_residual_m,i.normal_velocity_before_m_s,i.normal_velocity_after_m_s,i.impulse_n_s,i.momentum_defect,i.restitution_defect_m_s,i.kinetic_before_j,i.kinetic_after_j,i.predicted_energy_change_j,i.energy_defect_j);
        } else { print!("null"); }
        print!("}}");
    });
    print!("]");
    match result {
        Err(error) => print!(",\"admission_error\":\"{error:?}\",\"result\":null"),
        Ok(r) => {
            print!(
                ",\"admission_error\":null,\"result\":{{\"status\":\"{:?}\",\"remaining_interval_s\":{},\"consumed_interval_s\":{},\"accepted_segments\":{},\"accepted_impacts\":{},\"accounting\":",
                r.status,
                r.remaining_interval_s,
                r.consumed_interval_s,
                r.accepted_segments,
                r.accepted_impacts
            );
            if let Some(a) = r.accounting {
                print!(
                    "{{\"summed_impulse_n_s\":{:?},\"momentum_defect\":{:?},\"summed_predicted_energy_change_j\":{},\"summed_event_energy_change_j\":{},\"kinetic_change_j\":{},\"energy_defect_j\":{},\"summed_segment_duration_s\":{},\"represented_elapsed_s\":{},\"clock_defect_s\":{},\"duration_defect_s\":{}}}",
                    a.summed_impulse_n_s,
                    a.momentum_defect,
                    a.summed_predicted_energy_change_j,
                    a.summed_event_energy_change_j,
                    a.kinetic_change_j,
                    a.energy_defect_j,
                    a.summed_segment_duration_s,
                    a.represented_elapsed_s,
                    a.clock_defect_s,
                    a.duration_defect_s
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
