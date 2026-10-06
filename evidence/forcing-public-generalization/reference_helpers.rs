impl CoupledDiscreteFlow {
    pub(super) fn clone_candidate<'a>(&self) -> super::candidate::CoupledDiscreteFlow<'a> {
        super::candidate::CoupledDiscreteFlow::from_cloned_parts(
            self.flow.clone(),
            self.work.geometry.clone(),
            self.work.r,
            self.work.diagnostic,
            self.work.pressure,
            self.pressure,
            self.allocated_bytes,
        )
    }
    pub(super) fn clone_native(&self) -> Self {
        Self {
            flow: self.flow.clone(),
            work: Work {
                geometry: self.work.geometry.clone(),
                r: self.work.r,
                diagnostic: self.work.diagnostic,
                pressure: self.work.pressure,
            },
            pressure: self.pressure,
            allocated_bytes: self.allocated_bytes,
        }
    }
    pub(super) fn accepted_coefficients(&self) -> [f64; 22] {
        coefficients(self.state().velocity, &self.work.r).unwrap()
    }
}
pub(super) fn layout() -> (usize, usize, usize, usize, usize) {
    (
        STEP_STACK_BYTES,
        THIRD_STACK_BYTES,
        FORCE_STACK_BYTES,
        std::mem::size_of::<CoupledDiscreteFlow>(),
        std::mem::size_of::<Work>(),
    )
}
#[inline(never)]
pub(super) fn public_call(
    owner: &mut CoupledDiscreteFlow,
    control: &mut super::Control,
    h: f64,
    forces: &[BodyForce],
) -> Result<CoupledForcedExtrudedReport, CoupledDiscreteError> {
    owner.step_extruded_with_forces(h, forces, &mut |s| super::observe(control, s as usize))
}
