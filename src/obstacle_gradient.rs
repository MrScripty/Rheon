//! Requested uniform-grid velocity derivatives on an immutable owned snapshot.
//! Local P1 consistency and finite transpose pairing do not qualify physical
//! loads, full gradient quadrature, evolution, or IEEE error bounds.
use crate::obstacle_pressure::{Sum, checked, checkpoint, div, mul, positive};
use crate::{
    Axis, MAX_OBSTACLE_STATE_BYTES, ObstacleFlowError, ObstacleFlowStage, ObstacleFlowState,
    ObstacleStateQualification,
};
use std::{fmt, mem::size_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleGradientSite {
    Normal {
        axis: Axis,
        cell: [usize; 3],
    },
    /// Quadrant bits use ascending axes: bit0 first, bit1 second.
    Cross {
        component: Axis,
        derivative: Axis,
        edge: [usize; 3],
        quadrant: u8,
    },
    CoarseFine,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleGradientBoundary {
    Normal,
    Interior,
    FlatWallRay,
    FlatStationaryTrace,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleGradientSource {
    VelocityFace { face: usize },
    StationarySolid,
    StationaryOuterNormal,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleGradientEndpoint {
    pub position: [f64; 3],
    pub coefficient: f64,
    pub source: ObstacleGradientSource,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleGradientRow {
    pub site: ObstacleGradientSite,
    pub component: Axis,
    pub derivative: Axis,
    pub boundary: ObstacleGradientBoundary,
    /// Stored positive cell/sector volume; no global completeness claim.
    pub weight: f64,
    pub endpoints: [ObstacleGradientEndpoint; 2],
}
impl ObstacleGradientRow {
    pub fn active_term_count(&self) -> usize {
        self.endpoints
            .iter()
            .filter(|e| matches!(e.source, ObstacleGradientSource::VelocityFace { .. }))
            .count()
    }
    fn evaluate(&self, state: &ObstacleFlowState<'_>) -> Result<f64, ObstacleFlowError> {
        let mut sum = Sum::default();
        for e in self.endpoints {
            if let ObstacleGradientSource::VelocityFace { face } = e.source {
                sum.add(mul(
                    e.coefficient,
                    state.velocity()[self.component.index()][face],
                )?)?;
            }
        }
        sum.finish()
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum ObstacleGradientError {
    InvalidBudget { limit: usize },
    InvalidSite,
    UnsupportedCorner,
    UnsupportedOuterEdge,
    UnsupportedSectorPattern,
    UnsupportedCoarseFine,
    Flow(ObstacleFlowError),
}
impl From<ObstacleFlowError> for ObstacleGradientError {
    fn from(e: ObstacleFlowError) -> Self {
        Self::Flow(e)
    }
}
impl fmt::Display for ObstacleGradientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "obstacle gradient refused: {self:?}")
    }
}
impl std::error::Error for ObstacleGradientError {}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleGradientWork {
    pub row_pairing: f64,
    pub face_pairing: f64,
    /// Nearest-rounded diagnostic, not a bound or acceptance gate.
    pub unenclosed_defect: f64,
}
/// One state/geometry is shared; only requested rows are owned. No mutable
/// state, solve, force/load, pressure, stepping or raw-field gather is exposed.
pub struct ObstacleVelocityGradient<'s, 'g> {
    state: &'s ObstacleFlowState<'g>,
    rows: Vec<ObstacleGradientRow>,
    owned_payload_bytes: usize,
}
fn plane(state: &ObstacleFlowState<'_>, d: usize, i: usize) -> f64 {
    let g = state.geometry().grid();
    g.origin()[d] + i as f64 * g.spacing()[d]
}
fn center(state: &ObstacleFlowState<'_>, d: usize, i: usize) -> f64 {
    let g = state.geometry().grid();
    g.origin()[d] + (i as f64 + 0.5) * g.spacing()[d]
}
fn endpoint(
    state: &ObstacleFlowState<'_>,
    component: Axis,
    p: [usize; 3],
) -> Result<ObstacleGradientEndpoint, ObstacleGradientError> {
    let geometry = state.geometry();
    let g = geometry.grid();
    let d = component.index();
    let face = g
        .face_index(component, p)
        .ok_or(ObstacleGradientError::InvalidSite)?;
    let source = if p[d] == 0 || p[d] == g.counts()[d] {
        ObstacleGradientSource::StationaryOuterNormal
    } else if geometry.open_areas(component)[face] == 0. {
        ObstacleGradientSource::StationarySolid
    } else {
        ObstacleGradientSource::VelocityFace { face }
    };
    Ok(ObstacleGradientEndpoint {
        position: g
            .face_position(component, p)
            .ok_or(ObstacleGradientError::InvalidSite)?,
        source,
        coefficient: 0.,
    })
}
fn coefficients(row: &mut ObstacleGradientRow) -> Result<(), ObstacleGradientError> {
    let d = row.derivative.index();
    let separation = positive(checked(
        row.endpoints[1].position[d] - row.endpoints[0].position[d],
    )?)?;
    let inverse = div(1., separation)?;
    row.endpoints[0].coefficient = -inverse;
    row.endpoints[1].coefficient = inverse;
    Ok(())
}
pub(crate) fn make_row(
    state: &ObstacleFlowState<'_>,
    site: ObstacleGradientSite,
) -> Result<ObstacleGradientRow, ObstacleGradientError> {
    let geometry = state.geometry();
    let g = geometry.grid();
    let n = g.counts();
    match site {
        ObstacleGradientSite::CoarseFine => Err(ObstacleGradientError::UnsupportedCoarseFine),
        ObstacleGradientSite::Normal { axis, cell } => {
            let index = g
                .cell_index(cell)
                .ok_or(ObstacleGradientError::InvalidSite)?;
            let weight = geometry.fluid_volumes()[index];
            if weight == 0. {
                return Err(ObstacleGradientError::InvalidSite);
            }
            let weight = positive(weight)?;
            let mut upper = cell;
            upper[axis.index()] += 1;
            let mut row = ObstacleGradientRow {
                site,
                component: axis,
                derivative: axis,
                boundary: ObstacleGradientBoundary::Normal,
                weight,
                endpoints: [endpoint(state, axis, cell)?, endpoint(state, axis, upper)?],
            };
            coefficients(&mut row)?;
            Ok(row)
        }
        ObstacleGradientSite::Cross {
            component,
            derivative,
            edge,
            quadrant,
        } => {
            let component_index = component.index();
            let d = derivative.index();
            if component == derivative || quadrant > 3 {
                return Err(ObstacleGradientError::InvalidSite);
            }
            let a = component_index.min(d);
            let b = component_index.max(d);
            let c = 3 - a - b;
            if edge[a] > n[a] || edge[b] > n[b] || edge[c] >= n[c] {
                return Err(ObstacleGradientError::InvalidSite);
            }
            if edge[a] == 0 || edge[a] == n[a] || edge[b] == 0 || edge[b] == n[b] {
                return Err(ObstacleGradientError::UnsupportedOuterEdge);
            }
            let mut cells = [None; 4];
            for (q, slot) in cells.iter_mut().enumerate() {
                let mut cell = edge;
                if q & 1 == 0 {
                    cell[a] -= 1;
                }
                if q & 2 == 0 {
                    cell[b] -= 1;
                }
                let index = g
                    .cell_index(cell)
                    .ok_or(ObstacleGradientError::InvalidSite)?;
                if geometry.fluid_volumes()[index] > 0. {
                    *slot = Some(cell);
                }
            }
            let sector = cells[quadrant as usize].ok_or(ObstacleGradientError::InvalidSite)?;
            let count = cells.iter().flatten().count();
            if count == 3 {
                return Err(ObstacleGradientError::UnsupportedCorner);
            }
            if count != 2 && count != 4 {
                return Err(ObstacleGradientError::UnsupportedSectorPattern);
            }
            let width_a =
                positive(checked(center(state, a, sector[a]) - plane(state, a, edge[a]))?.abs())?;
            let width_b =
                positive(checked(center(state, b, sector[b]) - plane(state, b, edge[b]))?.abs())?;
            let width_c = positive(checked(
                plane(state, c, edge[c] + 1) - plane(state, c, edge[c]),
            )?)?;
            let weight = positive(mul(mul(width_a, width_b)?, width_c)?)?;
            let mut lower = edge;
            lower[d] -= 1;
            let mut row = ObstacleGradientRow {
                site,
                component,
                derivative,
                boundary: ObstacleGradientBoundary::Interior,
                weight,
                endpoints: [
                    endpoint(state, component, lower)?,
                    endpoint(state, component, edge)?,
                ],
            };
            if count == 2 {
                let first = cells.iter().position(Option::is_some).unwrap();
                let last = cells.iter().rposition(Option::is_some).unwrap();
                let normal = match first ^ last {
                    1 => b,
                    2 => a,
                    _ => return Err(ObstacleGradientError::UnsupportedSectorPattern),
                };
                if d == normal {
                    row.boundary = ObstacleGradientBoundary::FlatWallRay;
                    let bit = if d == a { 1 } else { 2 };
                    let plus = first & bit != 0;
                    let active_endpoint = if plus { 1 } else { 0 };
                    let trace_endpoint = 1 - active_endpoint;
                    // Keep only the actual fluid-side sample; the wall trace is
                    // at the admitted plane, never at a blocked sample center.
                    row.endpoints[trace_endpoint].position =
                        row.endpoints[active_endpoint].position;
                    row.endpoints[trace_endpoint].position[d] = plane(state, d, edge[d]);
                    row.endpoints[trace_endpoint].source = ObstacleGradientSource::StationarySolid;
                } else {
                    row.boundary = ObstacleGradientBoundary::FlatStationaryTrace;
                    for e in &mut row.endpoints {
                        e.position[component_index] =
                            plane(state, component_index, edge[component_index]);
                        e.source = ObstacleGradientSource::StationarySolid;
                    }
                }
            }
            coefficients(&mut row)?;
            Ok(row)
        }
    }
}
fn gate(required: usize, limit: usize) -> Result<(), ObstacleGradientError> {
    if required > limit {
        return Err(ObstacleFlowError::BufferLimit { required, limit }.into());
    }
    Ok(())
}
impl<'s, 'g> ObstacleVelocityGradient<'s, 'g> {
    pub fn planned_payload_bytes(
        state: &ObstacleFlowState<'_>,
        row_count: usize,
    ) -> Result<usize, ObstacleGradientError> {
        row_count
            .checked_mul(size_of::<ObstacleGradientRow>())
            .and_then(|n| n.checked_add(size_of::<Self>()))
            .and_then(|n| n.checked_add(state.combined_payload_bytes()))
            .ok_or(ObstacleFlowError::CapacityOverflow.into())
    }
    pub fn new(
        state: &'s ObstacleFlowState<'g>,
        sites: &[ObstacleGradientSite],
        limit: usize,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<Self, ObstacleGradientError> {
        if limit == 0 || limit > MAX_OBSTACLE_STATE_BYTES {
            return Err(ObstacleGradientError::InvalidBudget { limit });
        }
        gate(Self::planned_payload_bytes(state, sites.len())?, limit)?;
        for (i, &site) in sites.iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, i)?;
            make_row(state, site)?;
        }
        let mut rows = Vec::new();
        rows.try_reserve_exact(sites.len())
            .map_err(|_| ObstacleFlowError::AllocationFailure)?;
        let owned_payload_bytes = rows
            .capacity()
            .checked_mul(size_of::<ObstacleGradientRow>())
            .and_then(|n| n.checked_add(size_of::<Self>()))
            .ok_or(ObstacleFlowError::CapacityOverflow)?;
        gate(
            state
                .combined_payload_bytes()
                .checked_add(owned_payload_bytes)
                .ok_or(ObstacleFlowError::CapacityOverflow)?,
            limit,
        )?;
        for (i, &site) in sites.iter().enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, i)?;
            rows.push(make_row(state, site)?);
        }
        Ok(Self {
            state,
            rows,
            owned_payload_bytes,
        })
    }
    pub fn state(&self) -> &'s ObstacleFlowState<'g> {
        self.state
    }
    pub fn rows(&self) -> &[ObstacleGradientRow] {
        &self.rows
    }
    pub fn owned_payload_bytes(&self) -> usize {
        self.owned_payload_bytes
    }
    pub fn combined_payload_bytes(&self) -> usize {
        self.state.combined_payload_bytes() + self.owned_payload_bytes
    }
    pub fn qualification(&self) -> ObstacleStateQualification {
        ObstacleStateQualification::Unqualified
    }
    /// No allocations; output can be partially written after arithmetic/cancellation.
    pub fn gather(
        &self,
        out: &mut [f64],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<(), ObstacleGradientError> {
        if out.len() != self.rows.len() {
            return Err(ObstacleFlowError::ShapeMismatch.into());
        }
        for (i, (row, value)) in self.rows.iter().zip(out).enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Correction, i)?;
            *value = row.evaluate(self.state)?;
        }
        Ok(())
    }
    fn validate_test(&self, q: &[f64], out: &[&mut [f64]; 3]) -> Result<(), ObstacleGradientError> {
        if q.len() != self.rows.len()
            || (0..3).any(|d| out[d].len() != self.state.velocity()[d].len())
        {
            return Err(ObstacleFlowError::ShapeMismatch.into());
        }
        if q.iter().any(|&v| v != 0. && !v.is_normal()) {
            return Err(ObstacleFlowError::NonFiniteInput.into());
        }
        Ok(())
    }
    /// Matched G^T W q. Caller owns scratch; no physical force or update is implied.
    pub fn transpose(
        &self,
        q: &[f64],
        out: [&mut [f64]; 3],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<(), ObstacleGradientError> {
        self.validate_test(q, &out)?;
        let [x, y, z] = out;
        let mut buffers = [x, y, z];
        for buffer in buffers.iter_mut() {
            buffer.fill(0.);
        }
        for (i, (row, &test)) in self.rows.iter().zip(q).enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Correction, i)?;
            let value = mul(row.weight, test)?;
            for e in row.endpoints {
                if let ObstacleGradientSource::VelocityFace { face } = e.source {
                    let slot = &mut buffers[row.component.index()][face];
                    *slot = checked(*slot + mul(e.coefficient, value)?)?;
                }
            }
        }
        Ok(())
    }
    /// Row side independently evaluated; face side uses actual transpose scratch.
    pub fn diagnose(
        &self,
        q: &[f64],
        out: [&mut [f64]; 3],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<ObstacleGradientWork, ObstacleGradientError> {
        self.validate_test(q, &out)?;
        let mut lhs = Sum::default();
        for (i, (row, &test)) in self.rows.iter().zip(q).enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Correction, i)?;
            lhs.add(mul(mul(row.weight, row.evaluate(self.state)?)?, test)?)?;
        }
        let [x, y, z] = out;
        self.transpose(q, [&mut *x, &mut *y, &mut *z], &mut cancel)?;
        let mut rhs = Sum::default();
        for (d, buffer) in [x, y, z].iter().enumerate() {
            for (face, (&u, &v)) in self.state.velocity()[d]
                .iter()
                .zip(buffer.iter())
                .enumerate()
            {
                checkpoint(&mut cancel, ObstacleFlowStage::Correction, face)?;
                rhs.add(mul(u, v)?)?;
            }
        }
        let row_pairing = lhs.finish()?;
        let face_pairing = rhs.finish()?;
        Ok(ObstacleGradientWork {
            row_pairing,
            face_pairing,
            unenclosed_defect: checked(row_pairing - face_pairing)?,
        })
    }
}
