# Plan: bounded CPU compressible flow and solver-data contracts

**Plan status:** `Active`

**Current phase:** S1 — native periodic ideal-gas Euler implementation

**Next slice:** S1 — native fixed-Cartesian periodic ideal-gas Euler baseline

**Acceptance status:** `pending`

Parent accepted independent plan review and started S1 worker
`01a13913-999b-7511-b9cd-f4677019e018` from reviewed head
`67baa10576a2120f314ec8e56bd7b7129d18d9f0`. This status/acceptance pair
governs the solver roadmap; plan delivery is tracked separately in
[verification](reports/verification.md). No implementation acceptance is implied.

**Execution ledger:** [execution-ledger.md](execution-ledger.md)

**Issues:** [issues.md](issues.md)

## Objective

Give a separate implementation worker a bounded, standards-reviewed route to a
CPU compressible solver with truthful capabilities and reproducible evidence.
Eventually expose accepted numerical fields and generic generation/data controls
in Rheon, with Rheon Surrogate Lab as the home for Tuldok training-project files.
Tuldok remains a generic solver-model tool with an optional Rheon producer.

This change delivers only the plan, independent review, branch and PR. It changes
no solver, training project, published capability, proof, source admission or UI.
The roadmap is not an executed cross-repository campaign. Later slices require
their own current-state admission and evidence; the first handoff is
[S1](reports/first-slice.md). [Design](reports/design.md) owns proposed numerical
and boundary details; shipped API documents remain the existing contract owners.

## Objective Acceptance

All rows below are future evidence, not claims obtained by publishing research.
No numerical or aircraft/submarine acceptance follows from acceptance of this plan.

| ID | Observable criterion | Kind | Environment | Mode | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| C1 | Validated five-component state, ideal-gas EOS, bounded allocation and failed-call rollback through native API | focused + contract | representative: pinned Rust CPU | automated | pending | S1 worker; [first slice](reports/first-slice.md) |
| C2 | Shared-face periodic update preserves uniform state and volume-weighted budgets; evolved acoustic and planar shock records pass declared errors | integration | representative: CPU + pinned NumPy | automated | pending | S1 worker; independent continuum oracles |
| C3 | Reconstruction/time accuracy, rarefaction/near-vacuum handling and all three directions meet frozen method-specific gates | integration | representative: CPU | automated | pending | S2 numerical owner |
| C4 | Characteristic/open and fitted slip boundaries satisfy information count, force/work and reflection/error budgets | integration | representative: CPU | automated | pending | S3 boundary owner |
| C5 | Public session → accepted recording → separate-process selection/export → real desktop inspection preserves model, field, time and missing-data meaning | contract + system + user-workflow | representative: CPU; required-real: native display for UI smoke | either | pending | S4 API/recording/UI owner after PR69/71 |
| C6 | Viscous stress and heat conserve compatible energy, pass laminar/thermal references, and disclose timestep restrictions | integration | representative: CPU | automated | pending | S6 viscous owner |
| C7 | Generic project captures complete producer inputs; independent consumer verifies dataset and whole-trajectory splits; real learned rollouts meet accuracy and end-to-end timing including rejection/fallback | contract + system | representative: CPU and verified Tuldok consumer | automated | pending | S5 producer owner / separate Tuldok owner; blocked until consumer contract observed |

Arbitrary solids and vehicle capstones are **Deferred**, outside these bounded
acceptance claims. Their promotion requires the gates in design §Vehicle paths
and fresh physical-validation data. C6 need not block acceptance of S1–S4 as an
explicitly inviscid capability; each accepted slice must advertise its own scope.

## Scope and constraints

- Native CPU Rust implementation; existing pinned Rust 1.92.0, safe Rust and
  locked dependency policy. S1 adds no dependencies. Any later dependency must
  have verified authoritative Apache-2-compatible terms and required notices;
  GPL integration, copied GPL code, and GPL linked dependencies are excluded.
- Fixed Cartesian single-material ideal gas first. Separate density, momentum
  and total thermodynamic energy from passive appearance/tracer fields. Preserve
  physical sound speed; no all-speed or calibrated water claim.
- Elementary references precede airfoil, wing, passenger-jet or submarine
  geometries. The passenger jet path is compressible external aerodynamics;
  forward-and-rising submerged submarine flow first requires validated
  incompressible viscous flow and moving-body volume/force/work contracts.
- GPU compute, remote workers, deployment, main merge, training execution and
  the complete cross-repository campaign are outside this task.
- Preserve existing incompressible, particle, provider and dense3D contracts.
  Existing pressure providers operate on sealed constant-density MAC boxes;
  labels alone cannot extend them to compressible Euler steps.
- Generated numerical fields, logs, SQLite archives and renders stay in ignored
  or external storage under [repository output policy](../../REPOSITORY_OUTPUTS.md).

## Binding Decisions

| Decision | Owner | Evidence and consequence | Supersedes |
| --- | --- | --- | --- |
| One separate cell-centered conservative Euler owner; no reuse of projection pressure as EOS pressure | numerical worker | Chapters 26–28, `src/simulation.rs`, `src/lib.rs`; [design](reports/design.md) | none |
| Serial first-order LLF, unsplit forward Euler, periodic fixed grid before new flux families or threads | S1 worker | Original reference has transparent semantics; simplest independently verifiable scoped step | none |
| Every accepted stage finite/admissible; bounded retry from unchanged state; no silent floors | numerical worker | Lean does not establish LLF positivity; runtime checks and empirical evidence remain necessary | none |
| Reuse PR69/71 generic recording after their integration, through an explicitly compressible field adapter | recording integration owner | Read-only inspections at `b1acf2e` / `ecfc741`; binary named fields and cold selection are owned there | none |
| Generic Surrogate Lab projects and data controls, optional Rheon producer, independent Tuldok authority | producer + Tuldok owners | Explicit user direction; existing dense3D v1/v2 are constrained transport formats | none |
| Lean assumptions and empirical accuracy are separate acceptance domains | proof + numerical owners | `proofs/Rheon/Compressible.lean`; [design proof map](reports/design.md) | none |
| S1 worker is sole editor of the minimal additive compressible module/export block in `src/lib.rs`; recording and other exports remain untouched | S1 worker; parent integrates | Explicit parent delegation after accepting review and starting S1 at 67baa10; [handoff](reports/first-slice.md) | Earlier parent-only S1 export application; retained in ledger |

Publication prerequisite observed: main
`8baf459b91bb53ebd4a7230c4d9f3399dd5cb0bc`,
[Pages run 38507276934](https://github.com/MrScripty/Rheon/actions/runs/38507276934)
completed successfully for that SHA. Source review found no AGENTS.md in the
workspace ancestors or either inspected Rheon tree. Current main lacks a Rust
compressible module. Collision queries, reduced obstacle/shear and moving-plate
models remain bounded; none implements general vehicle flow.

| Remaining assumption | Decision affected | Owner / smallest adequate real observation | Stopping condition |
| --- | --- | --- | --- |
| PR69/71 contracts remain as inspected | S4 adapter and UI wiring | integration owner re-fetches both heads, diffs recording API/storage/desktop contracts before any edit | Integrated contract identified and real record/reopen example passes; otherwise re-plan S4 |
| Tuldok can consume desired fields/controls | S5 schema and optional producer | Tuldok owner inspects actual parser/project contract at a verified source revision; run one native-produced elementary trajectory through real consumer | Accepted and unsupported versions/associations observed; no broad dataset generation beforehand |
| Native display and representative host available | UI acceptance and timing claims | UI owner runs selected-field/cold-open workflow on native display; benchmark owner records host/CPU/storage/build/runtime | Real result recorded; headless widget tests remain supporting evidence |
| Later material, closure and vehicle benchmarks are suitable and licensed | S6/S7 and vehicle promotion | named physics/licensing owner verifies transport laws, data provenance, regime and uncertainty before incorporation | Frozen matched conditions and independent validation oracle exist |

## Simplicity And Ownership Review

**Applicability:** `applicable`

This plan proposes new state/API/recording and
producer composition. The artifact inspected now is this bounded plan; future
implementation must recheck its actual composition at admission.

- Independent concepts and dimensions: Euler owner advances/accepts thermodynamic contents
   on CPU per interval; recorder retains publications after process exit; UI
   selects/views them; producer captures reproducible datasets; Tuldok owns
   training/evaluation. Each changes for a different reason.
- State, identity, value, time, policy, and mechanism: accepted state, EOS and clock publish together. Run ID is
   distinct from publication generation, geometry identity and dataset family.
   Physical evaluation time is distinct from wall-clock cost. Numerical policy
   stays in the solver; retention stays in recording; splits stay in the
   dataset/training project. No umbrella manifest becomes authority for them.
   The solver references geometry; dataset provenance references source/binary
   identities without authenticating them. API, archive, dataset and model
   versions own separate promises. Existing archive-v1 and dense3D-v1/v2 readers
   retain their current support; no speculative version overlap is introduced.
   EOS/geometry changes invalidate a run; accepted increments invalidate only
   its publication. Dataset contents/split/normalization changes invalidate
   evaluation identity, not unrelated archive schemas.
- Caller and composition-root knowledge: configure grid/EOS/budget, initialize validated state,
   request step/cancellation, read borrowed accepted fields/report. Callers
   never assemble face fluxes or repair thermodynamics. Composition root owns
   recorder/producer paths, lifecycle, UI selection and optional model loading.
- Representative change paths and forced owners: new EOS touches numerical closure and its oracle, plus
   capability metadata; new recording field touches its adapter/decoder and UI
   consumer only; a retention change stays with recorder; a split changes the
   project manifest/evaluator; turbulence adds a distinct physical model rather
   than rewriting a pressure-provider label. Shared export wiring is serial.
- Stable Interfaces versus hidden knowledge: interfaces carry values, units, associations and accepted clocks;
   they expose no scratch buffers, private SQLite tables or hidden projection
   gauges. Serialized contracts have their own strict readers and evolution
   owners; a caller cannot infer an absent field from a name.
- Independent evolution, testing, failure, and replacement: solver verification runs independently of storage/UI/Tuldok. Recorder failure
   cannot roll back an accepted solver state; it stops capture and discloses a
   potentially unrecorded tail. Rejected model output falls back only through
   a same-domain qualified numerical path. UI can fail without inventing data.
- Deletion and cumulative machinery result: removing the Euler owner scatters necessary flux/EOS/acceptance
   logic into callers; retain it. Removing the compressible recording adapter
   scatters field/time interpretation; retain only when S4's consumer exists.
   Decline generic flux registries, remote services, extra persistence engines,
   new proof/receipt frameworks, speculative EOS plugins and early model seams.
   Reuse existing archive integrity rather than a second recorder. A small
   independent analytic verifier is retained only for distinct numerical checks.
- Necessary complexity and containment: inherent complexity is compressible conservation, admissibility, geometry
   and model error. Contain it in the numerical owner and scoped verification.
   Later schema/UI machinery is admitted with actual consumers, budgets and
   deletion tests; no framework is created during planning.

## Milestones

S1 is the only next implementation slice. Later write sets are proposals for
scope review, not permission to modify shared files concurrently. Before
starting them, resolve names/consumers and refine the exact set in this plan. Tool/verifier
and benchmark harness paths in these proposed sets belong to the independent
validation owner; they are not implementation-worker writes. Interface/benchmark
requirements go through parent before either owner edits.

| Slice / state | One goal and coherent unit | Allowed write set | Preserved/replaced semantics; focused gate | Re-plan condition |
| --- | --- | --- | --- | --- |
| S1 / Active | Native periodic five-field ideal-gas Euler step and reproducible headless reference | Exact list in [first slice](reports/first-slice.md) | Preserve all existing solvers; C1/C2, uniform/asymmetric grids, periodic budgets, acoustic refinement, planar Sod and rollback | Unsupported grid reuse, changed EOS, new dependency, altered CFL/error meaning |
| S2 / Deferred | Accuracy/admissibility upgrade of that operator | `src/compressible.rs`, `tests/compressible_contract.rs`, `examples/compressible_reference.rs`, `tools/verify_compressible.py`, `tools/test_compressible_verifier.py`, `docs/api/compressible.md`, this plan directory | Conservative reconstruction, SSPRK2, method-specific positivity and HLLE fallback; smooth order, near vacuum, expansion/strong shock and Mach sweeps | New theorem assumptions, low-Mach remedy, dual energy or material domain |
| S3 / Deferred | Elementary open/fitted boundary runs | S2 set plus `src/compressible/boundary.rs`, `tests/compressible_boundary_contract.rs` | C4: fixed-state endpoint Sod, characteristic counts, acoustic reflection, stationary wall pressure forces and oblique wedge | Arbitrary cuts, moving topology or mixed-regime BC requires new admission |
| S4 / Deferred | Accepted compressible session, recording and desktop inspection | `src/compressible/session.rs`, `src/recording/compressible.rs`, `src/lib.rs`, `tests/compressible_session_contract.rs`, `tests/recording_compressible_contract.rs`, `examples/recorded_compressible.rs`, `src/bin/rheon-desktop.rs`, `src/bin/desktop/inspector.rs`, `src/bin/desktop/archive_view.rs`, `src/bin/desktop/settings.rs`, `docs/api/compressible.md`, `docs/api/recording.md`, this plan directory | C5: actual run/stop/failure, per-step recording, selected ordinal/export and cold native UI; reuse integrated PR69/71 | Pending recording changed, field cap/layout unsuitable, source admission owner unavailable |
| S5 / Deferred | Generic producer/project and one accepted elementary dataset | `src/bin/desktop/surrogate_lab.rs`, `src/bin/rheon-desktop.rs`, `tools/export_compressible_sequence.py`, `tools/test_compressible_sequence.py`, `examples/compressible_sequence.rs`, `docs/api/compressible-datasets.md`, this plan directory; Tuldok edits separately owned | C7: native fields + actual interval controls → durable completion → real independent consumer; whole-trajectory grouping, accuracy/timing including fallback | Consumer schema unknown, new source families, contract/deployment change |
| S6 / Deferred | Conservative molecular stress and heat, laminar references | `src/compressible/viscous.rs`, `src/compressible.rs`, `tests/compressible_viscous_contract.rs`, `examples/compressible_laminar.rs`, `tools/verify_compressible_laminar.py`, `docs/api/compressible.md`, this plan directory | C6: Newtonian stress/heat/work, diffusion stability, Couette/Poiseuille/conduction then flat plate | Thermal EOS law, implicit solve, tensor/cut-cell geometry changes |
| S7 / Deferred | Evidence-selected turbulence model | Proposed `src/compressible/turbulence.rs`, its native tests/example/verifier and API docs; enumerate at admission | Frozen closure, coefficients, wall/heat treatment and independent statistical validation; no noise-as-turbulence claim | New closure, wall regime, calibration or inadequate resolution/data |

General cut-cell assembly, swept moving geometry, body reaction and capstones are
deferred follow-ons with the separate geometry/physics owner, before S7 is
advertised on vehicles. S6 can follow the fixed-wall path without arbitrary CAD.
No milestone schedule implies a delivery date or a performance guarantee.

## Repository isolation and concurrent work

Plan branch `plan/cpu-compressible-solver` starts at published main `8baf459`.
Planning agent owns the canonical plan documents in this directory; worker owns
its `reports/implementation.md`. Target is main; parent is serial
integration owner. Branch is shared for PR review, retained until parent disposition;
no main merge, force push, checkout reset or branch/worktree deletion is authorized.
Old `work` and `implementation/rust-smoke-milestone` refs remain intact.

Independent standards reviewer is read-only and reports to the planning owner.
Parent admitted and started S1 worker `01a13913-999b-7511-b9cd-f4677019e018`
at reviewed head `67baa10576a2120f314ec8e56bd7b7129d18d9f0`. It owns numerical
source, focused Rust tests, native example, API doc, its implementation report,
and solely the minimal additive compressible module/export block in `src/lib.rs`.
No other worker or parent concurrently edits that S1 block. Recording and other
exports remain untouched. This planning agent owns canonical plan/ledger/issues
updates; parent routes authority changes and integrates serially. Independent
validation owns numerical/origin harnesses; output contracts and gates are
coordinated with `01a13906-b6ec-7398-820a-387f852c3015` through parent.
The implementation worker owns no recording/desktop/schema/lockfile/publication
edits. This plan-only branch contains no solver changes. PR69/71 and all source
inventories, receipts, workflow files and other shared exports remain serial owner writes. Concurrent
Plan Integration profile is not applicable to a single serial plan proposal plus
read-only review; if multiple outstanding proposals later share mutable plan
authority, select that profile at their admission.

Updated parent coordination assigns independent validation/acceptance research
to `01a13906-b6ec-7398-820a-387f852c3015`, Tuldok execution to
`01a13906-0d11-7681-87b7-439a7097c039`, and lab projects to
`01a13906-4c49-7097-a9ea-1ba58cd0802f`. Parent routes requirements and integrates
cross-repository changes. User authorization permits those parallel streams;
this worker retains plan-only ownership. The only newly delegated shared write
is the parent-authorized S1 compressible export block specified above.

### Pinned release checkpoints

| Checkpoint | Immutable subject and interface | Deciding observation / integration owner |
| --- | --- | --- |
| R0 research | Rheon 8baf459; published chapters/proof inventory | Observed Pages success; plan/research scope only |
| R1 native baseline | Clean committed S1 SHA/tree/binary and `docs/api/compressible.md`; validation harness revision independently pinned | Actual native fields pass C1/C2; parent freezes both subjects before downstream consumption |
| R2 recording/session | Owner-integrated PR69/71 + S4 exact source and field adapter version | Real accepted run/cold reader/UI checks; recording/API owner, parent qualifies changed source |
| R3 producer/project | Exact Rheon producer/schema plus Tuldok consumer and lab project-schema commits, artifact hashes and split identity | One complete native trajectory consumed and saved/reopened through real Tuldok/lab path; producer/Tuldok/lab owners |
| R4 model/runtime | Exact training/evaluator/runtime/checkpoint identities and R3 dataset split | Whole-rollout accuracy and complete timing/rejection/fallback at frozen workload; validation owner and parent release decision |

Unknown future pins remain unavailable until these observations exist; never
consume a moving main as a tested release. Any changed semantic interface
invalidates affected checkpoint evidence and requires consumer disposition,
focused re-verification and parent checkpoint admission. These are tested
interface/artifact checkpoints, not a prescribed Git parentage or commit count.

## Blockers

No known blocker to plan delivery or S1. S4 waits for owner disposition/integration
of PR69/71; S5 waits for an observed Tuldok consumer and agreed project contract.
General vehicle readiness is blocked by missing geometry/physical validation,
not by research publication. Source-admission changes need the existing book
integration owner and fresh qualification; unchanged admission must fail on
changed native sources rather than accepting stale evidence.

## Re-Plan Triggers

Re-plan on changed objective/authority, overlapping work, unsupported oracle,
different EOS/boundary/geometry domain, numerical failure that invalidates the
method, missing source/consumer facts, new dependency/license issue, changed
retention or compatibility promise, or implementation complexity exceeding the
composition above. Ordinary repairs remain within a valid admitted slice.

## Final Acceptance

Implementation acceptance remains pending. Close each bounded milestone only
with its named evidence and advertise only that domain. Follow-ons remain owned
in [issues](issues.md); [verification](reports/verification.md) records plan
delivery, separate from all future numerical and user-workflow acceptance.

- Acceptance status: `pending`
- Final status: `Active`
