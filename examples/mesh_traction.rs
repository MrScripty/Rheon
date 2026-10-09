//! Bounded stdin bridge for physical triangle loads; no simulation or stepping.
use rheon::{SurfaceLoading, SurfaceSettings, SurfaceStamp, TriangleMeshLoad, TriangleSurface};
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
    let report = owner.reduce(velocity, angular, |_, _| false)?;
    print!(
        "{{\"force\":{:?},\"torque\":{:?},\"rigid_power\":{},\"nodal_power\":{},\"power_defect\":{},\"triangles\":{},\"triangle_loads\":[",
        report.force,
        report.torque,
        report.rigid_power,
        report.nodal_power,
        report.power_defect,
        report.triangles
    );
    for i in 0..nt {
        let load = owner.triangle_load(i)?;
        if i > 0 {
            print!(",");
        }
        print!(
            "{{\"triangle\":{},\"vertices\":{:?},\"area_m2\":{},\"nodal_force\":{:?},\"force\":{:?},\"torque\":{:?}}}",
            load.triangle, load.vertices, load.area_m2, load.nodal_force, load.force, load.torque
        );
    }
    println!("]}}");
    Ok(())
}
