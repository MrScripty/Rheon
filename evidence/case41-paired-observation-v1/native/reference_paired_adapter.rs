type PairedWork<'a> = Work;
fn paired_new_work(geometry: FittedHeightWorkspace) -> Work {
    Work {
        geometry,
        r: [[[0.; V]; 2]; N],
        diagnostic: [Default::default(); N],
        pressure: [0.; P],
    }
}
impl Work {
    fn paired_coordinates(
        &self,
        old: &[[f64; 3]; N],
        chart: Option<&mut scalar::ChartWorkspace>,
    ) -> Result<([f64; 3], [f64; 6]), CoupledDiscreteError> {
        assert!(chart.is_none());
        coordinates(&self.geometry, old, &self.r)
    }
    fn paired_attach_chart(&mut self, chart: Option<&mut scalar::ChartWorkspace>) {
        assert!(chart.is_none());
    }
}
