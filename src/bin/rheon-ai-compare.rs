//! Native Rheon water comparison: qualified reference and raw CPU AI diagnostic.
#[path = "desktop/ai_comparison/mod.rs"]
mod ai_comparison;
#[path = "desktop/camera.rs"]
mod camera;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut deployment = String::new();
    let mut directory = std::env::current_dir()?.join(".generated/ai-comparison");
    let mut steps = 64;
    let mut auto = false;
    let mut headless = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--deployment" => deployment = args.next().ok_or("Deployment path required")?,
            "--project-dir" => directory = args.next().ok_or("Project directory required")?.into(),
            "--steps" => {
                steps = args
                    .next()
                    .ok_or("Steps required")?
                    .parse()
                    .map_err(|_| "Invalid steps")?
            }
            "--auto-run" => auto = true,
            "--verify-run" => headless = true,
            _ => return Err(format!("Unknown argument {arg}").into()),
        }
    }
    if headless {
        return ai_comparison::headless(deployment.into(), directory, steps)
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
            app.set_steps(steps);
            if auto {
                app.start();
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(Into::into)
}
