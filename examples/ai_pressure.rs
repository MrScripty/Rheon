//! CPU qualification of actual frozen model weights against both PCG references.
//! Usage: cargo run --release --no-default-features --example ai_pressure -- MODEL_EXPORT OUTPUT_DIR
use rheon::solver_provider::*;
use rheon::*;
use std::{error::Error, fs, io::Write, time::Instant};
fn main() -> Result<(), Box<dyn Error>> {
    let process = Instant::now();
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("expected MODEL_EXPORT OUTPUT_DIR".into());
    }
    let output = std::path::Path::new(&args[2]);
    fs::create_dir(output)?;
    let bytes = fs::read(&args[1])?;
    let mut model = LabModalPressure::from_export(&bytes).map_err(|e| format!("{e:?}"))?;
    let g = GridGeometry::new([32; 3], [1.0 / 32.0; 3], [0.0; 3])?;
    let op = PressureOperator::new(&g, 1.25)?;
    let settings = PressureSettings {
        relative_residual: 1e-8,
        absolute_residual: 1e-12,
        divergence_limit: 1e-7,
        max_iterations: 3000,
    };
    let mut rhs = vec![0.0; g.cell_len()];
    let mut div = rhs.clone();
    let mut candidate: [Vec<f32>; 3] =
        std::array::from_fn(|d| vec![0.0; g.face_len([Axis::X, Axis::Y, Axis::Z][d])]);
    let mut corrected = candidate.clone();
    let mut workspace =
        ProviderWorkspace::new(&g, 64 * 1024 * 1024, PressureImplementation::JacobiPcgV1)?;
    let setup = process.elapsed();
    let mut csv = fs::File::create(output.join("comparison.csv"))?;
    writeln!(
        csv,
        "seed,repetition,method,selected,total_ms,inference_ms,qualification_ms,fallback_ms,corrected_call_ms,residual_l2,relative_pressure_l2,actual_divergence_max"
    )?;
    for seed in [900_u64, 901, 902, 903] {
        // Independent deterministic broadband MAC field, not training labels.
        for (d, field) in candidate.iter_mut().enumerate() {
            let shape = g.face_counts([Axis::X, Axis::Y, Axis::Z][d]);
            for (i, v) in field.iter_mut().enumerate() {
                let p = [
                    i % shape[0],
                    i / shape[0] % shape[1],
                    i / (shape[0] * shape[1]),
                ];
                let mut bits = (i as u64)
                    .wrapping_add(seed * 7919)
                    .wrapping_add(d as u64 * 997);
                bits = (bits ^ (bits >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                bits = (bits ^ (bits >> 27)).wrapping_mul(0x94d049bb133111eb);
                bits ^= bits >> 31;
                *v = if p[d] == 0 || p[d] + 1 == shape[d] {
                    0.0
                } else {
                    ((2.0 * ((bits >> 40) as f64 / 16_777_216.0) - 1.0) * 0.001) as f32
                };
            }
        }
        op.build_rhs(
            [&candidate[0], &candidate[1], &candidate[2]],
            1.0 / 60.0,
            &mut rhs,
        )?;
        let request = PressureRequest {
            grid: &g,
            density: 1.25,
            dt: 1.0 / 60.0,
            rhs: &rhs,
        };
        let reference = workspace.solve(request, settings, None, || false)?;
        let reference_pressure = workspace.pressure().to_vec();
        assert!(reference.accepted.true_residual_l2.is_finite());
        for repetition in 0..3 {
            // Rotate order to avoid always granting the learned method warm caches.
            for offset in 0..3 {
                let method = (offset + repetition) % 3;
                let method_id = ["jacobi", "sgs", "learned"][method];
                let fallback = if method == 1 {
                    PressureImplementation::SymmetricGaussSeidelPcgV1
                } else {
                    PressureImplementation::JacobiPcgV1
                };
                let mut work = ProviderWorkspace::new(&g, 64 * 1024 * 1024, fallback)?;
                let begin = Instant::now();
                let report = work.solve(
                    request,
                    settings,
                    if method == 2 { Some(&mut model) } else { None },
                    || false,
                )?;
                let [x, y, z] = &mut corrected;
                op.correct_velocity(
                    [&candidate[0], &candidate[1], &candidate[2]],
                    work.pressure(),
                    request.dt,
                    [x, y, z],
                )?;
                op.divergence([&corrected[0], &corrected[1], &corrected[2]], &mut div)?;
                let actual = div.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
                let complete = begin.elapsed();
                if actual > 1e-5 {
                    return Err("actual divergence gate".into());
                }
                // A fallback is a valid production outcome, but fails model qualification.
                if method == 2 && report.selected != "lab-modal-pressure-v1" {
                    return Err(format!("model rejected: {:?}", report.rejection).into());
                }
                let error = (work
                    .pressure()
                    .iter()
                    .zip(&reference_pressure)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    / reference_pressure.iter().map(|v| v * v).sum::<f64>())
                .sqrt();
                if !error.is_finite() || error > 1e-5 {
                    return Err("pressure error gate".into());
                }
                writeln!(
                    csv,
                    "{seed},{repetition},{method_id},{},{},{},{},{},{},{},{},{actual}",
                    report.selected,
                    report.total.as_secs_f64() * 1000.0,
                    report.inference.as_secs_f64() * 1000.0,
                    report.qualification.as_secs_f64() * 1000.0,
                    report.fallback.as_secs_f64() * 1000.0,
                    complete.as_secs_f64() * 1000.0,
                    report.accepted.true_residual_l2,
                    error
                )?;
                if seed == 900 && repetition == 0 && method == 2 {
                    for (name, values) in [
                        ("rhs.f64", rhs.as_slice()),
                        ("native-pressure.f64", work.pressure()),
                    ] {
                        let mut file = fs::File::create(output.join(name))?;
                        for v in values {
                            file.write_all(&v.to_le_bytes())?;
                        }
                    }
                }
            }
        }
    }
    println!(
        "setup_including_file_read_model_and_workspace_ms={}",
        setup.as_secs_f64() * 1000.0
    );
    println!(
        "complete_example_ms={}",
        process.elapsed().as_secs_f64() * 1000.0
    );
    println!("model={:?}", model.metadata());
    Ok(())
}
