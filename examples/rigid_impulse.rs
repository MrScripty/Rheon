//! Bounded stdin bridge for actual owned fixed-pose mesh impulse response.
use rheon::{
    FrozenRigidBody, RigidSnapshot, RigidStamp, SurfaceLoading, SurfaceSettings, SurfaceStamp,
    TriangleMeshLoad, TriangleSurface,
};
use std::{
    error::Error,
    io::{self, Read},
    str::{FromStr, SplitWhitespace},
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn token<T: FromStr>(input: &mut SplitWhitespace<'_>) -> Result<T> {
    input
        .next()
        .ok_or("missing token")?
        .parse()
        .map_err(|_| "invalid token".into())
}
fn vector(input: &mut SplitWhitespace<'_>) -> Result<[f64; 3]> {
    Ok([token(input)?, token(input)?, token(input)?])
}
fn main() -> Result<()> {
    let mut text = String::new();
    io::stdin().take(65537).read_to_string(&mut text)?;
    if text.len() > 65536 {
        return Err("input byte cap".into());
    }
    let mut input = text.split_whitespace();
    let mode = input.next().ok_or("missing loading mode")?;
    if mode == "inertia" {
        let count: usize = token(&mut input)?;
        if count > 1024 {
            return Err("inertia fixture cap".into());
        }
        print!("[");
        for i in 0..count {
            let moments = vector(&mut input)?;
            let state = RigidSnapshot {
                stamp: RigidStamp {
                    id: 7,
                    generation: 2,
                },
                surface: SurfaceStamp { id: 17, version: 4 },
                center_of_mass: [0.; 3],
                mass_kg: 1.,
                inertia_kg_m2: moments,
                velocity_m_s: [0.; 3],
                angular_velocity_rad_s: [0.; 3],
            };
            if i > 0 {
                print!(",");
            }
            print!("{}", FrozenRigidBody::new(state).is_ok());
        }
        if input.next().is_some() {
            return Err("trailing input".into());
        }
        println!("]");
        return Ok(());
    }
    if !["pressure", "traction"].contains(&mode) {
        return Err("loading mode".into());
    }
    let nv: usize = token(&mut input)?;
    let nt: usize = token(&mut input)?;
    if nv > 192 || nt > 64 {
        return Err("fixture count cap".into());
    }
    let mut vertices = Vec::with_capacity(nv);
    for _ in 0..nv {
        vertices.push(vector(&mut input)?);
    }
    let mut triangles = Vec::with_capacity(nt);
    for _ in 0..nt {
        triangles.push([token(&mut input)?, token(&mut input)?, token(&mut input)?]);
    }
    let reference = vector(&mut input)?;
    let velocity = vector(&mut input)?;
    let angular = vector(&mut input)?;
    let mass = token(&mut input)?;
    let inertia = vector(&mut input)?;
    let duration = token(&mut input)?;
    let mut traction = Vec::new();
    let mut pressure = Vec::new();
    if mode == "pressure" {
        for _ in 0..nt {
            pressure.push(vector(&mut input)?);
        }
    } else {
        for _ in 0..nt {
            traction.push([
                vector(&mut input)?,
                vector(&mut input)?,
                vector(&mut input)?,
            ]);
        }
    }
    if input.next().is_some() {
        return Err("trailing input".into());
    }
    let stamp = SurfaceStamp { id: 17, version: 4 };
    let surface = TriangleSurface::new(
        stamp,
        vertices,
        triangles,
        SurfaceSettings {
            memory_limit: 65536,
            ..Default::default()
        },
    )?;
    let loading = if mode == "pressure" {
        SurfaceLoading::Pressure(&pressure)
    } else {
        SurfaceLoading::Traction(&traction)
    };
    let owner = TriangleMeshLoad::new(&surface, stamp, loading, reference, 64, |_, _| false)?;
    let initial = RigidSnapshot {
        stamp: RigidStamp {
            id: 7,
            generation: 2,
        },
        surface: stamp,
        center_of_mass: reference,
        mass_kg: mass,
        inertia_kg_m2: inertia,
        velocity_m_s: velocity,
        angular_velocity_rad_s: angular,
    };
    let mut body = FrozenRigidBody::new(initial)?;
    let report = body.apply_mesh_impulse(initial.stamp, &owner, duration, |_, _| false)?;
    let stored = body.snapshot();
    println!(
        "{{\"force_n\":{:?},\"torque_n_m\":{:?},\"impulse_n_s\":{:?},\"angular_impulse_n_m_s\":{:?},\"velocity_before_m_s\":{:?},\"velocity_after_m_s\":{:?},\"angular_before_rad_s\":{:?},\"angular_after_rad_s\":{:?},\"momentum_defect\":{:?},\"angular_momentum_defect\":{:?},\"kinetic_before_j\":{},\"kinetic_after_j\":{},\"impulse_work_j\":{},\"update_work_j\":{},\"energy_defect_j\":{},\"update_energy_defect_j\":{},\"generation_before\":{},\"generation_after\":{},\"equivalent_duration_s\":{}}}",
        report.force_n,
        report.torque_n_m,
        report.impulse_n_s,
        report.angular_impulse_n_m_s,
        initial.velocity_m_s,
        stored.velocity_m_s,
        initial.angular_velocity_rad_s,
        stored.angular_velocity_rad_s,
        report.momentum_defect,
        report.angular_momentum_defect,
        report.kinetic_before_j,
        report.kinetic_after_j,
        report.impulse_work_j,
        report.update_work_j,
        report.energy_defect_j,
        report.update_energy_defect_j,
        initial.stamp.generation,
        stored.stamp.generation,
        duration
    );
    Ok(())
}
