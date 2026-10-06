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
        let descriptor = std::mem::size_of::<Self>()
            - std::mem::size_of::<super::reference::CoupledDiscreteFlow>();
        let allocated_bytes = allocated_bytes.checked_add(descriptor).unwrap();
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
        let descriptor = std::mem::size_of::<Self>()
            - std::mem::size_of::<super::reference::CoupledDiscreteFlow>();
        let reservation = self
            .allocated_bytes
            .checked_add(65536 - descriptor)
            .unwrap();
        assert!(reservation <= self.flow.settings().memory_limit);
        self.allocated_bytes = reservation;
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
