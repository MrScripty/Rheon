//! Fresh paired native / CPU-checkpoint water laboratory with explicit AI verdicts.
mod engine;
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
    closing: bool,
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
            status: "Choose a CPU deployment and start a fresh paired simulation".into(),
            closing: false,
        }
    }
    pub fn set_steps(&mut self, steps: usize) {
        self.scene.steps = steps;
    }
    pub fn start(&mut self) {
        if self.running() {
            return;
        }
        match Worker::start(
            self.scene.clone(),
            PathBuf::from(&self.deployment),
            self.project_directory.clone(),
        ) {
            Ok(worker) => {
                self.worker = Some(worker);
                self.active_scene = self.scene.clone();
                self.selected = 0;
                self.cut = self.cut.min(self.scene.counts[2] - 1);
                self.probe = None;
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
            ui.add_enabled_ui(!running,|ui|{
                ui.collapsing("CPU checkpoint deployment",|ui|{ui.text_edit_singleline(&mut self.deployment);ui.small("Select the exported deployment.json from the qualified training Run.");});
                ui.horizontal(|ui|{ui.label("Seed");ui.add(egui::DragValue::new(&mut self.scene.seed).range(0..=((1_u64<<53)-1)));if ui.button("New seed").clicked(){self.scene.seed=self.scene.seed.wrapping_add(0x9e3779b97f4a7c15)&((1_u64<<53)-1);}});
                ui.label("Cell counts X / Y / Z");ui.horizontal(|ui|{for count in &mut self.scene.counts{ui.add(egui::DragValue::new(count).range(5..=128));}});
                ui.horizontal(|ui|{ui.label("Cells per metre");ui.add(egui::DragValue::new(&mut self.scene.cells_per_metre).range(1..=128));});
                ui.small(format!("Domain {:?} m · cubic cell {:.5} m",self.scene.extents(),self.scene.h()));ui.small("Training: 5–16 cells/axis; 8 or 16 cells/m. Larger grids are extrapolation.");
                ui.horizontal(|ui|{ui.label("Timesteps");ui.add(egui::DragValue::new(&mut self.scene.steps).range(1..=512));});ui.horizontal(|ui|{ui.label("Native warm-up steps");ui.add(egui::DragValue::new(&mut self.scene.warmup_steps).range(0..=511));});ui.small("Fixed Δt = 1/60 s. Shared native prefix, then independent AI history. Checkpoint minimum is enforced.");
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
            ui.separator();ui.label("AI is a raw diagnostic track. Failed pressure checks stay visible. Courant, nonfinite or wall/flux failure stops the paired run.");
        });
        let mut selected_frame = None;
        let mut count = 0;
        let mut pressure_range = 0.0;
        let mut delta_range = 0.0;
        let mut checkpoint = String::new();
        let playback_delta = self.speed * f64::from(ctx.input(|i| i.stable_dt.min(0.1)));
        if let Some(worker) = &self.worker
            && let Ok(shared) = worker.shared.lock()
        {
            count = shared.frames.len();
            pressure_range = shared.pressure_range;
            delta_range = shared.difference_range;
            checkpoint = shared.checkpoint.clone();
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
        egui::TopBottomPanel::top("ai-comparison-status").show(ctx, |ui| {
            ui.heading("Rheon solver / AI solver · fresh water simulations");
            ui.label(&self.status);
            if !checkpoint.is_empty() {
                ui.small(format!(
                    "Frozen CPU checkpoint {} · optimizer updates during comparison: 0",
                    &checkpoint[..12]
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
    let scene = scene::Scene {
        steps,
        ..Default::default()
    };
    let shared = Mutex::new(worker::Shared::default());
    let cancel = AtomicBool::new(false);
    worker::run(
        scene,
        deployment,
        directory,
        &shared,
        &cancel,
        Arc::new(Mutex::new(None)),
    )?;
    let data = shared.lock().map_err(|_| "History lock")?;
    let last = data.frames.last().ok_or("No frames")?;
    println!(
        "{}",
        serde_json::json!({"state":data.state,"message":data.message,"frames":data.frames.len(),"step":last.step,"time_s":last.time_s,"reference_qualified":last.reference_quality.as_ref().map(|q|q.qualified),"ai_qualified":last.ai_quality.as_ref().map(|q|q.qualified),"checkpoint_sha256":data.checkpoint,"directory":data.directory})
    );
    Ok(())
}
