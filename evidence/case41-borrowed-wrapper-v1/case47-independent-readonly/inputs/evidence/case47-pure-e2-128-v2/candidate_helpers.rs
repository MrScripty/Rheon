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
    pub(super) fn select_working_affine(&mut self, enabled: bool) {
        self.work
            .research
            .as_deref_mut()
            .unwrap()
            .select_working_affine(enabled);
    }
    pub(super) fn set_chart(&mut self, chart: &'a mut scalar::ChartWorkspace) {
        assert!(self.work.research.is_none());
        let descriptor = std::mem::size_of::<Self>()
            - std::mem::size_of::<super::reference::CoupledDiscreteFlow>();
        let reservation = self
            .allocated_bytes
            .checked_add(67584 - descriptor)
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
    h: f64,
    forces: &[BodyForce],
) -> Result<CoupledForcedExtrudedReport, CoupledDiscreteError> {
    owner.step_extruded_with_forces(h, forces, &mut |s| super::observe(control, s as usize))
}

#[inline(never)]
pub(super) fn public_constructor<'a>(
    velocity: &[[f64; 3]; 16],
) -> Result<CoupledDiscreteFlow<'a>, CoupledDiscreteError> {
    CoupledDiscreteFlow::new_forced_extruded(
        FittedHeightGeometry {
            cap: &[[0., 1.], [0.5, 1.25], [1., 1.]],
            bottom_x: &[0., 0.5, 1.],
            extrusion_width: 1.,
            density: 3.,
            dynamic_viscosity: 0.05,
        },
        velocity,
        Default::default(),
        Default::default(),
        131,
    )
}
