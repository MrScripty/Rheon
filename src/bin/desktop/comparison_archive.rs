//! Immutable derived native comparison evidence over M2's exact run-set sources.
//! Opening a saved result never verifies physics or silently resolves newer data.
use super::native_comparison::{
    ANALYSIS_VERSION, Coverage, FieldComparison, FieldRole, PressurePolicy, Recipe, RecipeError,
    Selection, SelectionIdentity, align,
};
use super::run_sets::{
    MemberState, RunSetError, RunSetRevision, publish_new, read_versioned,
    source::{SavedMember, SavedRunSet},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{fmt, path::Path};

const SCHEMA_VERSION: u32 = 1;
const MAX_DOCUMENT_BYTES: u64 = 1024 * 1024;
const MAX_SELECTIONS_PER_SOURCE: usize = 128;
const MAX_SELECTED_FIELD_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Reference,
    Candidate,
}
#[derive(Debug)]
pub enum SourceProblem {
    Missing,
    Incomplete,
    Unpinned,
    NotRun,
    Unsupported(&'static str),
    Invalid(RunSetError),
    SelectionChanged,
}
impl fmt::Display for SourceProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => write!(f, "missing exact archive"),
            Self::Incomplete => write!(f, "archive incomplete"),
            Self::Unpinned => write!(f, "archive not pinned"),
            Self::NotRun => write!(f, "member not run"),
            Self::Unsupported(reason) => write!(f, "unsupported archive: {reason}"),
            Self::Invalid(error) => write!(f, "invalid archive: {error}"),
            Self::SelectionChanged => write!(f, "exact publication identity changed"),
        }
    }
}
#[derive(Debug)]
pub enum ComparisonArchiveError {
    Artifact(RunSetError),
    Recipe(RecipeError),
    Invalid(&'static str),
    Unsupported(&'static str),
    Integrity(&'static str),
    Source { side: Side, problem: SourceProblem },
}
impl fmt::Display for ComparisonArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Artifact(e) => write!(f, "comparison artifact: {e}"),
            Self::Recipe(e) => write!(f, "comparison alignment: {e}"),
            Self::Invalid(r) => write!(f, "invalid comparison artifact: {r}"),
            Self::Unsupported(r) => write!(f, "unsupported comparison artifact: {r}"),
            Self::Integrity(r) => write!(f, "comparison artifact integrity: {r}"),
            Self::Source { side, problem } => write!(f, "comparison {side:?} source: {problem}"),
        }
    }
}
impl std::error::Error for ComparisonArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Artifact(e) => Some(e),
            Self::Recipe(e) => Some(e),
            Self::Source {
                problem: SourceProblem::Invalid(e),
                ..
            } => Some(e),
            _ => None,
        }
    }
}
impl From<RunSetError> for ComparisonArchiveError {
    fn from(e: RunSetError) -> Self {
        Self::Artifact(e)
    }
}
impl From<RecipeError> for ComparisonArchiveError {
    fn from(e: RecipeError) -> Self {
        Self::Recipe(e)
    }
}

type Result<T> = std::result::Result<T, ComparisonArchiveError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SelectedMember {
    pub member_id: String,
    /// Ordered explicit source ordinals and exact analysis-v1 identities.
    pub selections: Vec<SelectionIdentity>,
}

/// Immutable intent; source existence/identity is checked by `compute`, not `open`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SavedRecipe {
    pub analysis_version: u32,
    pub recipe: Recipe,
    /// Full immutable M2 revision; its members retain terminal archive file pins.
    pub run_set: SavedRunSet,
    pub run_set_sha256: [u8; 32],
    pub reference: SelectedMember,
    pub candidate: SelectedMember,
}
impl SavedRecipe {
    /// Capture explicit publications from closed, pinned native sources only.
    pub fn capture(
        root: &Path,
        run_set: RunSetRevision,
        reference_member: &str,
        candidate_member: &str,
        reference_ordinals: &[usize],
        candidate_ordinals: &[usize],
        recipe: Recipe,
    ) -> Result<Self> {
        Self::capture_source(
            root,
            SavedRunSet::Native(run_set),
            reference_member,
            candidate_member,
            reference_ordinals,
            candidate_ordinals,
            recipe,
        )
    }
    /// Capture exact publications through the source variant's existing owner.
    pub fn capture_source(
        root: &Path,
        run_set: SavedRunSet,
        reference_member: &str,
        candidate_member: &str,
        reference_ordinals: &[usize],
        candidate_ordinals: &[usize],
        recipe: Recipe,
    ) -> Result<Self> {
        recipe.validate()?;
        run_set.validate()?;
        let mut selected_bytes = 0;
        let reference = capture_member(
            root,
            &run_set,
            reference_member,
            reference_ordinals,
            Side::Reference,
            &mut selected_bytes,
        )?;
        let candidate = capture_member(
            root,
            &run_set,
            candidate_member,
            candidate_ordinals,
            Side::Candidate,
            &mut selected_bytes,
        )?;
        let value = Self {
            analysis_version: ANALYSIS_VERSION,
            recipe,
            run_set_sha256: run_set.sha256()?,
            run_set,
            reference,
            candidate,
        };
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        analysis_version(self.analysis_version)?;
        self.recipe.validate()?;
        self.run_set.validate()?;
        if self.run_set_sha256 != self.run_set.sha256()? {
            return Err(ComparisonArchiveError::Integrity("run-set revision digest"));
        }
        for selected in [&self.reference, &self.candidate] {
            validate_selected(&self.run_set, selected)?;
        }
        Ok(())
    }
    pub fn sha256(&self) -> Result<[u8; 32]> {
        self.validate()?;
        digest(self)
    }
    pub fn save_new(&self, path: &Path) -> Result<()> {
        self.validate()?;
        save(
            path,
            ArtifactKind::Recipe,
            self,
            self.run_set.schema_version(),
        )
    }
    /// Decode/hash validation only; no archive, worker or model is opened.
    pub fn open(path: &Path) -> Result<Self> {
        let envelope: Envelope<Self> = read_envelope(path)?;
        validate_envelope(
            &envelope,
            ArtifactKind::Recipe,
            envelope.payload.run_set.schema_version(),
        )?;
        envelope.payload.validate()?;
        Ok(envelope.payload)
    }
    /// Reopen exact pinned sources under the existing combined64MiB field budget.
    pub(super) fn sources(&self, root: &Path) -> Result<[Vec<Selection>; 2]> {
        self.validate()?;
        let mut selected_bytes = 0;
        let reference = load_selected(
            root,
            &self.run_set,
            &self.reference,
            Side::Reference,
            &mut selected_bytes,
        )?;
        let candidate = load_selected(
            root,
            &self.run_set,
            &self.candidate,
            Side::Candidate,
            &mut selected_bytes,
        )?;
        Ok([reference, candidate])
    }
    pub fn compute(&self, root: &Path) -> Result<VerifiedComparison> {
        let [reference, candidate] = self.sources(root)?;
        let coverage = align(&reference, &candidate, self.recipe)?;
        let record = ComparisonRecord {
            recipe: self.clone(),
            coverage,
        };
        record.validate()?;
        Ok(VerifiedComparison { record })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ComparisonRecord {
    recipe: SavedRecipe,
    coverage: Coverage,
}
impl ComparisonRecord {
    fn validate(&self) -> Result<()> {
        self.recipe.validate()?;
        analysis_version(self.coverage.analysis_version)?;
        if self.coverage.recipe != self.recipe.recipe {
            return Err(ComparisonArchiveError::Invalid("result recipe binding"));
        }
        for pair in &self.coverage.pairs {
            analysis_version(pair.analysis_version)?;
            if pair.recipe != self.recipe.recipe {
                return Err(ComparisonArchiveError::Invalid("pair recipe binding"));
            }
            if let Ok(field) = &pair.outcome {
                validate_field(field, self.recipe.recipe)?;
            }
        }
        let reference = self
            .coverage
            .pairs
            .iter()
            .map(|p| &p.reference)
            .chain(self.coverage.reference_unmatched.iter())
            .collect::<Vec<_>>();
        let candidate = self
            .coverage
            .pairs
            .iter()
            .map(|p| &p.candidate)
            .chain(self.coverage.candidate_unmatched.iter())
            .collect::<Vec<_>>();
        for (selected, actual) in [
            (&self.recipe.reference, reference),
            (&self.recipe.candidate, candidate),
        ] {
            if selected.selections.len() != actual.len()
                || selected
                    .selections
                    .iter()
                    .any(|s| actual.iter().filter(|a| ***a == *s).count() != 1)
            {
                return Err(ComparisonArchiveError::Invalid("result selection coverage"));
            }
        }
        Ok(())
    }
}

/// A decoded result remains unverified until exact source recomputation succeeds.
pub struct UnverifiedComparison {
    record: ComparisonRecord,
}
impl UnverifiedComparison {
    pub fn open(path: &Path) -> Result<Self> {
        let envelope: Envelope<ComparisonRecord> = read_envelope(path)?;
        validate_envelope(
            &envelope,
            ArtifactKind::Result,
            envelope.payload.recipe.run_set.schema_version(),
        )?;
        envelope.payload.validate()?;
        Ok(Self {
            record: envelope.payload,
        })
    }
    pub fn recipe(&self) -> &SavedRecipe {
        &self.record.recipe
    }
    pub fn recorded_coverage(&self) -> &Coverage {
        &self.record.coverage
    }
    pub fn verify(&self, root: &Path) -> Result<VerifiedComparison> {
        let recomputed = self.record.recipe.compute(root)?;
        if digest(&self.record.coverage)? != digest(&recomputed.record.coverage)? {
            return Err(ComparisonArchiveError::Integrity(
                "result differs from exact source recomputation",
            ));
        }
        Ok(recomputed)
    }
}
/// Verified against exact archive pins in this operation; sources remain caller-owned.
/// This is derived analysis, never accepted-step or model-accuracy qualification.
pub struct VerifiedComparison {
    record: ComparisonRecord,
}
impl VerifiedComparison {
    pub fn recipe(&self) -> &SavedRecipe {
        &self.record.recipe
    }
    pub fn coverage(&self) -> &Coverage {
        &self.record.coverage
    }
    pub fn sha256(&self) -> Result<[u8; 32]> {
        self.record.validate()?;
        digest(&self.record)
    }
    pub fn save_new(&self, path: &Path) -> Result<()> {
        self.record.validate()?;
        save(
            path,
            ArtifactKind::Result,
            &self.record,
            self.record.recipe.run_set.schema_version(),
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ArtifactKind {
    Recipe,
    Result,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<T> {
    schema_version: u32,
    kind: ArtifactKind,
    sha256: [u8; 32],
    payload: T,
}
fn save<T: Serialize>(
    path: &Path,
    kind: ArtifactKind,
    payload: &T,
    source_version: u32,
) -> Result<()> {
    let envelope = Envelope {
        schema_version: source_version,
        kind,
        sha256: digest(payload)?,
        payload,
    };
    publish_new(
        path,
        &serde_json::to_vec(&envelope).map_err(RunSetError::from)?,
        MAX_DOCUMENT_BYTES,
    )?;
    Ok(())
}
fn read_envelope<T: DeserializeOwned>(path: &Path) -> Result<Envelope<T>> {
    match read_versioned(path, SCHEMA_VERSION, MAX_DOCUMENT_BYTES) {
        Ok(envelope) => Ok(envelope),
        Err(RunSetError::Unsupported("manifest version")) => {
            Ok(read_versioned(path, 2, MAX_DOCUMENT_BYTES)?)
        }
        Err(error) => Err(error.into()),
    }
}
fn validate_envelope<T: Serialize>(
    envelope: &Envelope<T>,
    kind: ArtifactKind,
    source_version: u32,
) -> Result<()> {
    if envelope.schema_version != source_version {
        return Err(ComparisonArchiveError::Unsupported(
            "artifact version for source profile",
        ));
    }
    if envelope.kind != kind {
        return Err(ComparisonArchiveError::Invalid("artifact kind"));
    }
    if envelope.sha256 != digest(&envelope.payload)? {
        return Err(ComparisonArchiveError::Integrity("artifact payload digest"));
    }
    Ok(())
}
fn digest<T: Serialize>(value: &T) -> Result<[u8; 32]> {
    Ok(Sha256::digest(serde_json::to_vec(value).map_err(RunSetError::from)?).into())
}
fn analysis_version(version: u32) -> Result<()> {
    if version != ANALYSIS_VERSION {
        Err(ComparisonArchiveError::Unsupported("analysis version"))
    } else {
        Ok(())
    }
}
fn member<'a>(set: &'a SavedRunSet, id: &str) -> Result<SavedMember<'a>> {
    set.member(id)
        .ok_or(ComparisonArchiveError::Invalid("comparison member missing"))
}
fn validate_ordinals(ordinals: &[usize]) -> Result<()> {
    if ordinals.is_empty()
        || ordinals.len() > MAX_SELECTIONS_PER_SOURCE
        || ordinals.windows(2).any(|w| w[0] >= w[1])
    {
        return Err(ComparisonArchiveError::Invalid(
            "ordered bounded selections",
        ));
    }
    Ok(())
}
fn validate_selected(set: &SavedRunSet, selected: &SelectedMember) -> Result<()> {
    let m = member(set, &selected.member_id)?;
    let attempt = m
        .attempt()
        .ok_or(ComparisonArchiveError::Invalid("selected attempt missing"))?;
    let pin = attempt
        .finalized
        .ok_or(ComparisonArchiveError::Invalid("selected attempt unpinned"))?;
    validate_ordinals(
        &selected
            .selections
            .iter()
            .map(|s| s.ordinal)
            .collect::<Vec<_>>(),
    )?;
    if selected.selections.iter().any(|s| {
        s.ordinal >= pin.recorded_count
            || s.run_id != attempt.expected_run_id
            || s.source_revision != attempt.expected_source_revision
    }) {
        return Err(ComparisonArchiveError::Invalid(
            "selected publication binding",
        ));
    }
    Ok(())
}
fn open_source(
    root: &Path,
    m: SavedMember<'_>,
    side: Side,
) -> Result<rheon::recording::RecordingReader> {
    let problem = match m.open(root) {
        MemberState::Ready(reader) => return Ok(*reader),
        MemberState::Missing => SourceProblem::Missing,
        MemberState::Incomplete { .. } => SourceProblem::Incomplete,
        MemberState::Unpinned { .. } => SourceProblem::Unpinned,
        MemberState::NotRun => SourceProblem::NotRun,
        MemberState::Unsupported(r) => SourceProblem::Unsupported(r),
        MemberState::Invalid(e) => SourceProblem::Invalid(e),
    };
    Err(ComparisonArchiveError::Source { side, problem })
}
fn select(
    reader: &rheon::recording::RecordingReader,
    ordinal: usize,
    side: Side,
    selected_bytes: &mut usize,
) -> Result<Selection> {
    let selected =
        Selection::read(reader, ordinal).map_err(|e| ComparisonArchiveError::Source {
            side,
            problem: SourceProblem::Invalid(RunSetError::Recording(e)),
        })?;
    let bytes = selected
        .publication()
        .fields
        .iter()
        .try_fold(0usize, |total, f| total.checked_add(f.data.len()))
        .ok_or(ComparisonArchiveError::Invalid("selected field bytes"))?;
    *selected_bytes = selected_bytes
        .checked_add(bytes)
        .ok_or(ComparisonArchiveError::Invalid("selected field bytes"))?;
    if *selected_bytes > MAX_SELECTED_FIELD_BYTES {
        return Err(ComparisonArchiveError::Invalid(
            "selected field byte budget",
        ));
    }
    Ok(selected)
}
fn capture_member(
    root: &Path,
    set: &SavedRunSet,
    id: &str,
    ordinals: &[usize],
    side: Side,
    selected_bytes: &mut usize,
) -> Result<SelectedMember> {
    validate_ordinals(ordinals)?;
    let reader = open_source(root, member(set, id)?, side)?;
    let mut selections = Vec::new();
    for ordinal in ordinals {
        selections.push(
            select(&reader, *ordinal, side, selected_bytes)?
                .identity()
                .clone(),
        );
    }
    Ok(SelectedMember {
        member_id: id.into(),
        selections,
    })
}
fn load_selected(
    root: &Path,
    set: &SavedRunSet,
    pin: &SelectedMember,
    side: Side,
    selected_bytes: &mut usize,
) -> Result<Vec<Selection>> {
    let reader = open_source(root, member(set, &pin.member_id)?, side)?;
    let mut selections = Vec::new();
    for expected in &pin.selections {
        let actual = select(&reader, expected.ordinal, side, selected_bytes)?;
        if actual.identity() != expected {
            return Err(ComparisonArchiveError::Source {
                side,
                problem: SourceProblem::SelectionChanged,
            });
        }
        selections.push(actual);
    }
    Ok(selections)
}
fn validate_field(field: &FieldComparison, recipe: Recipe) -> Result<()> {
    let m = field.metrics;
    if m.support_count == 0
        || [m.l2, m.rmse, m.linf]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
        || m.relative_l2.is_some_and(|v| !v.is_finite() || v < 0.0)
        || !field.color_bounds.min.is_finite()
        || !field.color_bounds.max.is_finite()
        || field.color_bounds.min > field.color_bounds.max
    {
        return Err(ComparisonArchiveError::Invalid(
            "finite result metrics and bounds",
        ));
    }
    let expected_units = match recipe.field {
        FieldRole::Tracer => "1",
        FieldRole::IntervalPressure => "Pa",
        FieldRole::VelocityDivergence => "1/s",
        _ => "m/s",
    };
    if field.units != expected_units {
        return Err(ComparisonArchiveError::Invalid("result units"));
    }
    if recipe.field == FieldRole::IntervalPressure {
        let gauge = field
            .gauge
            .ok_or(ComparisonArchiveError::Invalid("pressure gauge report"))?;
        if !gauge.removed_offset_pa.is_finite()
            || gauge.diagnostic != (recipe.pressure == PressurePolicy::RemoveMeanOffsetDiagnostic)
            || (!gauge.diagnostic
                && (!gauge.reference_canonical
                    || !gauge.candidate_canonical
                    || gauge.removed_offset_pa != 0.0))
        {
            return Err(ComparisonArchiveError::Invalid("pressure gauge report"));
        }
    } else if field.gauge.is_some() {
        return Err(ComparisonArchiveError::Invalid("nonpressure gauge report"));
    }
    Ok(())
}
