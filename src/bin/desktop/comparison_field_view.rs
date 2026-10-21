//! Bounded linked native sample slices. Numerical reduction, gauge and recording
//! custody remain with the existing public comparison and M2 source contracts.
use super::comparison::SelectedNativePair;
use super::run_sets::{MemberState, archive_path, source::SavedMember};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke};
use rheon::SnapshotStage;
use rheon::recording::comparison::{
    ComparedValues, ComparedValuesError, FieldRole, PairComparison, Recipe, SampleAssociation,
    Selection, Unavailable, compared_values,
};
use std::path::PathBuf;

const MAX_SAMPLES: usize = 1_048_576;
const MAX_SELECTED_NATIVE_BYTES: usize = 32 * 1024 * 1024;
const MAX_PROMOTED_BYTES: usize = 16 * 1024 * 1024;
const MAX_SOURCE_PUBLICATIONS: usize = 4096;
const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_SLICE_SAMPLES: usize = 16_384;
const MAX_CANVAS_SIDE: f32 = 4096.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceAxis {
    X,
    Y,
    Z,
}
impl SliceAxis {
    fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
    fn plane(self) -> [usize; 2] {
        match self {
            Self::X => [1, 2],
            Self::Y => [0, 2],
            Self::Z => [0, 1],
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PublicationFacts {
    pub time_s: f64,
    pub dt_s: Option<f64>,
    pub pressure_interval_start_s: Option<f64>,
    pub stage: SnapshotStage,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbeReadout {
    pub native_index: [usize; 3],
    pub physical_m: [f64; 3],
    pub reference: f64,
    pub candidate: f64,
    pub association: SampleAssociation,
    pub units: &'static str,
}
#[derive(Debug)]
pub enum FieldViewError {
    Invalid(&'static str),
    Limit(&'static str),
    Source { side: &'static str, detail: String },
    Unavailable(Unavailable),
    Comparison(ComparedValuesError),
    SummaryMismatch,
    Canvas(&'static str),
}
impl std::fmt::Display for FieldViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(r) => write!(f, "invalid linked field view: {r}"),
            Self::Limit(r) => write!(f, "linked field view limit: {r}"),
            Self::Source { side, detail } => write!(f, "{side} exact source: {detail}"),
            Self::Unavailable(r) => write!(f, "native field unavailable: {r:?}"),
            Self::Comparison(e) => write!(f, "native field comparison: {e}"),
            Self::SummaryMismatch => write!(
                f,
                "computed native summary differs from the selected verified pair"
            ),
            Self::Canvas(r) => write!(f, "native sample canvas unavailable: {r}"),
        }
    }
}
impl std::error::Error for FieldViewError {}
#[derive(Clone)]
struct Key {
    root: PathBuf,
    run_set: [u8; 32],
    analysis_version: u32,
    reference_member: String,
    candidate_member: String,
    recipe: Recipe,
    pair: PairComparison,
}
impl Key {
    fn new(s: &SelectedNativePair<'_>) -> Self {
        Self {
            root: s.root.into(),
            run_set: s.recipe.run_set_sha256,
            analysis_version: s.recipe.analysis_version,
            reference_member: s.recipe.reference.member_id.clone(),
            candidate_member: s.recipe.candidate.member_id.clone(),
            recipe: s.recipe.recipe,
            pair: s.pair.clone(),
        }
    }
    fn matches(&self, s: &SelectedNativePair<'_>) -> bool {
        self.root == s.root
            && self.run_set == s.recipe.run_set_sha256
            && self.analysis_version == s.recipe.analysis_version
            && self.reference_member == s.recipe.reference.member_id
            && self.candidate_member == s.recipe.candidate.member_id
            && self.recipe == s.recipe.recipe
            && self.pair == *s.pair
    }
}
struct Loaded {
    key: Key,
    data: ComparedValues,
    times: [PublicationFacts; 2],
    dimensions: [usize; 3],
}
pub struct ComparisonFieldView {
    loaded: Option<Loaded>,
    axis: SliceAxis,
    slice: usize,
    probe: [usize; 3],
    message: String,
    #[cfg(test)]
    controls: std::collections::BTreeMap<&'static str, Rect>,
}
impl Default for ComparisonFieldView {
    fn default() -> Self {
        Self {
            loaded: None,
            axis: SliceAxis::Z,
            slice: 0,
            probe: [0; 3],
            message: "Load one verified native pair to inspect its samples.".into(),
            #[cfg(test)]
            controls: Default::default(),
        }
    }
}
impl ComparisonFieldView {
    pub fn clear(&mut self) {
        self.loaded = None;
        self.probe = [0; 3];
        self.slice = 0;
        self.message =
            "Cached native fields cleared. Load the current verified pair explicitly.".into();
    }
    pub fn is_loaded(&self) -> bool {
        self.loaded.is_some()
    }
    pub fn data(&self) -> Option<&ComparedValues> {
        self.loaded.as_ref().map(|v| &v.data)
    }
    pub fn times(&self) -> Option<&[PublicationFacts; 2]> {
        self.loaded.as_ref().map(|v| &v.times)
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    /// Explicit synchronous source verification/materialization. Never called by paint.
    pub fn load(&mut self, current: SelectedNativePair<'_>) -> Result<(), FieldViewError> {
        self.clear();
        let result = self.load_inner(current);
        match result {
            Ok(loaded) => {
                if plane_count(loaded.dimensions, self.axis)? > MAX_SLICE_SAMPLES {
                    self.axis = [SliceAxis::X, SliceAxis::Y, SliceAxis::Z]
                        .into_iter()
                        .find(|a| {
                            plane_count(loaded.dimensions, *a).is_ok_and(|n| n <= MAX_SLICE_SAMPLES)
                        })
                        .ok_or(FieldViewError::Limit("no slice plane fits the work budget"))?;
                }
                self.loaded = Some(loaded);
                self.message =
                    "Exact native samples last explicitly verified and loaded. No background file monitoring."
                        .into();
                Ok(())
            }
            Err(e) => {
                self.message = e.to_string();
                Err(e)
            }
        }
    }
    fn load_inner(&self, current: SelectedNativePair<'_>) -> Result<Loaded, FieldViewError> {
        current
            .recipe
            .validate()
            .map_err(|_| FieldViewError::Invalid("saved recipe validation"))?;
        if current.pair.analysis_version != current.recipe.analysis_version
            || current.pair.recipe != current.recipe.recipe
        {
            return Err(FieldViewError::Invalid("pair recipe binding"));
        }
        let summary = current
            .pair
            .outcome
            .as_ref()
            .map_err(|r| FieldViewError::Unavailable(*r))?;
        if summary.metrics.support_count == 0 || summary.metrics.support_count > MAX_SAMPLES {
            return Err(FieldViewError::Limit("selected summary sample count"));
        }
        if !summary.color_bounds.min.is_finite()
            || !summary.color_bounds.max.is_finite()
            || summary.color_bounds.min > summary.color_bounds.max
            || [
                summary.metrics.l2,
                summary.metrics.rmse,
                summary.metrics.linf,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
            || summary
                .metrics
                .relative_l2
                .is_some_and(|v| !v.is_finite() || v < 0.0)
            || summary
                .gauge
                .is_some_and(|g| !g.removed_offset_pa.is_finite())
        {
            return Err(FieldViewError::Invalid("finite selected summary"));
        }
        let mut members = Vec::new();
        let mut native_bytes = 0usize;
        for (side, selected, expected) in [
            (
                "reference",
                &current.recipe.reference,
                &current.pair.reference,
            ),
            (
                "candidate",
                &current.recipe.candidate,
                &current.pair.candidate,
            ),
        ] {
            if !selected.selections.iter().any(|s| s == expected) {
                return Err(FieldViewError::Invalid("pair publication membership"));
            }
            let member = current
                .recipe
                .run_set
                .member(&selected.member_id)
                .ok_or(FieldViewError::Invalid("selected member missing"))?;
            let estimate = preflight_member(member, current.recipe.recipe.field)?;
            native_bytes = native_bytes
                .checked_add(estimate)
                .filter(|n| *n <= MAX_SELECTED_NATIVE_BYTES)
                .ok_or(FieldViewError::Limit("estimated native selected payload"))?;
            members.push((side, member, expected));
        }
        // Check both sources and all declared work before opening either SQLite reader.
        for (side, member, _) in &members {
            let attempt = member
                .attempt()
                .ok_or(FieldViewError::Invalid("attempt missing"))?;
            let path = archive_path(current.root, attempt.archive_relative_path)
                .map_err(|e| source_error(side, e))?;
            let metadata = std::fs::metadata(path).map_err(|e| source_error(side, e))?;
            if metadata.len() > MAX_ARCHIVE_BYTES {
                return Err(FieldViewError::Limit("archive file bytes"));
            }
        }
        let mut sources = Vec::new();
        for (side, member, expected) in members {
            let reader = match member.open(current.root) {
                MemberState::Ready(r) => r,
                MemberState::Missing => return Err(source_error(side, "missing exact archive")),
                MemberState::Invalid(e) => return Err(source_error(side, e)),
                MemberState::Unsupported(e) => return Err(source_error(side, e)),
                _ => return Err(source_error(side, "archive is not terminal and pinned")),
            };
            let selected =
                Selection::read(&reader, expected.ordinal).map_err(|e| source_error(side, e))?;
            if selected.identity() != expected {
                return Err(source_error(side, "exact publication identity changed"));
            }
            sources.push(selected);
        }
        let data = compared_values(&sources[0], &sources[1], current.recipe.recipe)
            .map_err(FieldViewError::Comparison)?;
        if data.comparison != *current.pair {
            return Err(FieldViewError::SummaryMismatch);
        }
        let dimensions = dimensions(data.sampling.dimensions)?;
        let n = product(dimensions)?;
        if n > MAX_SAMPLES || n.checked_mul(16).is_none_or(|n| n > MAX_PROMOTED_BYTES) {
            return Err(FieldViewError::Limit("actual promoted arrays"));
        }
        if data.reference.len() != n
            || data.candidate.len() != n
            || summary.metrics.support_count != n
            || data.reference.iter().chain(&data.candidate).any(|v| {
                !v.is_finite() || *v < summary.color_bounds.min || *v > summary.color_bounds.max
            })
        {
            return Err(FieldViewError::Invalid(
                "actual native arrays and common bounds",
            ));
        }
        validate_coordinates(&data, dimensions)?;
        if ![SliceAxis::X, SliceAxis::Y, SliceAxis::Z]
            .into_iter()
            .any(|a| plane_count(dimensions, a).is_ok_and(|n| n <= MAX_SLICE_SAMPLES))
        {
            return Err(FieldViewError::Limit("slice work"));
        }
        let times = [
            facts(&sources[0], current.recipe.recipe.field),
            facts(&sources[1], current.recipe.recipe.field),
        ];
        Ok(Loaded {
            key: Key::new(&current),
            data,
            times,
            dimensions,
        })
    }
    pub fn set_slice(&mut self, axis: SliceAxis, index: usize) -> Result<(), FieldViewError> {
        let result = (|| {
            let loaded = self
                .loaded
                .as_ref()
                .ok_or(FieldViewError::Invalid("no loaded samples"))?;
            if index >= loaded.dimensions[axis.index()] {
                return Err(FieldViewError::Invalid("slice index"));
            }
            if plane_count(loaded.dimensions, axis)? > MAX_SLICE_SAMPLES {
                return Err(FieldViewError::Limit("slice work"));
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.clear();
            self.message = e.to_string();
            return Err(e);
        }
        self.axis = axis;
        self.slice = index;
        self.probe[axis.index()] = index;
        Ok(())
    }
    pub fn set_probe(&mut self, index: [usize; 3]) -> Result<(), FieldViewError> {
        let loaded = self
            .loaded
            .as_ref()
            .ok_or(FieldViewError::Invalid("no loaded samples"))?;
        if index.iter().zip(loaded.dimensions).any(|(i, n)| *i >= n) {
            self.clear();
            self.message = "Native probe index is out of range".into();
            return Err(FieldViewError::Invalid("probe index"));
        }
        self.probe = index;
        self.slice = index[self.axis.index()];
        Ok(())
    }
    pub fn probe(&self) -> Option<ProbeReadout> {
        let loaded = self.loaded.as_ref()?;
        let i = linear(self.probe, loaded.dimensions);
        let summary = loaded.data.comparison.outcome.as_ref().ok()?;
        Some(ProbeReadout {
            native_index: self.probe,
            physical_m: point(&loaded.data, self.probe),
            reference: loaded.data.reference[i],
            candidate: loaded.data.candidate[i],
            association: loaded.data.sampling.association,
            units: summary.units,
        })
    }
    #[cfg(test)]
    pub fn control_rect(&self, id: &str) -> Option<Rect> {
        self.controls.get(id).copied()
    }
    fn remember(&mut self, _id: &'static str, r: egui::Response) -> egui::Response {
        #[cfg(test)]
        {
            self.controls.insert(_id, r.rect);
        }
        r
    }
    /// Paint is source-I/O free. A changed/unverified/unavailable current key clears
    /// the old cache before any pane is drawn; an explicit Load is required again.
    pub fn show(&mut self, ui: &mut egui::Ui, mut current: Option<SelectedNativePair<'_>>) {
        #[cfg(test)]
        self.controls.clear();
        if self
            .loaded
            .as_ref()
            .is_some_and(|v| current.as_ref().is_none_or(|s| !v.key.matches(s)))
        {
            self.clear();
        }
        if let Some(reason) = current.as_ref().and_then(|s| s.pair.outcome.as_ref().err()) {
            self.clear();
            self.message = format!("Native field unavailable: {reason:?}");
        }
        ui.heading("Linked native sample slices");
        let ready = current.as_ref().is_some_and(|s| s.pair.outcome.is_ok());
        let response = ui.add_enabled(ready, egui::Button::new("Load exact native pair"));
        if self.remember("load", response).clicked()
            && let Some(selected) = current.take()
        {
            let _ = self.load(selected);
        }
        ui.label(&self.message);
        let Some(loaded) = &self.loaded else {
            ui.small("A verified matched pair and an explicit load are required. No live or latest data is substituted.");
            return;
        };
        let dimensions = loaded.dimensions;
        let mut axis = self.axis;
        let mut slice = self.slice;
        ui.horizontal(|ui| {
            ui.label("Shared slice normal");
            for a in [SliceAxis::X, SliceAxis::Y, SliceAxis::Z] {
                ui.selectable_value(&mut axis, a, a.label());
            }
            let response = ui.add(
                egui::DragValue::new(&mut slice)
                    .range(0..=dimensions[axis.index()] - 1)
                    .prefix("index "),
            );
            self.remember("slice-index", response);
        });
        if (axis != self.axis || slice != self.slice) && self.set_slice(axis, slice).is_err() {
            return;
        }
        let mut probe = self.probe;
        ui.horizontal(|ui| {
            ui.label("Shared native probe X/Y/Z");
            for d in 0..3 {
                ui.add(egui::DragValue::new(&mut probe[d]).range(0..=dimensions[d] - 1));
            }
        });
        if probe != self.probe && self.set_probe(probe).is_err() {
            return;
        }
        let Some(loaded) = &self.loaded else {
            return;
        };
        let summary = loaded
            .data
            .comparison
            .outcome
            .as_ref()
            .expect("loaded available summary");
        ui.label(format!("Native association {:?} · sample dimensions {:?} · offsets {:?} cells · Cartesian XYZ, x-fast",loaded.data.sampling.association,loaded.data.sampling.dimensions,loaded.data.sampling.offset_cells));
        ui.monospace(format!(
            "ONE common color range [{:.17e}, {:.17e}] {}{}",
            summary.color_bounds.min,
            summary.color_bounds.max,
            summary.units,
            if summary.color_bounds.min == summary.color_bounds.max {
                " · constant range, neutral color"
            } else {
                ""
            }
        ));
        if let Some(g) = summary.gauge {
            ui.label(format!("{} · removed candidate offset {:.17e} Pa · original canonical gauges reference={} candidate={}",if g.diagnostic{"Mean-offset diagnostic pressure"}else{"Direct canonical pressure"},g.removed_offset_pa,g.reference_canonical,g.candidate_canonical));
        }
        ui.small("Tiles represent native sample locations, including native faces. They are not cell-volume averages.");
        let [horizontal, vertical] = self.axis.plane();
        ui.label(format!(
            "Slice index {} · horizontal {} / vertical {} · positive coordinates right / up",
            self.slice,
            ["X", "Y", "Z"][horizontal],
            ["X", "Y", "Z"][vertical]
        ));
        let physical = point(&loaded.data, self.probe);
        let probe_index = linear(self.probe, loaded.dimensions);
        let mut clicked = None;
        let mut error = None;
        let mut rects = [None, None];
        ui.columns(2, |columns| {
            for side in 0..2 {
                columns[side].label(if side == 0 {
                    "Reference native samples"
                } else {
                    "Candidate native samples"
                });
                show_time(&mut columns[side], loaded.times[side]);
                let value = if side == 0 {
                    loaded.data.reference[probe_index]
                } else {
                    loaded.data.candidate[probe_index]
                };
                columns[side].monospace(format!(
                    "{:?} native {:?}\nXYZ [{:.17e}, {:.17e}, {:.17e}] m\nSample {:.17e} {}",
                    loaded.data.sampling.association,
                    self.probe,
                    physical[0],
                    physical[1],
                    physical[2],
                    value,
                    summary.units
                ));
                let response = columns[side].allocate_response(
                    egui::vec2(columns[side].available_width().min(MAX_CANVAS_SIDE), 280.0),
                    Sense::click(),
                );
                match canvas(
                    loaded,
                    self.axis,
                    self.slice,
                    self.probe,
                    side,
                    &columns[side],
                    &response,
                ) {
                    Ok((rect, index)) => {
                        rects[side] = Some(rect);
                        if index.is_some() {
                            clicked = index;
                        }
                    }
                    Err(e) => {
                        columns[side].colored_label(Color32::LIGHT_RED, e.to_string());
                        error = Some(e.to_string());
                    }
                }
            }
        });
        #[cfg(test)]
        for (name, rect) in ["reference-canvas", "candidate-canvas"]
            .into_iter()
            .zip(rects)
        {
            if let Some(rect) = rect {
                self.controls.insert(name, rect);
            }
        }
        #[cfg(not(test))]
        let _ = rects;
        if let Some(index) = clicked {
            let _ = self.set_probe(index);
            ui.ctx().request_repaint();
        }
        if let Some(error) = error {
            ui.colored_label(Color32::LIGHT_RED, error);
        }
        if let Some(p) = self.probe() {
            ui.monospace(format!("Shared probe native {:?} · physical XYZ [{:.17e}, {:.17e}, {:.17e}] m\nReference {:.17e} {} · candidate {:.17e} {}",p.native_index,p.physical_m[0],p.physical_m[1],p.physical_m[2],p.reference,p.units,p.candidate,p.units));
        }
    }
}
fn source_error(side: &'static str, error: impl ToString) -> FieldViewError {
    FieldViewError::Source {
        side,
        detail: error.to_string(),
    }
}
fn dimensions(raw: [u64; 3]) -> Result<[usize; 3], FieldViewError> {
    let mut out = [0; 3];
    for d in 0..3 {
        out[d] = usize::try_from(raw[d]).map_err(|_| FieldViewError::Limit("native dimension"))?;
        if out[d] == 0 {
            return Err(FieldViewError::Invalid("empty native dimension"));
        }
    }
    Ok(out)
}
fn product(d: [usize; 3]) -> Result<usize, FieldViewError> {
    d.into_iter().try_fold(1usize, |n, v| {
        n.checked_mul(v)
            .ok_or(FieldViewError::Limit("sample product"))
    })
}
fn preflight_member(member: SavedMember<'_>, role: FieldRole) -> Result<usize, FieldViewError> {
    let attempt = member
        .attempt()
        .ok_or(FieldViewError::Invalid("attempt missing"))?;
    let pin = attempt
        .finalized
        .ok_or(FieldViewError::Invalid("terminal pin missing"))?;
    if pin.recorded_count > MAX_SOURCE_PUBLICATIONS {
        return Err(FieldViewError::Limit("source publication count"));
    }
    let counts = dimensions(match member {
        SavedMember::Native(m) => m.spec.spec.counts_xyz,
        SavedMember::Configured(m) => m.spec.constructor.counts_xyz,
    })?;
    let cells = product(counts)?;
    let mut faces = 0usize;
    let mut selected = counts;
    for axis in 0..3 {
        let mut shape = counts;
        shape[axis] = shape[axis]
            .checked_add(1)
            .ok_or(FieldViewError::Limit("face dimension"))?;
        faces = faces
            .checked_add(product(shape)?)
            .ok_or(FieldViewError::Limit("face payload"))?;
        if matches!(
            (role, axis),
            (FieldRole::VelocityX, 0) | (FieldRole::VelocityY, 1) | (FieldRole::VelocityZ, 2)
        ) {
            selected = shape;
        }
    }
    let n = product(selected)?;
    if n > MAX_SAMPLES || n.checked_mul(16).is_none_or(|n| n > MAX_PROMOTED_BYTES) {
        return Err(FieldViewError::Limit("estimated promoted samples"));
    }
    // Configured publications additionally retain f64 liquid fraction.
    let cell_bytes = if matches!(member, SavedMember::Configured(_)) {
        20
    } else {
        12
    };
    faces
        .checked_mul(4)
        .and_then(|n| cells.checked_mul(cell_bytes).and_then(|p| n.checked_add(p)))
        .ok_or(FieldViewError::Limit("estimated native selected payload"))
}
fn facts(s: &Selection, role: FieldRole) -> PublicationFacts {
    let p = s.publication();
    PublicationFacts {
        time_s: p.time,
        dt_s: p.dt,
        pressure_interval_start_s: if role == FieldRole::IntervalPressure {
            p.field("interval_pressure").and_then(|f| f.evaluation_time)
        } else {
            None
        },
        stage: p.stage,
    }
}
fn validate_coordinates(data: &ComparedValues, dims: [usize; 3]) -> Result<(), FieldViewError> {
    for (d, count) in dims.into_iter().enumerate() {
        let first =
            data.geometry.origin_m[d] + data.sampling.offset_cells[d] * data.geometry.spacing_m[d];
        let last = data.geometry.origin_m[d]
            + ((count - 1) as f64 + data.sampling.offset_cells[d]) * data.geometry.spacing_m[d];
        let span = count as f64 * data.geometry.spacing_m[d];
        if !first.is_finite() || !last.is_finite() || !span.is_finite() || span <= 0.0 {
            return Err(FieldViewError::Canvas(
                "finite native physical sample extent",
            ));
        }
    }
    Ok(())
}
fn linear(i: [usize; 3], d: [usize; 3]) -> usize {
    i[0] + d[0] * (i[1] + d[1] * i[2])
}
fn point(data: &ComparedValues, i: [usize; 3]) -> [f64; 3] {
    std::array::from_fn(|d| {
        data.geometry.origin_m[d]
            + (i[d] as f64 + data.sampling.offset_cells[d]) * data.geometry.spacing_m[d]
    })
}
fn plane_count(d: [usize; 3], a: SliceAxis) -> Result<usize, FieldViewError> {
    let [u, v] = a.plane();
    d[u].checked_mul(d[v])
        .ok_or(FieldViewError::Limit("slice work"))
}
fn fit_canvas(available: Rect, span: [f64; 2]) -> Result<Rect, FieldViewError> {
    if ![
        available.left(),
        available.right(),
        available.top(),
        available.bottom(),
    ]
    .iter()
    .all(|v| v.is_finite())
        || available.width() < 1.0
        || available.height() < 1.0
        || span.iter().any(|v| !v.is_finite() || *v <= 0.0)
    {
        return Err(FieldViewError::Canvas(
            "finite positive panel and sample spans",
        ));
    }
    let scale = span[0].max(span[1]);
    let unit = [span[0] / scale, span[1] / scale];
    let fit = (f64::from(available.width()) / unit[0]).min(f64::from(available.height()) / unit[1]);
    let size = [unit[0] * fit, unit[1] * fit];
    if size
        .iter()
        .any(|v| !v.is_finite() || *v < 1.0 || *v > f64::from(MAX_CANVAS_SIDE))
    {
        return Err(FieldViewError::Canvas(
            "sample aspect ratio cannot be represented in this panel",
        ));
    }
    let rect = Rect::from_center_size(
        available.center(),
        egui::vec2(size[0] as f32, size[1] as f32),
    );
    if !rect.is_finite() {
        return Err(FieldViewError::Canvas("finite canvas geometry"));
    }
    Ok(rect)
}
fn color(value: f64, min: f64, max: f64) -> Result<Color32, FieldViewError> {
    if !value.is_finite()
        || !min.is_finite()
        || !max.is_finite()
        || min > max
        || value < min
        || value > max
    {
        return Err(FieldViewError::Canvas(
            "finite color input and common bounds",
        ));
    }
    if min == max {
        return Ok(Color32::from_rgb(128, 128, 128));
    }
    let range = max - min;
    let t = if range.is_finite() {
        (value - min) / range
    } else {
        (value / 2.0 - min / 2.0) / (max / 2.0 - min / 2.0)
    };
    if !t.is_finite() {
        return Err(FieldViewError::Canvas("finite color normalization"));
    }
    let t = t.clamp(0.0, 1.0);
    Ok(Color32::from_rgb(
        (255.0 * t).round() as u8,
        64,
        (255.0 * (1.0 - t)).round() as u8,
    ))
}
fn canvas(
    loaded: &Loaded,
    axis: SliceAxis,
    slice: usize,
    probe: [usize; 3],
    side: usize,
    ui: &egui::Ui,
    response: &egui::Response,
) -> Result<(Rect, Option<[usize; 3]>), FieldViewError> {
    let dims = loaded.dimensions;
    if plane_count(dims, axis)? > MAX_SLICE_SAMPLES {
        return Err(FieldViewError::Limit("slice canvas cells"));
    }
    let [u, v] = axis.plane();
    let rect = fit_canvas(
        response.rect,
        [
            dims[u] as f64 * loaded.data.geometry.spacing_m[u],
            dims[v] as f64 * loaded.data.geometry.spacing_m[v],
        ],
    )?;
    let summary = loaded
        .data
        .comparison
        .outcome
        .as_ref()
        .map_err(|r| FieldViewError::Unavailable(*r))?;
    let values = if side == 0 {
        &loaded.data.reference
    } else {
        &loaded.data.candidate
    };
    let cell = egui::vec2(
        rect.width() / dims[u] as f32,
        rect.height() / dims[v] as f32,
    );
    if !cell.x.is_finite() || !cell.y.is_finite() || cell.x <= 0.0 || cell.y <= 0.0 {
        return Err(FieldViewError::Canvas("finite native sample tile"));
    }
    for row in 0..dims[v] {
        for col in 0..dims[u] {
            let mut i = [0; 3];
            i[axis.index()] = slice;
            i[u] = col;
            i[v] = row;
            let lower = Pos2::new(
                rect.left() + col as f32 * cell.x,
                rect.bottom() - (row + 1) as f32 * cell.y,
            );
            let tile = Rect::from_min_size(lower, cell);
            ui.painter().rect_filled(
                tile,
                0.0,
                color(
                    values[linear(i, dims)],
                    summary.color_bounds.min,
                    summary.color_bounds.max,
                )?,
            );
        }
    }
    let center = Pos2::new(
        rect.left() + (probe[u] as f32 + 0.5) * cell.x,
        rect.bottom() - (probe[v] as f32 + 0.5) * cell.y,
    );
    ui.painter().circle_stroke(
        center,
        cell.x.min(cell.y).max(3.0) * 0.35,
        Stroke::new(1.5, Color32::WHITE),
    );
    let clicked = if response.clicked() {
        response
            .interact_pointer_pos()
            .filter(|p| rect.contains(*p))
            .map(|p| {
                let mut i = probe;
                i[axis.index()] = slice;
                i[u] = (((p.x - rect.left()) / rect.width()) * dims[u] as f32).floor() as usize;
                i[v] = (((rect.bottom() - p.y) / rect.height()) * dims[v] as f32).floor() as usize;
                i[u] = i[u].min(dims[u] - 1);
                i[v] = i[v].min(dims[v] - 1);
                i
            })
    } else {
        None
    };
    Ok((rect, clicked))
}
fn show_time(ui: &mut egui::Ui, t: PublicationFacts) {
    ui.monospace(format!(
        "Actual endpoint {:.17e} s · {}",
        t.time_s,
        if t.stage == SnapshotStage::Accepted {
            "accepted"
        } else {
            "constructor"
        }
    ));
    if let Some(dt) = t.dt_s {
        ui.monospace(format!("Actual dt {dt:.17e} s"));
    }
    if let Some(start) = t.pressure_interval_start_s {
        ui.monospace(format!(
            "Projection interval [{start:.17e}, {:.17e}] s",
            t.time_s
        ));
    }
}

#[cfg(test)]
mod canvas_contract {
    use super::*;

    #[test]
    fn common_color_normalization_handles_overflow_and_constant_ranges_explicitly() {
        assert_eq!(
            color(0.0, -f64::MAX, f64::MAX).unwrap(),
            Color32::from_rgb(128, 64, 128)
        );
        assert_eq!(
            color(-f64::MAX, -f64::MAX, f64::MAX).unwrap(),
            Color32::from_rgb(0, 64, 255)
        );
        assert_eq!(
            color(f64::MAX, -f64::MAX, f64::MAX).unwrap(),
            Color32::from_rgb(255, 64, 0)
        );
        assert_eq!(
            color(4.0, 4.0, 4.0).unwrap(),
            Color32::from_rgb(128, 128, 128)
        );
        assert!(matches!(
            color(f64::NAN, 0.0, 1.0),
            Err(FieldViewError::Canvas(_))
        ));
        assert!(matches!(
            color(2.0, 0.0, 1.0),
            Err(FieldViewError::Canvas(_))
        ));
    }

    #[test]
    fn native_canvas_fits_single_sample_axes_and_rejects_nonfinite_or_unrepresentable_extents() {
        let available = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 280.0));
        let fit = fit_canvas(available, [0.5, 0.75]).unwrap();
        assert!(fit.is_finite());
        assert!(available.contains_rect(fit));
        assert!((f64::from(fit.width() / fit.height()) - 2.0 / 3.0).abs() < 1e-6);
        assert!(matches!(
            fit_canvas(available, [f64::MAX, f64::MIN_POSITIVE]),
            Err(FieldViewError::Canvas(_))
        ));
        assert!(matches!(
            fit_canvas(available, [0.0, 1.0]),
            Err(FieldViewError::Canvas(_))
        ));
        assert!(matches!(
            fit_canvas(available, [f64::INFINITY, 1.0]),
            Err(FieldViewError::Canvas(_))
        ));
        assert!(matches!(
            fit_canvas(
                Rect::from_min_max(Pos2::ZERO, egui::pos2(f32::NAN, 1.0)),
                [1.0, 1.0]
            ),
            Err(FieldViewError::Canvas(_))
        ));
    }
}
