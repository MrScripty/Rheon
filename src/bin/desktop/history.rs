//! Bounded immutable preview history shared between a run and the desktop.
use rheon::{
    ParticleFluidSimulation, ParticleStepReport, PressureImplementation, Simulation, StepReport,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const HISTORY_BUDGET: usize = 96 * 1024 * 1024;
pub const MAX_PREVIEW_EDGE: usize = 512;
pub struct PreviewPlan {
    pub edge: usize,
    pub bytes: usize,
}
pub struct PreviewFrame {
    pub method: PressureImplementation,
    pub step: usize,
    pub time: f64,
    pub seconds: f64,
    pub edge: usize,
    pub nonzero_tracer_cells: usize,
    pub density: Vec<u8>,
    pub report: Option<StepReport>,
    pub particle_report: Option<ParticleStepReport>,
    pub material_color: Option<[u8; 3]>,
}
impl PreviewFrame {
    pub fn capture(
        sim: &Simulation,
        edge: usize,
        step: usize,
        seconds: f64,
        report: Option<StepReport>,
    ) -> Result<Arc<Self>, String> {
        let count = edge * edge * edge;
        let mut density = Vec::new();
        density
            .try_reserve_exact(count)
            .map_err(|e| format!("Cannot allocate preview: {e}"))?;
        density.resize(count, 0);
        let [nx, ny, nz] = sim.grid().counts();
        let tracer = sim.state().tracer;
        let mut nonzero_tracer_cells = 0;
        // Maximum concentration over each preview cell preserves narrow jets;
        // simulation values remain untouched, including at non-divisible edges.
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let cell = x + nx * (y + ny * z);
                    nonzero_tracer_cells += usize::from(tracer[cell] != 0.0);
                    let preview =
                        (x * edge / nx) + edge * ((y * edge / ny) + edge * (z * edge / nz));
                    let value = (tracer[cell].clamp(0.0, 1.0) * 255.0).round() as u8;
                    density[preview] = density[preview].max(value);
                }
            }
        }
        Ok(Arc::new(Self {
            method: sim.implementation(),
            step,
            time: sim.state().time,
            seconds,
            edge,
            nonzero_tracer_cells,
            density,
            report,
            particle_report: None,
            material_color: None,
        }))
    }
    pub fn capture_particles(
        sim: &ParticleFluidSimulation,
        method: PressureImplementation,
        edge: usize,
        step: usize,
        seconds: f64,
        report: Option<ParticleStepReport>,
        color: [u8; 3],
    ) -> Result<Arc<Self>, String> {
        let mut density = Vec::new();
        density
            .try_reserve_exact(edge.pow(3))
            .map_err(|e| format!("Cannot allocate preview: {e}"))?;
        density.resize(edge.pow(3), 0_u8);
        let grid = sim.grid();
        let extent = std::array::from_fn::<_, 3, _>(|d| grid.upper()[d] - grid.origin()[d]);
        // Only the emitted material is highlighted. The initial surrounding
        // material still participates in every physical neighbor evaluation.
        for p in sim.particles().iter().filter(|p| p.material == 0) {
            let c = std::array::from_fn::<_, 3, _>(|d| {
                (p.position[d] - grid.origin()[d]) / extent[d] * edge as f64
            });
            let radius = std::array::from_fn::<_, 3, _>(|d| {
                // sqrt(3)/2 is the furthest nearest voxel center; retain a
                // visible sample even when a parcel lands on a voxel corner.
                (0.65 * sim.particle_spacing() / extent[d] * edge as f64).max(0.9)
            });
            let lo = std::array::from_fn::<_, 3, _>(|d| {
                ((c[d] - radius[d]).floor() as usize).min(edge - 1)
            });
            let hi = std::array::from_fn::<_, 3, _>(|d| {
                ((c[d] + radius[d]).ceil() as usize).min(edge - 1)
            });
            for z in lo[2]..=hi[2] {
                for y in lo[1]..=hi[1] {
                    for x in lo[0]..=hi[0] {
                        let distance = (0..3)
                            .map(|d| (([x, y, z][d] as f64 + 0.5 - c[d]) / radius[d]).powi(2))
                            .sum::<f64>();
                        if distance < 1.0 {
                            let value = ((1.0 - distance) * 255.0).round() as u8;
                            let i = x + edge * (y + edge * z);
                            density[i] = density[i].max(value);
                        }
                    }
                }
            }
        }
        let occupied = density.iter().filter(|v| **v != 0).count();
        Ok(Arc::new(Self {
            method,
            step,
            time: sim.time(),
            seconds,
            edge,
            nonzero_tracer_cells: occupied,
            density,
            report: None,
            particle_report: report,
            material_color: Some(color),
        }))
    }
}
pub struct LiveHistory {
    tracks: Mutex<Vec<Vec<Arc<PreviewFrame>>>>,
    pub active_track: AtomicUsize,
    pub preview_edge: usize,
    pub planned_bytes: usize,
    max_frames: usize,
    timing: Mutex<Option<(usize, usize, Instant)>>,
}
impl LiveHistory {
    #[cfg(test)]
    pub fn new(grid_edge: usize, steps: usize, tracks: usize) -> Result<Arc<Self>, String> {
        Self::with_budget(grid_edge, steps, tracks, HISTORY_BUDGET)
    }
    pub fn plan(
        grid_edge: usize,
        steps: usize,
        tracks: usize,
        budget: usize,
    ) -> Result<PreviewPlan, String> {
        let frame_count = steps
            .checked_add(1)
            .and_then(|n| n.checked_mul(tracks))
            .ok_or("Preview budget overflow")?;
        if grid_edge == 0 || frame_count == 0 {
            return Err("Invalid preview settings".into());
        }
        let overhead = std::mem::size_of::<PreviewFrame>()
            + 2 * std::mem::size_of::<usize>()
            + std::mem::size_of::<Arc<PreviewFrame>>();
        let mut edge = grid_edge.min(MAX_PREVIEW_EDGE);
        while frame_count
            .checked_mul(edge.pow(3) + overhead)
            .is_none_or(|n| n > budget)
        {
            if edge == 1 {
                return Err("Requested timeline exceeds the preview budget".into());
            }
            edge -= 1;
        }
        Ok(PreviewPlan {
            edge,
            bytes: frame_count * (edge.pow(3) + overhead),
        })
    }
    pub fn with_budget(
        grid_edge: usize,
        steps: usize,
        tracks: usize,
        budget: usize,
    ) -> Result<Arc<Self>, String> {
        let plan = Self::plan(grid_edge, steps, tracks, budget)?;
        let mut storage = Vec::new();
        for _ in 0..tracks {
            let mut track = Vec::new();
            track
                .try_reserve_exact(steps + 1)
                .map_err(|e| format!("Cannot allocate timeline: {e}"))?;
            storage.push(track);
        }
        Ok(Arc::new(Self {
            tracks: Mutex::new(storage),
            active_track: AtomicUsize::new(0),
            preview_edge: plan.edge,
            planned_bytes: plan.bytes,
            max_frames: steps + 1,
            timing: Mutex::new(None),
        }))
    }
    pub fn publish(&self, track: usize, frame: Arc<PreviewFrame>) -> Result<(), String> {
        let mut tracks = self.tracks.lock().map_err(|_| "Timeline owner failed")?;
        let frames = tracks.get_mut(track).ok_or("Unknown timeline track")?;
        if frame.edge != self.preview_edge
            || frame.step != frames.len()
            || frames.len() >= self.max_frames
        {
            return Err("Preview sequence or budget mismatch".into());
        }
        frames.push(frame);
        self.active_track.store(track, Ordering::Relaxed);
        Ok(())
    }
    pub fn begin_step(&self, track: usize, step: usize) -> Result<(), String> {
        *self.timing.lock().map_err(|_| "Step timing owner failed")? =
            Some((track, step, Instant::now()));
        Ok(())
    }
    pub fn end_step(&self) -> Result<(), String> {
        *self.timing.lock().map_err(|_| "Step timing owner failed")? = None;
        Ok(())
    }
    pub fn timing(&self) -> Option<(usize, usize, f64)> {
        self.timing.lock().ok().and_then(|t| {
            t.map(|(track, step, start)| (track, step, start.elapsed().as_secs_f64()))
        })
    }
    pub fn view(
        &self,
        track: usize,
        selected: usize,
        follow: bool,
    ) -> Result<(usize, Option<Arc<PreviewFrame>>), String> {
        let tracks = self.tracks.lock().map_err(|_| "Timeline owner failed")?;
        let frames = tracks.get(track).ok_or("Unknown timeline track")?;
        let index = if follow {
            frames.len().saturating_sub(1)
        } else {
            selected.min(frames.len().saturating_sub(1))
        };
        Ok((frames.len(), frames.get(index).cloned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn previews_above_128_are_real_and_long_timelines_obey_the_selected_budget() {
        let short = LiveHistory::new(256, 1, 1).unwrap();
        assert_eq!(short.preview_edge, 256);
        assert!(short.planned_bytes <= HISTORY_BUDGET);
        let small = LiveHistory::plan(256, 1024, 1, HISTORY_BUDGET).unwrap();
        let large = LiveHistory::plan(256, 1024, 1, 3072 * 1024 * 1024).unwrap();
        assert!(small.edge < 128);
        assert!(large.edge >= 128 && large.edge > small.edge);
        assert!(large.bytes <= 3072 * 1024 * 1024);
        assert!(LiveHistory::plan(512, usize::MAX, 1, HISTORY_BUDGET).is_err());
    }
    #[test]
    fn occupancy_counts_numerical_cells_even_below_the_preview_threshold() {
        let grid = rheon::GridGeometry::new([8; 3], [0.125; 3], [0.0; 3]).unwrap();
        let mut sim = rheon::Simulation::new(
            grid,
            rheon::SimulationConfig {
                density: 1.0,
                memory_limit: 1024 * 1024,
                pressure: rheon::PressureSettings {
                    relative_residual: 1e-9,
                    absolute_residual: 1e-12,
                    divergence_limit: 1e-7,
                    max_iterations: 2000,
                },
                actual_divergence_limit: 1e-5,
                max_courant: 1.0,
            },
        )
        .unwrap();
        let report = sim
            .step(
                0.02,
                Some(rheon::SmokeSource {
                    lower: [0.25; 3],
                    upper: [0.75; 3],
                    tracer_rate: 1e-20,
                    vertical_acceleration: 0.0,
                }),
                |_| false,
            )
            .unwrap();
        let frame = PreviewFrame::capture(&sim, 4, 1, 0.01, Some(report)).unwrap();
        assert!(frame.density.iter().all(|v| *v == 0));
        assert_eq!(frame.nonzero_tracer_cells, 64);
    }
    #[test]
    fn long_comparisons_keep_every_step_with_a_bounded_preview_resolution() {
        let h = LiveHistory::new(128, 10000, 2).unwrap();
        assert!(h.preview_edge < 32);
        assert!(h.planned_bytes <= HISTORY_BUDGET);
    }
    #[test]
    fn scrubbing_does_not_follow_new_frames_and_rejects_out_of_order_publication() {
        let h = LiveHistory::new(4, 4, 1).unwrap();
        let make = |step| {
            Arc::new(PreviewFrame {
                method: PressureImplementation::default(),
                step,
                time: step as f64 * 0.02,
                seconds: 0.01,
                edge: 4,
                nonzero_tracer_cells: if step == 0 { 0 } else { 64 },
                density: vec![step as u8; 64],
                report: None,
                particle_report: None,
                material_color: None,
            })
        };
        h.publish(0, make(0)).unwrap();
        h.publish(0, make(1)).unwrap();
        let (_, selected) = h.view(0, 0, false).unwrap();
        h.publish(0, make(2)).unwrap();
        assert_eq!(selected.unwrap().step, 0);
        assert_eq!(h.view(0, 0, true).unwrap().1.unwrap().step, 2);
        assert!(h.publish(0, make(4)).is_err());
        assert_eq!(h.view(0, 1, false).unwrap().1.unwrap().step, 1);
    }
}
