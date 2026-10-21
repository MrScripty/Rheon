# T5 desktop force custody implementation admission

Base8148fa239a60502a1245d5a69d0eeb8ceb905c16. Parent now explicitly assigns minimal
Inspector begin/record interfaces and desktop-only helpers to this UI owner.
Write set: main Request/freeze/launch/rerun/run_material; carrier_execution.rs;
inspector.rs recorder methods only; run_sets.rs versioned native intent/binding;
scene_ui.rs retained force projection; scene_view.rs historical display context;
test fixture Request literal updates required to compile, no test execution unless
individually inspected and pure. Configured/session owner source/core untouched.

Backend a4326a78a747bc868874fd16e434d780fd5a6d35 fetched exactly; diff from43ec505
is only admission review/inventory. Core source remains byte-identical. No copied,
refreshed or claimed backend admission for this changed desktop source closure.

Request adds per-track optional immutable ForceSnapshot values. Freeze before Worker/output admission;
None retains exact legacy methods/recording; Some (including empty) preserves scene
metadata and uses force-aware adapters/publication wrappers. Native saved comparisons
may hold per-member scenes: dispatch and recording use each member's snapshot, never
substitute the current draft or silently share a differing baseline. Material retains
snapshot in Request/history and core recorded material metadata. Draft/Retained scene
selection uses the retained document; forces selectable read-only in Retained, no
handles/edit; unknown legacy force metadata remains unknown, not inferred.

Inspector existing legacy entry points delegate with None; new optional-snapshot
begin/record methods preserve identity, recorder, reader attach, selection and error
owners. Build RecordingRun.with_force_fields before StepRecorder::create; matching
constructor/accepted publication wrappers verify the same expected snapshot. Rebuild
actual run header with same force binding before existing config equality checks.
No projection/archive UI or core recording change; no new worker/session owner.

FrozenSpec version1 remains byte/hash compatible: NativeCarrierSpec unchanged;
new optional ForceScene field omitted when None. Force-bearing FrozenSpec uses own
schema_version2 (global manifest/version1 remains unchanged), domain-separated
SHA256 over length-delimited legacy spec bytes + canonical binary force metadata.
Version1 requires None, version2 requires Some(valid scene), other/mixed versions
refuse. Bound original bytes before parsing, preserve duplicate-key rejection.
Native intent declaration version follows frozen spec, run header force digest and
selected publications must match. New comparison difference explicitly identifies
force metadata changes; reuse existing D1 paths. Rerun clones each stored scene and
preserves historical hashes, without automatic migration or current-draft fallback.

Configured/provider/session/transport/free-surface authored requests refuse before
Worker/output creation. Existing immutable legacy reruns remain independent of draft.
Native carrier/emitter and particle-material dispatch use canonical backend methods;
no alternative evaluator, approximation, density conversion, gravity or dt policy.

UI force launch stays visibly gated until separately authorized bounded integration
execution validates source custody. No numerical/SQLite/model tests in this turn.
Pure tests: literal FrozenSpec v1 golden encoding/hash, v2 canonical metadata roundtrip,
mutation/order/disabled/units/signedzero/version/unknown-field/oversize refusal; pure
Request metadata/support eligibility/freeze immutability + Scene retained projection.
Inspect entire helper paths, list-only and one full --exact name per execution. Any
solver/recorder/filesystem fixture is individually ignored/prepared only; no broad
filter/fallback. Compile/production Clippy/fmt + independent source review. Preserve
all earlier native/evidence/branches; rollback distinct incremental commit only.

Reviewed refinements: Request stores a bounded per-track Vec<Option<ForceSnapshot>>
(empty vector means all legacy; otherwise exactly methods.len). One fields_for_track
resolver drives steps, recorder setup/append and retained projection. Explicit None
in a saved member never falls back to another member/current draft. Freeze every
member before Worker/output, compare exact canonical bytes with stored intent.
Retained binding includes track identity and resets on each new Request; changing
track/document clears selection even when numeric IDs coincide. Resolve follow-live track before tree/properties and cache that track’s frame for
viewport composition. Timeline track intents project consistently on the next
paint; retained properties are read-only and labelled.

All production launch/rerun routes remain gated before worker/filesystem admission.
A private cfg(test) Request admission flag may bypass only that temporary gate for
individually ignored, reviewed, separately authorized future integration probes.
No such workload is executed now. Version2 metadata differences use canonical bytes
including signedzero/None-vs-empty; run_sets/source.rs gains one exhaustive label.
Nested version failures use frozen-spec-specific errors, never trigger configured
manifest fallback. Inspector verifies expected force digest before create; None
rejects bound runs, Some matches exactly and must not double-bind an existing run.
