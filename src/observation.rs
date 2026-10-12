//! Optional per-worker diagnostics. Every marker comes from a real kernel
//! visit; consumers sample independently. No simulation state is shared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CalculationStage {
    FieldUpdate = 1,
    VelocityAdvection,
    TracerAdvection,
    PressureOperator,
    Jacobi,
    SgsForward,
    SgsReverse,
    Emitter,
    Reduction,
    ParticleDensity,
    ParticleForces,
}

#[cfg(feature = "work-observation")]
mod enabled {
    use super::CalculationStage;
    use crate::GridGeometry;
    use std::{
        cell::RefCell,
        fmt,
        sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        },
    };
    const ACTIVE: u64 = 1 << 63;
    const INDEX_MASK: u64 = (1 << 56) - 1;
    #[repr(align(64))]
    struct Slot(AtomicU64);
    thread_local! { static CURRENT: RefCell<Option<Arc<CalculationObserver>>> = const { RefCell::new(None) }; }

    #[derive(Debug)]
    pub enum ObservationError {
        InvalidSize,
        AllocationFailed,
    }
    impl fmt::Display for ObservationError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "calculation observer: {self:?}")
        }
    }
    impl std::error::Error for ObservationError {}
    #[derive(Debug, Clone, Copy)]
    pub struct WorkerVoxel {
        pub worker: usize,
        pub coordinate: [usize; 3],
        pub active: bool,
        pub stage: CalculationStage,
    }
    pub struct CalculationObserver {
        counts: [usize; 3],
        cells: usize,
        slots: Vec<Slot>,
    }
    impl CalculationObserver {
        /// One cache-aligned marker per worker; at most 4096 workers. Payload
        /// and thread-local bindings are outside numerical simulation budgets.
        pub fn new(grid: &GridGeometry, threads: usize) -> Result<Arc<Self>, ObservationError> {
            if threads == 0 || threads > 4096 || grid.cell_len() as u64 >= INDEX_MASK {
                return Err(ObservationError::InvalidSize);
            }
            let mut slots = Vec::new();
            slots
                .try_reserve_exact(threads)
                .map_err(|_| ObservationError::AllocationFailed)?;
            slots.resize_with(threads, || Slot(AtomicU64::new(0)));
            Ok(Arc::new(Self {
                counts: grid.counts(),
                cells: grid.cell_len(),
                slots,
            }))
        }
        /// Bind on every scoped Rayon thread around `thread.run()`. The binding
        /// restores its predecessor even on unwind; it never changes solver data.
        pub fn observe<T>(self: &Arc<Self>, operation: impl FnOnce() -> T) -> T {
            struct Restore(Option<Arc<CalculationObserver>>);
            impl Drop for Restore {
                fn drop(&mut self) {
                    CURRENT.with(|c| *c.borrow_mut() = self.0.take());
                }
            }
            let _restore = Restore(CURRENT.with(|c| c.replace(Some(Arc::clone(self)))));
            operation()
        }
        pub fn snapshot(&self) -> Vec<WorkerVoxel> {
            self.slots
                .iter()
                .enumerate()
                .filter_map(|(worker, slot)| {
                    let word = slot.0.load(Ordering::Relaxed);
                    let encoded = word & INDEX_MASK;
                    if encoded == 0 {
                        return None;
                    }
                    let row = (encoded - 1) as usize;
                    let stage = match (word >> 56) & 0x7f {
                        1 => CalculationStage::FieldUpdate,
                        2 => CalculationStage::VelocityAdvection,
                        3 => CalculationStage::TracerAdvection,
                        4 => CalculationStage::PressureOperator,
                        5 => CalculationStage::Jacobi,
                        6 => CalculationStage::SgsForward,
                        7 => CalculationStage::SgsReverse,
                        8 => CalculationStage::Emitter,
                        9 => CalculationStage::Reduction,
                        10 => CalculationStage::ParticleDensity,
                        11 => CalculationStage::ParticleForces,
                        _ => return None,
                    };
                    let [nx, ny, _] = self.counts;
                    Some(WorkerVoxel {
                        worker,
                        coordinate: [row % nx, (row / nx) % ny, row / (nx * ny)],
                        active: word & ACTIVE != 0,
                        stage,
                    })
                })
                .collect()
        }
    }
    pub(crate) struct Probe {
        observer: Option<Arc<CalculationObserver>>,
        worker: usize,
        stage: CalculationStage,
    }
    impl Probe {
        pub(crate) fn new(stage: CalculationStage) -> Self {
            Self {
                observer: CURRENT.with(|c| c.borrow().clone()),
                worker: rayon::current_thread_index().unwrap_or(0),
                stage,
            }
        }
        #[inline]
        pub(crate) fn visit_index(&self, row: usize) {
            if let Some(observer) = &self.observer
                && row < observer.cells
                && let Some(slot) = observer.slots.get(self.worker)
            {
                slot.0.store(
                    ACTIVE | ((self.stage as u64) << 56) | (row as u64 + 1),
                    Ordering::Relaxed,
                );
            }
        }
        #[inline]
        pub(crate) fn visit(&self, p: [usize; 3]) {
            if let Some(observer) = &self.observer {
                let p: [usize; 3] = std::array::from_fn(|d| p[d].min(observer.counts[d] - 1));
                self.visit_index(p[0] + observer.counts[0] * (p[1] + observer.counts[1] * p[2]));
            }
        }
    }
    impl Drop for Probe {
        fn drop(&mut self) {
            if let Some(observer) = &self.observer
                && let Some(slot) = observer.slots.get(self.worker)
            {
                slot.0.fetch_and(!ACTIVE, Ordering::Relaxed);
            }
        }
    }
}
#[cfg(feature = "work-observation")]
pub(crate) use enabled::Probe as WorkProbe;
#[cfg(feature = "work-observation")]
pub use enabled::{CalculationObserver, ObservationError, WorkerVoxel};
#[cfg(not(feature = "work-observation"))]
pub(crate) struct WorkProbe;
#[cfg(not(feature = "work-observation"))]
impl WorkProbe {
    #[inline]
    pub(crate) fn new(_: CalculationStage) -> Self {
        Self
    }
    #[inline]
    pub(crate) fn visit_index(&self, _: usize) {}
    #[inline]
    pub(crate) fn visit(&self, _: [usize; 3]) {}
}
