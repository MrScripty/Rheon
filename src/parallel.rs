//! Disjoint scratch writes on the caller's Rayon pool when `parallel` is enabled.
//! Small grids stay sequential. Reductions and cancellation callbacks retain
//! their original order on the calling thread.
use crate::observation::CalculationStage;
#[cfg(feature = "parallel")]
use rayon::prelude::*;

// A 16³ grid is cheaper to process without scheduler handoffs. Chunking limits
// handoffs on large grids while leaving enough tasks for desktop CPU pools.
#[cfg(feature = "parallel")]
const MIN_PARALLEL_VALUES: usize = 16 * 1024;
const CHUNK_VALUES: usize = 2048;

// Traverse arbitrary flat chunks as contiguous X-row fragments. Divide once
// per fragment rather than once per voxel, without restricting batch sizes.
pub(crate) fn for_each_row_mut<T, E>(
    mut values: &mut [T],
    mut start: usize,
    dimensions: [usize; 3],
    mut operation: impl FnMut(usize, [usize; 3], &mut [T]) -> Result<(), E>,
) -> Result<(), E> {
    let [nx, ny, _] = dimensions;
    while !values.is_empty() {
        let p = [start % nx, (start / nx) % ny, start / (nx * ny)];
        let count = (nx - p[0]).min(values.len());
        let (row, remaining) = values.split_at_mut(count);
        operation(start, p, row)?;
        start += count;
        values = remaining;
    }
    Ok(())
}

pub(crate) fn try_for_each_chunk_mut<T: Send, E: Send>(
    values: &mut [T],
    stage: CalculationStage,
    preferred: usize,
    operation: impl Fn(usize, &mut [T]) -> Result<(), E> + Send + Sync,
) -> Result<(), E> {
    #[cfg(feature = "parallel")]
    if values.len() >= minimum_parallel_values(stage) && rayon::current_num_threads() > 1 {
        let run = crate::scheduling::BatchRun::new(stage, values.len(), preferred);
        let result = values
            .par_chunks_mut(run.batch)
            .enumerate()
            .try_for_each(|(chunk, values)| operation(chunk * run.batch, values));
        if result.is_ok() {
            run.complete();
        }
        return result;
    }
    // Retain the serial loop shape: no scheduling or chunk boundaries.
    let _ = (stage, preferred);
    operation(0, values)
}

#[cfg(feature = "parallel")]
fn minimum_parallel_values(stage: CalculationStage) -> usize {
    // Neighbor searches do substantially more work per item than grid vector
    // passes, so sparse material scenes merit parallel work at a smaller size.
    match stage {
        CalculationStage::ParticleDensity | CalculationStage::ParticleForces => 512,
        _ => MIN_PARALLEL_VALUES,
    }
}

pub(crate) fn for_each_mut<T: Send>(
    values: &mut [T],
    offset: usize,
    operation: impl Fn(usize, &mut T) + Send + Sync,
) {
    let result: Result<(), std::convert::Infallible> = try_for_each_chunk_mut(
        values,
        CalculationStage::FieldUpdate,
        CHUNK_VALUES,
        |start, values| {
            let probe = crate::observation::WorkProbe::new(
                crate::observation::CalculationStage::FieldUpdate,
            );
            for (i, value) in values.iter_mut().enumerate() {
                probe.visit_index(offset + start + i);
                operation(start + i, value);
            }
            Ok(())
        },
    );
    match result {
        Ok(()) => (),
        Err(never) => match never {},
    }
}

pub(crate) fn for_each_pair_mut<T: Send, U: Send>(
    a: &mut [T],
    b: &mut [U],
    offset: usize,
    operation: impl Fn(usize, &mut T, &mut U) + Send + Sync,
) {
    assert_eq!(a.len(), b.len(), "paired simulation scratch lengths");
    #[cfg(feature = "parallel")]
    if a.len() >= MIN_PARALLEL_VALUES && rayon::current_num_threads() > 1 {
        let run =
            crate::scheduling::BatchRun::new(CalculationStage::FieldUpdate, a.len(), CHUNK_VALUES);
        a.par_chunks_mut(run.batch)
            .zip(b.par_chunks_mut(run.batch))
            .enumerate()
            .for_each(|(chunk, (a, b))| {
                let probe = crate::observation::WorkProbe::new(
                    crate::observation::CalculationStage::FieldUpdate,
                );
                for (i, (a, b)) in a.iter_mut().zip(b).enumerate() {
                    probe.visit_index(offset + chunk * run.batch + i);
                    operation(chunk * run.batch + i, a, b);
                }
            });
        run.complete();
        return;
    }
    let probe =
        crate::observation::WorkProbe::new(crate::observation::CalculationStage::FieldUpdate);
    for (i, (a, b)) in a.iter_mut().zip(b).enumerate() {
        probe.visit_index(offset + i);
        operation(i, a, b);
    }
}

#[cfg(test)]
mod row_tests {
    use super::*;
    #[test]
    fn row_fragments_cover_partial_first_and_last_rows_without_gaps() {
        let mut values = vec![usize::MAX; 31];
        let mut fragments = Vec::new();
        for_each_row_mut(&mut values, 7, [5, 3, 3], |start, p, values| {
            fragments.push((start, p, values.len()));
            for (i, value) in values.iter_mut().enumerate() {
                *value = start + i;
            }
            Ok::<(), ()>(())
        })
        .unwrap();
        assert_eq!(values, (7..38).collect::<Vec<_>>());
        assert_eq!(
            fragments,
            vec![
                (7, [2, 1, 0], 3),
                (10, [0, 2, 0], 5),
                (15, [0, 0, 1], 5),
                (20, [0, 1, 1], 5),
                (25, [0, 2, 1], 5),
                (30, [0, 0, 2], 5),
                (35, [0, 1, 2], 3)
            ]
        );
    }
}
