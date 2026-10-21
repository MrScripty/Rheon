# T5 runtime/archive force integration boundary (preflight only)

UI owner: Request/freeze/launch/run_material in rheon-desktop.rs;
carrier_execution.rs dispatch; run_sets.rs and run_sets/{source.rs,configured.rs,
configured/archive.rs} desktop frozen intent/load/rerun. Backend PR145 owns canonical
ForceScene/ForceSnapshot/evaluator/adapters and core recording custody. Reuse exact
canonical ancestor43ec505; no duplicate force model or evaluator. No T5 source edit,
solver, SQLite workload or force-enabled launch is admitted by this preflight.

Inspected runtime seams: CarrierExecution::advance currently uses legacy step or
step_with_emitter; run_material uses legacy particle step and Inspector::begin/
record. Carrier recorder likewise uses Inspector owners. Core force-aware APIs now
exist, but these desktop wrappers currently cannot bind the expected snapshot at
run/constructor creation. Therefore runtime wiring needs a minimal additional
Inspector interface contract before editing that file, which has separate validated
projection/archive work. Do not copy/merge/rebase that separate slice speculatively.

Proposed minimal Inspector addition (coordination pending): explicit begin-carrier/
begin-material-with-force-fields paths (or one typed optional snapshot argument)
construct the existing RecordingRun.with_force_fields before Recorder creation and
use matching constructor publication wrapper; record paths retain existing owner
and call expected-snapshot publication wrappers. Preserve reader attach, errors,
recording completeness/integrity distinctions and historical legacy paths. No
projection UI change. Signature/write ownership must be agreed with Inspector owner
before source edits; backend core recording remains untouched.

Once interface is coordinated, freeze the draft before Worker/output allocation,
retain one immutable Request snapshot for the entire run/history, and serialize its
canonical scene for intent/record metadata. Dispatch ONLY native carrier and native
particle-material through their canonical force adapters; legacy None paths retain
exact existing calls. Empty authored document has an explicit policy (None for an
unchanged legacy workflow; a retained Some(empty) document must bind its metadata).
Configured/session/provider/transport/free-surface remain explicitly refused until
those existing owners carry force custody; never route around them. Existing lab
preview modes are not a substitute for two-solver runtime integration.

Versioning must preserve historical hashes, constructor validation and reruns.
Current FrozenSpec schema and global manifest version share1; do not blindly bump
all manifest formats or add serialized null fields that alter old spec digests.
Prepare an explicit versioned force-bearing frozen spec and bounded canonical scene
bytes, reject mixed/unknown/mismatched versions, keep legacy version1 reserialization
and digest behavior exactly. Archive cold-open/rerun uses recorded run scene, never
current draft. Disabled objects/IDs/order/units/transforms/support/time are retained.
Material run/history requires equivalent immutable metadata even without a saved-set
UI. Reuse D1 comparison; metadata differences become explicit, not reimplemented.

First independent source-only slice after coordination: pure versioned intent codecs,
legacy golden digest/roundtrip/refusal probes using literal bounded documents only.
Inspect helper paths, list-only, individual --exact. Compile actual dispatch later;
no solver/model/SQLite fixture execution under the current no-workload authority.
Separate bounded desktop/recording integration execution contracts and authorization
are needed before enabling launch readiness. Source compilation alone cannot accept
runtime/archive custody, genuine cancellation or coherent two-solver behavior.
