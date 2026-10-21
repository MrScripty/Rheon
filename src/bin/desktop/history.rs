//! Bounded immutable preview history shared between a run and the desktop.
use rheon::{
    ParticleFluidSimulation, ParticleStepReport, PressureImplementation, Simulation, StepReport,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const HISTORY_BUDGET: usize = 96 * 1024 * 1024;
pub const MAX_PREVIEW_EDGE: usize = 10_000;
/// View configuration is independent of saved numerical geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "mode",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum PreviewResolution {
    #[default]
    Adaptive,
    CellsPerMetre(u32),
    Edge(usize),
}
impl PreviewResolution {
    pub fn edge(self, side_m: f64, fallback: usize) -> Result<usize, String> {
        let edge = match self {
            Self::Adaptive => return Ok(fallback),
            Self::CellsPerMetre(density) => edge_at_density(side_m, density)?,
            Self::Edge(edge) => {
                voxel_count(edge)?;
                edge
            }
        };
        // One representational ulp admits decimal SI extents such as 9 mm;
        // it does not admit another cell or alter a numerical acceptance gate.
        let actual = edge as f64 / side_m;
        if !side_m.is_finite()
            || side_m <= 0.0
            || actual > f64::from(rheon::MAX_CELLS_PER_METRE).next_up()
        {
            return Err(format!(
                "Exact preview {edge} cells across {side_m:.9} m resolves to {actual:.6} cells/m, above the 1000/m limit. Adjust extent/counts; preview is never silently reduced"
            ));
        }
        Ok(edge)
    }
    pub fn exact(self) -> bool {
        self != Self::Adaptive
    }
}
/// Convert physical density to a covering cubic preview. Rounding is explicit:
/// ceil(side * density), so the resolved density can exceed the request slightly.
pub fn edge_at_density(side_m: f64, cells_per_metre: u32) -> Result<usize, String> {
    if !side_m.is_finite()
        || side_m <= 0.0
        || !(1..=rheon::MAX_CELLS_PER_METRE).contains(&cells_per_metre)
    {
        return Err("Preview requires a positive domain and 1–1000 cells/m".into());
    }
    let edge = (side_m * f64::from(cells_per_metre)).ceil();
    if !edge.is_finite() || edge > MAX_PREVIEW_EDGE as f64 {
        return Err(
            "Requested preview exceeds 10000 cells per axis; reduce physical extent or density"
                .into(),
        );
    }
    Ok((edge as usize).max(1))
}
fn voxel_count(edge: usize) -> Result<usize, String> {
    if !(1..=MAX_PREVIEW_EDGE).contains(&edge) {
        return Err("Preview edge must be 1–10000".into());
    }
    edge.checked_pow(3)
        .ok_or_else(|| "Preview voxel count overflow".into())
}
#[derive(Debug)]
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
        Self::capture_state(
            sim.grid(),
            sim.state(),
            sim.implementation(),
            edge,
            step,
            seconds,
            report,
        )
    }
    /// Borrow accepted carrier fields from either existing numerical owner.
    pub fn capture_state(
        grid: &rheon::GridGeometry,
        state: rheon::StateView<'_>,
        method: PressureImplementation,
        edge: usize,
        step: usize,
        seconds: f64,
        report: Option<StepReport>,
    ) -> Result<Arc<Self>, String> {
        let count = voxel_count(edge)?;
        let mut density = Vec::new();
        density
            .try_reserve_exact(count)
            .map_err(|e| format!("Cannot allocate preview: {e}"))?;
        density.resize(count, 0);
        let [nx, ny, nz] = grid.counts();
        let tracer = state.tracer;
        let mut nonzero_tracer_cells = 0;
        // Display-only conservative max resampling. Each target voxel covers
        // at least one source cell, including finer/nondivisible preview grids.
        // This replicates appearance; it never creates numerical samples.
        for z in 0..edge {
            for y in 0..edge {
                for x in 0..edge {
                    let p = [x, y, z];
                    let counts = [nx, ny, nz];
                    let lo = std::array::from_fn::<_, 3, _>(|d| {
                        (p[d] as u128 * counts[d] as u128 / edge as u128) as usize
                    });
                    let hi = std::array::from_fn::<_, 3, _>(|d| {
                        (((p[d] + 1) as u128 * counts[d] as u128).div_ceil(edge as u128) as usize)
                            .min(counts[d])
                    });
                    let mut value = 0_u8;
                    for sz in lo[2]..hi[2] {
                        for sy in lo[1]..hi[1] {
                            for sx in lo[0]..hi[0] {
                                let cell = sx + nx * (sy + ny * sz);
                                value =
                                    value.max((tracer[cell].clamp(0.0, 1.0) * 255.0).round() as u8);
                            }
                        }
                    }
                    density[x + edge * (y + edge * z)] = value;
                }
            }
        }
        nonzero_tracer_cells += tracer.iter().filter(|&&v| v != 0.0).count();
        Ok(Arc::new(Self {
            method,
            step,
            time: state.time,
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
        let count = voxel_count(edge)?;
        let mut density = Vec::new();
        density
            .try_reserve_exact(count)
            .map_err(|e| format!("Cannot allocate preview: {e}"))?;
        density.resize(count, 0_u8);
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
    pub inspector: super::inspector::Inspector,
    pub carrier_inspectors: Vec<super::inspector::Inspector>,
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
    #[cfg(test)]
    pub fn plan(
        grid_edge: usize,
        steps: usize,
        tracks: usize,
        budget: usize,
    ) -> Result<PreviewPlan, String> {
        Self::plan_resolution(grid_edge, steps, tracks, budget, false)
    }
    pub fn plan_resolution(
        requested_edge: usize,
        steps: usize,
        tracks: usize,
        budget: usize,
        exact: bool,
    ) -> Result<PreviewPlan, String> {
        let frame_count = steps
            .checked_add(1)
            .and_then(|n| n.checked_mul(tracks))
            .ok_or("Preview budget overflow")?;
        if requested_edge == 0 || frame_count == 0 {
            return Err("Invalid preview settings".into());
        }
        if exact && requested_edge > MAX_PREVIEW_EDGE {
            return Err("Preview edge exceeds 10000".into());
        }
        let overhead = std::mem::size_of::<PreviewFrame>()
            + 2 * std::mem::size_of::<usize>()
            + std::mem::size_of::<Arc<PreviewFrame>>();
        // Preserve the previous adaptive default's 512-axis ceiling. Explicit
        // physical density requests have their own larger checked ceiling.
        let mut edge = requested_edge.min(if exact { MAX_PREVIEW_EDGE } else { 512 });
        loop {
            let bytes = voxel_count(edge)?
                .checked_add(overhead)
                .and_then(|n| frame_count.checked_mul(n));
            if let Some(bytes) = bytes.filter(|&n| n <= budget) {
                return Ok(PreviewPlan { edge, bytes });
            }
            if exact {
                return Err(format!(
                    "Exact preview {edge}³ for {frame_count} frames needs {} bytes; timeline budget {budget} bytes. Reduce preview density, horizon or tracks, or raise the timeline budget",
                    bytes.map_or_else(|| "more than addressable".into(), |n| n.to_string())
                ));
            }
            if edge == 1 {
                return Err("Requested timeline exceeds the preview budget".into());
            }
            edge -= 1;
        }
    }
    #[cfg(test)]
    pub fn with_budget(
        grid_edge: usize,
        steps: usize,
        tracks: usize,
        budget: usize,
    ) -> Result<Arc<Self>, String> {
        Self::with_resolution(grid_edge, steps, tracks, budget, false)
    }
    pub fn with_resolution(
        grid_edge: usize,
        steps: usize,
        tracks: usize,
        budget: usize,
        exact: bool,
    ) -> Result<Arc<Self>, String> {
        let plan = Self::plan_resolution(grid_edge, steps, tracks, budget, exact)?;
        let mut storage = Vec::new();
        for _ in 0..tracks {
            let mut track = Vec::new();
            track
                .try_reserve_exact(steps + 1)
                .map_err(|e| format!("Cannot allocate timeline: {e}"))?;
            storage.push(track);
        }
        Ok(Arc::new(Self {
            inspector: super::inspector::Inspector::default(),
            carrier_inspectors: (0..tracks)
                .map(|_| super::inspector::Inspector::default())
                .collect(),
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
    fn explicit_preview_counts_density_and_serialization_are_independent() {
        let a = PreviewResolution::CellsPerMetre(1000);
        assert_eq!(a.edge(0.016, 32).unwrap(), 16);
        assert!(
            a.edge(0.0171, 32)
                .unwrap_err()
                .contains("above the 1000/m limit")
        );
        assert_eq!(
            PreviewResolution::CellsPerMetre(500)
                .edge(0.0171, 32)
                .unwrap(),
            9
        );
        assert_eq!(a.edge(0.032, 32).unwrap(), 32);
        for count in 1..=64 {
            assert_eq!(a.edge(count as f64 / 1000.0, 32).unwrap(), count);
        }
        assert!(PreviewResolution::Edge(17).edge(0.016, 32).is_err());
        assert!(
            PreviewResolution::CellsPerMetre(1001)
                .edge(0.016, 32)
                .is_err()
        );
        for value in [
            PreviewResolution::Adaptive,
            a,
            PreviewResolution::Edge(10000),
        ] {
            let bytes = serde_json::to_vec(&value).unwrap();
            assert_eq!(
                serde_json::from_slice::<PreviewResolution>(&bytes).unwrap(),
                value
            );
        }
        assert_eq!(
            PreviewResolution::Edge(10000).edge(10.0, 32).unwrap(),
            10000
        );
        assert!(PreviewResolution::Edge(10001).edge(0.016, 32).is_err());
        assert!(
            LiveHistory::plan_resolution(10000, 1, 1, HISTORY_BUDGET, true)
                .unwrap_err()
                .contains("Exact preview 10000³")
        );
        assert!(LiveHistory::plan_resolution(16, usize::MAX, 1, HISTORY_BUDGET, true).is_err());
        let exact = LiveHistory::plan_resolution(16, 2, 2, HISTORY_BUDGET, true).unwrap();
        assert_eq!(exact.edge, 16);
        assert!(LiveHistory::plan_resolution(16, 2, 2, exact.bytes - 1, true).is_err());
        assert_eq!(
            LiveHistory::plan_resolution(16, 2, 2, exact.bytes, true)
                .unwrap()
                .bytes,
            exact.bytes
        );
        assert!(LiveHistory::plan_resolution(usize::MAX, 1, 1, HISTORY_BUDGET, true).is_err());
    }
    #[test]
    fn finer_nondivisible_preview_has_no_holes_and_does_not_change_numerical_state() {
        let grid = rheon::GridGeometry::at_cell_density([8; 3], 1000, [0.0; 3]).unwrap();
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
        sim.step(
            0.001,
            Some(rheon::SmokeSource {
                lower: [0.0; 3],
                upper: [0.008; 3],
                tracer_rate: 2.0,
                vertical_acceleration: 0.0,
            }),
            |_| false,
        )
        .unwrap();
        let before = sim.state().tracer.to_vec();
        for edge in [5, 8, 11] {
            let frame = PreviewFrame::capture(&sim, edge, 1, 0.0, None).unwrap();
            assert_eq!(frame.density.len(), edge.pow(3));
            assert!(
                frame
                    .density
                    .iter()
                    .all(|&v| v == frame.density[0] && v > 0)
            );
            assert_eq!(frame.nonzero_tracer_cells, 512);
            assert_eq!(sim.state().tracer, before);
        }
        assert!(PreviewFrame::capture(&sim, usize::MAX, 1, 0.0, None).is_err());
    }
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
