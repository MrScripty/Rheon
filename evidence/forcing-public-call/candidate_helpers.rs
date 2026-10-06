impl<'a> CoupledDiscreteFlow<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_cloned_parts(
        flow: TranslatedViscousFlow,
        geometry: FittedHeightWorkspace,
        r: [[[f64; 22]; 2]; 16],
        diagnostic: [FittedHeightNodeDiagnostic; 16],
        work_pressure: [f64; 16],
        pressure: [f64; 16],
        allocated_bytes: usize,
    ) -> Self {
        Self {
            flow,
            work: Work {
                research: None,
                geometry,
                r,
                diagnostic,
                pressure: work_pressure,
            },
            pressure,
            allocated_bytes,
        }
    }
    pub(super) fn set_chart(&mut self, chart: &'a mut scalar::ChartWorkspace) {
        assert!(self.work.research.is_none());
        self.work.research = Some(chart);
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
    owner: &mut CoupledDiscreteFlow<'_>,
    control: &mut super::Control,
) -> Result<CoupledForcedExtrudedReport, CoupledDiscreteError> {
    owner.step_extruded_with_forces(super::H, &super::FORCES, &mut |s| {
        super::observe(control, s as usize)
    })
}
