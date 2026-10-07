type PairedWork<'a> = Work<'a>;
fn paired_new_work<'a>(geometry: FittedHeightWorkspace) -> PairedWork<'a> {
    Work {
        research: None,
        geometry,
        r: [[[0.; V]; 2]; N],
        diagnostic: [Default::default(); N],
        pressure: [0.; P],
    }
}
impl Work<'_> {
    fn paired_coordinates(
        &self,
        old: &[[f64; 3]; N],
        chart: Option<&mut scalar::ChartWorkspace>,
    ) -> Result<([f64; 3], [f64; 6]), CoupledDiscreteError> {
        coordinates(&self.geometry, old, &self.r, chart)
    }
}
impl<'a> Work<'a> {
    fn paired_attach_chart(&mut self, chart: Option<&'a mut scalar::ChartWorkspace>) {
        self.research = chart;
    }
}
