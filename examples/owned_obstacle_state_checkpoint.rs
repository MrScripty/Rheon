//! Explicit caller-declared known-rest snapshot and streamed checkpoint example.
//! No analytic oracle, physical solve, pressure or stepping. Digests are REQUIRED
//! arguments for external problem/boundary/initial/forcing declarations.
#[path = "viscous_boundary_wrench.rs"]
#[allow(dead_code)]
mod fixture;
use rheon::*;
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::Path,
};
fn geometry(n: usize) -> Result<StaticObstacleGeometry, Box<dyn Error>> {
    if ![3, 6, 12].contains(&n) {
        return Err("only bounded N3/6/12 fixtures supported".into());
    }
    fixture::geometry(
        [n as u64; 3],
        [3. / n as f64; 3],
        [0.; 3],
        [n / 3; 3],
        [2 * n / 3; 3],
    )
}
fn evidence(id: u64, hex: &str) -> Result<ObstacleEvidenceRef, Box<dyn Error>> {
    if hex.len() != 64 || !hex.is_ascii() {
        return Err("required SHA256 must contain exactly 64 ASCII hex digits".into());
    }
    let mut sha = [0; 32];
    for (i, b) in sha.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)?;
    }
    Ok(ObstacleEvidenceRef::new(id, 0, sha)?)
}
fn output(path: &Path) -> Result<BufWriter<File>, Box<dyn Error>> {
    Ok(BufWriter::with_capacity(
        8192,
        OpenOptions::new().write(true).create_new(true).open(path)?,
    ))
}
fn main() -> Result<(), Box<dyn Error>> {
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    if a.len() == 3 && a[0] == "check" {
        let g = geometry(a[1].parse()?)?;
        let s = ObstacleFlowState::read_checkpoint(
            &g,
            &mut BufReader::with_capacity(8192, File::open(&a[2])?),
            MAX_OBSTACLE_STATE_BYTES,
        )?;
        println!(
            "{{\"structurally_valid\":true,\"qualification\":\"Unqualified\",\"pressure_available\":false,\"combined_payload_bytes\":{}}}",
            s.combined_payload_bytes()
        );
        return Ok(());
    }
    if a.len() != 6 {
        return Err("usage: owned_obstacle_state_checkpoint NEW_EXTERNAL_DIR N PROBLEM_SHA BOUNDARY_SHA INITIAL_SHA FORCING_SHA; or check N FILE".into());
    }
    let path = Path::new(&a[0]);
    let parent = path
        .parent()
        .ok_or("output needs a parent directory")?
        .canonicalize()?;
    if parent.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize()?) {
        return Err("generated evidence must stay outside the Git worktree".into());
    }
    let n: usize = a[1].parse()?;
    let g = geometry(n)?;
    let inputs = ObstaclePhysicalInputs {
        units: ObstacleStateUnits::Si,
        model: ObstacleStateModel::TransientStokes,
        material: ObstacleStateMaterial {
            density: 1000.,
            dynamic_viscosity: 0.001,
        },
        problem: evidence(1, &a[2])?,
        boundary_evidence: evidence(2, &a[3])?,
        boundary: ObstacleStateBoundary::StationaryNoSlipSolidSealedFreeSlipOuter,
        initial: ObstacleInitialData::KnownRest(evidence(3, &a[4])?),
        initial_time: 0.,
        forcing: ObstacleForcingHistory {
            kind: ObstacleForcingKind::ExplicitNoForcing,
            evidence: evidence(4, &a[5])?,
            start_time: 0.,
            end_time: 0.,
        },
    };
    let frame = ObstacleStateFrame {
        time: 0.,
        generation: 0,
        origin: ObstacleStateOrigin::InitialData,
        errors: ObstacleVelocityErrors::unknown(),
    };
    // The supplied initial declaration explicitly says rest; these are caller
    // buffers. The owner itself has no generator or missing-input fallback.
    let axes = [Axis::X, Axis::Y, Axis::Z];
    let velocities: [Vec<f64>; 3] = std::array::from_fn(|d| vec![0.; g.grid().face_len(axes[d])]);
    let caller_bytes = velocities.iter().map(|v| v.capacity() * 8).sum::<usize>();
    let s = ObstacleFlowState::new(
        &g,
        inputs,
        frame,
        [&velocities[0], &velocities[1], &velocities[2]],
        MAX_OBSTACLE_STATE_BYTES,
    )?;
    let example_live_bound =
        g.allocation().retained_bytes + 2 * s.owned_payload_bytes() + caller_bytes + 8192;
    if example_live_bound > MAX_OBSTACLE_STATE_BYTES {
        return Err("coexisting example payload exceeds unchanged cap".into());
    }
    std::fs::create_dir(path)?;
    {
        let mut w = output(&path.join("state.rheon-os1"))?;
        s.write_checkpoint(&mut w)?;
        w.flush()?;
    }
    let t = ObstacleFlowState::read_checkpoint(
        &g,
        &mut BufReader::with_capacity(8192, File::open(path.join("state.rheon-os1"))?),
        MAX_OBSTACLE_STATE_BYTES,
    )?;
    {
        let mut w = output(&path.join("roundtrip.rheon-os1"))?;
        t.write_checkpoint(&mut w)?;
        w.flush()?;
    }
    println!(
        "{{\"n\":{n},\"qualification\":\"Unqualified\",\"pressure_available\":false,\"errors\":\"all_unknown\",\"geometry_retained_bytes\":{},\"geometry_constructor_peak_bytes\":{},\"owner_payload_bytes\":{},\"combined_payload_bytes\":{},\"caller_velocity_payload_bytes\":{caller_bytes},\"coexisting_example_payload_bound_bytes\":{example_live_bound},\"cap_bytes\":{MAX_OBSTACLE_STATE_BYTES}}}",
        g.allocation().retained_bytes,
        g.allocation().constructor_peak_bytes,
        s.owned_payload_bytes(),
        s.combined_payload_bytes()
    );
    Ok(())
}
