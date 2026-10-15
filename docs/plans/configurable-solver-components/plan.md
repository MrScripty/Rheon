# Configurable Rheon components and saved comparisons

**Document status:** Independently reviewed research/design proposal. Documentation publication is authorized; production implementation is not. This document has not entered the implementation-plan lifecycle. It is not `Planned`, `Active` or `Accepted` implementation merely because it exists.

**Current phase:** Documentation published for review; future implementation remains blocked/deferred.

**Next proposed slice:** M0, reconcile the exact new Rheon integration and admit one bounded vertical slice.

**Future implementation acceptance:** `blocked`: the reported new native source remains unavailable after repository-side branch discovery and exact-commit fetch. No implementation or numerical/runtime acceptance commands have run for this roadmap. Documentation/source review and link/scope checks do not satisfy implementation acceptance.

**Canonical repository path:** `docs/plans/configurable-solver-components/plan.md`, selected by the parent for this documentation task. Parent/integration owner must admit a concrete implementation operation and exact write set before code work; existing unambiguous authorization may be reused without ceremonial reapproval.

**Supporting records:** [Research/design report](reports/research.md), [proposed typed contracts](reports/contracts.md), [execution ledger](execution-ledger.md), [issues](issues.md), [standards route](reports/standards-and-design.md), [independent review](reports/review.md). The report is reviewed research evidence; this plan owns the current proposal and handoff, while contracts detail boundary semantics. Their review is not implementation acceptance.

## Objective

Let an operator select a compatible independently packaged pressure component, save the exact simulation and run-set configuration, execute it through Rheon's existing numerical/worker owners, and cold-open quantitative comparisons of the resulting native/model runs. Compatible model channel, normalization, weight and runtime changes should stay in the model package after the seam exists. Scientific domain, operator or stage changes remain explicit numerical work.

The first product outcome is a native desktop vertical path, backed by public numerical contracts and durable data: select provider → see support/preflight → run → inspect actual selected/fallback behavior → save → close producer → reopen and compare exact recorded publications.

## Source versions and applicable standards

| Source | Reviewed material / limit |
|---|---|
| Rheon | Main `43ed26b645dd42de906d0fac7d1ec7d5ab6e351a`; [provider](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/src/solver_provider.rs), [session](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/src/session.rs), [recording](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/api/recording.md), desktop and VTK contracts |
| New Rheon integration | Reported `feat/live-ai-pressure-comparison`, head `66e50388ec7a788dbda0ad1227e330996edf506e`; inaccessible and **not reviewed**. M0 is prerequisite to relying on it |
| rheon-surrogate-lab | `a8426061380e82951b4dba1bbfa38ff93dc7ebb2`; [deployment/bridge](https://github.com/MrScripty/rheon-surrogate-lab/tree/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1), [channel/reconstruction code](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/cuda-pressure-v1/model.py#L154-L188), [reported diagnostic results](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/docs/plans/live-ai-pressure-comparison/results.md) |
| Adopted Coding-Standards | `188beda1fa477d21d576c233dd8c7f4c4c267d23`, confirmed by Rheon's [existing recording plan](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/plans/accepted-step-recording/plan.md) and [standards route](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/plans/submarine-incompressible-cpu/reports/standards-and-design.md) |

This is source-guided application of the adopted standards, not an automated conformance certificate. No ancestor or Rheon `AGENTS.md` was found in the actual checkout. Recheck actual instructions when admitting future work.

Current documentation work selects [Core](https://github.com/MrScripty/Coding-Standards/blob/188beda1fa477d21d576c233dd8c7f4c4c267d23/CORE-STANDARDS.md), [Router](https://github.com/MrScripty/Coding-Standards/blob/188beda1fa477d21d576c233dd8c7f4c4c267d23/STANDARDS-ROUTER.md), [Planning](https://github.com/MrScripty/Coding-Standards/blob/188beda1fa477d21d576c233dd8c7f4c4c267d23/workflows/planning.md), Discovery, Development Proportionality, Implementation/Verification prerequisites, Documentation, Architecture/Code Design, Contracts/Evolution/Replay and claim-directed independent oracles. The complete composed-design probe below applies because this design introduces a material seam and runtime/persistence obligations.

Before actual code, route the affected Library, Rust/API/Security/Dependencies/Tooling, IPC, Persistence, Concurrency, Resilience, Diagnostics, Performance, Licensing and native GUI/platform verification requirements through current facts. Web frontend, Godot, FFI/unsafe, generated-contract, release or concurrent-plan-integration profiles are not automatic: select them only if the admitted implementation actually creates those conditions. One serial plan writer and read-only reviewers do not trigger concurrent mutable-plan integration.

Repository constraints are concrete: [Rust 1.92.0](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/rust-toolchain.toml), edition 2024, `unsafe_code = forbid`, Apache-2.0 and existing feature boundaries in [Cargo.toml](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/Cargo.toml). Respect [source/generated-output policy](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/REPOSITORY_OUTPUTS.md): authored plans/configuration/source and compact audit manifests may be tracked; simulation arrays, SQLite archives, logs, CSVs, renders and PDFs remain generated artifacts in ignored storage.

## Scope and preserved contracts

### Proposed implementation scope

- An additive typed pressure-context contract, explicit prior accepted pressure and host-prepared residual where required.
- Inert provider discovery, explicit binding and one qualified trusted-local CPU runtime using reviewed existing deployment machinery where suitable.
- The existing sealed constant-density pressure-candidate role, native qualification and explicit same-request fallback.
- Accepted provider/session recording, stable saved simulation/run-set specifications and cold inspection.
- Checked comparison alignment, native numerical metrics, linked desktop views and explicit warmup/fallback/diagnostic labels.
- Teacher-stage evaluation and an independent **stop-on-reject** diagnostic mode, with a new diagnostic record schema under the existing recording owner where standalone diagnostics are persisted.

### Deferred scope

- Continuing from failed AI states, until a separately typed Rheon-owned diagnostic advance contract is reviewed and admitted.
- Viscosity/provider composition beyond current main, arbitrary stage graphs, new physical operators, variable-density/free-surface support and obstacle coupling.
- Recurrent provider state, learned initial-guess/preconditioner roles, GPU IPC, new runtime families, a marketplace, distributed scheduling, browser plugin panels and integrator restart.
- Model retraining, new benchmark/speed/physical-accuracy claims, production deployment and main merge.

Existing numerical gates and accepted-publication semantics remain authoritative. Existing archive v1 and VTK/dense3D versions are not relabeled. A rejected standalone candidate cannot become an accepted archive ordinal. A finite rejected-candidate metric may accompany a genuinely accepted fallback publication through its provider report.

The reviewed lab network is stateless but pressure-conditioned; 15 native warmup steps precede its first supported call. Current scalar spacing means cubic cells, not equal axis counts. Nonzero physical origin and anisotropic spacing are unqualified for the reviewed adapter. Its deployment declares viscosity `0.001 Pa·s`, while main's full-step provider has no viscosity parameter. Until reconciled, preflight must not claim the same physical plan is admitted.

## Objective acceptance claims

All criteria are future claims. No `satisfied` implementation status is asserted here.

| ID | Observable criterion | Kind | Environment / mode | Current status |
|---|---|---|---|---|
| A0 | Exact integration/source/consumer inventory establishes which existing pieces are reused and one executable first envelope | source/design review | Accessible pinned Rheon and lab / read-only review | blocked |
| A1 | Two real compatible adapter/package revisions change channel preparation/weights without edits to numerical or UI owners; incompatible requests reject before inference | public contract + real-runtime integration | Pinned Rust/Linux CPU and actual model bundle / automated | pending |
| A2 | Full-step acceptance, state/history rollback, same-request fallback and cancellation retain existing meaning | numerical/API contract + independent oracle | Native Rust CPU / automated | pending |
| A3 | A completed accepted provider run is closed and selected fields/provenance reopen exactly in a fresh process without loading model code | persistence + system | Local SQLite and separate record/inspect processes / automated | pending |
| A4 | A saved set preserves immutable variant intent, actual outcomes and missing/corrupt/incomplete states without substituting live/latest data | persisted contract + workflow | Native desktop plus real archives / automated and native interaction | pending |
| A5 | Metrics/linked views compare compatible actual stages/times/geometry and report unmatched coverage, warmup, fallback and rejected diagnostics correctly | oracle + user workflow | Native numerical fields and window / automated plus manual interaction | pending |
| A6 | Teacher-stage inputs remain native; independent stop-on-reject inputs/history never leak across tracks; diagnostic schema cannot pass accepted decoders | isolation/contract + real integration | Distinct native/model contexts and real storage / automated | pending |
| A7 | Process failure, cancellation, Close, repeated Start and storage failure reach honest terminal states with no stale publication or owned child left running | lifecycle/system | Supported OS, real runtime/processes/window / automated plus native interaction | pending |
| A8 | Pinned formatting, lints, affected feature/API/recording/VTK/browser regressions pass for final material; unsupported platforms remain explicit | static/build/regression | Actual supported toolchains/targets / automated | pending |

Native window claims remain pending until actual interaction is observed in an appropriate connected execution environment. This documentation task has not qualified one. Headless egui tests and screenshots of a started window cannot substitute for the requested interactive workflow.

## Proposed decisions and owners

These become binding only upon implementation admission and current-state reconciliation.

| Decision | Owner and reason |
|---|---|
| Typed finite stage plan; no universal tensor graph | Rheon numerical owner; ordering/operator validity is scientific, rather than metadata compatibility |
| Full pressure candidate is the initial replaceable role | Rheon dispatcher; correction reconstruction remains adapter-owned and independently qualified |
| Explicit host history, accepted-step commit | Numerical/session owner; retries cannot advance hidden model history |
| Package preprocessing/weights/support | Lab project owner; compatible model evolution stays local to its source and evidence |
| CPU subprocess selected explicitly | Rheon host/runtime owner; reuse current reviewed process machinery and keep imports/GPU initialization outside the application |
| Trusted-local capability label | Runtime/security owner; same-user subprocess isolation is not malicious-code containment |
| Existing worker and recording modules retain custody | Application/recording owners; no competing solver loop, supervisor or numerical archive |
| Saved specs reference numerical contracts and package content | Application configuration owner; packaging facts does not transfer physics/training authority |
| Comparison recipe/version owns analysis semantics | Analysis owner; alignment/reduction are distinct from numerical acceptance |
| Tuldok remains generic | Tuldok owners; no Rheon pressure equation or physics-validity policy moves into its application core |

Parent/integration owner coordinates source baseline, shared files, review, publication and eventual merge disposition. Independent reviewers own challenge/evidence review, not production mutation or plan admission. Evidence owners are: A0 integration; A1 runtime plus lab; A2 numerical/session; A3 recording; A4 application/configuration; A5 comparison/UI; A6 numerical plus diagnostic-recording; A7 runtime/worker; A8 the owner of each affected build/consumer path. Their deciding procedures are the milestone gates and verification section below; evidence links remain absent until those procedures actually run.

## Simplicity and ownership review

**Applicability:** `applicable`. Candidate answers to every [Architecture composed-design probe](https://github.com/MrScripty/Coding-Standards/blob/188beda1fa477d21d576c233dd8c7f4c4c267d23/topics/architecture.md#composed-design-admission):

1. **Independent concerns and dimensions:** numerical core decides what physical step is valid, at which actual interval and native geometry; lab adapter decides how its captured model maps that physical request to a candidate; runtime decides where/how a selected executable runs and ends; recording decides durable publication/selection; application decides which saved variants to run; analysis decides what compatible recorded values mean in a comparison. Each has its own owner, lifecycle and change reason.
2. **Required versus accidental interleavings:** required: candidate, operator/RHS, history and actual interval must be bound; accepted field/time/provider report must publish coherently. Accidental: model channel order in desktop controls, database handles in model code, display quantization in metrics, native teacher history in an independent track, or one package version claiming to version every physical/runtime/storage concern. Keep stage, adapter, runtime, content and recording/analysis identities distinct.
3. **Caller/composition knowledge:** numerical caller selects an admitted provider/workspace and sees typed support/result/fallback obligations. It does not know Python imports, channels, transport files or archive layout. Host composition resolves package/runtime, budgets and lifecycle once; UI uses bounded descriptions and existing run controls. Model code receives immutable named context, never a mutable session/database handle.
4. **Representative change paths:** new weights/normalization alter lab package plus qualification evidence; compatible channels alter its adapter version and conformance evidence; new physical quantity/order touches numerical contract and dependent adapter/tests; new runtime touches runtime and compatibility tests; archive evolution touches recording and actual readers; colormap/camera touches view owner; metric definition touches analysis recipe/evidence. A compatible checkpoint must not force numerical/UI-owner edits.
5. **Stable values versus hidden knowledge:** exchange physical descriptors, immutable input lineage, explicit candidate results and verified archive selections. Do not expose stencil internals, mutable histories, process pipes, SQLite row structure or inferred configuration strings as cross-owner contracts. Unknown versions fail explicitly; only real supported compatibility windows are retained.
6. **Independent evolution/failure/replacement:** built-in numerical providers remain usable without external runtime; incompatible/unavailable models do not prevent cold inspection; failed model execution can use only the qualified same-request fallback; storage failure preserves numerical acceptance and reports a possible unrecorded tail; analysis failure does not mutate source runs. Actual public/independently deployed consumers determine version overlap; coordinated private interfaces change atomically.
7. **Deletion result:** deleting typed stage context scatters physical/history meaning into adapters and callers; deleting adapter scatters channel/reconstruction knowledge into UI/core; deleting runtime supervision scatters child/resource ownership; deleting run-set manifest removes saved grouping/intent, not numerical data; deleting comparison validator scatters alignment policy across views. Do not add a new archive, queue/service, marketplace, universal graph or GPU sharing layer: deleting those proposed mechanisms removes incidental machinery without losing the first outcome.
8. **Necessary cumulative complexity:** retain one numeric owner, one worker lifecycle, one recording owner, a narrow pressure context, captured provider binding/runtime, saved specifications and a comparison recipe. Contain unavoidable stage/geometry/time/provenance/resource knowledge at these boundaries. M1's change-locality evidence and M2/M3's cold/lifecycle tests decide whether this composition is actually simpler; broader propagation triggers redesign.

**Authority-scope admission:** a package or saved spec may reference stage, model, runtime, source and analysis identities, but owns only that binding/recipe. It does not become canonical for those referenced concerns. Old archive interpretation is retained by its versioned reader; rerun identity includes required material closure; neither hash integrity nor saved metadata establishes authenticity, numerical reproduction or restartability.

## Design observations and dependency gates

| Uncertainty | Decision / least costly adequate observation | Owner / stopping condition |
|---|---|---|
| New native branch inaccessible | Read exact source and compare its runtime/diagnostic/worker/recording boundaries before adding replacements | Integration owner / source/consumer matrix and exact reusable write paths complete |
| Model physical envelope differs from main provider | Trace viscosity, residual sign/scaling, gauge, origin/spacing, actual dt and warmup through one real captured request | Numerical + lab owners / compatible plan proved or exact unsupported outcome selected |
| Current process protocol reuse | Run one real CPU bundle through reviewed existing protocol, including cancel/restart, in an admitted executor | Runtime owner / exact output parity and terminal cleanup observed; no timing claim |
| Stable saved configuration codec | Inventory actual controls and consumers; select a reputable versioned codec if current dependencies do not supply one | Config owner / closed schema and reconstruction coverage established; no parsing Debug text |
| Diagnostic persistence/continuation | v1 is accepted-only; design a separate typed diagnostic record path under same recording owner; stop-on-reject initially | Recording + numerical owners / schema/reader dispatch proves no accepted-state confusion |

Investigations end at these decision-relevant results. Do not delay the first reversible admitted slice for GPU benchmarks, universal framework comparisons or unrelated solver research. Conversely, do not expand around an unavailable real-interface observation that could invalidate it.

No new package is selected by this draft. Prefer current approved Rust/SQLite/hash/UI/work scheduling and reviewed deployment code. Any new parser/runtime/test dependency requires purpose, exact version/transitive/license/toolchain/feature review. No GPL implementation adoption is proposed; scientific references do not confer rights to code, weights or datasets.

## Milestones and integration sequence

All milestone entries below are **proposals, not admitted implementation states**. Only M0 is the next proposed slice. Exact source paths below match reviewed main where they exist; new filenames are candidates. M0 must replace them with the actual exact write set after inspecting the new branch, before M1 starts. This is not authority for broad directory edits.

### M0: reconcile source and admit the vertical slice

**Goal:** one current source/consumer/authority matrix and a reviewable first-slice design.

**Draft write set:** this plan, ledger/issues and a source-review report; no production files.

**Tasks:** inspect current checkout status/instructions; pin accessible native/lab material; preserve unrelated work; reconcile stage/domain/history/runtime and recording gaps; resolve the viscosity mismatch and origin/spacing admission; identify actual consumers; choose version/compatibility strategy and exact M1 write paths. Confirm the canonical plan operation and existing authorization through the parent.

**Gate:** A0; complete composed-design review and all materially changed assumptions dispositioned. Missing source blocks only dependent implementation, not documentation delivery.

**Replan:** a different scientific owner, domain, trust tier or execution environment; meaningful new work already implemented by the inaccessible branch.

### M1: select and execute a typed CPU pressure component

**Goal:** a real desktop operator selects a compatible captured package and obtains honestly qualified/selected/fallback behavior through the existing session.

**Proposed write set:** `src/solver_provider.rs`, new `src/solver_provider/context.rs` only if justified; `src/provider_step.rs`, `src/session.rs`, `src/simulation.rs`, `src/liquid_step.rs` at composition points; additive necessary `src/lib.rs` exports; new `src/bin/desktop/provider.rs` and existing `settings.rs`; `src/bin/rheon-desktop.rs`; `tests/{solver_provider_contract,provider_step_contract,session_contract}.rs`; one real-package integration test; `docs/api/full-step-provider.md` and owned plan reports. `Cargo.toml`, root/browser lockfiles and workflow changes are serial adjacent writes only after an actual dependency/build need is established.

**Preserve:** numerical operator/gates, native defaults, core-only behavior, synchronous library lifecycle and existing archive contracts. Keep runtime-specific code in the host/qualified optional boundary rather than exposing speculative public APIs.

**Focused tests:** named-field/unit/stage/gauge/layout checks; unequal counts with cubic cells; explicit anisotropic-spacing/nonzero-origin/viscosity rejection; exact actual dt; absent/stale/other-run pressure history; host residual convention; zero-residual bypass; invalid/short/nonfinite output; unknown version; mutated package material; same-request fallback; no history advance on final rejection. Two compatible real packages/revisions prove change locality; a test double proves only its stated contract.

**Gate:** A1–A2 plus the M1 portion of A7/A8; actual runtime parity, no optimizer updates/device-policy violations, real selection and a native-window provider workflow. No accuracy pass is required of the currently failing checkpoint to prove honest rejection/fallback, and no model failure may be relabeled a success.

**Replan:** compatible model changes require core/UI-specific channel edits; the actual bridge cannot support the selected lifecycle; no physically compatible plan exists; hidden state requires commit/rollback support.

### M2: save accepted runs and reopen a run set

**Goal:** execute, persist, close and cold-open an immutable saved simulation/run set with complete accepted provider provenance.

**Dependency:** accepted M1 contracts, or a separately admitted native-only configuration slice if independent of the missing model envelope.

**Proposed write set:** `src/recording.rs`, a narrowly owned new `src/recording/provider.rs`; accepted capture in `src/session.rs` only if required; new `src/bin/desktop/run_sets.rs`; `src/bin/desktop/{archive_view,identity,inspector,settings,history}.rs`; `src/bin/rheon-desktop.rs`; `tests/{recording_contract,recording_carrier_contract,session_contract}.rs`; new `examples/recorded_provider.rs`; `docs/api/recording.md`, necessary saved-spec contract and plan records. Dependency/schema/lock changes remain serial and explicit.

**Preserve:** v1 interpretation, exact native bits and independent public/VTK versions; no restart claim; accepted capture before another attempt invalidates source stamps; recording failure reports an unrecorded tail rather than rolling back physics.

**Focused tests:** fresh-process record/verify/select/export; no model load on cold inspection; immutable revision capture; missing/corrupt/incomplete/unsupported member; same ordinal while another track advances; unknown field stays unavailable; capacity/poisoned writer/uncertain commit/terminal-status failure; no overwrite/deletion on recovery; saved Debug text is not executable configuration.

**Gate:** A3–A4 plus affected A7/A8. One real saved two-variant set survives producer exit and reopens exact fields, source times, identities and selected/fallback outcomes.

**Replan:** a second numerical archive or source-of-truth catalog is introduced; automatic migration/restart/retention deletion becomes necessary; a saved spec cannot reconstruct the promised controls.

### M3: compare actual numerical evidence and isolate diagnostics

**Goal:** native linked views and exported metrics distinguish accepted hybrid behavior, equal-request teacher-stage disagreement and independent stop-on-reject rollout.

**Dependencies:** M1 typed context and M2 durable selections. Standalone diagnostics require the new typed record/schema path; they cannot be smuggled through accepted `StepRecorder` rows. Teacher-stage fixtures need bounded capture of authoritative RHS/operator/context at stage preparation, since accepted archives do not retain them. Stop-on-reject needs a numerical-owned candidate-attempt path without fallback: current ProviderWorkspace overwrites rejected pressure with its reference fallback. Observing a completed production call cannot implement this diagnostic mode.

**Proposed write set:** the numerical-owned bounded stage-capture and candidate-attempt path in `src/solver_provider.rs` and `src/provider_step.rs` (exact API/export/test ownership resolved before admission); recording-owned diagnostic schema/decoder in `src/recording.rs` and a bounded new `src/recording/diagnostic.rs`; new `src/bin/desktop/comparison.rs`; `src/bin/desktop/{run_sets,archive_view,inspector,viewport,history}.rs`; `src/bin/rheon-desktop.rs`; focused recording/comparison tests; an independently authored metric/alignment verifier under `tools/`; `docs/api/{recording,vtk-interchange}.md` only when those owned contracts change, comparison contract and plan records.

**Focused tests:** equal ordinal/different time; wrong interval/endpoint stage; same shape/different geometry; native face divergence versus display reconstruction; zero reference norm; gauge-offset labeling; valid overlap and missing support; fallback/warmup counts; native/model array/cache isolation; teacher requests unaffected by biased predictions; rejected diagnostic cannot decode as accepted; nonfinite payload not replaced by zeros; selected publication unaffected by live appends. No cross-run particle-index matching.

**Gate:** A5–A6 plus affected A7/A8. Produce at least one nonempty matched-interval native-field comparison bound to exact source publications alongside unmatched coverage; all-unmatched output alone is insufficient. Real desktop interaction selects the same physical slice/probe/time, displays common units/color bounds and reports unaligned data honestly. A separately authored analytic/native-bit oracle verifies metric formulas and negative cases reach their intended rejection.

**Replan:** failed-state continuation is required, remapping is scientifically necessary, or recorded data is insufficient for a promised metric. Do not weaken the accepted-state boundary to finish the UI.

### Deferred expansion

Owner: parent/numerical integration lead. Trigger: a concrete next requested capability and its admitted evidence, after the first path is useful. Continuing failed states, viscosity composition, stateful providers, warm starts/preconditioners, GPU/ONNX, remapping and restart each get a bounded new contract and qualification. No speculative implementation is authorized by this list.

## Verification, evidence and reporting

The current CI [feature/executable workflow](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/.github/workflows/rust-rheon.yml) is source context, not evidence run here. Future owners must inspect actual current workflows and run applicable commands on final material:

- `cargo fmt --all --check`
- `cargo clippy --locked --all-targets -- -D warnings` and the supported `--no-default-features` / `--features desktop` variants
- `cargo test --locked` and those same supported feature variants, plus focused changed contracts
- Real executable/runtime, recording/cold-process and affected VTK/browser checks selected from changed consumers
- Native-window repeated Start, select/preflight/run, scrub/live, Cancel/restart and Close-through-terminal observations

Do not blindly require `--all-features`, unrelated benchmarks or a new full-platform claim. No Python test dependencies should be inferred from ambient installation. Use locked/qualified test environments for real model and independent reader checks.

| Claim family | Deciding authority / failure isolation |
|---|---|
| Physical candidate and acceptance | Existing numerical contract plus independent manufactured/assembled operator oracle; fake self-reported residual must fail the host check |
| Model adaptation | Captured exact model/normalization/output fixture and real runtime; package hashes alone establish identity, not numerical validity |
| Storage | Real SQLite transactions, fresh independent reader process and native/hand-decoded bits; readable last record does not prove completed lifecycle |
| Comparison | Independently derived metric expectations and actual geometry/time/stage descriptors; alter one valid input condition per negative case and assert its exact diagnostic |
| Lifecycle/UI | Real child processes, terminal exit/cleanup and native interaction; headless controlled substitutes retain narrower claim labels |

Preserve commands, source/material identities, environment/device/build facts, outcome, relevant outputs and unresolved limits in the ledger/report. Generated logs/data stay ignored or in CI artifacts; compact authored audit references may be tracked. Rerun affected evidence after code or contract changes. Distinguish passed, failed, blocked and never-run claims; no old test count, historical receipt or independent design review proves new implementation.

For timing, preserve scope distinctions between inference/IPC/qualification/fallback/full step, startup, recording and rendering. Retained numerical bytes are not RSS. This roadmap admits no speedup, physical-accuracy, real-time or cross-platform benchmark claim.

## Coordination, publication and stopping

One serial integration owner owns the active plan, schemas, public contracts, shared fixtures, lockfiles and workflows. Independent workers may research/review or implement separately owned bounded paths only after exact assignments; each handoff states primary/adjacent/forbidden paths, dependencies, output/evidence and serial integration order. Reconcile stale proposals before integration if actual mutable-plan concurrency makes that necessary.

The documentation executor created branch `docs/configurable-solver-experiments-20261015` from fetched main `43ed26b645dd42de906d0fac7d1ec7d5ab6e351a` in existing `/workspace/Rheon`; no new worktree was created. Only authored Markdown under `docs/plans/configurable-solver-components/**` is in this publication write set. Parent owns PR disposition and eventual integration. Retain the branch while public review is open, synchronize on material baseline/ownership changes and retire only under the parent's disposition; no merge or shared-history rewrite is authorized. Preserve unrelated work and shared history; normal push and draft PR are separate from merge/deploy. Do not invent branch-per-milestone or commit-count rules. Any task-created worktree's purpose, owner and retained/removal disposition must be recorded before terminal cleanup; no repository-wide pruning or force removal is implied.

### Current blockers

- Exact new native integration source is inaccessible, so dependent source/compatibility decisions remain `unavailable`.
- Repository execution is available, fetched main and adopted standards were inspected, and independent documentation review completed. Remote branch discovery returned no target; exact fetch of the reported integration OID returned `not our ref`. This proves the access limit, not a source-level defect or implementation result.
- Physical viscosity and origin/spacing compatibility require M0 disposition before the present lab package can be advertised for the chosen production plan.

### Replan and stop rules

Repair checks within the current admitted slice while objective, authority, owner, contract, risk and evidence meaning remain valid. Stop the dependent work and replan when those facts change, or when observed propagation/cumulative machinery contradicts the composed design.

Specific triggers: bypassed acceptance; teacher leakage; a generic archive accepting rejected states; unowned hidden state; a second loop/supervisor/archive; malicious-code containment becoming a requirement; new supported physical domain; missing public/persisted consumer; changed dependency/platform promise; unavailable oracle; restart/migration/retention deletion; or source reconciliation showing the proposed work already exists with different ownership.

Record trigger/evidence, replace the candidate/binding decision, disposition affected consumers and milestones, rerun the composed-design probe when composition changes, and ask the smallest necessary question only when evidence and existing authority cannot settle it. Do not reduce numerical gates or relabel lower-fidelity evidence to close a milestone.

**Completion rule:** implementation can be accepted only when every non-deferred milestone has its named evidence, all A0–A8 claims are satisfied for the admitted scope, final material/consumer disposition is reviewed, required terminal lifecycle results are observed, and final evidence/publication status is linked. Until then keep the appropriate pending, blocked or verifying status. This proposal closes only research/design and authorized documentation publication.

## Documentation delivery acceptance

This section concerns the present documentation task only. D1 is complete: the
proposal was independently reviewed, committed, pushed and opened as public
[draft PR #84](https://github.com/MrScripty/Rheon/pull/84). Next documentation slice:
`none`. M0 remains the next **proposed** implementation slice, blocked as above.
The [ledger](execution-ledger.md) records actual publication evidence.

| ID | Observable criterion | Kind / environment / mode | Status |
| --- | --- | --- | --- |
| D-A1 | Current source baseline, scientific envelope and alternatives are source-backed | contract / required-real repositories and primary sources / manual | satisfied |
| D-A2 | Typed stage/model/history/runtime/recording boundaries preserve existing authority | contract / not-applicable / independent manual review | satisfied: [review](reports/review.md) |
| D-A3 | Saved scenarios/run sets, physical-time metrics and phased handoff are reviewable | contract / not-applicable / independent manual review | satisfied: [review](reports/review.md) |
| D-A4 | Local links/fences, whitespace and staged/range scope contain only authorized authored prose | focused / not-applicable / automated and manual | satisfied: [ledger](execution-ledger.md) |
| D-A5 | Authorized commit pushed and public documentation PR confirmed, without merge | release-artifact / required-real GitHub / either | satisfied: [PR #84](https://github.com/MrScripty/Rheon/pull/84) |

Documentation acceptance cannot satisfy A0–A8 or authorize implementation.
Documentation delivery acceptance is satisfied; PR disposition belongs to the parent.
