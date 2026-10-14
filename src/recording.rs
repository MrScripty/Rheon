//! Disk-backed accepted publications. Version 1 is native, local, single-writer,
//! keep-all/stop-at-limit storage. This preserves inspection fields, not complete
//! integrator state. See `docs/api/recording.md` for the SQLite consumer contract.
mod material;
use crate::{GridGeometry, SessionIdentity, SnapshotStage};
pub use material::{
    RecordedMaterial, material_publication, material_run, material_snapshot_publication,
    material_snapshot_run,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension, limits::Limit, params};
use sha2::{Digest, Sha256};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
    time::Duration,
};

const VERSION: i64 = 1;
const PAGE_BYTES: u64 = 4096;
const MAX_FIELDS: usize = 32;
const MAX_TEXT: usize = 16 * 1024;
const SCHEMA: &str = "
PRAGMA user_version=1;
CREATE TABLE run (id INTEGER PRIMARY KEY CHECK(id=1), run_id BLOB NOT NULL,
 revision TEXT NOT NULL, solver TEXT NOT NULL, configuration TEXT NOT NULL,
 geometry BLOB NOT NULL, provenance TEXT NOT NULL, digest BLOB NOT NULL,
 max_database_bytes INTEGER NOT NULL, max_record_bytes INTEGER NOT NULL,
 max_records INTEGER NOT NULL, retention TEXT NOT NULL CHECK(retention='keep_all_stop'));
CREATE TABLE lifecycle (id INTEGER PRIMARY KEY CHECK(id=1), outcome TEXT NOT NULL,
 detail TEXT NOT NULL, accepted_generation BLOB, digest BLOB NOT NULL);
CREATE TABLE steps (ordinal INTEGER PRIMARY KEY, generation BLOB NOT NULL UNIQUE,
 time BLOB NOT NULL, dt BLOB, stage INTEGER NOT NULL, controls TEXT NOT NULL,
 field_count INTEGER NOT NULL, payload_bytes INTEGER NOT NULL, digest BLOB NOT NULL);
CREATE TABLE fields (ordinal INTEGER NOT NULL REFERENCES steps(ordinal),
 field_index INTEGER NOT NULL, name TEXT NOT NULL, units TEXT NOT NULL,
 association TEXT NOT NULL, stage TEXT NOT NULL, evaluation_time BLOB,
 dimensions BLOB NOT NULL, components INTEGER NOT NULL, scalar_type TEXT NOT NULL,
 data BLOB NOT NULL, PRIMARY KEY(ordinal,field_index), UNIQUE(ordinal,name));";

/// Logical database and per-publication payload limits, independent of physics.
/// SQLite cache/allocator/process overhead is excluded. Rollback journal can
/// temporarily use additional disk; no eviction, automatic cleanup or resume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordingLimits {
    pub max_database_bytes: u64,
    pub max_record_bytes: usize,
    /// Includes an optional constructor publication.
    pub max_records: usize,
}
impl Default for RecordingLimits {
    fn default() -> Self {
        Self {
            max_database_bytes: 1024 * 1024 * 1024,
            max_record_bytes: 64 * 1024 * 1024,
            max_records: 10001,
        }
    }
}
impl RecordingLimits {
    /// Validate supported native archive limits before admitting resource use.
    pub fn validate(self) -> Result<(), RecordingError> {
        if self.max_database_bytes < 16 * PAGE_BYTES
            || self.max_database_bytes > i64::MAX as u64
            || self.max_record_bytes == 0
            || self.max_record_bytes > 64 * 1024 * 1024
            || self.max_records == 0
            || i64::try_from(self.max_records).is_err()
        {
            return Err(RecordingError::Invalid("recording limits"));
        }
        Ok(())
    }
}

/// Actual producer configuration/geometry plus caller-declared provenance.
/// Opaque configuration text is retained verbatim; it is not executable input.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordingRun {
    pub identity: SessionIdentity,
    pub solver: String,
    pub configuration: String,
    pub counts: [u64; 3],
    pub spacing: [f64; 3],
    pub origin: [f64; 3],
    pub provenance: String,
}
impl RecordingRun {
    fn validate(&self) -> Result<(), RecordingError> {
        if !self.identity.is_valid() {
            return Err(RecordingError::Invalid("run identity"));
        }
        for value in [&self.solver, &self.configuration, &self.provenance] {
            valid_text(value)?;
        }
        GridGeometry::new(self.counts, self.spacing, self.origin)
            .map_err(|_| RecordingError::Invalid("recorded geometry"))?;
        Ok(())
    }
    fn geometry(&self) -> Vec<u8> {
        self.counts
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .chain(self.spacing.iter().flat_map(|v| v.to_le_bytes()))
            .chain(self.origin.iter().flat_map(|v| v.to_le_bytes()))
            .collect()
    }
    fn digest(&self, limits: RecordingLimits) -> Vec<u8> {
        let mut h = Sha256::new();
        h.update(self.identity.run_id.to_le_bytes());
        for v in [
            &self.identity.source_revision,
            &self.solver,
            &self.configuration,
            &self.provenance,
        ] {
            hash_text(&mut h, v);
        }
        h.update(self.geometry());
        h.update(limits.max_database_bytes.to_le_bytes());
        h.update((limits.max_record_bytes as u64).to_le_bytes());
        h.update((limits.max_records as u64).to_le_bytes());
        hash_text(&mut h, "keep_all_stop");
        h.finalize().to_vec()
    }
}

/// Little-endian numerical field; dimensions use x-fast ordering. Association
/// and stage explicitly describe the producer's sample locations and times.
#[derive(Debug, PartialEq)]
pub struct RecordedField {
    pub name: String,
    pub units: String,
    pub association: String,
    pub stage: String,
    pub evaluation_time: Option<f64>,
    pub dimensions: [u64; 3],
    pub components: usize,
    /// Version 1 supports f64, f32, u64 and u8 only.
    pub scalar_type: String,
    pub data: Vec<u8>,
}
impl RecordedField {
    fn validate(&self) -> Result<usize, RecordingError> {
        for v in [
            &self.name,
            &self.units,
            &self.association,
            &self.stage,
            &self.scalar_type,
        ] {
            valid_text(v)?;
        }
        let width = match self.scalar_type.as_str() {
            "f64" | "u64" => 8,
            "f32" => 4,
            "u8" => 1,
            _ => return Err(RecordingError::Unsupported("scalar type")),
        };
        if !(1..=16).contains(&self.components) {
            return Err(RecordingError::Invalid("components"));
        }
        let mut n = self.components;
        for dim in self.dimensions {
            n = n
                .checked_mul(
                    usize::try_from(dim).map_err(|_| RecordingError::Invalid("dimensions"))?,
                )
                .ok_or(RecordingError::Invalid("field size overflow"))?;
        }
        if n.checked_mul(width) != Some(self.data.len()) {
            return Err(RecordingError::Invalid("field shape"));
        }
        if self
            .evaluation_time
            .is_some_and(|t| !t.is_finite() || t < 0.0)
        {
            return Err(RecordingError::Invalid("evaluation time"));
        }
        if (self.scalar_type == "f64"
            && self
                .data
                .chunks_exact(8)
                .any(|b| !f64::from_le_bytes(b.try_into().unwrap()).is_finite()))
            || (self.scalar_type == "f32"
                && self
                    .data
                    .chunks_exact(4)
                    .any(|b| !f32::from_le_bytes(b.try_into().unwrap()).is_finite()))
        {
            return Err(RecordingError::Invalid("nonfinite field"));
        }
        Ok(self.data.len()
            + self.name.len()
            + self.units.len()
            + self.association.len()
            + self.stage.len()
            + self.scalar_type.len())
    }
}

/// One complete producer publication. Missing fields stay absent. Fields may
/// have an evaluation time earlier than the publication's accepted time.
#[derive(Debug)]
pub struct RecordedStep {
    pub ordinal: usize,
    pub generation: u64,
    pub time: f64,
    pub dt: Option<f64>,
    pub stage: SnapshotStage,
    pub controls: String,
    pub fields: Vec<RecordedField>,
}
impl RecordedStep {
    pub fn field(&self, name: &str) -> Option<&RecordedField> {
        self.fields.iter().find(|f| f.name == name)
    }
    fn payload_bytes(&self) -> Result<usize, RecordingError> {
        valid_text(&self.controls)?;
        if self.fields.is_empty()
            || self.fields.len() > MAX_FIELDS
            || !self.time.is_finite()
            || self.time < 0.0
            || match self.stage {
                SnapshotStage::Constructor => {
                    self.dt.is_some() || self.generation != 0 || self.time != 0.0
                }
                SnapshotStage::Accepted => {
                    self.generation == 0
                        || self
                            .dt
                            .is_none_or(|dt| !dt.is_finite() || dt <= 0.0 || dt > self.time)
                }
            }
        {
            return Err(RecordingError::Invalid("publication metadata"));
        }
        let mut total = self.controls.len();
        for (i, field) in self.fields.iter().enumerate() {
            if self.fields[..i].iter().any(|f| f.name == field.name)
                || field.evaluation_time.is_some_and(|t| t > self.time)
            {
                return Err(RecordingError::Invalid(
                    "duplicate field or future evaluation",
                ));
            }
            total = total
                .checked_add(field.validate()?)
                .ok_or(RecordingError::Limit("payload overflow"))?;
        }
        Ok(total)
    }
    fn digest(&self, run_digest: &[u8]) -> Vec<u8> {
        let mut h = Sha256::new();
        h.update(run_digest);
        h.update((self.ordinal as u64).to_le_bytes());
        h.update(self.generation.to_le_bytes());
        h.update(self.time.to_le_bytes());
        hash_optional(&mut h, self.dt);
        h.update([stage_id(self.stage) as u8]);
        hash_text(&mut h, &self.controls);
        h.update((self.fields.len() as u64).to_le_bytes());
        for f in &self.fields {
            for v in [&f.name, &f.units, &f.association, &f.stage, &f.scalar_type] {
                hash_text(&mut h, v);
            }
            hash_optional(&mut h, f.evaluation_time);
            for d in f.dimensions {
                h.update(d.to_le_bytes());
            }
            h.update((f.components as u64).to_le_bytes());
            h.update((f.data.len() as u64).to_le_bytes());
            h.update(&f.data);
        }
        h.finalize().to_vec()
    }
}

#[derive(Debug)]
pub enum RecordingError {
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    Invalid(&'static str),
    Unsupported(&'static str),
    Limit(&'static str),
    Missing(usize),
    Integrity(&'static str),
    Closed,
}
impl fmt::Display for RecordingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "numerical recording: {self:?}")
    }
}
impl std::error::Error for RecordingError {}
impl From<std::io::Error> for RecordingError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for RecordingError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

/// Terminal outcome is separate from step publication. `active` on cold reopen
/// means incomplete/unknown: steps beyond the committed index may be unrecorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingStatus {
    pub outcome: String,
    pub detail: String,
    pub accepted_generation: Option<u64>,
}

/// Exclusive fresh archive owner. Append failures poison admission; never retry
/// an uncertain publication on this writer. Drop deliberately leaves `active`.
pub struct StepRecorder {
    connection: Connection,
    path: PathBuf,
    run: RecordingRun,
    limits: RecordingLimits,
    count: usize,
    last: Option<(u64, f64)>,
    closed: bool,
}
impl StepRecorder {
    /// Create a fresh local database, never replace an existing file. The caller
    /// must own/protect the path and ancestors from concurrent replacement.
    pub fn create(
        path: &Path,
        run: RecordingRun,
        limits: RecordingLimits,
    ) -> Result<Self, RecordingError> {
        limits.validate()?;
        run.validate()?;
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.sync_all()?;
        let connection = connect(path, limits)?;
        connection.pragma_update(None, "page_size", PAGE_BYTES)?;
        let pages = limits.max_database_bytes / PAGE_BYTES;
        connection.pragma_update(None, "max_page_count", pages)?;
        let actual: u64 = connection.pragma_query_value(None, "max_page_count", |r| r.get(0))?;
        if actual != pages {
            return Err(RecordingError::Limit("database pages"));
        }
        connection.execute_batch(
            "PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;",
        )?;
        connection.execute_batch("BEGIN IMMEDIATE")?;
        connection.execute_batch(SCHEMA)?;
        connection.execute(
            "INSERT INTO run VALUES(1,?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'keep_all_stop')",
            params![
                run.identity.run_id.to_le_bytes().as_slice(),
                run.identity.source_revision,
                run.solver,
                run.configuration,
                run.geometry(),
                run.provenance,
                run.digest(limits),
                limits.max_database_bytes as i64,
                limits.max_record_bytes as i64,
                limits.max_records as i64
            ],
        )?;
        let detail = "producer has not declared completion; tail may be unrecorded";
        connection.execute(
            "INSERT INTO lifecycle VALUES(1,'active',?1,NULL,?2)",
            params![
                detail,
                status_digest(&run.digest(limits), "active", detail, None)
            ],
        )?;
        connection.execute_batch("COMMIT")?;
        Ok(Self {
            connection,
            path: path.to_owned(),
            run,
            limits,
            count: 0,
            last: None,
            closed: false,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn run(&self) -> &RecordingRun {
        &self.run
    }
    pub fn limits(&self) -> RecordingLimits {
        self.limits
    }
    /// Acknowledged commits only. After an append error, use a fresh reader to
    /// reconcile the actual committed prefix before making a completion claim.
    pub fn recorded_count(&self) -> usize {
        self.count
    }
    /// Commit exactly one consistent accepted publication (or the first
    /// constructor). Require adjacent generations: omitted accepted steps are
    /// rejected, never silently relabelled as a continuous record.
    pub fn append(&mut self, step: &RecordedStep) -> Result<(), RecordingError> {
        if self.closed {
            return Err(RecordingError::Closed);
        }
        let result = self.append_inner(step);
        if result.is_err() {
            self.closed = true;
        }
        result
    }
    fn append_inner(&mut self, step: &RecordedStep) -> Result<(), RecordingError> {
        if self.count >= self.limits.max_records {
            return Err(RecordingError::Limit("record count"));
        }
        let bytes = step.payload_bytes()?;
        if bytes > self.limits.max_record_bytes {
            return Err(RecordingError::Limit("record payload"));
        }
        if step.ordinal != self.count {
            return Err(RecordingError::Invalid("ordinal"));
        }
        if let Some((g, t)) = self.last {
            if g.checked_add(1) != Some(step.generation)
                || step.time <= t
                || step.dt.is_none_or(|dt| {
                    (step.time - t - dt).abs() > 8.0 * f64::EPSILON * step.time.max(dt)
                })
            {
                return Err(RecordingError::Invalid("nonadjacent generation/time"));
            }
        } else if step.generation > 1 || (step.generation == 1 && step.dt != Some(step.time)) {
            return Err(RecordingError::Invalid("unrecorded initial steps"));
        }
        let digest = step.digest(&self.run.digest(self.limits));
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT INTO steps VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                step.ordinal as i64,
                step.generation.to_le_bytes().as_slice(),
                step.time.to_le_bytes().as_slice(),
                step.dt.map(f64::to_le_bytes).as_ref().map(|b| b.as_slice()),
                stage_id(step.stage),
                step.controls,
                step.fields.len() as i64,
                bytes as i64,
                digest
            ],
        )?;
        for (i, f) in step.fields.iter().enumerate() {
            let dimensions: Vec<u8> = f.dimensions.iter().flat_map(|d| d.to_le_bytes()).collect();
            tx.execute(
                "INSERT INTO fields VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    step.ordinal as i64,
                    i as i64,
                    f.name,
                    f.units,
                    f.association,
                    f.stage,
                    f.evaluation_time
                        .map(f64::to_le_bytes)
                        .as_ref()
                        .map(|b| b.as_slice()),
                    dimensions,
                    f.components as i64,
                    f.scalar_type,
                    f.data
                ],
            )?;
        }
        tx.commit()?;
        self.last = Some((step.generation, step.time));
        self.count += 1;
        Ok(())
    }
    /// End lifecycle without deleting data. This remains allowed after append
    /// failure to disclose the last accepted (possibly unrecorded) generation.
    /// If this also fails, cold reopen remains active/unknown; no success claim.
    pub fn finish(
        &mut self,
        outcome: &str,
        detail: &str,
        accepted_generation: Option<u64>,
    ) -> Result<(), RecordingError> {
        if !["completed", "cancelled", "failed"].contains(&outcome) {
            return Err(RecordingError::Invalid("terminal outcome"));
        }
        valid_text(detail)?;
        if outcome == "completed"
            && (self.count == 0 || accepted_generation != self.last.map(|v| v.0))
        {
            return Err(RecordingError::Invalid(
                "completion must cover accepted tail",
            ));
        }
        let changed=self.connection.execute("UPDATE lifecycle SET outcome=?1,detail=?2,accepted_generation=?3,digest=?4 WHERE id=1 AND outcome='active'",params![outcome,detail,accepted_generation.map(u64::to_le_bytes).as_ref().map(|b|b.as_slice()),status_digest(&self.run.digest(self.limits),outcome,detail,accepted_generation)])?;
        self.closed = true;
        if changed != 1 {
            return Err(RecordingError::Closed);
        }
        Ok(())
    }
}

/// Indexed reader. Retains metadata and a connection, never an in-memory step
/// index or unbounded frame cache. Each selection returns one bounded record.
pub struct RecordingReader {
    connection: Connection,
    run: RecordingRun,
    limits: RecordingLimits,
    recorded_limits: RecordingLimits,
}
impl RecordingReader {
    /// The caller owns a local path protected from replacement. Opens without
    /// create and allows SQLite hot-journal recovery; no writer resumption.
    pub fn open(path: &Path, limits: RecordingLimits) -> Result<Self, RecordingError> {
        limits.validate()?;
        let m = fs::symlink_metadata(path)?;
        if !m.is_file() || m.file_type().is_symlink() {
            return Err(RecordingError::Invalid("regular database required"));
        }
        if m.len() > limits.max_database_bytes {
            return Err(RecordingError::Limit("database bytes"));
        }
        let connection = connect(path, limits)?;
        let version: i64 = connection.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version != VERSION {
            return Err(RecordingError::Unsupported("archive version"));
        }
        connection.execute_batch("PRAGMA query_only=ON; PRAGMA trusted_schema=OFF;")?;
        let check: String = connection.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        if check != "ok" {
            return Err(RecordingError::Integrity("SQLite quick_check"));
        }
        let run = read_run(&connection)?;
        run.validate()?;
        let expected: Vec<u8> =
            connection.query_row("SELECT digest FROM run WHERE id=1", [], |r| r.get(0))?;
        let recorded_limits = read_limits(&connection)?;
        recorded_limits.validate()?;
        let retention: String =
            connection.query_row("SELECT retention FROM run WHERE id=1", [], |r| r.get(0))?;
        if retention != "keep_all_stop" {
            return Err(RecordingError::Unsupported("retention"));
        }
        if expected != run.digest(recorded_limits) {
            return Err(RecordingError::Integrity("run metadata"));
        }
        if m.len() > recorded_limits.max_database_bytes {
            return Err(RecordingError::Limit("recorded database bytes"));
        }
        let reader = Self {
            connection,
            run,
            limits,
            recorded_limits,
        };
        reader.count()?;
        reader.status()?;
        Ok(reader)
    }
    pub fn run(&self) -> &RecordingRun {
        &self.run
    }
    pub fn recorded_limits(&self) -> RecordingLimits {
        self.recorded_limits
    }
    pub fn count(&self) -> Result<usize, RecordingError> {
        let (n, last, first): (i64, Option<i64>, Option<i64>) = self.connection.query_row(
            "SELECT count(*),max(ordinal),min(ordinal) FROM steps",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let n = usize::try_from(n).map_err(|_| RecordingError::Integrity("count"))?;
        if n > self
            .limits
            .max_records
            .min(self.recorded_limits.max_records)
        {
            return Err(RecordingError::Limit("record count"));
        }
        if n != 0 && (last != Some(n as i64 - 1) || first != Some(0)) {
            return Err(RecordingError::Integrity("noncontiguous index"));
        }
        Ok(n)
    }
    pub fn status(&self) -> Result<RecordingStatus, RecordingError> {
        let (outcome, detail, g, expected): (String, String, Option<Vec<u8>>, Vec<u8>) =
            self.connection.query_row(
                "SELECT outcome,detail,accepted_generation,digest FROM lifecycle WHERE id=1",
                [],
                |r| {
                    Ok((
                        bounded_text(r, 0)?,
                        bounded_text(r, 1)?,
                        r.get(2)?,
                        r.get(3)?,
                    ))
                },
            )?;
        if !["active", "completed", "cancelled", "failed"].contains(&outcome.as_str()) {
            return Err(RecordingError::Integrity("lifecycle"));
        }
        valid_text(&detail)?;
        let accepted_generation = g.map(|b| number(&b).map(u64::from_le_bytes)).transpose()?;
        if expected
            != status_digest(
                &self.run.digest(self.recorded_limits),
                &outcome,
                &detail,
                accepted_generation,
            )
        {
            return Err(RecordingError::Integrity("lifecycle digest"));
        }
        Ok(RecordingStatus {
            outcome,
            detail,
            accepted_generation,
        })
    }
    pub fn select_generation(&self, generation: u64) -> Result<RecordedStep, RecordingError> {
        let ordinal: Option<i64> = self
            .connection
            .query_row(
                "SELECT ordinal FROM steps WHERE generation=?1",
                [generation.to_le_bytes().as_slice()],
                |r| r.get(0),
            )
            .optional()?;
        match ordinal {
            Some(n) => {
                self.select(usize::try_from(n).map_err(|_| RecordingError::Integrity("ordinal"))?)
            }
            None => Err(RecordingError::Invalid("generation not recorded")),
        }
    }
    pub fn select(&self, ordinal: usize) -> Result<RecordedStep, RecordingError> {
        // A read transaction binds row metadata and all fields to one publication.
        let _transaction = self.connection.unchecked_transaction()?;
        self.select_record(ordinal)
    }
    fn select_record(&self, ordinal: usize) -> Result<RecordedStep, RecordingError> {
        self.verify_header()?;
        let ordinal_i64 = i64::try_from(ordinal).map_err(|_| RecordingError::Missing(ordinal))?;
        let metadata=self.connection.query_row("SELECT generation,time,dt,stage,controls,field_count,payload_bytes,digest FROM steps WHERE ordinal=?1",[ordinal_i64],|r|Ok((r.get::<_,Vec<u8>>(0)?,r.get::<_,Vec<u8>>(1)?,r.get::<_,Option<Vec<u8>>>(2)?,r.get::<_,i64>(3)?,bounded_text(r,4)?,r.get::<_,i64>(5)?,r.get::<_,i64>(6)?,r.get::<_,Vec<u8>>(7)?))).optional()?.ok_or(RecordingError::Missing(ordinal))?;
        let (g, t, dt, stage, controls, count, bytes, expected) = metadata;
        if !(1..=MAX_FIELDS as i64).contains(&count)
            || bytes < 0
            || bytes
                > self
                    .limits
                    .max_record_bytes
                    .min(self.recorded_limits.max_record_bytes) as i64
        {
            return Err(RecordingError::Limit("record payload/fields"));
        }
        let aggregate:(i64,i64)=self.connection.query_row("SELECT count(*),coalesce(sum(length(data)+length(CAST(name AS BLOB))+length(CAST(units AS BLOB))+length(CAST(association AS BLOB))+length(CAST(stage AS BLOB))+length(CAST(scalar_type AS BLOB))),0) FROM fields WHERE ordinal=?1",[ordinal_i64],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if aggregate.0 != count || aggregate.1.checked_add(controls.len() as i64) != Some(bytes) {
            return Err(RecordingError::Integrity("field inventory"));
        }
        let mut fields = Vec::new();
        fields
            .try_reserve_exact(count as usize)
            .map_err(|_| RecordingError::Limit("allocation"))?;
        let mut query=self.connection.prepare("SELECT field_index,name,units,association,stage,evaluation_time,dimensions,components,scalar_type,data FROM fields WHERE ordinal=?1 ORDER BY field_index")?;
        let mut rows = query.query([ordinal_i64])?;
        while let Some(r) = rows.next()? {
            if r.get::<_, i64>(0)? != fields.len() as i64 {
                return Err(RecordingError::Integrity("field order"));
            }
            let d: Vec<u8> = r.get(6)?;
            if d.len() != 24 {
                return Err(RecordingError::Integrity("dimensions"));
            }
            let evaluation: Option<Vec<u8>> = r.get(5)?;
            let data = r
                .get_ref(9)?
                .as_blob()
                .map_err(|_| RecordingError::Integrity("field blob"))?;
            let mut owned = Vec::new();
            owned
                .try_reserve_exact(data.len())
                .map_err(|_| RecordingError::Limit("allocation"))?;
            owned.extend_from_slice(data);
            fields.push(RecordedField {
                name: bounded_text(r, 1)?,
                units: bounded_text(r, 2)?,
                association: bounded_text(r, 3)?,
                stage: bounded_text(r, 4)?,
                evaluation_time: evaluation
                    .map(|b| number(&b).map(f64::from_le_bytes))
                    .transpose()?,
                dimensions: std::array::from_fn(|i| {
                    u64::from_le_bytes(d[8 * i..8 * i + 8].try_into().unwrap())
                }),
                components: usize::try_from(r.get::<_, i64>(7)?)
                    .map_err(|_| RecordingError::Integrity("components"))?,
                scalar_type: bounded_text(r, 8)?,
                data: owned,
            });
        }
        let step = RecordedStep {
            ordinal,
            generation: u64::from_le_bytes(number(&g)?),
            time: f64::from_le_bytes(number(&t)?),
            dt: dt.map(|b| number(&b).map(f64::from_le_bytes)).transpose()?,
            stage: match stage {
                0 => SnapshotStage::Constructor,
                1 => SnapshotStage::Accepted,
                _ => return Err(RecordingError::Unsupported("publication stage")),
            },
            controls,
            fields,
        };
        if step.payload_bytes()? != bytes as usize
            || step.digest(&self.run.digest(self.recorded_limits)) != expected
        {
            return Err(RecordingError::Integrity("publication digest"));
        }
        Ok(step)
    }
    /// Structural integrity plus every publication hash and adjacent generation.
    /// O(number of records) time, one bounded publication in memory.
    pub fn verify(&self) -> Result<(), RecordingError> {
        // The inventory, fields and terminal tail must describe the same database
        // snapshot even if a producer is still active. A full audit may block
        // its writer; callers should pause the producer for long archives.
        let _transaction = self.connection.unchecked_transaction()?;
        self.verify_header()?;
        let result: String = self
            .connection
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        if result != "ok" {
            return Err(RecordingError::Integrity("SQLite integrity_check"));
        }
        let foreign_key_failure: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",
            [],
            |r| r.get(0),
        )?;
        if foreign_key_failure {
            return Err(RecordingError::Integrity("orphan fields"));
        }
        let mut last: Option<(u64, f64)> = None;
        let n = self.count()?;
        for ordinal in 0..n {
            let s = self.select_record(ordinal)?;
            if let Some((g, t)) = last {
                if g.checked_add(1) != Some(s.generation)
                    || s.time <= t
                    || s.dt.is_none_or(|dt| {
                        (s.time - t - dt).abs() > 8.0 * f64::EPSILON * s.time.max(dt)
                    })
                {
                    return Err(RecordingError::Integrity("step continuity"));
                }
            } else if s.generation > 1 || (s.generation == 1 && s.dt != Some(s.time)) {
                return Err(RecordingError::Integrity("initial gap"));
            }
            last = Some((s.generation, s.time));
        }
        let status = self.status()?;
        if status.outcome == "completed"
            && (n == 0 || status.accepted_generation != last.map(|v| v.0))
        {
            return Err(RecordingError::Integrity("completion tail"));
        }
        Ok(())
    }
    fn verify_header(&self) -> Result<(), RecordingError> {
        let version: i64 = self
            .connection
            .pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version != VERSION {
            return Err(RecordingError::Unsupported("archive version"));
        }
        let actual = read_run(&self.connection)?;
        let limits = read_limits(&self.connection)?;
        let (expected, retention): (Vec<u8>, String) =
            self.connection
                .query_row("SELECT digest,retention FROM run WHERE id=1", [], |r| {
                    Ok((r.get(0)?, bounded_text(r, 1)?))
                })?;
        if retention != "keep_all_stop" {
            return Err(RecordingError::Unsupported("retention"));
        }
        if actual != self.run
            || limits != self.recorded_limits
            || actual.digest(limits) != expected
            || self.run.digest(self.recorded_limits) != expected
        {
            return Err(RecordingError::Integrity("run metadata"));
        }
        Ok(())
    }
}
fn connect(path: &Path, limits: RecordingLimits) -> Result<Connection, RecordingError> {
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE;
    let c = Connection::open_with_flags(path, flags | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
    c.busy_timeout(Duration::from_millis(250))?;
    // Bounded page cache per connection; no memory mapped copy of the archive.
    c.execute_batch("PRAGMA cache_size=-1024; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;")?;
    c.set_limit(
        Limit::SQLITE_LIMIT_LENGTH,
        i32::try_from(limits.max_record_bytes + 4 * MAX_TEXT)
            .map_err(|_| RecordingError::Limit("SQLite length"))?,
    )?;
    Ok(c)
}
fn read_run(c: &Connection) -> Result<RecordingRun, RecordingError> {
    let (id, revision, solver, configuration, geometry, provenance): (
        Vec<u8>,
        String,
        String,
        String,
        Vec<u8>,
        String,
    ) = c.query_row(
        "SELECT run_id,revision,solver,configuration,geometry,provenance FROM run WHERE id=1",
        [],
        |r| {
            Ok((
                r.get(0)?,
                bounded_text(r, 1)?,
                bounded_text(r, 2)?,
                bounded_text(r, 3)?,
                r.get(4)?,
                bounded_text(r, 5)?,
            ))
        },
    )?;
    if geometry.len() != 72 {
        return Err(RecordingError::Integrity("geometry"));
    }
    Ok(RecordingRun {
        identity: SessionIdentity {
            run_id: u64::from_le_bytes(number(&id)?),
            source_revision: revision,
        },
        solver,
        configuration,
        counts: std::array::from_fn(|i| {
            u64::from_le_bytes(geometry[8 * i..8 * i + 8].try_into().unwrap())
        }),
        spacing: std::array::from_fn(|i| {
            f64::from_le_bytes(geometry[24 + 8 * i..32 + 8 * i].try_into().unwrap())
        }),
        origin: std::array::from_fn(|i| {
            f64::from_le_bytes(geometry[48 + 8 * i..56 + 8 * i].try_into().unwrap())
        }),
        provenance,
    })
}
fn valid_text(value: &str) -> Result<(), RecordingError> {
    if value.is_empty()
        || value.len() > MAX_TEXT
        || value
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        Err(RecordingError::Invalid("metadata text"))
    } else {
        Ok(())
    }
}
fn bounded_text(r: &rusqlite::Row<'_>, i: usize) -> rusqlite::Result<String> {
    let b = r.get_ref(i)?.as_str()?;
    if b.len() > MAX_TEXT {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(b.to_owned())
}
fn number(b: &[u8]) -> Result<[u8; 8], RecordingError> {
    b.try_into()
        .map_err(|_| RecordingError::Integrity("numeric metadata width"))
}
fn stage_id(stage: SnapshotStage) -> i64 {
    match stage {
        SnapshotStage::Constructor => 0,
        SnapshotStage::Accepted => 1,
    }
}
fn hash_text(h: &mut Sha256, v: &str) {
    h.update((v.len() as u64).to_le_bytes());
    h.update(v.as_bytes());
}
fn hash_optional(h: &mut Sha256, v: Option<f64>) {
    h.update([u8::from(v.is_some())]);
    if let Some(v) = v {
        h.update(v.to_le_bytes());
    }
}

fn read_limits(c: &Connection) -> Result<RecordingLimits, RecordingError> {
    let (database, record, count): (i64, i64, i64) = c.query_row(
        "SELECT max_database_bytes,max_record_bytes,max_records FROM run WHERE id=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    Ok(RecordingLimits {
        max_database_bytes: u64::try_from(database)
            .map_err(|_| RecordingError::Integrity("database limit"))?,
        max_record_bytes: usize::try_from(record)
            .map_err(|_| RecordingError::Integrity("record limit"))?,
        max_records: usize::try_from(count)
            .map_err(|_| RecordingError::Integrity("count limit"))?,
    })
}
fn status_digest(run: &[u8], outcome: &str, detail: &str, generation: Option<u64>) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(run);
    hash_text(&mut h, outcome);
    hash_text(&mut h, detail);
    h.update([u8::from(generation.is_some())]);
    if let Some(g) = generation {
        h.update(g.to_le_bytes());
    }
    h.finalize().to_vec()
}
