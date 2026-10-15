# Saved native specifications and run sets, version 1

`src/bin/desktop/run_sets.rs` owns immutable application intent and manifests over
existing [numerical SQLite recordings](recording.md). The desktop registers the
module and reuses its existing worker/Inspector/StepRecorder for native saved
runs. Integration and all merges belong to the configurable-solver lead.

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

An existing native worker can instead use `FrozenSpec::capture_constructor` to
freeze the actual Simulation's typed geometry, configuration and implementation
before its first step. The caller supplies its explicit future dt, step count,
source schedule and recording budget. A progressed or reset simulation is rejected;
the helper neither owns the worker loop nor interprets Debug text or accepted
state as a checkpoint.

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

## Native desktop workflow

In carrier mode without cylinder emitter or collision mesh, select **Save immutable
native run set**, then Simulate or Compare both pressure solvers. Before execution,
the app freezes the exact geometry, tolerances, requested dt/horizon, source schedule
and budgets in a fresh protected run-set directory. The existing worker constructs
Simulation from that frozen intent and applies its dt/source schedule. It records
each accepted publication through the existing Inspector/StepRecorder owner. It
closes the writer before pinning; no second numerical loop or archive is added.

`intent.json`, member-started/member-finished revisions and `completed.json` are
create-new immutable files. Each changed revision links the exact previous content
digest. Completion may use the same revision content as the last finished member.
An in-memory acknowledged-path receipt changes only after successful publication;
after joining success, cancellation or failure, the UI shows that exact path.
An uncertain terminal recording/publication retains its last acknowledged manifest,
raw archive and diagnostic error; it does not invent a finalized member. Cancelled
prefixes retain cancelled status, and unstarted members remain NotRun.

The v1 saved contract requires actual accepted dt to equal requested dt. Native
Simulation can legitimately shorten it. The worker first records that accepted
state, then stops this fixed-dt workflow with an explicit unsupported-intent message.
The exact archive remains available for ordinary numerical inspection; this does
not call valid shortened physics corrupt or silently relabel the intended schedule.

Open an explicit manifest under Saved native run sets. Member identity/integrity
and terminal status are checked once on open and cached for repaint; Inspect
rechecks the pinned artifact before opening the existing selected-field inspector
and its CSV/VTK exports. Failed open clears previous member capability. No runtime
is loaded and no current Debug configuration is interpreted.

**Rerun saved native set from zero** creates a new set/attempt/archive directory
and links the original revision digest. It uses the captured FrozenSpecs, not
current UI defaults, and never restores fields or mutates the source artifacts.
All direct source-member run IDs are excluded when assigning new process-local
run IDs, including after producer-process restart. This proves freshness relative
to that selected source revision and distinct new IDs within this process; it does
not assert global uniqueness across unrelated sets or the full ancestral chain.
Attempt/set identities and exact archive/publication pins remain separate facts.
This desktop execution envelope supports at most two distinct native pressure
methods with cubic zero-origin common geometry, requested dt/horizon and recording
budgets. Unsupported saved intent stays cold-inspectable. Existing native physics,
allocation and worker limits still apply; typed file support is broader than this
particular preview composer.

Fresh integration evidence: a real two-variant worker is joined through App::poll,
then cold-opened and rendered with egui; a captured-intent rerun has fresh run and
attempt IDs and identical native field/time bits while preserving original bytes.
A gate-controlled cancellation after a real accepted publication passes through
actual App::poll and reopens its cancelled pinned prefix with the second member
NotRun. The separate codec contracts include producer-process exit before a fresh
reader. These are automated Linux CPU/headless UI claims; native window/other-OS,
trained provider recording and full generic M2/M3 acceptance remain separate.
The app-private bounded publication/versioned-read/path/digest helpers are reused
by the saved-comparison document owner. Each caller supplies its own expected
schema version and practical byte budget; these are not executable configuration
or numerical archive versions. The native spec/run-set document budget remains
1 MiB. Zero and maximum-u64 budgets are invalid before oversize-read arithmetic.


Actual software-X11 integration observation on the pre-fix d8b0eb1 binary created
an 8-cube, two-step/0.003 s two-solver saved set, closed its producer, cold-opened
both Ready members in a fresh process, audited and exported selected ordinal2 VTI,
and reran captured intent despite fresh material-mode defaults. Native field/time
bits and original artifacts were identical. It also exposed source run-ID reuse
because the process-local counter restarted; that observation fails freshness and
is retained as a finding, not a pass. The root exclusion fix is covered by a
separate actual producer-exit/cold-App-rerun regression with a BEFORE snapshot of
all source files. Corrected native-window repetition remains pending. No numerical
kernel, accepted archive schema, global provenance or timeout was changed.
