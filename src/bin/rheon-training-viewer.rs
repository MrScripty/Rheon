//! Inspect one complete native training simulation, translucent voxels and force.
#[path = "desktop/camera.rs"]
mod camera;
#[path = "desktop/training_viewer.rs"]
mod training_viewer;
#[path = "desktop/voxel_view.rs"]
mod voxel_view;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut paths = Vec::new();
    while let Some(flag) = args.next() {
        if flag != "--session" || paths.len() == 16 {
            return Err("Use one to sixteen --session /path/to/session.json arguments".into());
        }
        paths.push(
            args.next()
                .ok_or("--session requires a timeline JSON path")?,
        );
    }
    if paths.is_empty() {
        return Err("Use --session /path/to/session.json".into());
    }
    eframe::run_native(
        "Rheon · Training simulation and force",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1400.0, 940.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());
            Ok(Box::new(training_viewer::TrainingViewer::with_presets(
                paths,
            )))
        }),
    )?;
    Ok(())
}
