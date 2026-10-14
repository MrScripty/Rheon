//! Run-owned batch sizing over Rayon's work-stealing scheduler. Timings change
//! task boundaries only: arithmetic and ordered reductions remain unchanged.
use crate::observation::CalculationStage;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
    time::Instant,
};

const STAGES: usize = 11;
const MIN_BATCH: usize = 64;
const MAX_BATCH: usize = 16 * 1024;
const RETUNE_CALLS: usize = 128;
const TRIAL_CALLS: usize = 8;

thread_local! { static CURRENT: RefCell<Option<Arc<CpuScheduler>>> = const { RefCell::new(None) }; }

/// Last completed batch plan and smoothed wall cost for one numerical kernel.
/// Tasks count the complete pass; SGS tasks become ready in dependent waves.
/// Particle stages count material particles rather than domain voxels.
#[derive(Clone, Copy, Debug)]
pub struct BatchStatistics {
    pub stage: CalculationStage,
    pub batch_cells: usize,
    pub tasks: usize,
    pub calls: usize,
    pub wall_nanoseconds_per_cell: f64,
    pub total_wall_nanoseconds: u64,
}
struct Cost {
    sample: Option<BatchStatistics>,
    shape: (usize, usize),
    shape_calls: usize,
    scores: [u64; 4],
    best: usize,
}
/// Optional run-owned feedback for CPU task sizing; no numerical buffers.
/// Bind this on every thread of a scoped Rayon pool using `scope` around
/// `thread.run()`. Dropping the scope restores its previous binding, even on
/// unwind. Unbound consumers use hardware/workload-based sizing without timing.
pub struct CpuScheduler {
    costs: [Mutex<Cost>; STAGES],
}
impl CpuScheduler {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            costs: std::array::from_fn(|_| {
                Mutex::new(Cost {
                    sample: None,
                    shape: (0, 0),
                    shape_calls: 0,
                    scores: [u64::MAX; 4],
                    best: 0,
                })
            }),
        })
    }
    pub fn scope<T>(self: &Arc<Self>, operation: impl FnOnce() -> T) -> T {
        struct Restore(Option<Arc<CpuScheduler>>);
        impl Drop for Restore {
            fn drop(&mut self) {
                CURRENT.with(|c| *c.borrow_mut() = self.0.take());
            }
        }
        let _restore = Restore(CURRENT.with(|c| c.replace(Some(Arc::clone(self)))));
        operation()
    }
    /// An independently sampled diagnostic, never a CPU-utilization estimate.
    pub fn snapshot(&self) -> Vec<BatchStatistics> {
        self.costs
            .iter()
            .filter_map(|c| c.lock().expect("scheduler cost owner").sample)
            .collect()
    }
}

// SGS owns tile selection. This records that owner's actual completed sweep
// for the same UI diagnostics without applying an independent linear plan.
pub(crate) fn record_sweep(
    stage: CalculationStage,
    cells: usize,
    batch: usize,
    tasks: usize,
    duration: std::time::Duration,
) {
    CURRENT.with(|current| {
        if let Some(owner) = current.borrow().as_ref() {
            let mut cost = owner.costs[stage as usize - 1]
                .lock()
                .expect("scheduler cost owner");
            let previous = cost.sample;
            let ns = duration.as_nanos() as f64 / cells as f64;
            cost.sample = Some(BatchStatistics {
                stage,
                batch_cells: batch,
                tasks,
                calls: previous.map_or(1, |p| p.calls.saturating_add(1)),
                wall_nanoseconds_per_cell: previous
                    .map_or(ns, |p| 0.75 * p.wall_nanoseconds_per_cell + 0.25 * ns),
                total_wall_nanoseconds: previous
                    .map_or(0, |p| p.total_wall_nanoseconds)
                    .saturating_add(duration.as_nanos().min(u128::from(u64::MAX)) as u64),
            });
        }
    });
}
pub(crate) struct SerialTimer {
    stage: CalculationStage,
    cells: usize,
    start: Option<Instant>,
}
impl SerialTimer {
    pub(crate) fn new(stage: CalculationStage, cells: usize) -> Self {
        Self {
            stage,
            cells,
            start: CURRENT.with(|c| c.borrow().is_some()).then(Instant::now),
        }
    }
}
impl Drop for SerialTimer {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            record_sweep(self.stage, self.cells, self.cells, 1, start.elapsed());
        }
    }
}
pub(crate) struct BatchRun {
    owner: Option<Arc<CpuScheduler>>,
    stage: CalculationStage,
    pub(crate) batch: usize,
    cells: usize,
    workers: usize,
    trial: Option<usize>,
    start: Instant,
}
impl BatchRun {
    pub(crate) fn new(stage: CalculationStage, cells: usize, preferred: usize) -> Self {
        let owner = CURRENT.with(|c| c.borrow().clone());
        let workers = rayon::current_num_threads();
        let candidates = batch_candidates(cells, workers, preferred);
        let (batch, trial) = owner.as_ref().map_or((candidates[0], None), |o| {
            let mut cost = o.costs[stage as usize - 1]
                .lock()
                .expect("scheduler cost owner");
            if cost.shape != (cells, workers) {
                cost.shape = (cells, workers);
                cost.shape_calls = 0;
                cost.best = 0;
            }
            let phase = cost.shape_calls % RETUNE_CALLS;
            if phase == 0 {
                cost.scores = [u64::MAX; 4];
            }
            let trial = (phase < TRIAL_CALLS).then_some(phase % candidates.len());
            (candidates[trial.unwrap_or(cost.best)], trial)
        });
        Self {
            owner,
            stage,
            batch,
            cells,
            workers,
            trial,
            start: Instant::now(),
        }
    }
    pub(crate) fn complete(&self) {
        let wall = self.start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        self.record(wall);
    }
    fn record(&self, wall: u64) {
        if self.cells == 0 {
            return;
        }
        if let Some(owner) = &self.owner {
            let cost = wall as f64 / self.cells as f64;
            let mut entry = owner.costs[self.stage as usize - 1]
                .lock()
                .expect("scheduler cost owner");
            let previous = entry.sample;
            if entry.shape == (self.cells, self.workers) {
                if let Some(trial) = self.trial {
                    entry.scores[trial] = entry.scores[trial].min(wall);
                    entry.best = entry
                        .scores
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, time)| **time)
                        .map_or(0, |(i, _)| i);
                }
                entry.shape_calls = entry.shape_calls.wrapping_add(1);
            }
            entry.sample = Some(BatchStatistics {
                stage: self.stage,
                batch_cells: self.batch,
                tasks: self.cells.div_ceil(self.batch),
                calls: previous.map_or(1, |p| p.calls.saturating_add(1)),
                wall_nanoseconds_per_cell: previous
                    .map_or(cost, |p| 0.75 * p.wall_nanoseconds_per_cell + 0.25 * cost),
                total_wall_nanoseconds: previous
                    .map_or(0, |p| p.total_wall_nanoseconds)
                    .saturating_add(wall),
            });
        }
    }
}
// Compare workload-preferred batches with one, two and four jobs per worker.
// Even the largest candidate exposes a full pool, except at the minimum grain.
// Feedback uses joined kernel wall time, including scheduling overhead, rather
// than assuming that smaller chunks or more busy workers mean faster execution.
fn batch_candidates(cells: usize, workers: usize, preferred: usize) -> [usize; 4] {
    let per_worker = cells.div_ceil(workers.max(1));
    [
        preferred.min(per_worker),
        per_worker,
        per_worker.div_ceil(2),
        per_worker.div_ceil(4),
    ]
    .map(|n| n.clamp(MIN_BATCH, MAX_BATCH))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plans_scale_to_workers_and_measured_cost_without_unbounded_tasks() {
        assert!(
            batch_candidates(16 * 1024, 24, 16 * 1024)
                .iter()
                .all(|&n| n < 1024)
        );
        assert!(
            batch_candidates(128_usize.pow(3), 8, 2048)[1]
                > batch_candidates(128_usize.pow(3), 8, 128)[0]
        );
        assert_eq!(batch_candidates(1, 1, 0), [MIN_BATCH; 4]);
        assert_eq!(batch_candidates(usize::MAX, 1, usize::MAX), [MAX_BATCH; 4]);
    }
    #[test]
    fn feedback_keeps_the_fastest_trial_and_restarts_when_workload_size_changes() {
        let scheduler = CpuScheduler::new();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(8)
            .build()
            .unwrap();
        pool.install(|| {
            scheduler.scope(|| {
                for time in [400, 300, 100, 200, 390, 290, 110, 210] {
                    BatchRun::new(CalculationStage::PressureOperator, 32768, 1024).record(time);
                }
                let selected = BatchRun::new(CalculationStage::PressureOperator, 32768, 1024);
                assert_eq!(selected.batch, 2048);
                assert!(selected.trial.is_none());
                let changed = BatchRun::new(CalculationStage::PressureOperator, 65536, 1024);
                assert_eq!(changed.trial, Some(0));
                assert_eq!(changed.batch, 1024);
            })
        });
    }

    #[test]
    fn successful_nonoverlapping_calls_explore_twice_then_retune_at_128() {
        let scheduler = CpuScheduler::new();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        pool.install(|| {
            scheduler.scope(|| {
                let candidates = batch_candidates(65536, 4, 1024);
                let mut visits = [0; 4];
                for call in 0..256 {
                    let run = BatchRun::new(CalculationStage::PressureOperator, 65536, 1024);
                    let phase = call % RETUNE_CALLS;
                    if phase < TRIAL_CALLS {
                        let candidate = phase % 4;
                        assert_eq!(run.trial, Some(candidate));
                        assert_eq!(run.batch, candidates[candidate]);
                        visits[candidate] += 1;
                    } else {
                        assert!(run.trial.is_none());
                        assert_eq!(run.batch, candidates[2]);
                    }
                    // Synthetic costs test policy only; no timing/performance claim.
                    run.record([300, 200, 100, 400][phase % 4]);
                }
                assert_eq!(visits, [4; 4]);
                assert_eq!(scheduler.snapshot()[0].calls, 256);
                assert_eq!(
                    BatchRun::new(CalculationStage::PressureOperator, 65536, 1024).trial,
                    Some(0)
                );
            })
        });
    }

    #[test]
    fn failed_parallel_pass_does_not_train_and_retry_covers_each_index_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let scheduler = CpuScheduler::new();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        pool.install(|| {
            scheduler.scope(|| {
                let mut values = vec![usize::MAX; 17011];
                let failed = crate::parallel::try_for_each_chunk_mut(
                    &mut values,
                    CalculationStage::PressureOperator,
                    1024,
                    |_, chunk| {
                        chunk[0] = 99;
                        Err::<(), _>("deliberate partial failure")
                    },
                );
                assert!(failed.is_err());
                assert!(scheduler.snapshot().is_empty());
                for _ in 0..3 {
                    let abandoned =
                        BatchRun::new(CalculationStage::PressureOperator, values.len(), 1024);
                    assert_eq!(abandoned.trial, Some(0));
                }
                let visits: Vec<_> = (0..values.len()).map(|_| AtomicUsize::new(0)).collect();
                crate::parallel::try_for_each_chunk_mut(
                    &mut values,
                    CalculationStage::PressureOperator,
                    1024,
                    |start, chunk| {
                        for (offset, value) in chunk.iter_mut().enumerate() {
                            let index = start + offset;
                            visits[index].fetch_add(1, Ordering::Relaxed);
                            *value = index * 17 + 3;
                        }
                        Ok::<(), ()>(())
                    },
                )
                .unwrap();
                assert_eq!(values, (0..17011).map(|i| i * 17 + 3).collect::<Vec<_>>());
                assert!(visits.iter().all(|v| v.load(Ordering::Relaxed) == 1));
                let sample = scheduler.snapshot()[0];
                assert_eq!(sample.calls, 1);
                assert_eq!(sample.tasks, values.len().div_ceil(sample.batch_cells));
                assert_eq!(
                    BatchRun::new(CalculationStage::PressureOperator, values.len(), 1024).trial,
                    Some(1)
                );
            })
        });
    }

    #[test]
    fn nested_scope_restores_owner_after_unwind_and_releases_bindings() {
        let outer = CpuScheduler::new();
        let inner = CpuScheduler::new();
        outer.scope(|| {
            BatchRun::new(CalculationStage::PressureOperator, 32768, 1024).record(100);
            let failed = std::panic::catch_unwind(|| {
                inner.scope(|| {
                    BatchRun::new(CalculationStage::PressureOperator, 32768, 1024).record(200);
                    panic!("deliberate nested scope unwind");
                })
            });
            assert!(failed.is_err());
            BatchRun::new(CalculationStage::PressureOperator, 32768, 1024).record(300);
        });
        assert_eq!(outer.snapshot()[0].calls, 2);
        assert_eq!(inner.snapshot()[0].calls, 1);
        assert_eq!(Arc::strong_count(&outer), 1);
        assert_eq!(Arc::strong_count(&inner), 1);
        assert!(CURRENT.with(|c| c.borrow().is_none()));
    }

    #[test]
    fn exploration_assumption_excludes_overlapping_same_stage_invocations() {
        // Deliberate counterexample: this owner is not an invocation queue.
        // Two outstanding calls can observe the same phase. A fairness claim
        // without the nonoverlap premise would therefore be false.
        let scheduler = CpuScheduler::new();
        scheduler.scope(|| {
            let first = BatchRun::new(CalculationStage::PressureOperator, 32768, 1024);
            let second = BatchRun::new(CalculationStage::PressureOperator, 32768, 1024);
            assert_eq!((first.trial, second.trial), (Some(0), Some(0)));
            first.record(100);
            second.record(120);
            assert_eq!(
                BatchRun::new(CalculationStage::PressureOperator, 32768, 1024).trial,
                Some(2)
            );
        });
    }
}
