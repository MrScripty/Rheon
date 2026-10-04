//! Native comparison shell. The solver remains independent of the UI runtime.
use eframe::egui;
use rheon::{
    GridGeometry, PressureImplementation, PressureSettings, Simulation, SimulationConfig,
    SmokeSource, guidance_pixels,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Request {
    size: u64,
    steps: usize,
    tight: bool,
    methods: Vec<PressureImplementation>,
}
struct ResultRow {
    method: PressureImplementation,
    seconds: f64,
    iterations: usize,
    divergence: f64,
    bytes: usize,
    time: f64,
    pixels: Vec<u8>,
}
struct Worker {
    cancel: Arc<AtomicBool>,
    progress: Arc<AtomicUsize>,
    total: usize,
    handle: JoinHandle<Result<Vec<ResultRow>, String>>,
}
impl Worker {
    fn start(request: Request) -> std::io::Result<Self> {
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&cancel);
        let p = Arc::clone(&progress);
        let total = request.methods.len() * request.steps;
        let handle = std::thread::Builder::new()
            .name("rheon-comparison".into())
            .spawn(move || run(request, &c, &p))?;
        Ok(Self {
            cancel,
            progress,
            total,
            handle,
        })
    }
}
fn run(
    request: Request,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
) -> Result<Vec<ResultRow>, String> {
    if !(1..=128).contains(&request.size)
        || !(1..=10000).contains(&request.steps)
        || request.methods.is_empty()
    {
        return Err("Invalid comparison settings".into());
    }
    let mut rows = Vec::new();
    for method in request.methods {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled; no completed comparison published".into());
        }
        let h = 1.0 / request.size as f64;
        let grid =
            GridGeometry::new([request.size; 3], [h; 3], [0.0; 3]).map_err(|e| e.to_string())?;
        let settings = SimulationConfig {
            density: 1.0,
            memory_limit: 192 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: if request.tight { 1e-11 } else { 1e-9 },
                absolute_residual: 1e-12,
                divergence_limit: if request.tight { 1e-9 } else { 1e-7 },
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        };
        let mut sim =
            Simulation::with_implementation(grid, settings, method).map_err(|e| e.to_string())?;
        let source = SmokeSource {
            lower: [0.35, 0.08, 0.35],
            upper: [0.65, 0.30, 0.65],
            tracer_rate: 2.0,
            vertical_acceleration: 1.0,
        };
        let mut iterations = 0;
        let mut divergence = 0.0_f64;
        let start = Instant::now();
        for step in 0..request.steps {
            let r = sim
                .step(0.02, (step < request.steps / 2).then_some(source), |_| {
                    cancel.load(Ordering::Relaxed)
                })
                .map_err(|e| e.to_string())?;
            iterations += r.pressure.iterations;
            divergence = divergence.max(r.actual_divergence_max);
            progress.fetch_add(1, Ordering::Relaxed);
        }
        let seconds = start.elapsed().as_secs_f64();
        let pixels = guidance_pixels(&sim, 128 * 128).map_err(|e| e.to_string())?;
        rows.push(ResultRow {
            method,
            seconds,
            iterations,
            divergence,
            bytes: sim.allocated_bytes(),
            time: sim.state().time,
            pixels,
        });
    }
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled; no completed comparison published".into());
    }
    Ok(rows)
}
struct App {
    selected: PressureImplementation,
    size: u64,
    steps: usize,
    tight: bool,
    worker: Option<Worker>,
    rows: Vec<ResultRow>,
    textures: Vec<egui::TextureHandle>,
    status: String,
    running_request: Option<Request>,
    closing: bool,
}
impl Default for App {
    fn default() -> Self {
        Self {
            selected: PressureImplementation::default(),
            size: 16,
            steps: 30,
            tight: false,
            worker: None,
            rows: Vec::new(),
            textures: Vec::new(),
            status: "Ready. Each run starts from rest; source turns off halfway.".into(),
            running_request: None,
            closing: false,
        }
    }
}
impl App {
    fn start(&mut self, all: bool) {
        if self.worker.is_some() {
            return;
        }
        let request = Request {
            size: self.size,
            steps: self.steps,
            tight: self.tight,
            methods: if all {
                PressureImplementation::ALL.to_vec()
            } else {
                vec![self.selected]
            },
        };
        match Worker::start(request.clone()) {
            Ok(worker) => {
                self.worker = Some(worker);
                self.running_request = Some(request);
                self.rows.clear();
                self.textures.clear();
                self.status = "Running isolated simulations…".into();
            }
            Err(e) => self.status = format!("Cannot start worker: {e}"),
        }
    }
    fn cancel(&self) {
        if let Some(w) = &self.worker {
            w.cancel.store(true, Ordering::Relaxed);
        }
    }
    fn poll(&mut self, ctx: &egui::Context) {
        if self.worker.as_ref().is_some_and(|w| w.handle.is_finished()) {
            let worker = self.worker.take().expect("finished worker");
            let cancelled = worker.cancel.load(Ordering::Relaxed);
            match worker.handle.join() {
                Ok(Ok(_)) if cancelled => {
                    self.status = "Cancelled; no completed comparison published".into();
                }
                Ok(Ok(rows)) => {
                    let size = self.running_request.as_ref().expect("worker request").size as usize;
                    self.textures = rows
                        .iter()
                        .map(|r| {
                            ctx.load_texture(
                                r.method.id(),
                                egui::ColorImage::from_gray([size, size], &r.pixels),
                                egui::TextureOptions::NEAREST,
                            )
                        })
                        .collect();
                    self.rows = rows;
                    self.status="Complete. Single-run timing is exploratory; use the benchmark harness for repeated measurements.".into();
                    if self.rows.windows(2).any(|w| w[0].time != w[1].time) {
                        self.status.push_str(
                            " Accepted times differ; these are not equal physical horizons.",
                        );
                    }
                }
                Ok(Err(e)) => self.status = format!("No completed comparison: {e}"),
                Err(_) => {
                    self.status =
                        "Worker failed unexpectedly; no completed comparison published".into()
                }
            }
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll(ctx);
        if ctx.input(|i| i.viewport().close_requested()) && self.worker.is_some() {
            self.closing = true;
            self.cancel();
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if self.closing && self.worker.is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        egui::CentralPanel::default().show(ctx,|ui|{
            ui.heading("Rheon / Fluid solver laboratory");
            ui.label("Preserved implementations · shared physical contracts · Rust CPU simulations");ui.add_space(16.0);
            ui.add_enabled_ui(self.worker.is_none()&&!self.closing,|ui|{
                ui.horizontal_wrapped(|ui|{
                    ui.label("Implementation");
                    egui::ComboBox::from_id_salt("implementation").selected_text(self.selected.label()).width(280.0).show_ui(ui,|ui|{
                        for method in PressureImplementation::ALL {ui.selectable_value(&mut self.selected,method,method.label());}
                    });
                    ui.label("Grid edge");ui.add(egui::DragValue::new(&mut self.size).range(1..=128));
                    ui.label("Steps");ui.add(egui::DragValue::new(&mut self.steps).range(1..=10000));
                    ui.checkbox(&mut self.tight,"Tight pressure tolerance");
                });
                ui.add_space(8.0);
                ui.horizontal(|ui|{
                    if ui.button("Run selected").clicked(){self.start(false);}
                    if ui.button("Compare all approaches").clicked(){self.start(true);}
                });
            });
            if let Some(w)=&self.worker {
                let n=w.progress.load(Ordering::Relaxed);
                ui.add(egui::ProgressBar::new(n as f32/w.total as f32).text(format!("{n}/{} accepted steps",w.total)));
                if ui.button("Cancel").clicked(){self.cancel();}
            }
            ui.add_space(8.0);ui.label(&self.status);ui.separator();
            if let Some(r)=&self.running_request {
                ui.label(format!("Results: {}³ cells, {} steps, dt ≤ 0.02 s, {} pressure accuracy",r.size,r.steps,if r.tight {"tight"}else{"standard"}));
            }
            egui::ScrollArea::vertical().show(ui,|ui|{
                egui::Grid::new("results").striped(true).show(ui,|ui|{
                    for h in ["Implementation","Step seconds","Iterations","Arrays MiB","Max divergence","Accepted time"]{ui.strong(h);}ui.end_row();
                    for r in &self.rows {
                        ui.label(r.method.id());ui.label(format!("{:.4}",r.seconds));ui.label(r.iterations.to_string());ui.label(format!("{:.3}",r.bytes as f64/1048576.0));ui.label(format!("{:.2e}",r.divergence));ui.label(format!("{:.6}",r.time));ui.end_row();
                    }
                });
                ui.add_space(16.0);
                ui.horizontal_wrapped(|ui|{
                    for (r,texture) in self.rows.iter().zip(&self.textures){ui.vertical(|ui|{ui.label(r.method.label());ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(240.0,240.0)));});}
                });
            });
            ui.separator();ui.label("+Z opacity guidance, +Y upward. Pressure accuracy is not physical ground truth. Memory excludes process/renderer overhead. No real-time or full 3D-viewer claim.");
        });
        if self.worker.is_some() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}
impl Drop for App {
    fn drop(&mut self) {
        // Normal window close waits asynchronously above; this is the final
        // ownership backstop if the framework shuts down without another frame.
        if let Some(worker) = self.worker.take() {
            worker.cancel.store(true, Ordering::Relaxed);
            let _ = worker.handle.join();
        }
    }
}
fn main() -> eframe::Result {
    eframe::run_native(
        "Rheon comparison",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1100.0, 700.0])
                .with_min_inner_size([800.0, 550.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(App::default()))),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn worker_runs_selected_and_equal_input_comparison() {
        let request = Request {
            size: 8,
            steps: 4,
            tight: false,
            methods: PressureImplementation::ALL.to_vec(),
        };
        let progress = AtomicUsize::new(0);
        let rows = run(request, &AtomicBool::new(false), &progress).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(progress.load(Ordering::Relaxed), 8);
        assert_eq!(rows[0].time, rows[1].time);
        assert_eq!(rows[0].bytes, rows[1].bytes);
        assert_eq!(rows[0].pixels.len(), 64);
        assert_eq!(rows[0].pixels, rows[1].pixels);
    }
    #[test]
    fn cancelled_worker_never_publishes_rows() {
        assert!(
            run(
                Request {
                    size: 8,
                    steps: 4,
                    tight: false,
                    methods: PressureImplementation::ALL.to_vec()
                },
                &AtomicBool::new(true),
                &AtomicUsize::new(0)
            )
            .is_err()
        );
    }
    #[test]
    fn repeated_start_cannot_replace_active_work() {
        let mut app = App::default();
        app.size = 8;
        app.steps = 4;
        app.start(false);
        let token = Arc::clone(&app.worker.as_ref().unwrap().cancel);
        app.start(true);
        assert!(Arc::ptr_eq(&token, &app.worker.as_ref().unwrap().cancel));
        assert_eq!(app.running_request.as_ref().unwrap().methods.len(), 1);
    }
    #[test]
    fn cancellation_before_publication_suppresses_already_finished_result() {
        let mut app = App::default();
        app.size = 4;
        app.steps = 2;
        app.start(false);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !app.worker.as_ref().unwrap().handle.is_finished() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        app.cancel();
        app.poll(&egui::Context::default());
        assert!(app.worker.is_none());
        assert!(app.rows.is_empty());
        assert!(app.status.starts_with("Cancelled"));
    }
}
