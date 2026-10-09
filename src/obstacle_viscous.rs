//! Restricted symmetric Newtonian stress and negative matched transpose force.
//! Rayleigh potential/dissipation are powers, not stored mechanical energy.
//! All diagnostics are unenclosed; this owner cannot authorize evolution.
use crate::obstacle_pressure::{Sum, checked, checkpoint, gate, mul};
use crate::{
    MAX_OBSTACLE_STATE_BYTES, ObstacleFlowError, ObstacleFlowStage,
    ObstacleGradientBoundary as Boundary, ObstacleGradientError, ObstacleGradientRow,
    ObstacleGradientSite as Site, ObstacleStateQualification, ObstacleVelocityGradient,
};
use std::{fmt, mem::size_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleViscousBlock {
    Normal { row: usize },
    Shear { first: usize, second: usize },
}
#[derive(Debug, Clone, PartialEq)]
pub enum ObstacleViscousError {
    InvalidBudget { limit: usize },
    DuplicateSite,
    MissingShearPartner,
    IncompatibleShearPair,
    Gradient(ObstacleGradientError),
    Flow(ObstacleFlowError),
}
impl From<ObstacleFlowError> for ObstacleViscousError {
    fn from(e: ObstacleFlowError) -> Self {
        Self::Flow(e)
    }
}
impl From<ObstacleGradientError> for ObstacleViscousError {
    fn from(e: ObstacleGradientError) -> Self {
        Self::Gradient(e)
    }
}
impl fmt::Display for ObstacleViscousError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "obstacle viscous refused: {self:?}")
    }
}
impl std::error::Error for ObstacleViscousError {}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleViscousWork {
    /// R, watts; partial requested quadrature, not stored kinetic energy.
    pub rayleigh_potential: f64,
    /// D=2R, watts; exact-real nonnegativity is conditional on positive weights.
    pub dissipation: f64,
    /// Independently summed snapshot velocity dot actual force output, watts.
    pub force_work: f64,
    /// force_work + D: nearest-rounded observation, not an enclosure or gate.
    pub unenclosed_defect: f64,
}
/// Borrows one immutable state/geometry through its gradient. No action allocates.
/// No mutable snapshot, raw-field substitution, solve, pressure or step API.
pub struct ObstacleViscousStress<'o, 's, 'g> {
    gradient: &'o ObstacleVelocityGradient<'s, 'g>,
    blocks: Vec<ObstacleViscousBlock>,
    owned_payload_bytes: usize,
    constructor_peak_payload_bytes: usize,
}
fn key(row: &ObstacleGradientRow) -> [usize; 8] {
    match row.site {
        Site::Normal { axis, cell } => [
            0,
            axis.index(),
            axis.index(),
            cell[0],
            cell[1],
            cell[2],
            0,
            axis.index(),
        ],
        Site::Cross {
            component,
            derivative,
            edge,
            quadrant,
        } => {
            let a = component.index();
            let b = derivative.index();
            [
                1,
                a.min(b),
                a.max(b),
                edge[0],
                edge[1],
                edge[2],
                quadrant as usize,
                a,
            ]
        }
        // The immutable parent factory refuses this; do not silently turn it into a normal block.
        Site::CoarseFine => [2; 8],
    }
}
fn layout_block(
    rows: &[ObstacleGradientRow],
    indices: &[usize],
    at: usize,
) -> Result<(ObstacleViscousBlock, usize), ObstacleViscousError> {
    let first = indices[at];
    let k = key(&rows[first]);
    let mut end = at + 1;
    while end < indices.len() && key(&rows[indices[end]])[..7] == k[..7] {
        end += 1;
    }
    for pair in indices[at..end].windows(2) {
        if key(&rows[pair[0]]) == key(&rows[pair[1]]) {
            return Err(ObstacleViscousError::DuplicateSite);
        }
    }
    if k[0] == 0 {
        return Ok((ObstacleViscousBlock::Normal { row: first }, end));
    }
    if k[0] != 1 {
        return Err(ObstacleViscousError::IncompatibleShearPair);
    }
    if end - at != 2 {
        return Err(ObstacleViscousError::MissingShearPartner);
    }
    let second = indices[at + 1];
    let a = &rows[first];
    let b = &rows[second];
    let boundaries = matches!(
        (a.boundary, b.boundary),
        (Boundary::Interior, Boundary::Interior)
            | (Boundary::FlatWallRay, Boundary::FlatStationaryTrace)
            | (Boundary::FlatStationaryTrace, Boundary::FlatWallRay)
    );
    if a.component != b.derivative
        || a.derivative != b.component
        || a.weight.to_bits() != b.weight.to_bits()
        || !boundaries
    {
        return Err(ObstacleViscousError::IncompatibleShearPair);
    }
    Ok((ObstacleViscousBlock::Shear { first, second }, end))
}
fn add_bytes(base: usize, count: usize, item: usize) -> Result<usize, ObstacleFlowError> {
    count
        .checked_mul(item)
        .and_then(|n| n.checked_add(base))
        .ok_or(ObstacleFlowError::CapacityOverflow)
}
impl<'o, 's, 'g> ObstacleViscousStress<'o, 's, 'g> {
    pub fn new(
        gradient: &'o ObstacleVelocityGradient<'s, 'g>,
        limit: usize,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<Self, ObstacleViscousError> {
        if limit == 0 || limit > MAX_OBSTACLE_STATE_BYTES {
            return Err(ObstacleViscousError::InvalidBudget { limit });
        }
        let base = add_bytes(gradient.combined_payload_bytes(), 1, size_of::<Self>())?;
        let n = gradient.rows().len();
        gate(add_bytes(base, n, size_of::<usize>())?, limit)?;
        let mut indices = Vec::new();
        indices
            .try_reserve_exact(n)
            .map_err(|_| ObstacleFlowError::AllocationFailure)?;
        let indexed = add_bytes(base, indices.capacity(), size_of::<usize>())?;
        gate(indexed, limit)?;
        for i in 0..n {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, i)?;
            indices.push(i);
        }
        // Allocation-free in-place sort; key extraction also allocates nothing.
        indices.sort_unstable_by_key(|&i| key(&gradient.rows()[i]));
        let mut at = 0;
        let mut count = 0usize;
        while at < n {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, at)?;
            let (_, next) = layout_block(gradient.rows(), &indices, at)?;
            at = next;
            count = count
                .checked_add(1)
                .ok_or(ObstacleFlowError::CapacityOverflow)?;
        }
        gate(
            add_bytes(indexed, count, size_of::<ObstacleViscousBlock>())?,
            limit,
        )?;
        let mut blocks = Vec::new();
        blocks
            .try_reserve_exact(count)
            .map_err(|_| ObstacleFlowError::AllocationFailure)?;
        let peak = add_bytes(
            indexed,
            blocks.capacity(),
            size_of::<ObstacleViscousBlock>(),
        )?;
        gate(peak, limit)?;
        at = 0;
        while at < n {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, at)?;
            let (block, next) = layout_block(gradient.rows(), &indices, at)?;
            blocks.push(block);
            at = next;
        }
        let owned = add_bytes(
            size_of::<Self>(),
            blocks.capacity(),
            size_of::<ObstacleViscousBlock>(),
        )?;
        drop(indices);
        Ok(Self {
            gradient,
            blocks,
            owned_payload_bytes: owned,
            constructor_peak_payload_bytes: peak,
        })
    }
    pub fn gradient(&self) -> &'o ObstacleVelocityGradient<'s, 'g> {
        self.gradient
    }
    pub fn blocks(&self) -> &[ObstacleViscousBlock] {
        &self.blocks
    }
    pub fn dynamic_viscosity(&self) -> f64 {
        self.gradient
            .state()
            .physical_inputs()
            .material
            .dynamic_viscosity
    }
    pub fn owned_payload_bytes(&self) -> usize {
        self.owned_payload_bytes
    }
    pub fn combined_payload_bytes(&self) -> usize {
        self.gradient.combined_payload_bytes() + self.owned_payload_bytes
    }
    pub fn constructor_peak_payload_bytes(&self) -> usize {
        self.constructor_peak_payload_bytes
    }
    pub fn qualification(&self) -> ObstacleStateQualification {
        ObstacleStateQualification::Unqualified
    }
    fn validate_outputs(
        &self,
        scratch: &[f64],
        out: &[&mut [f64]; 3],
    ) -> Result<(), ObstacleViscousError> {
        if scratch.len() != self.gradient.rows().len()
            || (0..3).any(|d| out[d].len() != self.gradient.state().velocity()[d].len())
        {
            return Err(ObstacleFlowError::ShapeMismatch.into());
        }
        Ok(())
    }
    fn compose(
        &self,
        scratch: &mut [f64],
        measure: bool,
        cancel: &mut impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<f64, ObstacleViscousError> {
        self.gradient.gather(scratch, &mut *cancel)?;
        let mu = self.dynamic_viscosity();
        let rows = self.gradient.rows();
        let mut power = Sum::default();
        for (i, &block) in self.blocks.iter().enumerate() {
            checkpoint(cancel, ObstacleFlowStage::Correction, i)?;
            match block {
                ObstacleViscousBlock::Normal { row } => {
                    let g = scratch[row];
                    if measure {
                        power.add(mul(2., mul(mu, mul(rows[row].weight, mul(g, g)?)?)?)?)?;
                    }
                    scratch[row] = mul(2., mul(mu, g)?)?;
                }
                ObstacleViscousBlock::Shear { first, second } => {
                    let gamma = checked(scratch[first] + scratch[second])?;
                    if measure {
                        power.add(mul(mu, mul(rows[first].weight, mul(gamma, gamma)?)?)?)?;
                    }
                    let stress = mul(mu, gamma)?;
                    scratch[first] = stress;
                    scratch[second] = stress;
                }
            }
        }
        Ok(power.finish()?)
    }
    /// Caller output may be partial after cancellation/arithmetic failure.
    pub fn stress(
        &self,
        out: &mut [f64],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<(), ObstacleViscousError> {
        self.compose(out, false, &mut cancel)?;
        Ok(())
    }
    fn negative_transpose(
        &self,
        stress: &[f64],
        out: [&mut [f64]; 3],
        cancel: &mut impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<(), ObstacleViscousError> {
        let [x, y, z] = out;
        self.gradient
            .transpose(stress, [&mut *x, &mut *y, &mut *z], &mut *cancel)?;
        for buffer in [x, y, z] {
            for (face, value) in buffer.iter_mut().enumerate() {
                checkpoint(cancel, ObstacleFlowStage::Correction, face)?;
                if *value != 0. {
                    *value = -*value;
                }
            }
        }
        Ok(())
    }
    /// F=-G^T W tau. No mass inverse, physical wall load or state update.
    pub fn force(
        &self,
        stress_scratch: &mut [f64],
        out: [&mut [f64]; 3],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<(), ObstacleViscousError> {
        self.validate_outputs(stress_scratch, &out)?;
        self.compose(stress_scratch, false, &mut cancel)?;
        self.negative_transpose(stress_scratch, out, &mut cancel)
    }
    pub fn diagnose(
        &self,
        stress_scratch: &mut [f64],
        out: [&mut [f64]; 3],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<ObstacleViscousWork, ObstacleViscousError> {
        self.validate_outputs(stress_scratch, &out)?;
        let dissipation = self.compose(stress_scratch, true, &mut cancel)?;
        let [x, y, z] = out;
        self.negative_transpose(stress_scratch, [&mut *x, &mut *y, &mut *z], &mut cancel)?;
        let mut work = Sum::default();
        for (d, buffer) in [x, y, z].iter().enumerate() {
            for (face, (&u, &f)) in self.gradient.state().velocity()[d]
                .iter()
                .zip(buffer.iter())
                .enumerate()
            {
                checkpoint(&mut cancel, ObstacleFlowStage::Correction, face)?;
                work.add(mul(u, f)?)?;
            }
        }
        let force_work = work.finish()?;
        Ok(ObstacleViscousWork {
            rayleigh_potential: mul(0.5, dissipation)?,
            dissipation,
            force_work,
            unenclosed_defect: checked(force_work + dissipation)?,
        })
    }
}
