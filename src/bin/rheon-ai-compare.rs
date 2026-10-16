//! Native Rheon water comparison: qualified reference and raw CPU AI diagnostic.
#[path = "desktop/ai_comparison/mod.rs"]
mod ai_comparison;
#[path = "desktop/build_identity.rs"]
mod build_identity;
#[path = "desktop/camera.rs"]
mod camera;
#[path = "desktop/run_sets.rs"]
pub mod run_sets;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut deployment = String::new();
    let mut directory = std::env::current_dir()?.join(".generated/ai-comparison");
    let mut scene = ai_comparison::scene::Scene::default();
    let mut preflight = false;
    let mut auto = false;
    let mut headless = false;
    let mut policy = ai_comparison::ExecutionPolicy::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--deployment" => deployment = args.next().ok_or("Deployment path required")?,
            "--project-dir" => directory = args.next().ok_or("Project directory required")?.into(),
            "--cells-per-metre" => {
                scene.cells_per_metre = args.next().ok_or("Density required")?.parse()?
            }
            "--counts" => {
                let counts = args.next().ok_or("Counts X,Y,Z required")?;
                let values = counts
                    .split(',')
                    .map(str::parse)
                    .collect::<Result<Vec<usize>, _>>()?;
                scene.counts = values
                    .try_into()
                    .map_err(|_| "Exactly three counts X,Y,Z required")?;
            }
            "--warmup-steps" => {
                scene.warmup_steps = args.next().ok_or("Warm-up steps required")?.parse()?
            }
            "--preflight" => preflight = true,
            "--help" | "-h" => {
                println!(
                    "Rheon paired comparison: --cells-per-metre N (1–1000), --counts X,Y,Z (5–128/axis, ≤65536 cells), --steps N (1–512), --warmup-steps N, --preflight (no model execution/output), --deployment FILE, --project-dir DIR, --verify-run, --auto-run, --mode legacy|teacher|stop-on-reject. Model support, fixed 1/60 s step, history and resource limits are independent."
                );
                return Ok(());
            }
            "--steps" => {
                scene.steps = args
                    .next()
                    .ok_or("Steps required")?
                    .parse()
                    .map_err(|_| "Invalid steps")?
            }
            "--mode" => {
                policy = match args.next().ok_or("Mode required")?.as_str() {
                    "legacy" => ai_comparison::ExecutionPolicy::LegacyRawContinuation,
                    "teacher" => ai_comparison::ExecutionPolicy::TeacherStage,
                    "stop-on-reject" => ai_comparison::ExecutionPolicy::IndependentStopOnReject,
                    _ => return Err("Unknown execution mode".into()),
                }
            }
            "--auto-run" => auto = true,
            "--verify-run" => headless = true,
            _ => return Err(format!("Unknown argument {arg}").into()),
        }
    }
    scene.validate().map_err(std::io::Error::other)?;
    if preflight {
        let grid = rheon::GridGeometry::at_cell_density(
            scene.counts.map(|n| n as u64),
            scene.cells_per_metre,
            [0.0; 3],
        )?;
        let plan = rheon::BufferPlan::for_grid(&grid, usize::MAX)?;
        let support = if deployment.is_empty() {
            None
        } else {
            Some(
                ai_comparison::model_process::ModelProcess::preflight(
                    std::path::Path::new(&deployment),
                    &scene,
                )
                .map_err(std::io::Error::other)?,
            )
        };
        println!(
            "{}",
            serde_json::json!({"counts_xyz":scene.counts,"cells_per_metre":scene.cells_per_metre,"cell_size_m":scene.h(),"extents_m":scene.extents(),"cells":grid.cell_len(),"native_array_payload_bytes_per_track":plan.total_bytes,"paired_history_payload_bytes":scene.history_bytes()?,"pressure_cell_iteration_budget_per_track":grid.cell_len() as u128*scene.steps as u128*2000,"model_support_checked":support.is_some(),"model_executed":false,"whole_process_memory_cap_claimed":false})
        );
        return Ok(());
    }
    if headless {
        return ai_comparison::headless_scene(deployment.into(), directory, scene, policy)
            .map_err(|e| std::io::Error::other(e).into());
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1480.0, 940.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rheon · Solver / AI pressure comparison",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            let mut app = ai_comparison::ComparisonApp::new(deployment, directory);
            app.set_scene(scene);
            app.set_policy(policy);
            if auto {
                app.start();
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(Into::into)
}
