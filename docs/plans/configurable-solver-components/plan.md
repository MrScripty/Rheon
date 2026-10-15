# Configurable Rheon components and saved comparisons

**Document status:** Active implementation plan. The user subsequently authorized M1–M3 implementation, separate draft PRs, validated ready merges and safe branch retirement. This supersedes the earlier documentation-only scope recorded in the historical ledger; numerical and environment gates remain mandatory.

**Current phase:** bounded adapter/context, native saved-run, native quantitative-comparison and linked-view slices have passed independent review and serial integration gates. Typed teacher/stop runtime, schema2 capture, frozen reruns and cold pressure inspection have source-bound qualification and actual software-X11 fixture evidence. An independent follow-up challenges failed-start custody, frozen chronology, oracle range and cached synchronized inspection; its reviewed app-specific seed correction preserves generic archive semantics. Exact source, test and merge dispositions are recorded chronologically in the ledger and linked PRs. No accepted learned-provider composition is claimed.

**Next slice:** complete the remaining reviewed follow-up qualification and serial merge gates, retire only safe merged branches, then obtain the missing measured bundle/runtime/parity inputs for the separate model acceptance path. Preserve unresolved A1–A8 criteria and adverse accuracy evidence.

**Implementation acceptance:** pending against A1–A8. Actual stdlib CPU processes qualify the stated protocol/lifecycle fixtures, not the absent measured trained deployment; isolated Xvfb native-window fixture interaction is observed. The measured trained bundle/Torch remains unavailable; required model-parity and full generic milestone claims stay pending.

**Canonical repository path:** `docs/plans/configurable-solver-components/plan.md`. Exact M1 ownership and cross-thread consumer handoff: [M1 admission](reports/m1-handoff.md). The integration lead owns shared source and all merges; unrelated renderer/training work is excluded.

**Supporting records:** [Research/design report](reports/research.md), [proposed typed contracts](reports/contracts.md), [M0 source reconciliation](reports/m0-reconciliation.md), [execution ledger](execution-ledger.md), [issues](issues.md), [standards route](reports/standards-and-design.md), [independent review](reports/review.md). The report is reviewed research evidence; this plan owns the current proposal and handoff, while contracts detail boundary semantics. Their review is not implementation acceptance.

## Objective

Let an operator select a compatible independently packaged pressure component, save the exact simulation and run-set configuration, execute it through Rheon's existing numerical/worker owners, and cold-open quantitative comparisons of the resulting native/model runs. Compatible model channel, normalization, weight and runtime changes should stay in the model package after the seam exists. Scientific domain, operator or stage changes remain explicit numerical work.

The first product outcome is a native desktop vertical path, backed by public numerical contracts and durable data: select provider → see support/preflight → run → inspect actual selected/fallback behavior → save → close producer → reopen and compare exact recorded publications.

## Source versions and applicable standards

| Source | Reviewed material / limit |
|---|---|
| Rheon | Main `43ed26b645dd42de906d0fac7d1ec7d5ab6e351a`; [provider](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/src/solver_provider.rs), [session](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/src/session.rs), [recording](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/api/recording.md), desktop and VTK contracts |
| Published Rheon integration | Reviewed `integration/ai-pressure-comparison-20261015` at `0fbadbe733b5b521e192dc8a2aa980d6d69a1e01`; original `66e50388ec7a788dbda0ad1227e330996edf506e` is accessible and an unchanged ancestor. [M0](reports/m0-reconciliation.md) maps source, physical envelope, retained evidence and limits |
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

- Promotion of the existing experimental failed-AI-state continuation to a generic contract, until its typed diagnostic advance/hard-stop/history policy is independently qualified. Preserve and label the current raw diagnostic behavior; initial generic comparison policy remains stop-on-reject.
- Viscosity/provider composition beyond current main, arbitrary stage graphs, new physical operators, variable-density/free-surface support and obstacle coupling.
- Recurrent provider state, learned initial-guess/preconditioner roles, GPU IPC, new runtime families, a marketplace, distributed scheduling, browser plugin panels and integrator restart.
- Model retraining, new benchmark/speed/physical-accuracy claims and production deployment. Main merges are authorized only after applicable validation and independent review.

Existing numerical gates and accepted-publication semantics remain authoritative. Existing archive v1 and VTK/dense3D versions are not relabeled. A rejected standalone candidate cannot become an accepted archive ordinal. A finite rejected-candidate metric may accompany a genuinely accepted fallback publication through its provider report.

The reviewed lab network is stateless but pressure-conditioned; 15 native warmup steps precede its first supported call. Current scalar spacing means cubic cells, not equal axis counts. Nonzero physical origin and anisotropic spacing are unqualified for the reviewed adapter. The published diagnostic engine actually applies viscosity `0.001 Pa·s` before pressure, resolving the earlier local-physics mismatch. Main's unchanged full-step provider still has no viscosity composition; the diagnostic engine does not admit that production plan. [M0](reports/m0-reconciliation.md) establishes fixed dt/rho, zero origin, sealed free-slip walls, exact channel/reconstruction policy and continuation limits.

## Objective acceptance claims

A0 is current read-only source/design evidence, satisfied within its stated scope. A1–A8 are implementation acceptance claims with the bounded progress recorded in the ledger. No implementation, accuracy or fresh model/runtime execution acceptance follows from A0 or retained producer logs.

| ID | Observable criterion | Kind | Environment / mode | Current status |
|---|---|---|---|---|
| A0 | Exact integration/source/consumer inventory establishes reuse and one explicit existing diagnostic envelope | source/design review | Pinned Rheon/lab source plus retained receipts / independent read-only review and hash audit | satisfied: [M0](reports/m0-reconciliation.md); no runtime rerun or implementation admission |
| A1 | Two real compatible adapter/package revisions change channel preparation/weights without edits to numerical or UI owners; incompatible requests reject before inference | public contract + real-runtime integration | Pinned Rust/Linux CPU and actual model bundle / automated | pending |
| A2 | Full-step acceptance, state/history rollback, same-request fallback and cancellation retain existing meaning | numerical/API contract + independent oracle | Native Rust CPU / automated | pending |
| A3 | A completed accepted provider run is closed and selected fields/provenance reopen exactly in a fresh process without loading model code | persistence + system | Local SQLite and separate record/inspect processes / automated | pending |
| A4 | A saved set preserves immutable variant intent, actual outcomes and missing/corrupt/incomplete states without substituting live/latest data | persisted contract + workflow | Native desktop plus real archives / automated and native interaction | pending |
| A5 | Metrics/linked views compare compatible actual stages/times/geometry and report unmatched coverage, warmup, fallback and rejected diagnostics correctly | oracle + user workflow | Native numerical fields and window / automated plus manual interaction | pending |
| A6 | Teacher-stage inputs remain native; independent stop-on-reject inputs/history never leak across tracks; diagnostic schema cannot pass accepted decoders | isolation/contract + real integration | Distinct native/model contexts and real storage / automated | pending |
| A7 | Process failure, cancellation, Close, repeated Start and storage failure reach honest terminal states with no stale publication or owned child left running | lifecycle/system | Supported OS, real runtime/processes/window / automated plus native interaction | pending |
| A8 | Pinned formatting, lints, affected feature/API/recording/VTK/browser regressions pass for final material; unsupported platforms remain explicit | static/build/regression | Actual supported toolchains/targets / automated | pending |

Actual software-X11 interaction now covers the bounded native saved-run, linked slice/probe and typed diagnostic fixture workflows recorded in the execution ledger, including cold reopen, rerun, Cancel and Close. These observations do not qualify the absent measured deployment or every platform and acceptance criterion above. Headless egui tests and screenshots of a started window alone do not establish an interactive workflow.

## Proposed decisions and owners

These become binding only upon implementation admission and current-state reconciliation.

| Decision | Owner and reason |
|---|---|
| Typed finite stage plan; no universal tensor graph | Rheon numerical owner; ordering/operator validity is scientific, rather than metadata compatibility |
| Full pressure candidate is the initial replaceable role | Rheon dispatcher; correction reconstruction remains adapter-owned and independently qualified |
| Explicit host history, accepted-step commit | Numerical/session owner; retries cannot advance hidden model history |
| Package preprocessing/weights/support | Lab project owner; compatible model evolution stays local to its source and evidence |
| CPU subprocess selected explicitly | Existing ai_comparison process/worker owner; reuse direct-child/reader cleanup, preserve measured limits and keep imports/GPU initialization outside the application |
| Trusted-local capability label | Runtime/security owner; same-user subprocess isolation is not malicious-code containment |
| Existing engine, worker and recording modules retain custody | Existing diagnostic Engine/Worker/Record and production session/SQLite owners remain distinct. Preserve raw JSON evidence; no competing numerical loop/supervisor or relabeling as accepted data |
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
6. **Independent evolution/failure/replacement:** built-in numerical providers remain usable without external runtime; incompatible/unavailable models do not prevent cold inspection. Qualified production policy permits only same-request numerical fallback; independent diagnostics never silently fall back and retain their declared stop/continuation verdict. Storage failure preserves numerical acceptance and reports a possible unrecorded tail; analysis failure does not mutate source runs. Actual public/independently deployed consumers determine version overlap; coordinated private interfaces change atomically.
7. **Deletion result:** deleting typed stage context scatters physical/history meaning into adapters and callers; deleting adapter scatters channel/reconstruction knowledge into UI/core; deleting runtime supervision scatters child/resource ownership; deleting run-set manifest removes saved grouping/intent, not numerical data; deleting comparison validator scatters alignment policy across views. Do not add a new archive, queue/service, marketplace, universal graph or GPU sharing layer: deleting those proposed mechanisms removes incidental machinery without losing the first outcome.
8. **Necessary cumulative complexity:** retain one active numerical owner per explicit mode (existing diagnostic Engine or accepted Session), the existing worker lifecycle, recording ownership, a narrow pressure context, captured provider binding/runtime, saved specifications and a comparison recipe. Do not implement another loop for either mode. Contain unavoidable stage/geometry/time/provenance/resource knowledge at these boundaries. M1's change-locality evidence and M2/M3's cold/lifecycle tests decide whether this composition is actually simpler; broader propagation triggers redesign.

**Authority-scope admission:** a package or saved spec may reference stage, model, runtime, source and analysis identities, but owns only that binding/recipe. It does not become canonical for those referenced concerns. Old archive interpretation is retained by its versioned reader; rerun identity includes required material closure; neither hash integrity nor saved metadata establishes authenticity, numerical reproduction or restartability.

## Design observations and dependency gates

| Uncertainty | Decision / least costly adequate observation | Owner / stopping condition |
|---|---|---|
| Source access resolved | Fetched exact original and published descendant; complete worker/engine/process/record source and receipt audit | [M0](reports/m0-reconciliation.md) complete; integration owner retains implementation/merge disposition |
| Diagnostic physics reconciled; production admission remains | Source traces actual viscosity/residual/gauge/origin/spacing/dt/warmup and retained measured requests. Qualify any promotion through unchanged production session separately | Numerical + lab/API owners / existing diagnostic envelope explicit; no generic production support inferred |
| Existing process protocol and retained local run | Hash audit binds current worker and five retained logs; real ignored lifecycle test reports immediate-child cancel/restart. Fresh rerun requires the actual measured bundle/runtime | Runtime owner / normal direct-child behavior supported; descendant, blocking-write, cleanup-error and hard-timeout limits remain; no new execution claimed |
| Stable saved configuration codec | Inventory actual controls and consumers; select a reputable versioned codec if current dependencies do not supply one | Config owner / closed schema and reconstruction coverage established; no parsing Debug text |
| Diagnostic persistence/continuation | v1 is accepted-only; design a separate typed diagnostic record path under same recording owner; stop-on-reject initially | Recording + numerical owners / schema/reader dispatch proves no accepted-state confusion |

Investigations end at these decision-relevant results. Do not delay the first reversible admitted slice for GPU benchmarks, universal framework comparisons or unrelated solver research. Conversely, do not expand around an unavailable real-interface observation that could invalidate it.

No new package is selected by this draft. Prefer current approved Rust/SQLite/hash/UI/work scheduling and reviewed deployment code. Any new parser/runtime/test dependency requires purpose, exact version/transitive/license/toolchain/feature review. No GPL implementation adoption is proposed; scientific references do not confer rights to code, weights or datasets.

## Milestones and integration sequence

M0 source reconciliation is complete and its documentation PR is merged. M1–M3 are now admitted under the serial boundaries in the M1 handoff. M1 is active; independently authored new native M2/M3 contracts await integration. Conditional production writes still require physical qualification and are not authority for broad directory edits.

### M0: reconcile source and prepare vertical-slice admission

**Goal:** one current source/consumer/authority matrix and a reviewable first-slice design.

**Draft write set:** this plan, ledger/issues and a source-review report; no production files.

**Tasks/result:** inspected checkout/instructions and pinned native/lab source; preserved unrelated work; reconciled stage/domain/history/runtime/recording owners and diagnostic viscosity/origin/spacing envelope; retained independent version roles and actual consumers; replaced speculative M1 paths with existing composition points and conditional production boundaries. Exact implementation assignments/production qualification require separate owner admission and are not represented as completed code work.

**Gate/status:** A0 source/design reconciliation satisfied at `0fbadbe` + lab `a842606`: [M0 report](reports/m0-reconciliation.md) and [independent review](reports/review.md). The real executable diagnostic envelope and normal immediate-child lifecycle are supported by inspected source and audited retained observations. No fresh inference/runtime test ran here. Production promotion, generic contract work and new implementation admission are not part of this completion. The unchanged core/SQLite owners and existing experimental engine/process/worker/JSON owners remain distinct.

**Replan:** a different scientific owner, domain, trust tier or execution environment; material changes in the reviewed integration/source baseline or actual local ownership.

### M1: select and execute a typed CPU pressure component

**Goal:** extend the existing desktop comparison to select two compatible captured adapter/package revisions through a typed pressure context, with honest support/verdicts and preserved history. Existing raw diagnostic behavior remains distinct from a separately admitted qualified provider+fallback production path.

**Proposed write boundary:** retain and adapt `src/bin/desktop/ai_comparison/{engine,model_process,worker,mod}.rs`, their owned tests and the matching lab package's adapter/export. First derive typed request/support mapping at existing `Engine::advance` and `ModelProcess::predict`; retain worker supervision and UI entry points. The API owner alone may assign additive `src/solver_provider.rs` context/`src/lib.rs` exports and affected contract tests. `src/provider_step.rs`, `src/session.rs`, `src/simulation.rs`, `src/liquid_step.rs` and full-step documentation are **conditional separately qualified production-composition writes**, not presumed necessary for local package evolution. Core/defaults/recording ownership stays intact. Cargo/locks/workflows/shared renderer or other local-agent paths require explicit serial ownership and an actual need.

**Preserve:** numerical operator/gates, native defaults, core-only behavior, synchronous library lifecycle and archive contracts; existing experimental diagnostic verdict/continuation and original raw evidence. Do not introduce native fallback into the current independent AI track or call it accepted production. Reuse camera/voxel/training-viewer shared owners; coordinate their consumers before any source overlap.

**Focused tests:** named-field/unit/stage/gauge/layout checks; unequal counts with cubic cells; explicit anisotropic-spacing/nonzero-origin/unsupported-mu rejection (the existing diagnostic mu=0.001 Pa·s is supported); exact fixed diagnostic dt and declared production dt; absent/stale/other-track context; host residual convention; zero-residual bypass; invalid/short/nonfinite output; unknown version and mutated package material. Test diagnostic continuation/stop policy and lineage separately from accepted production history. Only a separately admitted production binding runs same-request fallback and no-history-advance-on-final-rejection gates. Two compatible real packages/revisions prove change locality; a test double proves only its stated contract.

**Gate:** A1 plus M1 portions of A7/A8: actual runtime parity, no optimizer updates/device-policy violations, real package selection and a native-window workflow under the selected explicit diagnostic or production mode. A2 is required only for a separately admitted physically compatible production provider binding/composition and remains pending if M1 is diagnostic-only. No accuracy pass is required of the currently failing checkpoint to prove honest diagnostic failure or qualified fallback; neither mode may relabel a model failure as success. Diagnostic-only M1 does not close the accepted-provider prerequisite for M2.

**Replan:** compatible model changes require core/UI-specific channel edits; the actual bridge cannot support the selected lifecycle; no physically compatible plan exists; hidden state requires commit/rollback support.

### M2: save accepted runs and reopen a run set

**Goal:** execute, persist, close and cold-open an immutable saved simulation/run set with complete accepted provider provenance.

**Dependency:** accepted M1 contracts **and**, for accepted provider/session recording, a separately admitted physically compatible production binding/composition with A2 satisfied. Diagnostic-only M1 is insufficient. A separately admitted accepted-native-only configuration/recording slice may proceed independently; diagnostic JSON remains outside accepted rows. Existing Scene/Record JSON and lab field import are useful custody evidence, not completion of a generic immutable SimulationSpec/run-set/SQLite cold workflow.

**Proposed write set:** `src/recording.rs`, a narrowly owned new `src/recording/provider.rs`; accepted capture in `src/session.rs` only if required; new `src/bin/desktop/run_sets.rs`; `src/bin/desktop/{archive_view,identity,inspector,settings,history}.rs`; `src/bin/rheon-desktop.rs`; `tests/{recording_contract,recording_carrier_contract,session_contract}.rs`; new `examples/recorded_provider.rs`; `docs/api/recording.md`, necessary saved-spec contract and plan records. Dependency/schema/lock changes remain serial and explicit.

**Preserve:** v1 interpretation, exact native bits and independent public/VTK versions; no restart claim; accepted capture before another attempt invalidates source stamps; recording failure reports an unrecorded tail rather than rolling back physics.

**Focused tests:** fresh-process record/verify/select/export; no model load on cold inspection; immutable revision capture; missing/corrupt/incomplete/unsupported member; same ordinal while another track advances; unknown field stays unavailable; capacity/poisoned writer/uncertain commit/terminal-status failure; no overwrite/deletion on recovery; saved Debug text is not executable configuration.

**Gate:** A3–A4 plus affected A7/A8. One real saved two-variant set survives producer exit and reopens exact fields, source times, identities and selected/fallback outcomes.

**Replan:** a second numerical archive or source-of-truth catalog is introduced; automatic migration/restart/retention deletion becomes necessary; a saved spec cannot reconstruct the promised controls.

### M3: compare actual numerical evidence and isolate diagnostics

**Goal:** native linked views and exported metrics distinguish accepted hybrid behavior, equal-request teacher-stage disagreement and independent stop-on-reject rollout.

**Dependencies:** M1 typed context and M2 durable selections. Existing Engine already prepares its own request and advances a raw no-fallback AI diagnostic track; reuse that owner, preserving current evidence and explicitly labeling its continuation policy. Initial generic comparison policy remains stop-on-reject; promotion of failed-state continuation requires its typed contract/qualification. Teacher fixtures still need bounded persistent capture of authoritative RHS/operator/context: transient Prepared values and current accepted/JSON pressure-tracer frames are not complete equal-request fixtures. Current production ProviderWorkspace still overwrites rejected candidates with fallback, so observing that returned call cannot implement raw candidate diagnostics. Standalone SQLite diagnostics need a separate typed record/schema under recording ownership, not accepted ordinals.

**Proposed write set:** bounded stage-capture/attempt/mode contracts around existing `src/bin/desktop/ai_comparison/engine.rs` and worker/record/view consumers; public `src/solver_provider.rs`/`src/provider_step.rs` changes only if the API owner admits a production diagnostic boundary; recording-owned diagnostic schema/decoder in `src/recording.rs` and a bounded new `src/recording/diagnostic.rs`; new `src/bin/desktop/comparison.rs`; `src/bin/desktop/{run_sets,archive_view,inspector,viewport,history}.rs`; `src/bin/rheon-desktop.rs`; focused recording/comparison tests; an independently authored metric/alignment verifier under `tools/`; `docs/api/{recording,vtk-interchange}.md` only when those owned contracts change, comparison contract and plan records.

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

- Source-access blocker resolved: original exact fetch and published descendant now accessible; [M0](reports/m0-reconciliation.md) supersedes the initial failed lookup.
- Existing diagnostic viscosity/profile is reconciled. Production viscous-session composition and any new supported semantics require numerical/API-owner qualification; nonzero origin, anisotropic spacing, other rho/dt/boundaries remain outside the current adapter.
- Actual measured deployment/installed runtime is absent here; retained local lifecycle logs were hash-audited, not rerun. Generic runtime robustness, required native-window workflow and supported-platform claims remain pending.
- Generic configurable contracts, typed saved specs/run sets, accepted provider recording and SQLite diagnostic integration remain future work. Actual implementation ownership must reconcile local-agent dirty/serving paths from the published handoff before overlapping.

### Replan and stop rules

Repair checks within the current admitted slice while objective, authority, owner, contract, risk and evidence meaning remain valid. Stop the dependent work and replan when those facts change, or when observed propagation/cumulative machinery contradicts the composed design.

Specific triggers: bypassed acceptance; teacher leakage; a generic archive accepting rejected states; unowned hidden state; a second loop/supervisor/archive; malicious-code containment becoming a requirement; new supported physical domain; missing public/persisted consumer; changed dependency/platform promise; unavailable oracle; restart/migration/retention deletion; or source reconciliation showing the proposed work already exists with different ownership.

Record trigger/evidence, replace the candidate/binding decision, disposition affected consumers and milestones, rerun the composed-design probe when composition changes, and ask the smallest necessary question only when evidence and existing authority cannot settle it. Do not reduce numerical gates or relabel lower-fidelity evidence to close a milestone.

**Completion rule:** implementation can be accepted only when every non-deferred milestone has its named evidence, all A0–A8 claims are satisfied for the admitted scope, final material/consumer disposition is reviewed, required terminal lifecycle results are observed, and final evidence/publication status is linked. Until then keep the appropriate pending, blocked or verifying status. Research/design publication is complete. Implementation remains Active until its admitted claims have their required evidence; bounded native slices do not close unimplemented provider/diagnostic claims.

## Documentation delivery acceptance

This section preserves the completed original documentation delivery. D1 was
independently reviewed, committed, pushed and published as [PR84](https://github.com/MrScripty/Rheon/pull/84), subsequently merged under the user's later authority.
M0 source/design reconciliation is complete. Subsequent M1–M3 implementation and
validated merge authority supersede the original prose-only scope; their active
status and remaining gates are recorded above and in the [ledger](execution-ledger.md).

| ID | Observable criterion | Kind / environment / mode | Status |
| --- | --- | --- | --- |
| D-A1 | Current source baseline, scientific envelope and alternatives are source-backed | contract / required-real repositories and primary sources / manual | satisfied |
| D-A2 | Typed stage/model/history/runtime/recording boundaries preserve existing authority | contract / not-applicable / independent manual review | satisfied: [review](reports/review.md) |
| D-A3 | Saved scenarios/run sets, physical-time metrics and phased handoff are reviewable | contract / not-applicable / independent manual review | satisfied: [review](reports/review.md) |
| D-A4 | Local links/fences, whitespace and staged/range scope contain only authorized authored prose | focused / not-applicable / automated and manual | satisfied: [ledger](execution-ledger.md) |
| D-A5 | Authorized commit pushed and public documentation PR confirmed, without merge | release-artifact / required-real GitHub / either | satisfied: [PR #84](https://github.com/MrScripty/Rheon/pull/84) |

Publication alone cannot satisfy source/runtime/implementation claims or authorize implementation; A0 is satisfied by the separately reviewed M0 source evidence, not publication.
Initial documentation delivery was satisfied. The source-reconciliation update has independent review and scoped checks recorded in the ledger; PR disposition belongs to the parent.
