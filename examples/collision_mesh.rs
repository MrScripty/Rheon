//! Reproducible CPU-only import/particle demonstration and collision microprofile.
//! Run with a mesh path, or --profile for deterministic reference workloads.
use rheon::{
    CollisionMesh, DomainFill, FluidPreset, GridGeometry, MeshPolicy, ParticleFluidConfig,
    ParticleFluidSimulation,
    mesh_import::{MeshPlacement, import_mesh},
};
use std::{error::Error, hint::black_box, path::Path, sync::Arc, time::Instant};

fn plane(n: usize) -> Result<CollisionMesh, Box<dyn Error>> {
    let mut vertices = Vec::with_capacity((n + 1) * (n + 1));
    let mut triangles = Vec::with_capacity(2 * n * n);
    for z in 0..=n {
        for x in 0..=n {
            vertices.push([x as f64 / n as f64, 0.4, z as f64 / n as f64]);
        }
    }
    for z in 0..n {
        for x in 0..n {
            let i = x + z * (n + 1);
            triangles.extend([[i, i + 1, i + n + 2], [i, i + n + 2, i + n + 1]]);
        }
    }
    Ok(CollisionMesh::new(
        vertices,
        triangles,
        MeshPolicy::TwoSidedSurface,
    )?)
}

fn simulation(
    phase: FluidPreset,
    mesh: Option<Arc<CollisionMesh>>,
) -> Result<ParticleFluidSimulation, Box<dyn Error>> {
    let grid = GridGeometry::new([16; 3], [1.0 / 16.0; 3], [0.0; 3])?;
    let config = ParticleFluidConfig {
        material: phase.material(),
        fill: DomainFill::EmittedFluid,
        samples_per_axis: 12,
        max_particles: 4096,
        ..ParticleFluidConfig::default()
    };
    let mut sim = ParticleFluidSimulation::new(grid, config)?;
    sim.set_collision_mesh(mesh)?;
    Ok(sim)
}

fn profile() -> Result<(), Box<dyn Error>> {
    println!("CPU wall time, release profile, serial kernels; no GPU/device or real-time claim.");
    println!(
        "kind,triangles,particles,repeat,build_ms,work_ms,queries_or_steps,retained_bytes,accepted_seconds"
    );
    for n in [1, 16, 64, 128] {
        for repeat in 0..5 {
            let start = Instant::now();
            let mesh = plane(n)?;
            let build = start.elapsed().as_secs_f64() * 1000.0;
            let start = Instant::now();
            for i in 0..20_000 {
                let x = ((i * 73 % 1009) as f64 + 0.5) / 1009.0;
                let z = ((i * 29 % 1013) as f64 + 0.5) / 1013.0;
                let (p, v) = mesh.resolve_segment([x, 0.9, z], [x, 0.1, z], [0.0, -1.0, 0.0])?;
                assert!(p[1] >= 0.4 && p[1] < 0.400_001);
                assert!(v[1].abs() < 1e-12);
                black_box((p, v));
            }
            println!(
                "sweep,{},{},{repeat},{build:.6},{:.6},20000,{},0",
                2 * n * n,
                0,
                start.elapsed().as_secs_f64() * 1000.0,
                mesh.allocated_bytes()
            );
        }
    }
    for phase in [FluidPreset::Water, FluidPreset::Air] {
        for enabled in [false, true] {
            for repeat in 0..5 {
                let mesh = if enabled {
                    Some(Arc::new(plane(16)?))
                } else {
                    None
                };
                let mut sim = simulation(phase, mesh)?;
                let start = Instant::now();
                for _ in 0..40 {
                    sim.step(0.001, None, || false)?;
                }
                println!(
                    "{:?}-{},{},{},{repeat},0,{:.6},40,{},{:.9}",
                    phase,
                    if enabled { "barrier" } else { "baseline" },
                    if enabled { 512 } else { 0 },
                    sim.particles().len(),
                    start.elapsed().as_secs_f64() * 1000.0,
                    sim.allocated_bytes(),
                    sim.time()
                );
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.as_slice() == ["--profile"] {
        return profile();
    }
    if args.is_empty() || args.len() > 3 {
        return Err(
            "usage: collision_mesh MODEL.obj|gltf|glb [solid|surface] [water|air], or --profile"
                .into(),
        );
    }
    let policy = match args.get(1).map(String::as_str).unwrap_or("solid") {
        "solid" => MeshPolicy::ConvexSolid,
        "surface" => MeshPolicy::TwoSidedSurface,
        _ => return Err("expected solid or surface".into()),
    };
    let phase = match args.get(2).map(String::as_str).unwrap_or("water") {
        "water" => FluidPreset::Water,
        "air" => FluidPreset::Air,
        _ => return Err("expected water or air".into()),
    };
    let start = Instant::now();
    let imported = import_mesh(Path::new(&args[0]), MeshPlacement::default())?;
    let mesh = Arc::new(CollisionMesh::new(
        imported.vertices,
        imported.triangles,
        policy,
    )?);
    println!(
        "Imported {:?}: {} triangles, bounds {:?}, retained bytes {}, import/build {:.3} ms",
        imported.source_format,
        mesh.triangles().len(),
        mesh.bounds(),
        mesh.allocated_bytes(),
        start.elapsed().as_secs_f64() * 1000.0
    );
    let mut sim = simulation(phase, Some(Arc::clone(&mesh)))?;
    println!(
        "Initial admitted particles: {} (solid interior excluded before time zero)",
        sim.particles().len()
    );
    let start = Instant::now();
    for _ in 0..120 {
        sim.step(0.001, None, || false)?;
    }
    let inside = sim
        .particles()
        .iter()
        .filter(|p| mesh.contains_or_touches(p.position))
        .count();
    println!(
        "{:?}: {} accepted steps, t={:.9}s, {} particles, {} centers inside solid, CPU {:.3}ms",
        phase,
        sim.generation(),
        sim.time(),
        sim.particles().len(),
        inside,
        start.elapsed().as_secs_f64() * 1000.0
    );
    assert_eq!(inside, 0);
    Ok(())
}
