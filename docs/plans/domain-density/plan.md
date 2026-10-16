# Independent simulation and preview cell density

Status: Active. Operation: start. Owner/integrator: Root. Authorized requirement:
select up to 1000 cells per metre, simulation counts up to 10000 per axis and
independent exact preview counts up to 10000 per axis, preserve other work and the measured A1 blocker, publish and merge
when independently reviewed and required checks pass.

Base: ad888bdbbbd5cae362439458af342196f51c58e2. Isolated branch/worktree:
implementation/domain-density-1000-20261016, /workspace/rheon-domain-density.
Target main; retain shared history. After accepted ancestry and no dependency
proof, retire only this task's branch/worktree. Other worktrees remain protected.

## Admitted change

Reuse GridGeometry, BufferPlan, current numerical/particle owners, immutable
Request, LiveHistory, viewport and current recorder. CLI density fixes spacing at selected counts. Desktop density resolves counts
from fixed physical extent, disclosing covering rounding. Preview selection changes visualization only;
new explicit preview requests are exact or refused, never silently reduced.
Existing adaptive preview defaults remain explicit and unchanged. A finer
preview replicates existing cell appearance; it does not add physical samples.

The observed 512 is preview axis count, not universal density. The former carrier count 128,
particle samples 4–64, scene density now 1000/m, scene count/total/history limits,
step 512, package budgets and fixed qualification fixtures have separate owners.
Raise scene density and add independent native CLI density/preflight. Existing
GridGeometry::new/recording spacing contracts remain general. Legacy model
support is not widened; unsupported high density rejects before child launch.

Write set: geometry/lib density convenience and tests; native CLI/options/tests; unit-box comparison manifest geometry guard/tests;
desktop Request/UI/history/viewport/carrier thumbnail and owner tests; comparison Scene/UI/CLI and
legacy inert support and CPU contract tests; API/desktop docs; this plan/ledger/review; independently
reviewed complete native source admission inventory. No numerical kernel,
mathematical tolerance, recorder codec/schema, Tuldok or surrogate-lab changes.

## Acceptance and sequence

1. Checked units/conversion and pre-allocation estimates: grid cells/faces,
   retained native arrays, preview history including constructors/tracks, preview
   voxel visits and worst-case surface mesh. These are not RSS, elapsed-time or
   model-accuracy guarantees. Oversized/overflow requests fail explicitly.
2. Prove small physical domains with 1 mm spacing, independent preview settings and physical
   geometry serialization/cold round trips, no-hole display upsampling and no
   numerical-state mutation. Preserve defaults and old adaptive behavior.
3. Inert model-support refusal, unchanged A1 inputs/qualification and protected
   Dense3d fixtures. Test native CLI real small run and nonallocating huge-domain
   refusal; affected core/desktop/recording tests, formatting and strict Clippy.
4. Independent final source/review; complete conservative source inventory
   admission and fresh native/rational qualification; exact-head hosted gates;
   review staged/range scope; commit/push/PR/merge and verified safe retirement.

Implementation complete; final expanded checks, independent source admission,
committed native qualification and hosted acceptance are in progress.
The configurable-solver plan's measured A1 blocker is unchanged. No training,
checkpoint substitution, sparse/out-of-core solver or 10000³ dense
capacity claim is admitted. Numerical stability failures retain their meaning.


2026-10-16 revised admission: user replaced the original target with maximum
1000 cells/m, 10000 native cells per axis and independent 10000-axis preview.
Exact preview density and count requests check realized density ≤1000/m.
Default adaptive compatibility and frozen historical numerical specs retain
their previous contracts. Review found/corrected the late 128²thumbnail cap,
particle-mode contamination of configured carrier counts and obsolete 128 clamp.

The initial forced 1000/m smoke/source-stop trial hit existing
Pressure(IncompatibleRhs), sum 2.03378186350074e-23 versus rounding budget
1.7512013017958464e-25. No kernel/tolerance change is admitted. Keep this as a
numerical owner follow-up; do not promote geometry/storage/setting evidence to
scientific qualification. Nonzero continuously forced CLI and zero-flow cold
reruns have distinct claims. The measured A1 blocker remains unchanged.
