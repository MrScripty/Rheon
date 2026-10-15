//! Explicit cold loads and cached, synchronized native-grid attempt slices.
use super::{
    diagnostic_runs::{DiagnosticRunSet, FrozenDiagnosticSpec, MemberState},
    view,
};
use eframe::egui;
use rheon::recording::{
    comparison::FieldComparison,
    diagnostic::{
        DiagnosticAttempt, DiagnosticReader, PredictionOutcome, PressureComparisonUnavailable,
    },
};
use std::{path::Path, sync::Arc};

/// One reduction over an exact immutable publication. The retained strong Arc
/// prevents address reuse and mutation through Arc::get_mut while displayed.
pub(super) struct AttemptProjection {
    attempt: Arc<DiagnosticAttempt>,
    comparison: Result<FieldComparison, PressureComparisonUnavailable>,
}
impl AttemptProjection {
    fn new(attempt: Arc<DiagnosticAttempt>) -> Self {
        let comparison = attempt.pressure_comparison();
        Self {
            attempt,
            comparison,
        }
    }
    pub(super) fn is_for(&self, attempt: &Arc<DiagnosticAttempt>) -> bool {
        Arc::ptr_eq(&self.attempt, attempt)
    }
}

/// Shared by cold and live callers. No arrays are copied and no file/digest is
/// read on a retained publication's paint path.
#[derive(Default)]
pub(super) struct AttemptCache {
    projection: Option<AttemptProjection>,
    #[cfg(test)]
    constructions: usize,
}
impl AttemptCache {
    pub(super) fn get(&self) -> Option<&AttemptProjection> {
        self.projection.as_ref()
    }
    #[cfg(test)]
    pub(super) fn constructions(&self) -> usize {
        self.constructions
    }
    pub(super) fn update(
        &mut self,
        attempt: Option<Arc<DiagnosticAttempt>>,
        cut: &mut usize,
        probe: &mut Option<usize>,
    ) {
        if self
            .projection
            .as_ref()
            .zip(attempt.as_ref())
            .is_some_and(|(previous, current)| previous.is_for(current))
        {
            return;
        }
        let same_owner_grid =
            self.projection
                .as_ref()
                .zip(attempt.as_ref())
                .is_some_and(|(previous, current)| {
                    let a = &previous.attempt.request;
                    let b = &current.request;
                    a.run_id() == b.run_id()
                        && a.grid().counts() == b.grid().counts()
                        && a.grid().spacing().map(f64::to_bits)
                            == b.grid().spacing().map(f64::to_bits)
                        && a.grid().origin().map(f64::to_bits)
                            == b.grid().origin().map(f64::to_bits)
                });
        if !same_owner_grid {
            *cut = 0;
            *probe = None;
        }
        self.projection = attempt.map(|attempt| {
            #[cfg(test)]
            {
                self.constructions += 1;
            }
            AttemptProjection::new(attempt)
        });
    }
}
#[derive(Default)]
pub(super) struct Inspector {
    pub path: String,
    pub visible: bool,
    set: Option<DiagnosticRunSet>,
    reader: Option<DiagnosticReader>,
    attempt: AttemptCache,
    member: usize,
    ordinal: usize,
    count: usize,
    cut: usize,
    probe: Option<usize>,
    difference: bool,
    status: String,
    #[cfg(test)]
    pub(super) rerun_rect: Option<egui::Rect>,
}
impl Inspector {
    pub fn report_execution_failure(&mut self, error: &str) {
        self.status = format!("Frozen rerun unavailable: {error}");
    }
    #[cfg(test)]
    pub(super) fn load_for_test(&mut self) {
        self.open().unwrap();
        self.visible = true;
    }
    pub fn set_path(&mut self, path: String) {
        if self.path != path {
            self.clear();
            self.set = None;
            self.status = "Unverified path; open explicitly".into();
            self.path = path;
        }
    }
    fn clear(&mut self) {
        self.reader = None;
        self.attempt.update(None, &mut self.cut, &mut self.probe);
        self.count = 0;
        self.probe = None;
    }
    fn open(&mut self) -> Result<(), String> {
        self.clear();
        self.set = None;
        let set = DiagnosticRunSet::open(Path::new(&self.path))?;
        self.member = 0;
        self.ordinal = 0;
        self.set = Some(set);
        self.load_member()
    }
    fn load_member(&mut self) -> Result<(), String> {
        self.clear();
        let set = self.set.as_ref().ok_or("Open saved intent first")?;
        let root = Path::new(&self.path)
            .parent()
            .ok_or("Saved intent directory")?;
        let (reader, state) = match set.open_member(root, self.member) {
            MemberState::Ready(r) => (r, "Pinned closed archive verified"),
            MemberState::Incomplete(r) => (r, "Incomplete archive: committed prefix only"),
            MemberState::Unpinned(r) => (r, "Closed archive is unpinned"),
            MemberState::NotRun => {
                self.status = "Not run; no archive".into();
                return Ok(());
            }
            MemberState::Missing => return Err("Missing diagnostic archive".into()),
            MemberState::Invalid(e) => return Err(e),
        };
        self.count = reader.count().map_err(|e| e.to_string())?;
        self.reader = Some(reader);
        self.status = state.into();
        self.ordinal = self.ordinal.min(self.count.saturating_sub(1));
        self.load_attempt()
    }
    fn load_attempt(&mut self) -> Result<(), String> {
        let selected = (|| {
            if self.count == 0 {
                return Ok(None);
            }
            self.reader
                .as_ref()
                .ok_or("No archive")?
                .select(self.ordinal)
                .map(|attempt| Some(Arc::new(attempt)))
                .map_err(|e| e.to_string())
        })();
        match selected {
            Ok(attempt) => {
                self.attempt.update(attempt, &mut self.cut, &mut self.probe);
                Ok(())
            }
            Err(error) => {
                self.attempt.update(None, &mut self.cut, &mut self.probe);
                Err(error)
            }
        }
    }
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        running: bool,
    ) -> Option<(FrozenDiagnosticSpec, [u8; 32])> {
        if !self.visible {
            return None;
        }
        let mut rerun = None;
        let mut visible = self.visible;
        egui::Window::new("Saved diagnostic attempts").open(&mut visible).default_size([1050.0,760.0]).vscroll(true).show(ctx,|ui|{
            ui.horizontal(|ui| {
                ui.label("Intent or finished JSON");
                if ui.text_edit_singleline(&mut self.path).changed() {
                    self.clear();
                    self.set = None;
                    self.status = "Unverified path; open explicitly".into();
                }
                if ui.button("Open / verify").clicked() && let Err(error) = self.open() {
                    self.status = error;
                }
            });
            ui.label(&self.status);
            if let Some(set)=self.set.clone(){
                ui.label(format!("Set {} · baseline {}",set.set_id,set.baseline_member));
                let previous=self.member;
                egui::ComboBox::from_id_salt("diagnostic-member").selected_text(&set.members[self.member].member_id).show_ui(ui,|ui|{for (index,m) in set.members.iter().enumerate(){ui.selectable_value(&mut self.member,index,&m.member_id);}});
                if previous!=self.member&&let Err(e)=self.load_member(){self.status=e;}
                let member=&set.members[self.member];
                ui.label(format!("{} · native prefix requested {} · horizon {} · recording budget {} attempts",member.spec.policy.label(),member.spec.scene.warmup_steps,member.spec.scene.steps,member.spec.recording.max_records));
                if let Some(o)=&member.observed{let s=&o.statistics;ui.label(format!("Observed {:?}: prefix {}, produced {}, dispatched {}, qualified {}, rejected {}, unavailable {}, durable {}",o.terminal,s.native_prefix_steps,s.produced_attempts,s.dispatched_calls,s.qualified,s.rejected,s.unavailable,s.recorded_attempts));ui.small(format!("Wall costs: startup {:.6}s · model calls {:.6}s · execution before finalization {:.6}s; includes protocol/waits and100ms UI pacing, no isolated model/device benchmark",s.model_start_seconds,s.model_prediction_seconds,s.run_wall_seconds));if s.produced_attempts>s.recorded_attempts{ui.colored_label(egui::Color32::YELLOW,"Unrecorded tail: observed category totals beyond the durable prefix cannot be reconstructed");}}
                if ui.button("Export exact selected attempt summary (new file)").clicked() {
                    let result=self.attempt.get().ok_or_else(||"Load an attempt first".to_owned()).and_then(|projection|export_attempt(Path::new(&self.path),&set,self.member,&projection.attempt));
                    match result {Ok(path)=>self.status=format!("Published {}",path.display()),Err(error)=>{self.clear();self.status=error;}}
                }
                let rerun_response=ui.add_enabled(!running,egui::Button::new("Rerun frozen member in fresh directory"));
                #[cfg(test)] {self.rerun_rect=Some(rerun_response.rect);}
                if rerun_response.clicked(){
                    // Reopen the selected document on the action; cached display is not execution authority.
                    match DiagnosticRunSet::open(Path::new(&self.path)).and_then(|fresh|{
                        if fresh.digest()!=set.digest(){return Err("Saved intent changed; reopen before rerun".into());}
                        Ok((fresh.members.get(self.member).ok_or("Member absent")?.spec.clone(),fresh.digest()?))
                    }){Ok(request)=>rerun=Some(request),Err(e)=>self.status=e}
                }
            }
            if self.count>0{ui.horizontal(|ui|{ui.add(egui::DragValue::new(&mut self.ordinal).range(0..=self.count-1).prefix("Attempt index "));if ui.button("Load attempt").clicked()&&let Err(e)=self.load_attempt(){self.status=e;}});}
            if let Some(projection)=self.attempt.get(){attempt_ui(ui,projection,&mut self.cut,&mut self.probe,&mut self.difference);}
        });
        self.visible = visible;
        rerun
    }
}
fn export_attempt(
    path: &Path,
    set: &DiagnosticRunSet,
    member: usize,
    attempt: &DiagnosticAttempt,
) -> Result<std::path::PathBuf, String> {
    let fresh = DiagnosticRunSet::open(path)?;
    if fresh.digest()? != set.digest()? {
        return Err("Saved intent changed before export".into());
    }
    let root = path.parent().ok_or("Saved directory")?;
    let reader = match fresh.open_member(root, member) {
        MemberState::Ready(reader) => reader,
        MemberState::Invalid(error) => return Err(error),
        _ => return Err("Export requires exact pinned closed diagnostic source".into()),
    };
    if reader
        .select(attempt.attempt_index)
        .map_err(|e| e.to_string())?
        != *attempt
    {
        return Err("Cached attempt differs from exact source".into());
    }
    let selected = &fresh.members[member];
    let (comparison, unavailable) = match attempt.pressure_comparison() {
        Ok(comparison) => (Some(comparison), None),
        Err(reason) => (None, Some(format!("{reason:?}"))),
    };
    let value = serde_json::json!({"schema":"rheon-diagnostic-pressure-comparison-v1","analysis_version":1,"run_set_file":path.file_name().and_then(|v|v.to_str()).ok_or("UTF8 saved file")?,"run_set_sha256":fresh.digest()?,"member_id":selected.member_id,"run_id":selected.run_id,"archive_sha256":selected.finalized.as_ref().ok_or("Source pin absent")?.sha256,"attempt_index":attempt.attempt_index,"request_sha256":attempt.request.digest(),"counts":attempt.request.grid().counts(),"interval_start":attempt.request.interval_start(),"interval_end":attempt.request.interval_end(),"dt":attempt.request.dt(),"comparison":comparison,"unavailable":unavailable});
    let output = root.join(format!(
        "diagnostic-comparison-{member}-{}.json",
        attempt.attempt_index
    ));
    crate::run_sets::publish_new(
        &output,
        &serde_json::to_vec(&value).map_err(|e| e.to_string())?,
        1024 * 1024,
    )
    .map_err(|e| e.to_string())?;
    Ok(output)
}
pub(super) fn attempt_ui(
    ui: &mut egui::Ui,
    projection: &AttemptProjection,
    cut: &mut usize,
    probe: &mut Option<usize>,
    difference: &mut bool,
) {
    let attempt = &projection.attempt;
    let r = &attempt.request;
    let counts = r.grid().counts();
    *cut = (*cut).min(counts[2] - 1);
    ui.separator();
    ui.label(format!(
        "Request {} · generation {} · [{:.17}, {:.17}] s · dt {:.17}s · {:?}",
        attempt.attempt_index,
        r.generation(),
        r.interval_start(),
        r.interval_end(),
        r.dt(),
        attempt.transition
    ));
    ui.small(format!("Native cells {:?}, spacing {:?} m, origin {:?} m · cell pressure · cell-zero gauge · dispatched {}",counts,r.grid().spacing(),r.grid().origin(),attempt.dispatched));
    ui.horizontal(|ui| {
        ui.add(egui::Slider::new(cut, 0..=counts[2] - 1).text("Shared Z slice"));
        ui.checkbox(difference, "Candidate − native");
    });
    // Keep the shared probe on the displayed plane, including programmatic cuts.
    // Preserve X/Y; invalid previous indices carry no reusable sample identity.
    let plane_len = counts[0] * counts[1];
    *probe = probe
        .filter(|index| *index < r.rhs().len())
        .map(|index| index % plane_len + plane_len * *cut);
    if let Some(native) = &attempt.native_reference {
        let q = &native.sample.observation;
        ui.small(format!(
            "Native same-request reference: host pressure gate qualified {} · hard stop {} · walls closed {} · history {:?}. This diagnostic disagreement is not an accuracy acceptance result.",
            q.qualified, q.hard_stop, q.walls_closed, native.history_status
        ));
    } else {
        ui.small("Native same-request reference unavailable; no qualification is inferred.");
    }
    let PredictionOutcome::Candidate(candidate) = &attempt.outcome else {
        ui.label(format!("No candidate: {:?}", attempt.outcome));
        ui.label("Candidate/native comparison unavailable; no replacement field is drawn.");
        return;
    };
    let q = &candidate.observation;
    ui.label(format!("Candidate host pressure gate: qualified {} · hard stop {} · walls closed {} · residual {:.6e} (relative {:?}, limit {:.3e}; absolute {:.3e})",q.qualified,q.hard_stop,q.walls_closed,q.residual_l2,q.relative_residual,q.relative_limit,q.absolute_limit));
    ui.small(format!("Predicted divergence {:.3e}/{:.3e} s⁻¹ · actual {:.3e}/{:.3e} s⁻¹ · Courant {:.3e}/{:.3e} · net flux {:.3e}, limit {:.3e}",q.predicted_divergence,q.predicted_limit,q.actual_divergence,q.actual_limit,q.courant,q.courant_limit,q.net_flux,q.flux_limit));
    match &projection.comparison {
        Ok(comparison) => {
            let m = comparison.metrics;
            ui.label(format!("Equal-request pressure: L2 {:.9e} Pa · RMSE {:.9e} Pa · L∞ {:.9e} Pa · relative {:?} · {} cells",m.l2,m.rmse,m.linf,m.relative_l2,m.support_count));
            let native = &attempt.native_reference.as_ref().unwrap().sample.pressure;
            let scale = comparison
                .color_bounds
                .min
                .abs()
                .max(comparison.color_bounds.max.abs());
            let delta = comparison.metrics.linf;
            ui.columns(2, |columns| {
                slice(
                    &mut columns[0],
                    (counts, *cut),
                    native,
                    None,
                    scale,
                    probe,
                    "Native same request · Pa",
                );
                slice(
                    &mut columns[1],
                    (counts, *cut),
                    &candidate.pressure,
                    if *difference { Some(native) } else { None },
                    if *difference { delta } else { scale },
                    probe,
                    if *difference {
                        "Candidate − native · Pa"
                    } else {
                        "Candidate · Pa (common scale)"
                    },
                );
            });
        }
        Err(e) => {
            ui.label(format!("Equal-request comparison unavailable: {e:?}"));
            return;
        }
    }
    if let Some(index) = probe.filter(|i| *i < r.rhs().len()) {
        let c = [
            index % counts[0],
            index / counts[0] % counts[1],
            index / (counts[0] * counts[1]),
        ];
        let position = std::array::from_fn::<_, 3, _>(|d| {
            r.grid().origin()[d] + (c[d] as f64 + 0.5) * r.grid().spacing()[d]
        });
        ui.label(format!("Cell {c:?} at {position:?} m: candidate {:.17} Pa · native {:?} Pa · RHS {:.17} · initial residual {:.17} · prior {:?} Pa",candidate.pressure[index],attempt.native_reference.as_ref().map(|v|v.sample.pressure[index]),r.rhs()[index],r.initial_residual()[index],r.prior().map(|v|v.pressure[index])));
    }
}
fn slice(
    ui: &mut egui::Ui,
    geometry: ([usize; 3], usize),
    values: &[f64],
    reference: Option<&[f64]>,
    scale: f64,
    probe: &mut Option<usize>,
    label: &str,
) {
    let (counts, cut) = geometry;
    view::legend(ui, scale, label);
    let width = ui.available_width().min(420.0);
    let height = width * counts[1] as f32 / counts[0] as f32;
    let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::click());
    let cell = egui::vec2(width / counts[0] as f32, height / counts[1] as f32);
    for j in 0..counts[1] {
        for i in 0..counts[0] {
            let index = i + counts[0] * (j + counts[1] * cut);
            let v = values[index] - reference.map_or(0.0, |r| r[index]);
            let rect = egui::Rect::from_min_size(
                response.rect.min
                    + egui::vec2(i as f32 * cell.x, (counts[1] - 1 - j) as f32 * cell.y),
                cell,
            );
            painter.rect_filled(rect, 0.0, view::color(v, scale));
            if *probe == Some(index) {
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    egui::StrokeKind::Inside,
                );
            }
            if response.clicked()
                && response
                    .interact_pointer_pos()
                    .is_some_and(|p| rect.contains(p))
            {
                *probe = Some(index);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_comparison::{
        cpu_contract_tests::Fixture, engine::ExecutionPolicy, scene::Scene, worker::Worker,
    };
    fn producer(p: &Fixture) -> Worker {
        let mut w = Worker::start_with_policy(
            Scene {
                counts: [5; 3],
                steps: 3,
                warmup_steps: 1,
                ..Default::default()
            },
            p.manifest.clone(),
            p.root.join("runs"),
            ExecutionPolicy::TeacherStage,
        )
        .unwrap();
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !w.finished() {
            assert!(std::time::Instant::now() < end);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        w.join_finished();
        w
    }
    fn render(
        cache: &AttemptCache,
        cut: &mut usize,
        probe: &mut Option<usize>,
    ) -> egui::FullOutput {
        egui::Context::default().run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 1600.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    attempt_ui(ui, cache.get().unwrap(), cut, probe, &mut false);
                });
            },
        )
    }
    fn text(output: &egui::FullOutput) -> String {
        output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.text()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    #[test]
    fn programmatic_namespace_change_and_failed_reopen_clear_old_attempts() {
        let p = Fixture::new(false);
        let w = producer(&p);
        let path = w
            .shared
            .lock()
            .unwrap()
            .diagnostic_manifest
            .clone()
            .unwrap();
        drop(w);
        let mut inspector = Inspector::default();
        inspector.set_path(path.to_string_lossy().into_owned());
        inspector.open().unwrap();
        assert!(inspector.attempt.get().is_some());
        assert!(inspector.reader.is_some());
        assert!(inspector.status.contains("verified"));
        // Exercise the actual slider, then verify that the text probe and both
        // pane highlights identify the newly displayed plane in the same frame.
        let context = egui::Context::default();
        let mut cut = 0;
        let mut probe = Some(0);
        let mut difference = false;
        let projection = inspector.attempt.get().unwrap();
        let attempt = &projection.attempt;
        let mut frame = |events| {
            context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1000.0, 1400.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        attempt_ui(ui, projection, &mut cut, &mut probe, &mut difference);
                    });
                },
            )
        };
        let first = frame(vec![]);
        let label = first
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.text() == "Shared Z slice" => {
                    Some(text.pos)
                }
                _ => None,
            })
            .expect("actual shared Z slider label");
        let pointer = label + egui::vec2(-64.0, 7.0);
        let modifiers = egui::Modifiers::default();
        let _ = frame(vec![
            egui::Event::PointerMoved(pointer),
            egui::Event::PointerButton {
                pos: pointer,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers,
            },
        ]);
        let rendered = frame(vec![egui::Event::PointerButton {
            pos: pointer,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers,
        }]);
        assert!(cut > 0, "the actual slider must move the slice");
        let counts = attempt.request.grid().counts();
        assert_eq!(probe, Some(counts[0] * counts[1] * cut));
        assert_eq!(
            rendered
                .shapes
                .iter()
                .filter(|shape| matches!(
                    &shape.shape, egui::epaint::Shape::Rect(rect)
                    if rect.stroke.color == egui::Color32::YELLOW && rect.stroke.width == 2.0
                ))
                .count(),
            2,
            "both visible panes must highlight the remapped probe"
        );
        let expected = format!("Cell [0, 0, {cut}]");
        assert!(rendered.shapes.iter().any(|shape| matches!(
            &shape.shape, egui::epaint::Shape::Text(text)
            if text.galley.text().starts_with(&expected)
        )));

        let expected_candidate = match &attempt.outcome {
            PredictionOutcome::Candidate(candidate) => format!(
                "Candidate host pressure gate: qualified {}",
                candidate.observation.qualified
            ),
            _ => panic!("producer candidate expected"),
        };
        assert!(text(&rendered).contains(&expected_candidate));

        let native = attempt.native_reference.as_ref().unwrap();
        let expected_native = format!(
            "Native same-request reference: host pressure gate qualified {}",
            native.sample.observation.qualified
        );
        let expected_history = format!("history {:?}", native.history_status);
        assert!(rendered.shapes.iter().any(|shape| matches!(
            &shape.shape, egui::epaint::Shape::Text(text)
            if text.galley.text().contains(&expected_native)
                && text.galley.text().contains(&expected_history)
                && text.galley.text().contains("not an accuracy acceptance result")
        )));

        // Export must reduce the reopened exact attempt, never the display cache.
        // Test-only corruption makes accidental cache authority observable.
        inspector
            .attempt
            .projection
            .as_mut()
            .unwrap()
            .comparison
            .as_mut()
            .unwrap()
            .metrics
            .l2 += 1234.0;
        let summary = export_attempt(
            &path,
            inspector.set.as_ref().unwrap(),
            0,
            &inspector.attempt.get().unwrap().attempt,
        )
        .unwrap();
        assert!(
            export_attempt(
                &path,
                inspector.set.as_ref().unwrap(),
                0,
                &inspector.attempt.get().unwrap().attempt
            )
            .is_err()
        );
        let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tools/verify_diagnostic_pressure.py");
        let mut changed: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&summary).unwrap()).unwrap();
        changed["comparison"]["metrics"]["rmse"] =
            serde_json::json!(1.0 + changed["comparison"]["metrics"]["rmse"].as_f64().unwrap());
        let damaged = summary.with_file_name("changed-metric.json");
        std::fs::write(&damaged, serde_json::to_vec(&changed).unwrap()).unwrap();
        for mode in 0..3 {
            let invoke = |input: &std::path::Path| {
                let mut command = std::process::Command::new("python3");
                if mode == 1 {
                    command.arg("-O");
                }
                if mode == 2 {
                    command.env("PYTHONOPTIMIZE", "1");
                }
                command.arg(&script).arg(input).output().unwrap()
            };
            let valid = invoke(&summary);
            assert!(
                valid.status.success(),
                "{}",
                String::from_utf8_lossy(&valid.stderr)
            );
            let invalid = invoke(&damaged);
            assert!(!invalid.status.success());
            assert!(String::from_utf8_lossy(&invalid.stderr).contains("metric rmse"));
        }

        drop(inspector.reader.take());
        let archive = Path::new(&inspector.path)
            .parent()
            .unwrap()
            .join(&inspector.set.as_ref().unwrap().members[0].archive_file);
        let hidden = archive.with_extension("hidden");
        std::fs::rename(&archive, &hidden).unwrap();
        let cached_paint = render(&inspector.attempt, &mut inspector.cut, &mut inspector.probe);
        assert!(text(&cached_paint).contains("Equal-request pressure:"));
        assert!(
            export_attempt(
                &path,
                inspector.set.as_ref().unwrap(),
                0,
                &inspector.attempt.get().unwrap().attempt
            )
            .is_err()
        );
        std::fs::rename(&hidden, &archive).unwrap();
        inspector.load_member().unwrap();
        inspector.ordinal = inspector.count;
        assert!(inspector.load_attempt().is_err());
        assert!(inspector.attempt.get().is_none());
        assert!(inspector.probe.is_none());
        inspector.member = inspector.set.as_ref().unwrap().members.len();
        assert!(inspector.load_member().is_err());
        assert!(inspector.attempt.get().is_none());
        assert!(inspector.reader.is_none());

        // Same entry point used by the actual live-run 'Inspect saved run' action.
        inspector.set_path(
            p.root
                .join("other-intent.json")
                .to_string_lossy()
                .into_owned(),
        );
        assert!(inspector.reader.is_none());
        assert!(inspector.attempt.get().is_none());
        assert!(inspector.set.is_none());
        assert_eq!(inspector.count, 0);
        assert!(inspector.status.starts_with("Unverified"));
        assert!(inspector.open().is_err());
        assert!(inspector.attempt.get().is_none());
        inspector.visible = true;
        let context = egui::Context::default();
        let _ = context.run(egui::RawInput::default(), |ctx| {
            assert!(inspector.show(ctx, false).is_none());
        });
        assert!(inspector.reader.is_none());
    }

    #[test]
    fn projection_cache_uses_exact_arc_retention_and_caches_unavailable_outcomes() {
        let package = Fixture::new(false);
        let worker = producer(&package);
        let mut snapshot = worker.shared.lock().unwrap().last_attempt.clone().unwrap();
        let weak = Arc::downgrade(&snapshot);
        let mut cache = AttemptCache::default();
        let mut cut = 2;
        let mut probe = Some(3);
        cache.update(Some(Arc::clone(&snapshot)), &mut cut, &mut probe);
        assert_eq!(cache.constructions(), 1);
        assert_eq!((cut, probe), (0, None));
        assert!(cache.get().unwrap().is_for(&snapshot));
        assert!(
            Arc::get_mut(&mut snapshot).is_none(),
            "cache must retain immutable strong ownership"
        );
        let original = cache.get().unwrap().comparison.clone().unwrap();
        let native_ptr = snapshot
            .native_reference
            .as_ref()
            .unwrap()
            .sample
            .pressure
            .as_ptr();
        assert_eq!(
            native_ptr,
            cache
                .get()
                .unwrap()
                .attempt
                .native_reference
                .as_ref()
                .unwrap()
                .sample
                .pressure
                .as_ptr()
        );
        cut = 2;
        probe = Some(3);
        for _ in 0..3 {
            cache.update(Some(Arc::clone(&snapshot)), &mut cut, &mut probe);
            let output = render(&cache, &mut cut, &mut probe);
            assert!(text(&output).contains("Cell [3, 0, 2]"));
            assert_eq!(probe, Some(3 + 25 * 2));
        }
        assert_eq!(
            cache.constructions(),
            1,
            "retained paint must not rebuild projection"
        );
        drop(worker);
        drop(snapshot);
        assert_eq!(
            weak.strong_count(),
            1,
            "only display cache now retains publication"
        );
        let snapshot = weak.upgrade().unwrap();
        let request = snapshot.request.digest();
        let equal = Arc::new(snapshot.as_ref().clone());
        cache.update(Some(Arc::clone(&equal)), &mut cut, &mut probe);
        assert_eq!(
            cache.constructions(),
            2,
            "new equal Arc is a new immutable publication"
        );
        assert_eq!(cache.get().unwrap().comparison.as_ref().unwrap(), &original);
        let mut changed = equal.as_ref().clone();
        let PredictionOutcome::Candidate(candidate) = &mut changed.outcome else {
            panic!("candidate")
        };
        candidate.pressure[1] += 1000.0;
        assert_eq!(changed.request.digest(), request);
        assert_eq!(changed.attempt_index, snapshot.attempt_index);
        let changed = Arc::new(changed);
        cache.update(Some(Arc::clone(&changed)), &mut cut, &mut probe);
        assert_eq!(cache.constructions(), 3);
        assert_ne!(cache.get().unwrap().comparison.as_ref().unwrap(), &original);
        assert_eq!(
            probe,
            Some(53),
            "same owner/grid keeps the physical location"
        );
        let mut missing = changed.as_ref().clone();
        missing.native_reference = None;
        let missing = Arc::new(missing);
        let missing_weak = Arc::downgrade(&missing);
        for _ in 0..3 {
            cache.update(Some(Arc::clone(&missing)), &mut cut, &mut probe);
            assert_eq!(
                cache.get().unwrap().comparison,
                Err(PressureComparisonUnavailable::NoNativeReference)
            );
            let labels = text(&render(&cache, &mut cut, &mut probe));
            assert!(labels.contains("Equal-request comparison unavailable: NoNativeReference"));
            assert!(!labels.contains("Native same request · Pa"));
            assert!(!labels.contains("Cell ["));
        }
        assert_eq!(
            cache.constructions(),
            4,
            "unavailable comparisons are cached too"
        );
        drop(missing);
        assert_eq!(missing_weak.strong_count(), 1);
        cache.update(None, &mut cut, &mut probe);
        assert!(cache.get().is_none());
        assert!(missing_weak.upgrade().is_none());
        assert_eq!((cut, probe), (0, None));
    }

    #[test]
    fn changed_grid_clears_physical_probe_and_native_labels_report_actual_failed_gate() {
        use rheon::{
            GridGeometry,
            solver_provider::context::{
                PressureContext, PressureContextInput, PressureContextMode, PressureHistoryStatus,
            },
        };
        let package = Fixture::new(false);
        let worker = producer(&package);
        let original = worker.shared.lock().unwrap().last_attempt.clone().unwrap();
        let mut cache = AttemptCache::default();
        let mut cut = 0;
        let mut probe = None;
        cache.update(Some(Arc::clone(&original)), &mut cut, &mut probe);
        cut = 3;
        probe = Some(78);
        let mut changed = original.as_ref().clone();
        let grid = GridGeometry::new([6, 5, 5], [0.125; 3], [0.0; 3]).unwrap();
        let rhs = vec![0.0; grid.cell_len()];
        let context = PressureContext::new(PressureContextInput {
            grid: &grid,
            density: 1000.0,
            dt: original.request.dt(),
            interval_start: 0.0,
            rhs: &rhs,
            prior: None,
            run_id: original.request.run_id(),
            generation: 1,
            mode: PressureContextMode::DiagnosticCandidate,
        })
        .unwrap();
        changed.request =
            rheon::recording::diagnostic::DiagnosticRequest::capture(&context, 0.001, 1 << 20)
                .unwrap();
        let native = changed.native_reference.as_mut().unwrap();
        native.request_digest = changed.request.digest();
        native.sample.pressure.resize(grid.cell_len(), 0.0);
        native.sample.observation.qualified = false;
        native.history_status = PressureHistoryStatus::Diagnostic { qualified: false };
        changed.transition = rheon::recording::diagnostic::HistoryTransition::NotAdvanced;
        let PredictionOutcome::Candidate(candidate) = &mut changed.outcome else {
            panic!("candidate")
        };
        candidate.pressure.resize(grid.cell_len(), 0.0);
        candidate.observation.qualified = true;
        // Explicit adversarial metadata variant over a real producer publication;
        // this does not claim that the adapter observed either altered gate.
        cache.update(Some(Arc::new(changed)), &mut cut, &mut probe);
        assert_eq!((cut, probe), (0, None));
        let labels = text(&render(&cache, &mut cut, &mut probe));
        assert!(
            labels.contains("Native same-request reference: host pressure gate qualified false")
        );
        assert!(labels.contains("history Diagnostic { qualified: false }"));
        assert!(labels.contains("Candidate host pressure gate: qualified true"));
        assert!(labels.contains("not an accuracy acceptance result"));
    }
}
