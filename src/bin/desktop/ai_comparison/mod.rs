//! Fresh paired native / CPU-checkpoint water laboratory with explicit AI verdicts.
mod accepted_provider;
mod adapter;
#[cfg(test)]
mod cpu_contract_tests;
mod diagnostic_runs;
mod diagnostic_view;
mod diagnostic_worker;
mod engine;
pub use engine::ExecutionPolicy;
mod model_process;
mod record;
mod scene;
mod view;
mod worker;
use crate::camera::OrbitCamera;
use eframe::egui::{self, Color32};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::Duration,
};
use worker::Worker;
pub struct ComparisonApp {
    pub deployment: String,
    pub project_directory: PathBuf,
    scene: scene::Scene,
    active_scene: scene::Scene,
    worker: Option<Worker>,
    camera: OrbitCamera,
    selected: usize,
    cut: usize,
    probe: Option<usize>,
    follow: bool,
    playing: bool,
    play_time: f64,
    speed: f64,
    difference: bool,
    fixed_range: bool,
    whole_run_range: bool,
    manual_range: f64,
    status: String,
    preflight_status: Option<String>,
    closing: bool,
    policy: engine::ExecutionPolicy,
    active_policy: engine::ExecutionPolicy,
    inspector: diagnostic_view::Inspector,
    show_attempt: bool,
    live_attempt: diagnostic_view::AttemptCache,
    attempt_cut: usize,
    attempt_probe: Option<usize>,
    attempt_difference: bool,
}
impl ComparisonApp {
    pub fn new(deployment: String, project_directory: PathBuf) -> Self {
        Self {
            deployment,
            project_directory,
            scene: Default::default(),
            active_scene: Default::default(),
            worker: None,
            camera: Default::default(),
            selected: 0,
            cut: 8,
            probe: None,
            follow: true,
            playing: false,
            play_time: 0.0,
            speed: 0.25,
            difference: true,
            fixed_range: false,
            whole_run_range: false,
            manual_range: 100.0,
            preflight_status: None,
            status: "Choose a CPU deployment and start a fresh paired simulation".into(),
            closing: false,
            policy: Default::default(),
            active_policy: Default::default(),
            inspector: Default::default(),
            show_attempt: false,
            live_attempt: Default::default(),
            attempt_cut: 0,
            attempt_probe: None,
            attempt_difference: true,
        }
    }
    fn check_package(&mut self) {
        self.preflight_status = Some(
            match model_process::ModelProcess::preflight(
                std::path::Path::new(&self.deployment),
                &self.scene,
            ) {
                Ok(p) => format!(
                    "Supported CPU package {} {} (no code executed)",
                    p.id, p.version
                ),
                Err(e) => e,
            },
        );
    }
    pub fn set_steps(&mut self, steps: usize) {
        self.scene.steps = steps;
    }
    #[allow(
        dead_code,
        reason = "Standalone comparison CLI entry, shared with desktop UI"
    )]
    pub fn set_policy(&mut self, policy: engine::ExecutionPolicy) {
        self.policy = policy;
    }
    pub fn start(&mut self) {
        if self.running() {
            return;
        }
        let worker = if self.policy == ExecutionPolicy::LegacyRawContinuation {
            Worker::start(
                self.scene.clone(),
                PathBuf::from(&self.deployment),
                self.project_directory.clone(),
            )
        } else {
            Worker::start_with_policy(
                self.scene.clone(),
                PathBuf::from(&self.deployment),
                self.project_directory.clone(),
                self.policy,
            )
        };
        match worker {
            Ok(worker) => {
                self.worker = Some(worker);
                self.active_scene = self.scene.clone();
                self.active_policy = self.policy;
                self.selected = 0;
                self.cut = self.cut.min(self.scene.counts[2] - 1);
                self.probe = None;
                self.live_attempt
                    .update(None, &mut self.attempt_cut, &mut self.attempt_probe);
                self.follow = true;
                self.playing = false;
                self.status = "Starting native warm-up, then independent tracks".into();
            }
            Err(error) => self.status = error,
        }
    }
    fn running(&self) -> bool {
        self.worker.as_ref().is_some_and(|w| !w.finished())
    }
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        if let Some(worker) = self.worker.as_mut() {
            worker.join_finished();
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.running() {
            self.closing = true;
            if let Some(worker) = &self.worker {
                worker.cancel();
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if self.closing && !self.running() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        let running = self.running();
        let mut stay = true;
        egui::SidePanel::left("ai-comparison-controls").resizable(true).default_width(300.0).show(ctx,|ui|{
            ui.heading("Water pressure comparison");if ui.add_enabled(!running,egui::Button::new("Back to simulation")).clicked(){stay=false;}
            ui.label("Filled sealed water · 1000 kg/m³ · 0.001 Pa·s · free-slip walls");
            ui.checkbox(&mut self.inspector.visible,"Inspect saved diagnostic attempts");
            ui.checkbox(&mut self.show_attempt,"Inspect latest live attempt");
            ui.add_enabled_ui(!running,|ui|{
                egui::ComboBox::from_label("Execution policy").selected_text(self.policy.label()).show_ui(ui,|ui|{ for policy in [engine::ExecutionPolicy::LegacyRawContinuation,engine::ExecutionPolicy::TeacherStage,engine::ExecutionPolicy::IndependentStopOnReject]{ui.selectable_value(&mut self.policy,policy,policy.label());}});
                ui.collapsing("CPU adapter package", |ui| {
                    ui.text_edit_singleline(&mut self.deployment);
                    if ui.button("Check package support").clicked() { self.check_package(); }
                    if let Some(result) = &self.preflight_status { ui.label(format!("Last inert support check: {result}")); }
                    ui.small("Select a measured v2 deployment or configurable CPU adapter manifest.");
                });
                ui.horizontal(|ui|{ui.label("Seed");ui.add(egui::DragValue::new(&mut self.scene.seed).range(0..=((1_u64<<53)-1)));if ui.button("New seed").clicked(){self.scene.seed=self.scene.seed.wrapping_add(0x9e3779b97f4a7c15)&((1_u64<<53)-1);}});
                ui.label("Cell counts X / Y / Z");ui.horizontal(|ui|{for count in &mut self.scene.counts{ui.add(egui::DragValue::new(count).range(5..=128));}});
                ui.horizontal(|ui|{ui.label("Cells per metre");ui.add(egui::DragValue::new(&mut self.scene.cells_per_metre).range(1..=128));});
                ui.small(format!("Domain {:?} m · cubic cell {:.5} m",self.scene.extents(),self.scene.h()));ui.small("Training: 5–16 cells/axis; 8 or 16 cells/m. Larger grids are extrapolation.");
                ui.horizontal(|ui|{ui.label("Timesteps");ui.add(egui::DragValue::new(&mut self.scene.steps).range(1..=512));});ui.horizontal(|ui|{ui.label("Native warm-up steps");ui.add(egui::DragValue::new(&mut self.scene.warmup_steps).range(0..=511));});ui.small("Fixed Δt = 1/60 s. Native prefix and model support are checked before execution.");
                ui.add(egui::Slider::new(&mut self.scene.acceleration,0.0..=4.0).text("Acceleration (m/s²)"));
                ui.collapsing("Force location and fluctuations",|ui|{for d in 0..3{ui.add(egui::Slider::new(&mut self.scene.center[d],0.0..=1.0).text(format!("{} center fraction",["X","Y","Z"][d])));}ui.add(egui::Slider::new(&mut self.scene.radius,0.1..=0.5).text("Radius fraction"));ui.add(egui::Slider::new(&mut self.scene.swirl,-1.0..=1.0).text("Swirl"));ui.add(egui::Slider::new(&mut self.scene.fluctuation,0.0..=0.8).text("Fluctuation"));ui.add(egui::Slider::new(&mut self.scene.frequency,0.0..=4.0).text("Frequency (Hz)"));});
                ui.checkbox(&mut self.scene.sgs,"SGS PCG reference (otherwise Jacobi)");
                if ui.button("Start fresh comparison").clicked(){self.start();}
            });
            if running && ui.button("Cancel and retain frames").clicked()&& let Some(worker)=&self.worker{worker.cancel();}
            ui.separator();ui.selectable_value(&mut self.difference,false,"AI pane: pressure");ui.selectable_value(&mut self.difference,true,"AI pane: AI − Rheon pressure");
            let depth=self.active_scene.counts[2];ui.add(egui::Slider::new(&mut self.cut,0..=depth-1).text("Z cut layer"));
            ui.checkbox(&mut self.fixed_range,"Fixed color scale");if self.fixed_range{ui.add(egui::DragValue::new(&mut self.manual_range).range(1e-9..=1e12).speed(1.0).prefix("± ").suffix(" Pa"));ui.small("Colors clamp at endpoints; voxel probes keep exact values.");}else{ui.checkbox(&mut self.whole_run_range,"Whole-run color range");ui.small(if self.whole_run_range{"Range over all retained frames, shared across slices."}else{"Current frame full-volume range, shared across slices."});}
            ui.horizontal(|ui|{if ui.button("Frame view").clicked(){self.camera=Default::default();}ui.checkbox(&mut self.camera.orthographic,"Orthographic");});
            ui.small("Drag: orbit · Shift-drag: pan · wheel: zoom · click voxel: probe");
            ui.separator();ui.label(self.active_policy.label()); ui.small(match self.active_policy { engine::ExecutionPolicy::LegacyRawContinuation=>"Raw continuation retains failed states until a hard stop.", engine::ExecutionPolicy::TeacherStage=>"Candidate is measured on native requests; only native history advances.", engine::ExecutionPolicy::IndependentStopOnReject=>"First rejected candidate is recorded without advancing state or history."});
        });
        let mut selected_frame = None;
        let mut count = 0;
        let mut pressure_range = 0.0;
        let mut delta_range = 0.0;
        let mut provider = None;
        let mut latest_attempt = None;
        let mut diagnostic_manifest = None;
        let playback_delta = self.speed * f64::from(ctx.input(|i| i.stable_dt.min(0.1)));
        if let Some(worker) = &self.worker
            && let Ok(shared) = worker.shared.lock()
        {
            count = shared.frames.len();
            pressure_range = shared.pressure_range;
            delta_range = shared.difference_range;
            provider = shared.provider.clone();
            latest_attempt = shared.last_attempt.clone();
            diagnostic_manifest = shared.diagnostic_manifest.clone();
            self.status = format!("{} · {}", shared.state, shared.message);
            if count > 0 {
                if self.follow {
                    self.selected = count - 1;
                }
                if self.playing {
                    self.play_time += playback_delta;
                    self.selected = (self.play_time / scene::DT).floor() as usize;
                    if self.selected >= count - 1 {
                        self.selected = count - 1;
                        self.playing = false;
                    }
                }
                self.selected = self.selected.min(count - 1);
                selected_frame = shared.frames.get(self.selected).cloned();
            }
        }
        if let Some((spec, parent)) = self.inspector.show(ctx, running) {
            match Worker::start_saved(spec.clone(), parent, self.project_directory.clone()) {
                Ok(worker) => {
                    self.worker = Some(worker);
                    self.active_scene = spec.scene;
                    self.active_policy = spec.policy;
                    self.selected = 0;
                    self.probe = None;
                    self.live_attempt
                        .update(None, &mut self.attempt_cut, &mut self.attempt_probe);
                    self.follow = true;
                    self.playing = false;
                    self.cut = self.cut.min(self.active_scene.counts[2] - 1);
                    // All local snapshots above belong to the previous owner.
                    // Paint the new scene only after taking its own snapshots.
                    ctx.request_repaint();
                    return true;
                }
                Err(e) => {
                    self.inspector.report_execution_failure(&e);
                    self.status = e;
                }
            }
        }
        self.live_attempt.update(
            if self.show_attempt {
                latest_attempt
            } else {
                None
            },
            &mut self.attempt_cut,
            &mut self.attempt_probe,
        );
        if self.show_attempt {
            egui::Window::new("Latest live diagnostic request")
                .open(&mut self.show_attempt)
                .vscroll(true)
                .default_size([1050.0, 700.0])
                .show(ctx, |ui| {
                    if let Some(projection) = self.live_attempt.get() {
                        diagnostic_view::attempt_ui(
                            ui,
                            projection,
                            &mut self.attempt_cut,
                            &mut self.attempt_probe,
                            &mut self.attempt_difference,
                        );
                    } else {
                        ui.label("No typed request recorded yet");
                    }
                });
        }
        egui::TopBottomPanel::top("ai-comparison-status").show(ctx, |ui| {
            ui.heading("Rheon solver / AI solver · fresh water simulations");
            ui.label(&self.status);
            if let Some(path) = &diagnostic_manifest {
                ui.horizontal(|ui| {
                    ui.small(path.display().to_string());
                    if ui.button("Inspect saved run").clicked() {
                        self.inspector.set_path(path.to_string_lossy().into_owned());
                        self.inspector.visible = true;
                    }
                });
            }
            if let Some(provider) = &provider {
                ui.small(format!(
                    "CPU provider {} · optimizer updates during comparison: 0",
                    provider.id.chars().take(24).collect::<String>()
                ));
            }
        });
        egui::TopBottomPanel::bottom("ai-comparison-time").min_height(90.0).show(ctx,|ui|{
            ui.horizontal(|ui|{ui.checkbox(&mut self.follow,"Follow live");if ui.add_enabled(count>1,egui::Button::new(if self.playing{"Pause"}else{"Play"})).clicked(){self.playing = !self.playing;self.follow=false;if self.selected+1>=count{self.selected=0;}self.play_time=self.selected as f64*scene::DT;}ui.add(egui::Slider::new(&mut self.speed,0.1..=2.0).text("Playback speed"));if count>0&& ui.add(egui::Slider::new(&mut self.selected,0..=count-1).text("Paired step")).changed(){self.follow=false;self.playing=false;}});
            if let Some(frame)=&selected_frame{ui.label(format!("Same timestamp: {:.6} s · step {} · both grids {:?}",frame.time_s,frame.step,self.active_scene.counts));if let Some(index)=self.probe.filter(|i|*i<frame.ai_pressure.len()){let counts=self.active_scene.counts;let c=[index%counts[0],index/counts[0]%counts[1],index/(counts[0]*counts[1])];let location=c.map(|n|(n as f64+0.5)*self.active_scene.h());ui.label(format!("Voxel {c:?} at {location:?} m: Rheon {:.6} Pa · AI {:.6} Pa · AI − Rheon {:+.6} Pa",frame.reference_pressure[index],frame.ai_pressure[index],frame.ai_pressure[index]-frame.reference_pressure[index]));}else{ui.small("Click a visible voxel to inspect matched values.");}}
        });
        egui::CentralPanel::default().show(ctx,|ui|{let Some(frame)=selected_frame else{ui.centered_and_justified(|ui|ui.label("Start a fresh comparison; native physics and CPU inference run in the owned worker."));return;};
            if !self.whole_run_range {(pressure_range,delta_range)=view::ranges(&frame);}
            let scale=if self.fixed_range{self.manual_range}else{pressure_range};let error_scale=if self.fixed_range{self.manual_range}else{delta_range};
            let area=ui.available_rect_before_wrap();
            ctx.input(|input| {
                if input.pointer.hover_pos().is_some_and(|p|area.contains(p)) {
                    if input.pointer.primary_down() && input.pointer.is_decidedly_dragging() {
                        let delta=input.pointer.delta();
                        if input.modifiers.shift {self.camera.pan(delta,(area.height()-180.0).max(1.0));}
                        else if input.modifiers.ctrl {self.camera.zoom(-delta.y*0.01);}
                        else {self.camera.orbit(delta);}
                    }
                    self.camera.zoom(input.smooth_scroll_delta.y*0.003);
                }
            });
            ui.columns(2,|columns|{
                columns[0].allocate_ui_with_layout(egui::vec2(columns[0].available_width(),180.0),egui::Layout::top_down(egui::Align::Min),|ui|{
                    ui.heading("Rheon reference pressure");
                    if let Some(q)=&frame.reference_quality{ui.colored_label(Color32::LIGHT_GREEN,format!("Qualified · relative residual {:.2e}",q.relative_residual.unwrap_or(0.0)));}else{ui.label("Initial state");}
                    view::legend(ui,scale,"Reference pressure · Pa");
                });
                view::voxels(&mut columns[0],view::Display{scene:&self.active_scene,frame:&frame,camera:&self.camera,cut:self.cut,field:view::Field::Reference,scale},&mut self.probe);
                columns[1].allocate_ui_with_layout(egui::vec2(columns[1].available_width(),180.0),egui::Layout::top_down(egui::Align::Min),|ui|{
                    ui.heading(if self.difference{"AI − Rheon pressure"}else{"AI pressure"});
                    if let Some(q)=&frame.ai_quality{ui.colored_label(if q.qualified{Color32::LIGHT_GREEN}else{Color32::LIGHT_RED},format!("AI diagnostic · {} · residual {:.2e}",if q.qualified{"pressure passed"}else{"pressure FAILED"},q.relative_residual.unwrap_or(0.0)));}else{ui.label(if frame.step==0{"Initial state"}else{"Native warm-up · AI not evaluated"});}
                    view::legend(ui,if self.difference{error_scale}else{scale},if self.difference{"Signed error · blue: AI lower · white: zero · red: AI higher · Pa"}else{"AI pressure · same range as reference · Pa"});
                });
                view::voxels(&mut columns[1],view::Display{scene:&self.active_scene,frame:&frame,camera:&self.camera,cut:self.cut,field:if self.difference{view::Field::Difference}else{view::Field::Model},scale:if self.difference{error_scale}else{scale}},&mut self.probe);
            });
        });
        ctx.request_repaint_after(Duration::from_millis(33));
        stay
    }
}
impl eframe::App for ComparisonApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if !self.show(ctx) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
#[allow(
    dead_code,
    reason = "Companion rheon-ai-compare CLI entry; this shared UI module is also compiled into rheon-desktop"
)]
pub fn headless(deployment: PathBuf, directory: PathBuf, steps: usize) -> Result<(), String> {
    headless_with_policy(
        deployment,
        directory,
        steps,
        engine::ExecutionPolicy::LegacyRawContinuation,
    )
}
#[allow(dead_code, reason = "CLI entry also compiled in the desktop binary")]
pub fn headless_with_policy(
    deployment: PathBuf,
    directory: PathBuf,
    steps: usize,
    policy: engine::ExecutionPolicy,
) -> Result<(), String> {
    let scene = scene::Scene {
        steps,
        ..Default::default()
    };
    let shared = Mutex::new(worker::Shared::default());
    let cancel = AtomicBool::new(false);
    if policy == ExecutionPolicy::LegacyRawContinuation {
        worker::run(
            scene,
            deployment,
            directory,
            &shared,
            &cancel,
            Arc::new(Mutex::new(None)),
        )?;
    } else {
        worker::run_with_policy(
            scene,
            deployment,
            directory,
            &shared,
            &cancel,
            Arc::new(Mutex::new(None)),
            policy,
        )?;
    }
    let data = shared.lock().map_err(|_| "History lock")?;
    let last = data.frames.last();
    println!(
        "{}",
        serde_json::json!({"state":data.state,"message":data.message,"frames":data.frames.len(),"step":last.map(|f|f.step),"time_s":last.map(|f|f.time_s),"reference_qualified":last.and_then(|f|f.reference_quality.as_ref().map(|q|q.qualified)),"ai_qualified":last.and_then(|f|f.ai_quality.as_ref().map(|q|q.qualified)),"provider":data.provider,"checkpoint_sha256":data.provider.as_ref().and_then(|p|p.checkpoint_sha256.as_ref()),"directory":data.directory})
    );
    if data.state == "failed" {
        return Err(data.message.clone());
    }
    Ok(())
}

#[cfg(test)]
mod saved_rerun_tests {
    use super::*;
    use eframe::egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
    fn worker(package: &cpu_contract_tests::Fixture, counts: [usize; 3]) -> Worker {
        let mut worker = Worker::start_with_policy(
            scene::Scene {
                counts,
                steps: 3,
                warmup_steps: 1,
                ..Default::default()
            },
            package.manifest.clone(),
            package.root.join("runs"),
            ExecutionPolicy::TeacherStage,
        )
        .unwrap();
        let end = std::time::Instant::now() + Duration::from_secs(10);
        while !worker.finished() {
            assert!(std::time::Instant::now() < end);
            std::thread::sleep(Duration::from_millis(5));
        }
        worker.join_finished();
        worker
    }
    #[test]
    fn live_attempt_cache_reuses_published_arc_and_clears_on_absence_or_new_worker() {
        let package = cpu_contract_tests::Fixture::new(false);
        let first = worker(&package, [5; 3]);
        let snapshot = first.shared.lock().unwrap().last_attempt.clone().unwrap();
        let shared = Arc::clone(&first.shared);
        let mut app = ComparisonApp::new(
            package.manifest.to_string_lossy().into_owned(),
            package.root.join("live-cache"),
        );
        app.worker = Some(first);
        app.active_scene = scene::Scene {
            counts: [5; 3],
            steps: 3,
            warmup_steps: 1,
            ..Default::default()
        };
        app.active_policy = ExecutionPolicy::TeacherStage;
        app.show_attempt = true;
        let context = egui::Context::default();
        let paint = |app: &mut ComparisonApp| {
            let _ = context.run(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1500.0, 1400.0))),
                    ..Default::default()
                },
                |ctx| {
                    assert!(app.show(ctx));
                },
            );
        };
        paint(&mut app);
        assert!(app.live_attempt.get().unwrap().is_for(&snapshot));
        app.attempt_cut = 2;
        app.attempt_probe = Some(3);
        app.cut = 4;
        app.probe = Some(7);
        for _ in 0..3 {
            paint(&mut app);
        }
        assert_eq!(app.live_attempt.constructions(), 1);
        assert_eq!(app.attempt_probe, Some(53));
        assert_eq!(
            (app.cut, app.probe),
            (4, Some(7)),
            "live diagnostic probe has independent owner from legacy frames"
        );
        shared.lock().unwrap().last_attempt = None;
        paint(&mut app);
        assert!(app.live_attempt.get().is_none());
        assert!(app.attempt_probe.is_none());
        shared.lock().unwrap().last_attempt = Some(Arc::clone(&snapshot));
        paint(&mut app);
        assert_eq!(app.live_attempt.constructions(), 2);
        let second = worker(&package, [5; 3]);
        let other = second.shared.lock().unwrap().last_attempt.clone().unwrap();
        assert_eq!(snapshot.attempt_index, other.attempt_index);
        assert_ne!(snapshot.request.run_id(), other.request.run_id());
        app.attempt_cut = 3;
        app.attempt_probe = Some(78);
        app.worker = Some(second);
        paint(&mut app);
        assert!(app.live_attempt.get().unwrap().is_for(&other));
        assert_eq!(app.live_attempt.constructions(), 3);
        assert_eq!((app.attempt_cut, app.attempt_probe), (0, None));
        app.show_attempt = false;
        paint(&mut app);
        assert!(app.live_attempt.get().is_none());
        app.show_attempt = true;
        paint(&mut app);
        assert_eq!(app.live_attempt.constructions(), 4);
        app.worker = None;
        paint(&mut app);
        assert!(app.live_attempt.get().is_none());
    }
    #[test]
    fn actual_saved_rerun_button_never_renders_old_arrays_with_new_grid() {
        let package = cpu_contract_tests::Fixture::new(false);
        let old = worker(&package, [5; 3]);
        let new = worker(&package, [6; 3]);
        let path = new
            .shared
            .lock()
            .unwrap()
            .diagnostic_manifest
            .clone()
            .unwrap();
        drop(new);
        let old_shared = old.shared.clone();
        let mut app = ComparisonApp::new(
            package.manifest.to_string_lossy().into_owned(),
            package.root.join("reruns"),
        );
        app.worker = Some(old);
        app.active_scene = scene::Scene {
            counts: [5; 3],
            steps: 3,
            warmup_steps: 1,
            ..Default::default()
        };
        app.inspector.set_path(path.to_string_lossy().into_owned());
        app.inspector.load_for_test();
        let context = egui::Context::default();
        let mut paint = |events: Vec<Event>| {
            let _ = context.run(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1500.0, 1100.0))),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    assert!(app.show(ctx));
                },
            );
            app.inspector.rerun_rect
        };
        for _ in 0..3 {
            paint(vec![]);
        }
        let position = paint(vec![]).unwrap().center();
        paint(vec![
            Event::PointerMoved(position),
            Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ]);
        paint(vec![
            Event::PointerMoved(position),
            Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ]);
        assert_eq!(app.active_scene.counts, [6; 3]);
        assert!(!Arc::ptr_eq(
            &old_shared,
            &app.worker.as_ref().unwrap().shared
        ));
        // Also paint the next frame, now using only the new owner's snapshots.
        let _ = context.run(RawInput::default(), |ctx| {
            assert!(app.show(ctx));
        });
    }
}
