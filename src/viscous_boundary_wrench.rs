//! Stationary variational viscous boundary wrench for the retained aligned box.
//! This restores virtual rigid traces to the *existing* strain reconstruction.
//! No physical moving-wall velocity, pressure, stepping or facet traction is
//! accepted. A row torque includes normal-trace derivatives; it is not a point
//! tangential traction placed on the wall. Exact-real work/closure identities
//! have separate premises; every native defect below is unenclosed diagnostic.
use crate::obstacle_pressure::{Sum, allocate, checked, checkpoint, div, gate, mul, positive};
use crate::{
    AlignedStrain, AlignedStrainBoundary, AlignedStrainError, ObstacleFlowError, ObstacleFlowStage,
};
use std::mem::size_of;

/// Columns are world translation xyz, then angular velocity xyz, about reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViscousBoundaryLift {
    pub solid: [f64; 6],
    pub outer: [f64; 6],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViscousBoundaryWrenchReport {
    /// Actual borrowed geometry stamp and material/reference identity. This
    /// diagnostic is not a capability ticket and cannot authorize an update.
    pub surface_stamp: crate::SurfaceStamp,
    pub reference: [f64; 3],
    pub density: f64,
    pub viscosity: f64,
    /// Force xyz (N), then torque xyz (N m), about the retained reference.
    pub solid_wrench: [f64; 6],
    pub outer_wrench: [f64; 6],
    /// Sum of active face forces and their moments at actual sample positions.
    pub fluid_wrench: [f64; 6],
    /// Fluid + solid + outer, nearest-rounded and NOT an enclosure or acceptance gate.
    pub balance_defect: [f64; 6],
    pub dissipation: f64,
    pub force_work: f64,
    pub work_defect: f64,
    /// Max |E R + Cs + Co| per rigid mode for the stored floating coefficients.
    /// An exact-real closure claim needs this identity as an explicit premise.
    pub common_rigid_residual_max: [f64; 6],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViscousBoundaryVirtualWork {
    /// Pairing of computed wrench with test twists; W for velocity-valued tests.
    pub wrench_work: f64,
    /// Independently summed -mu sum_r w_r (Eu)_r (Cs xi + Co eta)_r.
    pub row_work: f64,
    pub defect: f64,
}

/// Borrows the immutable aligned operator (and thus its stamped geometry).
/// Only two fixed six-column lifts per row are allocated. The constructor cap
/// covers their actual Vec capacity PLUS the borrowed operator's accounted
/// payload; geometry, stack, caller fields, allocator metadata and RSS excluded.
/// Actions allocate nothing. Caller force output may be partially written on
/// arithmetic failure/cancellation, matching the existing strain API contract.
pub struct AlignedViscousBoundaryWrench<'a, 'g> {
    operator: &'a AlignedStrain<'g>,
    reference: [f64; 3],
    lifts: Vec<ViscousBoundaryLift>,
    lift_bytes: usize,
    common_rigid_residual_max: [f64; 6],
}

fn basis(
    component: usize,
    position: [f64; 3],
    reference: [f64; 3],
) -> Result<[f64; 6], ObstacleFlowError> {
    let mut r = [0.0; 3];
    for d in 0..3 {
        r[d] = checked(position[d] - reference[d])?;
    }
    let mut out = [0.0; 6];
    out[component] = 1.0;
    match component {
        0 => {
            out[4] = r[2];
            out[5] = -r[1];
        }
        1 => {
            out[3] = -r[2];
            out[5] = r[0];
        }
        2 => {
            out[3] = r[1];
            out[4] = -r[0];
        }
        _ => unreachable!(),
    }
    Ok(out)
}
fn add_scaled(
    out: &mut [f64; 6],
    coefficient: f64,
    values: [f64; 6],
) -> Result<(), ObstacleFlowError> {
    for k in 0..6 {
        out[k] = checked(out[k] + mul(coefficient, values[k])?)?;
    }
    Ok(())
}
fn dot(a: [f64; 6], b: [f64; 6]) -> Result<f64, ObstacleFlowError> {
    let mut sum = Sum::default();
    for k in 0..6 {
        sum.add(mul(a[k], b[k])?)?;
    }
    sum.finish()
}
fn gather(row: &crate::AlignedStrainRow, u: &[f64]) -> Result<f64, ObstacleFlowError> {
    let mut sum = Sum::default();
    for term in row.terms() {
        sum.add(mul(term.coefficient, u[term.active])?)?;
    }
    sum.finish()
}

impl<'a, 'g> AlignedViscousBoundaryWrench<'a, 'g> {
    pub fn new(
        operator: &'a AlignedStrain<'g>,
        reference: [f64; 3],
        limit: usize,
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<Self, AlignedStrainError> {
        if reference.iter().any(|x| !x.is_finite()) {
            return Err(ObstacleFlowError::NonFiniteInput.into());
        }
        let base = operator.allocated_bytes();
        let planned = operator
            .rows()
            .len()
            .checked_mul(size_of::<ViscousBoundaryLift>())
            .and_then(|n| n.checked_add(base))
            .ok_or(ObstacleFlowError::CapacityOverflow)?;
        gate(planned, limit)?;
        let mut used = base;
        let mut lifts = allocate(
            operator.rows().len(),
            ViscousBoundaryLift {
                solid: [0.0; 6],
                outer: [0.0; 6],
            },
            &mut used,
            limit,
        )?;
        let grid = operator.geometry().grid();
        let plane = |d: usize, i: usize| grid.origin()[d] + i as f64 * grid.spacing()[d];
        let center = |d: usize, i: usize| grid.origin()[d] + (i as f64 + 0.5) * grid.spacing()[d];
        for (i, (row, lift)) in operator.rows().iter().zip(lifts.iter_mut()).enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, i)?;
            if row.axes[0] == row.axes[1] {
                let axis = row.axes[0];
                let d = axis.index();
                let p = row.coordinates;
                let inverse = div(1.0, positive(plane(d, p[d] + 1) - plane(d, p[d]))?)?;
                for side in 0..2 {
                    let mut face = p;
                    face[d] += side;
                    let index = grid
                        .face_index(axis, face)
                        .ok_or(ObstacleFlowError::InvalidParameter)?;
                    if operator.active_index(axis, index).is_some() {
                        continue;
                    }
                    let position = std::array::from_fn(|a| {
                        if a == d {
                            plane(a, face[a])
                        } else {
                            center(a, face[a])
                        }
                    });
                    let trace = basis(d, position, reference)?;
                    let coefficient = if side == 0 { -inverse } else { inverse };
                    // An admitted wet normal row's missing nonouter face is an
                    // actual obstacle face, not an interpolated wall location.
                    if face[d] == 0 || face[d] == grid.counts()[d] {
                        add_scaled(&mut lift.outer, coefficient, trace)?;
                    } else {
                        if operator.geometry().open_areas(axis)[index] != 0.0 {
                            return Err(ObstacleFlowError::InvalidParameter.into());
                        }
                        add_scaled(&mut lift.solid, coefficient, trace)?;
                    }
                }
            } else if matches!(
                row.boundary,
                AlignedStrainBoundary::ObstacleFlat | AlignedStrainBoundary::ObstacleCorner
            ) {
                // Derived affine background + sector residual reconstruction:
                // gamma(R)=0; replace each sample U by U-R_component(x_U).
                // This includes BOTH derivatives at flat walls, and each
                // actual reflected corner sector. It is not used for interior
                // rows, whose affine reproduction is a separate diagnostic.
                for term in row.terms() {
                    let face = &operator.active_faces()[term.active];
                    add_scaled(
                        &mut lift.solid,
                        -term.coefficient,
                        basis(face.axis.index(), face.position, reference)?,
                    )?;
                }
            }
        }
        let mut residual = [0.0_f64; 6];
        for (i, (row, lift)) in operator.rows().iter().zip(&lifts).enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Assembly, i)?;
            for k in 0..6 {
                let mut sum = Sum::default();
                for term in row.terms() {
                    let face = &operator.active_faces()[term.active];
                    sum.add(mul(
                        term.coefficient,
                        basis(face.axis.index(), face.position, reference)?[k],
                    )?)?;
                }
                sum.add(lift.solid[k])?;
                sum.add(lift.outer[k])?;
                residual[k] = residual[k].max(sum.finish()?.abs());
            }
        }
        Ok(Self {
            operator,
            reference,
            lifts,
            lift_bytes: used - base,
            common_rigid_residual_max: residual,
        })
    }
    pub fn operator(&self) -> &'a AlignedStrain<'g> {
        self.operator
    }
    pub fn reference(&self) -> [f64; 3] {
        self.reference
    }
    pub fn rows(&self) -> &[ViscousBoundaryLift] {
        &self.lifts
    }
    pub fn allocated_bytes(&self) -> usize {
        self.lift_bytes
    }
    pub fn combined_operator_bytes(&self) -> usize {
        self.operator.allocated_bytes() + self.lift_bytes
    }
    pub fn common_rigid_residual_max(&self) -> [f64; 6] {
        self.common_rigid_residual_max
    }
    fn validate_field(&self, u: &[f64]) -> Result<(), ObstacleFlowError> {
        if u.len() != self.operator.active_faces().len() {
            return Err(ObstacleFlowError::ShapeMismatch);
        }
        if u.iter().any(|x| !x.is_finite()) {
            return Err(ObstacleFlowError::NonFiniteInput);
        }
        Ok(())
    }
    fn boundary_wrenches(
        &self,
        u: &[f64],
        cancel: &mut impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<([f64; 6], [f64; 6]), ObstacleFlowError> {
        let mut solid: [Sum; 6] = std::array::from_fn(|_| Sum::default());
        let mut outer: [Sum; 6] = std::array::from_fn(|_| Sum::default());
        for (i, (row, lift)) in self.operator.rows().iter().zip(&self.lifts).enumerate() {
            checkpoint(cancel, ObstacleFlowStage::Correction, i)?;
            let scale = -mul(mul(self.operator.viscosity(), row.weight)?, gather(row, u)?)?;
            for k in 0..6 {
                solid[k].add(mul(lift.solid[k], scale)?)?;
                outer[k].add(mul(lift.outer[k], scale)?)?;
            }
        }
        let mut s = [0.0; 6];
        let mut o = [0.0; 6];
        for (k, (ss, oo)) in solid.into_iter().zip(outer).enumerate() {
            s[k] = ss.finish()?;
            o[k] = oo.finish()?;
        }
        Ok((s, o))
    }
    /// Physical obstacle and outer traces are stationary. Original force and
    /// work action is used unchanged. Defects do not authorize state advances.
    pub fn diagnose(
        &self,
        u: &[f64],
        force: &mut [f64],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<ViscousBoundaryWrenchReport, AlignedStrainError> {
        let old = self.operator.diagnose(u, force, &mut cancel)?;
        let (solid_wrench, outer_wrench) = self.boundary_wrenches(u, &mut cancel)?;
        let mut fluid: [Sum; 6] = std::array::from_fn(|_| Sum::default());
        for (i, (face, &f)) in self
            .operator
            .active_faces()
            .iter()
            .zip(force.iter())
            .enumerate()
        {
            checkpoint(&mut cancel, ObstacleFlowStage::Acceptance, i)?;
            let b = basis(face.axis.index(), face.position, self.reference)?;
            for k in 0..6 {
                fluid[k].add(mul(b[k], f)?)?;
            }
        }
        let mut fluid_wrench = [0.0; 6];
        let mut balance_defect = [0.0; 6];
        for (k, sum) in fluid.into_iter().enumerate() {
            fluid_wrench[k] = sum.finish()?;
            let mut total = Sum::default();
            total.add(fluid_wrench[k])?;
            total.add(solid_wrench[k])?;
            total.add(outer_wrench[k])?;
            balance_defect[k] = total.finish()?;
        }
        Ok(ViscousBoundaryWrenchReport {
            surface_stamp: self.operator.geometry().stamp(),
            reference: self.reference,
            density: self.operator.density(),
            viscosity: self.operator.viscosity(),
            solid_wrench,
            outer_wrench,
            fluid_wrench,
            balance_defect,
            dissipation: old.dissipation,
            force_work: old.force_work,
            work_defect: old.identity_error,
            common_rigid_residual_max: self.common_rigid_residual_max,
        })
    }
    /// Virtual tests only. Does not change s=Eu or prescribe moving geometry.
    pub fn virtual_boundary_work(
        &self,
        u: &[f64],
        solid_twist: [f64; 6],
        outer_twist: [f64; 6],
        mut cancel: impl FnMut(ObstacleFlowStage, usize) -> bool,
    ) -> Result<ViscousBoundaryVirtualWork, AlignedStrainError> {
        self.validate_field(u)?;
        if solid_twist
            .iter()
            .chain(&outer_twist)
            .any(|x| !x.is_finite())
        {
            return Err(ObstacleFlowError::NonFiniteInput.into());
        }
        let (s, o) = self.boundary_wrenches(u, &mut cancel)?;
        let mut work = Sum::default();
        work.add(dot(s, solid_twist)?)?;
        work.add(dot(o, outer_twist)?)?;
        let wrench_work = work.finish()?;
        let mut row_work = Sum::default();
        for (i, (row, lift)) in self.operator.rows().iter().zip(&self.lifts).enumerate() {
            checkpoint(&mut cancel, ObstacleFlowStage::Acceptance, i)?;
            let mut virtual_strain = Sum::default();
            virtual_strain.add(dot(lift.solid, solid_twist)?)?;
            virtual_strain.add(dot(lift.outer, outer_twist)?)?;
            row_work.add(-mul(
                mul(mul(self.operator.viscosity(), row.weight)?, gather(row, u)?)?,
                virtual_strain.finish()?,
            )?)?;
        }
        let row_work = row_work.finish()?;
        Ok(ViscousBoundaryVirtualWork {
            wrench_work,
            row_work,
            defect: checked(wrench_work - row_work)?,
        })
    }
}
