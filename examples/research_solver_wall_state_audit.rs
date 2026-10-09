//! Read-only accepted-state and type-layout audit. Never steps or solves.
use rheon::{
    GridGeometry, PressureSettings, Simulation, SimulationConfig, VelocitySampler,
    ViscosityWorkspace,
};
use std::{error::Error, fs::OpenOptions, io::Write, mem::size_of, path::Path};
const CAP: usize = 16_000_000;
#[allow(dead_code)]
struct ProposedLeafCell {
    key: u64,
    volume: f64,
    level: u32,
    flags: u32,
}
#[allow(dead_code)]
struct ProposedFace {
    minus: u32,
    plus: u32,
    axis: u32,
    flags: u32,
    center: [f64; 3],
    area: f64,
    distance: f64,
    mass: f64,
}
#[allow(dead_code)]
struct ProposedGhostTransfer {
    ids: [u32; 8],
    weights: [f64; 8],
}
#[allow(dead_code)]
struct ProposedQuadraticTransfer {
    ids: [u32; 27],
    weights: [f64; 27],
}
#[allow(dead_code)]
struct ProposedTerm {
    id: u32,
    coefficient: f64,
}
#[allow(dead_code)]
struct ProposedTileRow {
    weight: f64,
    count: u32,
    flags: u32,
    terms: [ProposedTerm; 64],
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        return Err("NEW_EXTERNAL_JSON".into());
    }
    let path = Path::new(&args[0]);
    if path
        .parent()
        .ok_or("parent")?
        .canonicalize()?
        .starts_with(env!("CARGO_MANIFEST_DIR"))
    {
        return Err("external output required".into());
    }
    let mut out = OpenOptions::new().write(true).create_new(true).open(path)?;
    write!(
        out,
        "{{\"schema\":\"actual-solver-state-read-only-audit-v1\",\"cap\":{CAP},\"proposed_layout\":{{\"cell\":{},\"face\":{},\"ghost_transfer\":{},\"quadratic_transfer\":{},\"tile_row\":{}}},\"states\":[",
        size_of::<ProposedLeafCell>(),
        size_of::<ProposedFace>(),
        size_of::<ProposedGhostTransfer>(),
        size_of::<ProposedQuadraticTransfer>(),
        size_of::<ProposedTileRow>()
    )?;
    for (index, n) in [3usize, 6, 12].into_iter().enumerate() {
        if index > 0 {
            write!(out, ",")?;
        }
        let grid = GridGeometry::new([n as u64; 3], [3. / n as f64; 3], [0.; 3])?;
        let owner = Simulation::new(
            grid.clone(),
            SimulationConfig {
                density: 1.,
                memory_limit: CAP,
                pressure: PressureSettings {
                    relative_residual: 1e-10,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-6,
                    max_iterations: 100,
                },
                actual_divergence_limit: 1e-6,
                max_courant: 0.25,
            },
        )?;
        let viscosity = ViscosityWorkspace::new(grid.clone(), CAP - owner.allocated_bytes())?;
        let state = owner.state();
        assert_eq!(state.generation, 0);
        assert_eq!(state.time, 0.);
        assert!(
            state
                .x
                .iter()
                .chain(state.y)
                .chain(state.z)
                .chain(state.tracer)
                .all(|v| v.to_bits() == 0)
        );
        let sampler = VelocitySampler::new(owner.grid(), [state.x, state.y, state.z])?;
        let mut queries = 0usize;
        let mut max: f64 = 0.;
        for s in [8usize, 16, 32] {
            let h = 1. / s as f64;
            for normal in 0..3 {
                let tangent = match normal {
                    0 => [1, 2],
                    1 => [0, 2],
                    _ => [0, 1],
                };
                for side in [-1., 1.] {
                    for i in 0..s {
                        for j in 0..s {
                            for layer in [0.5, 1.5] {
                                let mut p = [0.; 3];
                                p[normal] = if side < 0. { 1. } else { 2. };
                                p[normal] += side * layer * h;
                                p[tangent[0]] = 1. + (i as f64 + 0.5) * h;
                                p[tangent[1]] = 1. + (j as f64 + 0.5) * h;
                                for value in sampler.sample(p)? {
                                    max = max.max(value.abs());
                                }
                                queries += 1;
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(max, 0.);
        assert_eq!(owner.state().generation, 0);
        assert_eq!(owner.state().time, 0.);
        write!(
            out,
            "{{\"N\":{n},\"time\":0,\"generation\":0,\"simulation_bytes\":{},\"free_slip_viscosity_bytes\":{},\"face_values\":{},\"queried_positions\":{queries},\"max_interpolated_speed\":{max},\"new_independent_information\":false,\"obstacle_velocity_state\":false,\"read_only\":true}}",
            owner.allocated_bytes(),
            viscosity.allocated_bytes(),
            state.x.len() + state.y.len() + state.z.len()
        )?;
    }
    write!(out, "]}}\n")?;
    Ok(())
}
