# Saved native specifications and run sets, version 1

`src/bin/desktop/run_sets.rs` owns immutable application intent and manifests over
existing [numerical SQLite recordings](recording.md). In this draft the module is
compiled through `tests/saved_run_set_contract.rs`; it is not yet registered in
the desktop. Integration and all merges belong to the configurable-solver lead.

The admitted first configuration is constructor-zero native sealed-box carrier
with Jacobi PCG v1 or SGS PCG v1, constant requested dt, bounded requested interval
count and an optional initial contiguous SmokeSource schedule. It preserves grid
counts, spacing and origin, density, every existing pressure/full-step tolerance,
simulation memory limit and recording budgets. Initial velocity/tracer are zero.
No model, viscosity, obstacles, external initial assets, emitter, adaptive stepping,
restart, or provider fallback is implied. Accepted provider recording and diagnostic
attempt storage need the lead's actual M1 contracts before integration.

`FrozenSpec` validates and content-identifies these controls. The composer creates
the existing Simulation with `spec.create_simulation()` and uses the same frozen
spec's `requested_dt_s` and `source_at(interval)` for execution. `bound_native_run`
checks actual constructor geometry/configuration/method and puts a versioned
full-spec SHA256 declaration in existing RecordingRun provenance. Source revision
and build strings remain caller-declared provenance. This binds intent; it does
not authenticate a producer or independently prove its source schedule was applied.
Recorded native fields and actual intervals remain the numerical observations.

`RunSetRevision` embeds exact frozen member specs, baseline, an optional parent
content identity and an ordered exact list of intentional differences. Editing
membership or intent creates a fresh revision; there is no mutable latest alias.
Retry uses a fresh attempt ID and archive path. The manifest contains no numerical
arrays, archive index/catalog, runtime handle or model-loading operation. The
existing recording owner remains authoritative for all fields and lifecycle data.

## Persistence and reopening

JSON uses explicit schema version 1, fixed typed fields, bounded inputs and exact
round-trip f64 values through the existing pinned serde_json float-roundtrip
feature. Spec and run-set digests hash their typed serialized payloads; these are
content identities, not signatures. Unknown versions return Unsupported; unknown
keys, variants, duplicates or malformed JSON fail decoding. Changed content with an
old digest returns Integrity. Files are limited to 1 MiB and sets to 64 members.
No migration, reinterpretation of saved Debug text, automatic recovery overwrite,
retention deletion or execution occurs on open.

`save_new` stages a complete bounded manifest in a fresh sibling file, syncs it,
publishes with a no-replace hard link, syncs the directory, removes only its own
staged name, then syncs the directory again. Existing destinations are never
replaced, including when publishers race. This requires caller-owned protected
local directories with hard-link and directory-sync support; Linux is the tested
environment. A staging/interruption failure may leave a non-authoritative temporary
file. Once the destination link exists, a later failure is PublicationUncertain:
inspect the exact destination, preserve it and use a fresh path for a new revision.
No retry silently deletes data or guesses that publication failed. No filesystem
crash/power-loss test or other-platform durability claim has been made.

`RunAttempt::pin_closed` is called after the writer/producer is closed. It checks
identity, geometry, method, typed full-intent declaration, actual recorded budgets,
integrity, every native carrier publication, actual dt and the requested horizon.
Completed runs cover all requested intervals and include constructor zero. Failed
or cancelled runs can retain a shorter prefix, an explicit unrecorded accepted
tail within the saved horizon, or zero publications. The final pin stores the
whole-file SHA256, count and authoritative terminal/accepted-generation facts.

Cold `open_member` returns NotRun, Missing, Incomplete, Unpinned, Unsupported,
Invalid or Ready. Active archives remain incomplete even when their last record
is valid. An unpinned terminal archive is not silently promoted Ready. A pinned
archive must match its complete artifact identity, captured terminal facts and
native schema before a reader is returned. Ready means the reader is usable;
failed/cancelled terminal outcomes retain those meanings. Every selection uses an
explicit ordinal on that member's reader; no live or newer publication substitutes
for missing data. Unknown fields stay absent through RecordingReader.

Paths are bounded canonical relative paths; absolute/parent/backslash/drive paths
and symlink components are refused. Caller protection must exclude concurrent path
replacement throughout validation and use, matching RecordingReader's local-path
contract. Opening inherits that owner's SQLite hot-journal recovery semantics; it
is not an OS-enforced read-only open. Archives are inspection records, not restart
checkpoints. Finalized archives are kept immutable after producer closure.

## Verification boundary

The contract test drives both real native implementations and the existing
StepRecorder, checks exact f32/f64 field bytes/times, and runs a producer OS process
to successful exit before starting a separate cold reader OS process. That reader
opens only manifests/RecordingReader and independently checks selected native bytes
against SQLite BLOBs. Tests also exercise strict bounds/version/integrity failures,
immutable revisions, corrupt/missing/incomplete/unpinned states, changed source
intent, historic opaque Debug provenance, native schema failures, budget/horizon
mismatches, unrecorded tails, path rejection and racing manifest publication.

These establish the bounded native automated contracts. They do not establish
accepted-provider provenance, generic diagnostic storage, native-window operation,
model runtime/lifecycle behavior, crash durability, physical accuracy or M2–M3
acceptance as a whole.

The app-private bounded publication/versioned-read/path/digest helpers are reused
by the saved-comparison document owner. Each caller supplies its own expected
schema version and practical byte budget; these are not executable configuration
or numerical archive versions. The native spec/run-set document budget remains
1 MiB. Zero and maximum-u64 budgets are invalid before oversize-read arithmetic.
