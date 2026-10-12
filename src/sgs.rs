//! Exact SGS triangular sweeps in a topological order of independent tiles.
//! Tiles at bx+by+bz=w read only their own preceding cells or tiles completed
//! at earlier waves (later waves for the reverse sweep). Joining each wave
//! establishes visibility; relaxed atomic bit loads/stores avoid aliasing a
//! mutable f64 array across workers. No floating-point arithmetic is atomic.
use crate::{Axis, PressureError, PressureOperator};
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) const MIN_VALUES: usize = 32 * 32 * 32;
const TILE_EDGE: usize = 8;
const _: () = assert!(std::mem::size_of::<AtomicU64>() == 8);

#[derive(Default)]
pub(crate) struct SweepPlan {
    workers: usize,
    edge: usize,
}

pub(crate) fn apply(
    operator: &PressureOperator<'_>,
    residual: &[f64],
    values: &[AtomicU64],
    plan: &mut SweepPlan,
) -> Result<(), PressureError> {
    let workers = rayon::current_num_threads();
    if workers == 1
        || operator
            .geometry()
            .counts()
            .iter()
            .filter(|&&n| n > 8)
            .count()
            < 2
    {
        return apply_tiled(operator, residual, values, TILE_EDGE);
    }
    if plan.workers != workers {
        // Replay identical scratch-only sweeps to compare actual wall time.
        // Each forward pass overwrites all predecessors in topological order;
        // neither the residual nor numerical result depends on prior scratch.
        // Alternating order reduces cache/order bias. Keep minima to reduce
        // unrelated desktop preemption; this is local tuning, not a benchmark.
        let mut times = [std::time::Duration::MAX; 2];
        for trial in [0, 1, 1, 0] {
            let start = std::time::Instant::now();
            apply_tiled(operator, residual, values, [8, 4][trial])?;
            times[trial] = times[trial].min(start.elapsed());
        }
        plan.edge = if times[1] < times[0] { 4 } else { 8 };
        plan.workers = workers;
    }
    apply_tiled(operator, residual, values, plan.edge)
}

fn apply_tiled(
    operator: &PressureOperator<'_>,
    residual: &[f64],
    values: &[AtomicU64],
    tile_edge: usize,
) -> Result<(), PressureError> {
    let counts = operator.geometry().counts();
    let tiles = counts.map(|n| n.div_ceil(tile_edge));
    let waves = tiles.iter().sum::<usize>() - 2;
    if operator.has_gauge() {
        values[0].store(0.0_f64.to_bits(), Ordering::Relaxed);
    }
    if rayon::current_num_threads() == 1 || tiles.iter().filter(|&&n| n > 1).count() < 2 {
        sweep_tile::<false>(operator, residual, values, [0; 3], counts)?;
        return sweep_tile::<true>(operator, residual, values, [0; 3], counts);
    }
    for reverse in [false, true] {
        let start = std::time::Instant::now();
        for phase in 0..waves {
            let wave = if reverse { waves - 1 - phase } else { phase };
            let jobs = wave_jobs(tiles, wave);
            (0..jobs)
                .into_par_iter()
                .with_max_len(1)
                .try_for_each(|job| {
                    let tile = wave_tile(tiles, wave, job);
                    let lower = tile.map(|v| v * tile_edge);
                    let upper = std::array::from_fn(|d| (lower[d] + tile_edge).min(counts[d]));
                    if reverse {
                        sweep_tile::<true>(operator, residual, values, lower, upper)
                    } else {
                        sweep_tile::<false>(operator, residual, values, lower, upper)
                    }
                })?;
        }
        crate::scheduling::record_sweep(
            if reverse {
                crate::observation::CalculationStage::SgsReverse
            } else {
                crate::observation::CalculationStage::SgsForward
            },
            residual.len(),
            tile_edge.pow(3),
            tiles.iter().product(),
            start.elapsed(),
        );
    }
    Ok(())
}

fn wave_rows(tiles: [usize; 3], wave: usize, bx: usize) -> std::ops::Range<usize> {
    if bx > wave {
        return 0..0;
    }
    let remaining = wave - bx;
    remaining.saturating_sub(tiles[2] - 1)..(remaining + 1).min(tiles[1])
}
fn wave_jobs(tiles: [usize; 3], wave: usize) -> usize {
    (0..tiles[0])
        .map(|bx| wave_rows(tiles, wave, bx).len())
        .sum()
}
fn wave_tile(tiles: [usize; 3], wave: usize, mut job: usize) -> [usize; 3] {
    for bx in 0..tiles[0] {
        let rows = wave_rows(tiles, wave, bx);
        if job < rows.len() {
            let by = rows.start + job;
            return [bx, by, wave - bx - by];
        }
        job -= rows.len();
    }
    unreachable!("job came from this wave's exact tile count")
}

fn sweep_tile<const REVERSE: bool>(
    operator: &PressureOperator<'_>,
    residual: &[f64],
    values: &[AtomicU64],
    lower: [usize; 3],
    upper: [usize; 3],
) -> Result<(), PressureError> {
    let counts = operator.geometry().counts();
    let [nx, ny, _] = counts;
    let strides = [1, nx, nx * ny];
    let weights = operator.fixed_box_weights();
    let probe = crate::observation::WorkProbe::new(if REVERSE {
        crate::observation::CalculationStage::SgsReverse
    } else {
        crate::observation::CalculationStage::SgsForward
    });
    for kz in lower[2]..upper[2] {
        let k = if REVERSE {
            upper[2] - 1 - (kz - lower[2])
        } else {
            kz
        };
        for jy in lower[1]..upper[1] {
            let j = if REVERSE {
                upper[1] - 1 - (jy - lower[1])
            } else {
                jy
            };
            for ix in lower[0]..upper[0] {
                let i = if REVERSE {
                    upper[0] - 1 - (ix - lower[0])
                } else {
                    ix
                };
                let p = [i, j, k];
                probe.visit(p);
                let row = i + nx * (j + ny * k);
                if row == 0 && operator.has_gauge() {
                    continue;
                }
                let diagonal = if let Some(weights) = weights {
                    // Retain the operator's lower/upper, X/Y/Z summation order.
                    let mut diagonal = 0.0;
                    for d in 0..3 {
                        if p[d] > 0 {
                            diagonal += weights[d];
                        }
                        if p[d] + 1 < counts[d] {
                            diagonal += weights[d];
                        }
                    }
                    diagonal
                } else {
                    operator.diagonal(p)
                };
                let mut correction = 0.0;
                let mut value = residual[row];
                for d in 0..3 {
                    if REVERSE {
                        if p[d] + 1 < counts[d]
                            && (weights.is_some()
                                || (operator.is_wet(row) && operator.is_wet(row + strides[d])))
                        {
                            let mut upper = p;
                            upper[d] += 1;
                            let weight = weights.map_or_else(
                                || operator.neighbor_weight(Axis::ALL[d], upper),
                                |w| w[d],
                            );
                            correction += weight
                                * f64::from_bits(values[row + strides[d]].load(Ordering::Relaxed));
                        }
                    } else if p[d] > 0
                        && (weights.is_some()
                            || (operator.is_wet(row) && operator.is_wet(row - strides[d])))
                    {
                        let weight = weights
                            .map_or_else(|| operator.neighbor_weight(Axis::ALL[d], p), |w| w[d]);
                        value += weight
                            * f64::from_bits(values[row - strides[d]].load(Ordering::Relaxed));
                    }
                }
                let result = if REVERSE {
                    f64::from_bits(values[row].load(Ordering::Relaxed)) + correction / diagonal
                } else {
                    value / diagonal
                };
                if !result.is_finite() || diagonal <= 0.0 || !diagonal.is_finite() {
                    return Err(PressureError::Breakdown { iteration: 0 });
                }
                values[row].store(result.to_bits(), Ordering::Relaxed);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod scheduling_tests {
    use super::*;
    #[test]
    fn waves_enumerate_only_ready_tiles_once() {
        for tiles in [[1, 1, 1], [5, 4, 3], [1, 7, 2], [8, 8, 8]] {
            let mut seen = vec![false; tiles.iter().product()];
            for wave in 0..tiles.iter().sum::<usize>() - 2 {
                for job in 0..wave_jobs(tiles, wave) {
                    let p = wave_tile(tiles, wave, job);
                    assert_eq!(p.iter().sum::<usize>(), wave);
                    assert!((0..3).all(|d| p[d] < tiles[d]));
                    let row = p[0] + tiles[0] * (p[1] + tiles[1] * p[2]);
                    assert!(!seen[row]);
                    seen[row] = true;
                }
            }
            assert!(seen.iter().all(|v| *v));
        }
    }
}
