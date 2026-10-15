//! Typed diagnostic execution through the existing joinable Worker/ModelProcess.
//! Each attempt is appended before JSON/live publication; only closed archives pin.
use super::{
    diagnostic_runs::{
        DiagnosticRunSet, FrozenDiagnosticSpec, ProviderPin, RunObservation, RunStatistics,
        RunTerminal,
    },
    engine::{AdvanceStop, Engine, FramePhase},
    model_process::{ChildSlot, ModelProcess},
    record::Record,
    scene::Scene,
    worker::{RunOptions, Shared},
};
use rheon::recording::diagnostic::{
    DiagnosticRecorder, DiagnosticTerminal, HistoryTransition, NoCandidateReason, PredictionOutcome,
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
fn detail(s: &str) -> String {
    let clean: String = s.chars().filter(|c| !c.is_control()).take(2000).collect();
    if clean.is_empty() {
        "Diagnostic execution stopped".into()
    } else {
        clean
    }
}
fn ensure_recorder(
    engine: &Engine,
    spec: &FrozenDiagnosticSpec,
    directory: &Path,
    recorder: &mut Option<DiagnosticRecorder>,
) -> Result<(), String> {
    if recorder.is_none() {
        let mut header =
            engine.diagnostic_run(spec.provider.binding(), spec.source_revision.clone())?;
        spec.bind_header(&mut header)?;
        *recorder = Some(
            DiagnosticRecorder::create(
                &directory.join("attempts.sqlite"),
                header,
                spec.recording.limits(),
            )
            .map_err(|e| e.to_string())?,
        );
    }
    Ok(())
}
pub(super) fn run_typed(
    scene: Scene,
    deployment: PathBuf,
    root: PathBuf,
    shared: &Mutex<Shared>,
    flag: &AtomicBool,
    slot: ChildSlot,
    options: RunOptions,
) -> Result<(), String> {
    let RunOptions { policy, saved } = options;
    let mut record = Record::begin_policy(&root, &scene, policy)?;
    {
        let mut view = shared.lock().map_err(|_| "Comparison lock")?;
        view.directory = Some(record.directory.clone());
        view.policy = policy;
    }
    let mut engine: Option<Engine> = None;
    let mut recorder = None;
    let mut manifest: Option<DiagnosticRunSet> = None;
    let mut statistics = RunStatistics::default();
    let wall = Instant::now();
    let result = (|| -> Result<(RunTerminal, String), String> {
        record.activity("running", "Validating typed CPU deployment", 0)?;
        let identity = ModelProcess::preflight(&deployment, &scene)?;
        let (spec, parent) = if let Some((spec, parent)) = saved {
            if ProviderPin::capture(&identity)? != spec.provider {
                return Err("Saved diagnostic provider pin changed; no model code executed".into());
            }
            (spec, Some(parent))
        } else {
            (
                FrozenDiagnosticSpec::capture(scene.clone(), policy, &deployment, &identity)?,
                None,
            )
        };
        spec.validate()?;
        let run_id = record
            .directory
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or("Diagnostic directory identity")?
            .to_owned();
        let intent = DiagnosticRunSet::single(run_id.clone(), spec.clone(), parent);
        intent.save_new(&record.directory.join("intent.json"))?;
        shared
            .lock()
            .map_err(|_| "Comparison lock")?
            .diagnostic_manifest = Some(record.directory.join("intent.json"));
        manifest = Some(intent);
        let start = Instant::now();
        let mut startup_publication_failed = false;
        let model = ModelProcess::start_pinned(
            &deployment,
            slot,
            flag,
            &spec.provider.identity(),
            |phase| {
                shared.lock().map_err(|_| "Comparison lock")?.message = phase.into();
                record
                    .activity("running", phase, 0)
                    .inspect_err(|_| startup_publication_failed = true)
            },
        );
        statistics.model_start_seconds = start.elapsed().as_secs_f64();
        let mut model = match model {
            Ok(model) => model,
            Err(e) if flag.load(Ordering::Relaxed) && !startup_publication_failed => {
                return Ok((RunTerminal::Cancelled, e));
            }
            Err(e) => return Err(e),
        };
        record.model(&model.identity, &model.deployment_path)?;
        shared.lock().map_err(|_| "Comparison lock")?.provider = Some(model.identity.clone());
        engine = Some(Engine::with_policy(scene.clone(), policy, run_id)?);
        let engine = engine.as_mut().unwrap();
        let initial = Arc::new(engine.initial());
        record.frame(&initial)?;
        shared
            .lock()
            .map_err(|_| "Comparison lock")?
            .frames
            .push(initial);
        let deadline = Instant::now() + Duration::from_secs(600);
        for _ in 0..scene.steps {
            if flag.load(Ordering::Relaxed) {
                return Ok((
                    RunTerminal::Cancelled,
                    "Cancelled; typed recorded prefix retained".into(),
                ));
            }
            if Instant::now() >= deadline {
                return Err("Typed diagnostic 600s deadline reached".into());
            }
            if statistics.native_prefix_steps == scene.warmup_steps {
                ensure_recorder(engine, &spec, &record.directory, &mut recorder)?;
            }
            let result = engine.advance_typed(flag, |input| {
                let start = Instant::now();
                let value = model.predict_typed(input, flag);
                statistics.model_prediction_seconds += start.elapsed().as_secs_f64();
                value
            });
            let result = match result {
                Ok(result) => result,
                Err(e) if flag.load(Ordering::Relaxed) => return Ok((RunTerminal::Cancelled, e)),
                Err(e) => return Err(e),
            };
            let mut attempt_for_view = None;
            if let Some(attempt) = result.attempt {
                statistics.produced_attempts += 1;
                statistics.dispatched_calls += usize::from(attempt.dispatched);
                match &attempt.outcome {
                    PredictionOutcome::Candidate(p) => {
                        if p.observation.qualified {
                            statistics.qualified += 1;
                        } else {
                            statistics.rejected += 1;
                        }
                    }
                    PredictionOutcome::NoCandidate { .. } => statistics.unavailable += 1,
                }
                if matches!(
                    attempt.transition,
                    HistoryTransition::DiagnosticAdvanced
                        | HistoryTransition::NativeTeacherAdvanced
                ) {
                    statistics.last_advanced_generation = Some(attempt.request.generation());
                }
                let writer = recorder
                    .as_mut()
                    .ok_or("Typed request has no recording owner")?;
                writer.append(&attempt).map_err(|e| {
                    format!("Diagnostic append failed; stop with possible unrecorded attempt: {e}")
                })?;
                statistics.recorded_attempts = writer.recorded_count();
                attempt_for_view = Some(Arc::new(attempt));
            }
            let mut frame_for_view = None;
            if let Some(frame) = result.frame {
                if frame.phase == FramePhase::NativeWarmup {
                    statistics.native_prefix_steps += 1;
                }
                let frame = Arc::new(frame);
                record.frame(&frame)?;
                frame_for_view = Some(frame);
            }
            let message = format!(
                "{} · native prefix{} · requests{} · dispatched{} · qualified{} · rejected{} · unavailable{}",
                policy.label(),
                statistics.native_prefix_steps,
                statistics.produced_attempts,
                statistics.dispatched_calls,
                statistics.qualified,
                statistics.rejected,
                statistics.unavailable
            );
            let advanced = frame_for_view.as_ref().map_or_else(
                || {
                    shared
                        .lock()
                        .ok()
                        .and_then(|v| v.frames.last().map(|f| f.step))
                        .unwrap_or(0)
                },
                |f| f.step,
            );
            record.activity("running", &message, advanced)?;
            {
                let mut view = shared.lock().map_err(|_| "Comparison lock")?;
                view.message = message;
                view.statistics = Some(statistics.clone());
                if let Some(attempt) = attempt_for_view {
                    view.last_attempt = Some(attempt);
                }
                if let Some(frame) = frame_for_view {
                    let ranges = super::view::ranges(&frame);
                    view.pressure_range = view.pressure_range.max(ranges.0);
                    view.difference_range = view.difference_range.max(ranges.1);
                    view.frames.push(frame);
                }
            }
            if let Some(stop) = result.stop {
                return Ok(match stop{
                AdvanceStop::Rejected=>(RunTerminal::StoppedOnRejection,"First candidate rejection retained; diagnostic state/history did not advance".into()),
                AdvanceStop::Cancelled=>(RunTerminal::Cancelled,"Cancelled before diagnostic commit; recorded attempt retained".into()),
                AdvanceStop::ProviderFailure(NoCandidateReason::Cancelled)=>(RunTerminal::Cancelled,"Cancelled provider call; recorded request retained".into()),
                AdvanceStop::ProviderFailure(reason)=>(RunTerminal::Failed,format!("Provider produced no usable candidate ({reason:?}); exact request retained")),
            });
            }
            let next = Instant::now() + Duration::from_millis(100);
            while Instant::now() < next && !flag.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        drop(model);
        Ok((
            RunTerminal::Completed,
            "Typed diagnostic horizon complete; model drained".into(),
        ))
    })();
    // Model and its I/O owners have drained before recording/finalized publication.
    let (mut terminal, mut message) = match result {
        Ok(v) => v,
        Err(e) => (RunTerminal::Failed, e),
    };
    if let (Some(engine), Some(set)) = (&engine, &manifest) {
        let spec = &set.members[0].spec;
        if let Err(e) = ensure_recorder(engine, spec, &record.directory, &mut recorder) {
            terminal = RunTerminal::Failed;
            message = format!("{message}; diagnostic archive unavailable: {e}");
        }
    }
    let mut terminal_confirmed = false;
    if let Some(mut writer) = recorder.take() {
        statistics.native_prefix_steps = writer.run().native_prefix_steps;
        statistics.recorded_attempts = writer.recorded_count();
        let finish = writer.finish(DiagnosticTerminal {
            outcome: terminal.recording(),
            detail: detail(&message),
            produced_attempts: statistics.produced_attempts,
            last_advanced_generation: statistics.last_advanced_generation,
        });
        drop(writer);
        match finish {
            Ok(()) => terminal_confirmed = true,
            Err(e) => {
                terminal = RunTerminal::Failed;
                message = format!(
                    "{message}; typed terminal unconfirmed: {e}; preserve cold committed prefix"
                );
            }
        }
    }
    statistics.run_wall_seconds = wall.elapsed().as_secs_f64();
    if let Some(mut set) = manifest {
        let observed = RunObservation {
            terminal,
            detail: detail(&message),
            statistics: statistics.clone(),
        };
        set.members[0].observed = Some(observed.clone());
        if terminal_confirmed && let Err(e) = set.pin_closed(&record.directory, 0, observed) {
            terminal = RunTerminal::Failed;
            message = format!("{message}; closed archive pin unavailable: {e}");
            set.members[0].finalized = None;
            set.members[0].observed = Some(RunObservation {
                terminal,
                detail: detail(&message),
                statistics: statistics.clone(),
            });
        }
        let path = record.directory.join("finished.json");
        match set.save_new(&path) {
            Ok(()) => {
                shared
                    .lock()
                    .map_err(|_| "Comparison lock")?
                    .diagnostic_manifest = Some(path)
            }
            Err(e) => {
                terminal = RunTerminal::Failed;
                message = format!(
                    "{message}; saved revision publication unconfirmed: {e}; preserve intent and archive"
                );
            }
        }
    }
    let state = match terminal {
        RunTerminal::Completed => "completed",
        RunTerminal::StoppedOnRejection => "stopped_on_rejection",
        RunTerminal::Cancelled => "cancelled",
        RunTerminal::Failed => "failed",
    };
    let advanced = shared
        .lock()
        .map_err(|_| "Comparison lock")?
        .frames
        .last()
        .map_or(0, |f| f.step);
    record.finish(state, &message, advanced)?;
    let mut view = shared.lock().map_err(|_| "Comparison lock")?;
    view.state = state.into();
    view.message = message;
    view.statistics = Some(statistics);
    Ok(())
}

#[cfg(test)]
#[path = "diagnostic_worker_tests.rs"]
mod tests;
