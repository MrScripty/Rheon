# Rheon: configurable numerical components and saved comparisons

Research and design recommendation, 15 October 2026.

## Recommendation

Build a small, typed component seam around Rheon's existing numerical owners. A saved simulation selects a qualified stage plan and explicit provider bindings; model packages adapt named physical fields to their own tensors. The existing simulation/session remains the only owner of time advancement and accepted publication. Saved run sets group the resulting existing SQLite recordings and a versioned comparison recipe.

The first useful slice is deliberately narrow: sealed constant-density MAC pressure candidates, native reference fallback, explicit previous-pressure context, persisted provenance, and quantitative native/model comparison. It should establish the extension boundary once, so later compatible checkpoints and preprocessing revisions do not require editing Rheon's desktop UI or numerical owners.

Seven decisions matter most:

1. Separate physical stage contracts, model adaptation, execution runtime, and qualification.
2. Give fields explicit geometry, sampling, units, temporal stage and lineage. Tensor shape alone cannot establish compatibility.
3. Keep operator assembly, acceptance thresholds, corrected-state checks and final commit in Rheon.
4. Distinguish qualified fallback, teacher-stage evaluation and independent diagnostic rollout in both storage and UI.
5. Reuse `SimulationSession`, the joinable worker, `StepRecorder`/`RecordingReader`, and current VTK interchange. Add adapters and saved configuration, rather than another solver loop or numerical archive.
6. Make run-set membership and comparisons immutable and reproducible to the extent the actual captured material permits. Cold inspection must not require executing the original model.
7. Promise configuration within a qualified scientific envelope. Arbitrary operator changes, new state quantities or new coupling schemes can still require numerical work and a new typed contract.

This is a proposed architecture. No implementation, training, runtime qualification, performance measurement, repository publication or deployment was performed for this report. External precedents inform the design; none is proposed as a wholesale framework replacement.

## 1. Source baseline and the actual gap

### Verified Rheon main

The source baseline is `43ed26b645dd42de906d0fac7d1ec7d5ab6e351a`.

- `PressureRequest` supplies grid, constant density, actual dt and the integrated full ungauged RHS. It does not supply previous pressure. `SolverMetadata` currently describes one input/output tensor and uses static string identity fields. Merely replacing an artifact path cannot describe an arbitrary multi-field, history-dependent model. The same source already states that new stages need their own typed request/qualification API. [Pressure provider source](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/src/solver_provider.rs).
- The full-step provider path is sealed, constant-density and full-box. Its published contract has no viscosity parameter and explicitly excludes other domains. The dispatcher qualifies pressure, but later actual stored-velocity divergence, Courant, tracer, time and liquid checks still gate full publication. A later full-step failure does not silently trigger a whole-step retry with another solver. [Full-step provider contract](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/api/full-step-provider.md).
- `SimulationSession` owns accepted state, quotas and cooperative deadlines. Snapshots borrow one coherent publication; source/run identity is caller-declared, rather than authenticated provenance. Provider internal state and side effects are not automatically rolled back when a full step fails. [Session source](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/src/session.rs).
- SQLite already stores native fields and accepted-publication metadata atomically per publication, with bounded readers and indexed selection. Integrated producers cover material/SPH and the native fixed-box carrier. Provider/session/viscosity producers are explicitly not yet connected. The archive is inspection/interchange data, not a restart checkpoint; current exact configuration is opaque Rust Debug text, not a stable executable specification. [Recording contract](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/api/recording.md).
- The desktop already has one joinable worker, immutable run requests, independent comparison tracks, accepted-frame publication and cancellation/close ownership. Its appearance preview can be downsampled and is not a quantitative numerical checkpoint. VTK already preserves native field associations and distinguishes interval projection pressure from endpoint/absolute pressure. [Desktop ownership](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/src/bin/desktop/README.md), [VTK interchange](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/api/vtk-interchange.md).

### New integration context and review limit

The verified lab baseline is `a8426061380e82951b4dba1bbfa38ff93dc7ebb2`:

- Deployment v2 captures weights, normalization, source material and runtime identity, but declares the fixed `sealed-water-seven-channel-float32` profile and a specific physical/context policy. The CPU bridge consumes base pressure and native input residual, checks cubic water/dt bounds and returns full pressure. [Deployment exporter](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/deployment.py), [CPU bridge](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/cpu_bridge.py).
- Model preprocessing builds residual, previous/base pressure, X/Y/Z, density and dt channels, then converts to float32. Inference gauge-corrects a predicted correction and adds it to the base; exactly zero residual bypasses the network. These are legitimate adapter semantics, rather than seven universal Rheon fields. [Model input/reconstruction code](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/cuda-pressure-v1/model.py#L154-L188).
- The lab's published experiment report describes 15 native warmup steps, first inference at step 16 and independent histories afterward. It explicitly says this network is not recurrent: previous pressure is an input field, not a hidden all-frame memory. Warmup fixes the declared context mismatch but does not resolve the recorded pressure-accuracy failures. Those are the report's existing diagnostic findings, not a new benchmark or independent reproduction here. [Lab comparison results](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/docs/plans/live-ai-pressure-comparison/results.md).

Two specific physical-contract gaps need admission rules. The deployment declares dynamic viscosity `0.001 Pa·s`, while the reviewed main full-step provider has no viscosity parameter. The reviewed model builds coordinate channels as `(index + 0.5) × h`, while its CPU bridge carries scalar cell size and no physical origin. Nonzero-origin use is therefore unqualified by that adapter; do not silently infer a world-coordinate transform. “Cubic” here means cubic cells with one scalar spacing, not necessarily equal counts in X/Y/Z. These are limits of the reviewed interfaces, not source-level findings about the inaccessible new Rheon branch.

This establishes a useful existing adapter/deployment precedent. It does not establish a general stage contract or production-qualified viscosity/provider composition.

The newer Rheon target `feat/live-ai-pressure-comparison`, reported head `66e50388ec7a788dbda0ad1227e330996edf506e`, was inaccessible during this research. This report does not claim to have reviewed its changes, run its tests, or found source-level defects in it. Reconcile the recommendations against that implementation before planning a patch.

## 2. What the primary-source precedents actually support

| Precedent | Useful lesson | Limit for Rheon |
|---|---|---|
| FMI 3.0.2 | Separate importer-owned integration from self-stepping components; describe variables/capabilities outside executable code; advertise state restore support explicitly | An FMU is not a security sandbox, and an interface cannot establish the correctness of a PDE discretization |
| CSDMS BMI 2.0 | Query variable type, units, grid and location; keep model control separate from data queries | Its general field vocabulary is not a complete MAC staggering, pressure-gauge or accepted-state contract |
| preCICE | Coupled iterations require checkpoint/restore and explicit convergence ownership | A new coupling algorithm is numerical functionality, not just a different list of plugin names |
| VTK / ParaView | Preserve geometry and point/cell/field association; use linked views, explicit time, parameter studies and quantitative field operations | A screenshot or automatically resampled field is not evidence of numerical acceptance |
| PETSc KSP | Residual meaning and norm must be named; solver/preconditioner combinations carry mathematical restrictions | A low residual is not the same quantity as physical solution error |
| ONNX | Version executable graph/operator requirements and reject unsupported operators | Executable tensor compatibility does not encode pressure semantics, grid layout or training support |

Evidence: FMI defines Model Exchange, Co-Simulation and Scheduled Execution as different ownership models, exposes declarative variables/capabilities, defines SI-dimensional units, and makes get/set/serialized state optional capabilities. Its C API does not restrict operating-system services. These are reasons to borrow the separation, without implementing all of FMI. [FMI 3.0.2](https://fmi-standard.org/docs/3.0.2/).

BMI describes variable metadata and model grids independently. A particularly relevant warning: BMI's structured grid shape uses `ij` order and counts nodes; Rheon's native carrier fields use x-fast storage with distinct cell and face dimensions. An adapter needs a tested mapping, rather than matching three integers. [BMI variable metadata](https://bmi.csdms.io/en/stable/bmi.var_funcs.html), [BMI grid metadata](https://bmi.csdms.io/en/stable/bmi.grid_funcs.html).

preCICE's implicit-coupling instructions require saving all state needed to repeat a coupling iteration and advancing only after convergence. This supports an explicit state/commit contract for future coupled stages; it does not justify adding a second time loop now. [preCICE implicit coupling](https://precice.org/couple-your-code-implicit-coupling).

The architecture below is a design inference from these sources and the verified Rheon ownership boundaries, rather than a claim made by those projects.

## 3. Keep five owners distinct

| Owner | Responsibility |
|---|---|
| Rheon numerical core | Physical state, geometry/operator construction, actual dt, qualified stage ordering, acceptance and final commit |
| Rheon adapter/runtime layer | Resolve the selected provider; marshal immutable stage requests; supervise execution; validate transport; return candidates and diagnostics |
| Lab project | Training code, datasets/selection, checkpoint and model source, channel preparation, normalization, correction reconstruction, support declaration and evaluation evidence |
| Rheon application | Saved simulation drafts, run sets, worker scheduling, recordings, inspection and comparison UI |
| Tuldok | Generic project/source/runtime/artifact custody and training/evaluation orchestration, through its own owners; no Rheon-specific pressure equations or universal fluid validity policy |

A model package can contain lab-owned adapter code and a manifest consumed by Rheon. Sharing that package format or generic runtime facilities with Tuldok is useful. Moving fluid-stage semantics into Tuldok would make the generic product carry a scientific contract it cannot own.

Use four separate version axes:

- **Stage contract:** physical inputs, outputs, stage timing, invariants and ownership.
- **Adapter contract:** mapping, transforms, reconstruction and support constraints.
- **Runtime protocol:** transport, lifecycle, capabilities and error semantics.
- **Content identity:** exact source, weights, manifest, dependencies and configuration.

A package semantic version does not replace any of these. Unknown required schema/contract versions fail before execution. Historic recordings retain their original interpretation; upgrading a provider never relabels an old result.

## 4. Typed solver-stage contracts

### 4.1 Use a finite qualified stage plan first

Start with named host-supported plans, such as the existing sealed-box carrier step with one replaceable pressure-candidate slot. Expose the effective sequence read-only in the UI, with bounded options qualified by the numerical owner. Do not start with a universal node editor or arbitrary JSON graph whose edges imply scientific validity.

Within that plan, a provider binding names a stage contract, provider/adapter identity, configuration, runtime and qualification policy. A new model with a compatible physical request can be added without editing the solver loop. A new physical stage or changed ordering requires a new qualified plan or typed API and numerical tests.

For example, `advection → force → viscosity → pressure → projection` is only a conceptual sequence. The currently verified full-step provider path does not admit viscosity composition. Adding a viscosity checkbox to its manifest must not make that composition production-supported. Rheon must first implement and verify the relevant state, splitting order, constraints, cancellation and work accounting.

### 4.2 Field descriptors need scientific meaning

Each stage input/output should have a stable semantic key and a descriptor with:

- Quantity and role: full pressure, pressure correction, velocity face component, integrated RHS, residual, mask, density, viscosity, etc.
- Units as machine-checkable dimensions/scale where supported; retain display units separately.
- Geometry identity and actual counts, origin, spacing, coordinate frame, axis order and domain/boundary contract.
- Association: cell, X/Y/Z face, particle or global scalar; sample offset and component convention.
- Dtype, dimensions, strides/layout, byte length and device/copy requirements.
- Stage/clock: accepted state, tentative state or interval diagnostic; evaluation time or interval start/end; actual dt.
- Provenance: source run, accepted generation, producing stage and exact source field/revision.
- Validity/missingness policy and any mask with its own meaning. A liquid fraction or appearance tracer is not implicitly a solid/validity mask.

Rheon's existing archive descriptors already supply much of the storage-side vocabulary. Extend/reuse it at the execution boundary, while preserving the fact that an execution request can contain tentative fields that are not eligible for accepted recording.

VTK's explicit geometry and data association are useful interchange precedents, rather than a proposal to pass `vtkDataSet` objects through the Rust solver. [VTK XML data model](https://docs.vtk.org/en/v9.6.1/vtk_file_formats/vtkxml_file_format.html).

### 4.3 A pressure-context contract, rather than a seven-channel core API

Add an additive versioned pressure context with host-owned typed fields. Keep the existing request/API usable where feasible; decide the Rust compatibility strategy during implementation review.

The new context should bind:

- Retained operator/domain identity, grid, density and actual interval dt.
- The canonical full integrated RHS, including its units, sign and full/gauged convention.
- Optional previous accepted interval pressure, with originating run/generation, interval and gauge. Absence is explicit at construction.
- Any tentative native fields that a qualified adapter actually needs, exposed through named read-only views rather than a mutable simulation handle.
- Cancellation/deadline and input/output budgets.

Do not pass a generic callback permitting a model to replace operator assembly. The current sealed operator remains owned by the dispatcher. Extra context does not expand the admitted physical domain.

If a model requires an initial guess `p₀` and residual `r = b − A p₀`, the first implementation should have the host prepare that named residual with its authoritative retained operator and exact convention. The adapter packs the result; it does not rebuild a competing stencil. A model's residual channel is not automatically the RHS, a preconditioned residual, or the residual of its final reconstructed pressure.

For a correction model, the adapter's declared pipeline can be `physical context → normalized model channels → inference → inverse transform → full pressure candidate`. The exact reconstruction, such as `p = p₀ + Δp`, belongs to that adapter version and must match the training/export convention. Gauge treatment, coordinate channels, normalization constants and degenerate scaling are explicit. Rheon then qualifies the reconstructed full candidate independently.

Model channels remain adapter-owned. Another model can use fewer fields, a different axis order, or several tensors when it maps from the same qualified context. If it needs information the host does not possess or expose, the binding is incompatible. Configuration cannot conjure a missing physical field.

### 4.4 History and state must follow acceptance

Previous pressure means the selected track's previous accepted pressure, not the latest UI frame, a failed prediction, or another track's native pressure. Constructor zeros must not silently masquerade as solved history. A warmup policy states its length and initialization rule; those steps remain visible in provenance and metrics.

Prefer stateless adapters whose needed history is carried explicitly by host context. For a recurrent/stateful provider, require one of:

1. A tested prepare/commit/abort state protocol: only final accepted full steps advance provider state.
2. Complete provider snapshot/restore support, including RNG/history where relevant.
3. Diagnostic-only use, or a clearly documented non-retryable failure policy.

The current in-process provider API does not promise such rollback. Adding it is real work. Rejecting a full step while allowing hidden provider state to advance would make a retry a different experiment.

## 5. Three execution/evaluation modes

| Mode | Input lineage | Output authority | Failure behavior |
|---|---|---|---|
| Qualified provider with fallback | Current staged request from its own accepted simulation history | Full pressure candidate may be selected only after existing pressure qualification and all later step gates | Explicit numerical fallback uses the same staged request; cancellation remains cancellation; later step failure rejects the interval |
| Teacher-stage evaluation | Captured native/teacher stage requests and native accepted history | Diagnostic candidate and metrics; never a separate accepted simulation trajectory | Preserve rejected candidate diagnostics without contaminating native state |
| Independent diagnostic rollout | Candidate track uses its own prior states/history after declared initialization | Separate diagnostic trajectory, visibly outside accepted-production claims unless every required gate passes | Stop, or continue only under an explicit diagnostic policy with invalidity marked; never quietly reset from native history |

The fallback case is not a native-versus-AI independent comparison. A track that falls back often is a hybrid track; record attempted, selected, rejection reason, candidate diagnostics, fallback and accepted counts. Good final fields can otherwise conceal a failed model.

Teacher-stage evaluation isolates local candidate quality on equal inputs. Independent rollout reveals accumulated effects and distribution shift. Report both when judging a history-conditioned pressure component used in a rollout, including the current stateless pressure-conditioned network. If teacher history is injected into an otherwise independent track, record the injection as an intervention and split the analysis segments.

No diagnostic failure may be written as an accepted `SimulationSession` publication. Current archive version 1 permits constructor/accepted publications only; its generic named fields do not authorize rejected steps to acquire ordinals. Persisting diagnostic trajectories therefore requires a new versioned diagnostic record schema and producer under the existing recording owner. Give it a separately typed append/reader path, explicit outcome, lineage and diagnostic stage identity while reusing bounded SQLite field-storage mechanisms. Keep the existing `StepRecorder` accepted-publication contract and canonical decoders strict. This is an extension to the current recording system, not a second numerical archive.

This requirement concerns standalone rejected attempts and trajectory states outside numerical acceptance. Finite rejected-candidate metrics may still accompany a genuinely accepted fallback publication through its provider report; that does not turn the rejected candidate itself into accepted state.

The first implementation can run teacher-stage diagnostics and stop independent evaluation on rejection. Continuing from failed AI pressure requires an explicitly typed diagnostic advance contract owned by Rheon, with its own permitted-continuation conditions, hard-stop gates and diagnostic record schema. It cannot use `SimulationSession` to advance a rejected state. Review the inaccessible new branch's diagnostic implementation before deciding what can be retained; do not implement a competing application-owned solver loop.

Keep existing finite-field validation. For a nonfinite candidate, retain bounded rejection details such as counts/locations rather than serializing NaNs as a normal numerical field or replacing them with invented zeros. Any optional raw failed payload needs separate quarantined artifact handling.

### Scientific validation evidence

The original learned-simulation literature reinforces these distinctions:

- Solver-in-the-Loop distinguishes unmodified solver states from inputs influenced by earlier learned corrections. Use this to separate teacher-stage disagreement from rollout behavior; the selected numerical teacher is a discrete reference, not physical truth. [Um et al., NeurIPS 2020, §2](https://papers.nips.cc/paper_files/paper/2020/file/43e4e6a6f341e00671e123714de019a8-Paper.pdf#page=3).
- GNS defines autoregressive rollout from the model's own evolving states and evaluates one-step and rollout error separately. History and hidden caches belong to that trajectory. [Sanchez-Gonzalez et al., ICML 2020, §§3–4](https://proceedings.mlr.press/v119/sanchez-gonzalez20a/sanchez-gonzalez20a.pdf#page=3).
- MP-PDE describes accumulated-error distribution shift in autoregressive inference. Geometry, coefficients, dt, boundaries and history need held-out tests; a representation accepting these inputs does not prove generalization to arbitrary values. [Brandstetter et al., ICLR 2022, §3](https://arxiv.org/html/2202.03376v2#S3.SS1).
- PDEBench motivates local, boundary, conservation and frequency-sensitive metrics alongside averaged field error. Select physically applicable metrics and include source/boundary balances; do not impose compressible positivity rules on every incompressible intermediate field. [Takamoto et al., NeurIPS 2022, §3.3](https://papers.nips.cc/paper/2022/file/0a9747136d411fb83f0cf81820d44afb-Paper-Datasets_and_Benchmarks.pdf#page=5).
- MOOD illustrates candidate detection followed by robust recomputation from the old time level. Its PDE/scheme-specific guarantees do not transfer to a neural pressure adapter; it is an architectural precedent for preserving the same staged input and committing only after checks. [Dumbser et al., JCP, §4](https://arxiv.org/html/1406.7416v3#S4.SS4).
- Learned corrections, replacements, initial guesses and preconditioners are different roles. Hsieh et al.'s fixed-point/convergence results rely on a particular construction and assumptions; a generic adapter inherits no theorem from a matching tensor signature. [Hsieh et al., ICLR 2019, §3](https://arxiv.org/pdf/1906.01200#page=5).

For the first correction model, reconstructing full pressure inside its adapter still presents the current full-candidate role to Rheon. A future warm-start or preconditioner role needs its own typed algorithm contract. Keep validation/training splits and held-out rollout horizons with the lab's evidence, and do not tune acceptance thresholds on the reported test cohort.

## 6. Model/runtime package and safety

### Minimum inert manifest

A provider package should declare:

- Namespaced provider and adapter IDs/versions; required stage and runtime protocol versions.
- Source/weight/manifest digests and relative artifact inventory; declared origin/commit and dirty-source evidence.
- Named physical inputs, optional/required history, tensor ports, channel ordering, normalization and reconstruction policy.
- Supported domain, geometry/layout, density/dt ranges, dtype/precision and explicit unsupported cases.
- Runtime/dependency requirements, device policy, memory/output budgets, cancellation and state/rollback capabilities.
- Model, code, dependency and training-data license/provenance references, plus validation material identity.

Discovering a package reads bounded static metadata only. It must not import Python, load weights, execute a probe, build/install dependencies or execute a user-supplied command string. A capability probe is an explicit execution step under the selected runtime.

Declared support, observed runtime compatibility and numerical admission are three different results. A source digest identifies bytes; it does not authenticate their author, make them trustworthy or establish physical accuracy.

### Runtime recommendation

Keep native Rust providers for built-in numerical methods. For flexible lab code, use an explicitly trusted-local, supervised subprocess adapter as the first external runtime, attached to the existing worker lifecycle. Reuse any compatible deployed protocol after review instead of introducing another daemon. Start with CPU typed numeric payloads/files or mmap and bounded control messages; keep model/GPU state owned by its runtime process. Measure before adding shared GPU IPC.

A subprocess contains many dependency and crash failures; it is not a security sandbox against code running as the same OS user. Do not import source/checkpoints into the application process merely to inspect them. No network service, runtime installer or database handle is needed at the numerical API boundary. [Python subprocess behavior](https://docs.python.org/3/library/subprocess.html), [Python audit-hook sandbox warning](https://docs.python.org/3/library/sys.html#sys.addaudithook).

The control protocol needs request/attempt IDs, stage/version, bounded schemas, input content identity, output descriptors, status and typed errors. Keep request correlation identity separate from physical accepted generation and interval time. Separate logs from protocol bytes. Reject stale responses after cancellation or an attempt change. Supervise owned descendants, drain pipes, wait/reap on exit and record cooperative versus enforced cancellation honestly. A model process must never append to the numerical archive directly.

ONNX can be a later qualified runtime for compatible exported graphs. It declares graph/type/operator-set requirements, but does not remove the adapter or scientific support contract; unsupported/custom operators must fail compatibility checks. [ONNX IR](https://onnx.ai/onnx/repo-docs/IR.html).

Wasm/Component Model is a possible later restricted execution tier for suitable kernels. Its explicit imports/capabilities are relevant, but ML framework/GPU compatibility and data-copy costs require separate qualification. Wasmtime's sandbox guarantees also depend on safe host APIs and runtime security. Do not require a Wasm rewrite to ship the first lab adapter. [Wasmtime security](https://docs.wasmtime.dev/security.html), [Wasmtime introduction](https://docs.wasmtime.dev/introduction.html).

## 7. Integrate through the existing session and recorder

The proposed production path is:

1. Resolve a saved immutable simulation specification and admitted provider package.
2. Start the existing joinable worker with that captured request.
3. Construct the existing numerical session and separately budgeted adapter/workspace.
4. Let the numerical owner choose actual dt and prepare the existing staged request.
5. Invoke the selected candidate adapter, then the existing dispatcher qualification/fallback.
6. Apply native projection and all existing full-step gates.
7. Commit once through the numerical owner; advance history only after that success.
8. Immediately adapt that coherent accepted publication into `StepRecorder`, including exact selected/attempted provider provenance; publish the matching preview under current ordering.

Use one independently owned session per accepted comparison variant; never share mutable history or provider state. The production path above covers qualified accepted variants. Teacher-stage evaluation does not advance model state, and any continuing failed-state diagnostic trajectory uses the separately qualified diagnostic contract described in section 5. Initially run variants sequentially through the current worker's orchestration, retaining its cancellation, close/join and bounded-resource behavior. A future parallel comparison scheduler must budget total memory/CPU/device use and preserve those same owners.

The missing provider/session recording adapter is an explicit task. Dense3D frame output currently does not serialize the complete provider report, so exporting those frames alone cannot satisfy provenance. Capture source arrays and report metadata immediately after accepted publication, before a later attempt invalidates inspection stamps. [Full-step provider recording limitation](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/api/full-step-provider.md), [recording timing and failure contract](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/api/recording.md).

Keep recording failures separate from numerical rejection: an I/O error cannot undo an accepted numerical step. Preserve the durable recorded prefix, last accepted generation and potentially unrecorded tail exactly as the existing contract requires. A saved run set must not mark an uncertain/active archive completed merely because its last readable record is valid.

## 8. Saved simulations and run sets

### Three records, three purposes

**Simulation specification:** a versioned, machine-readable rerun recipe. Include scenario/domain and initial-state material, geometry, sources/forcing schedules, physical parameters, requested time policy, exact stage plan/provider bindings, solver tolerances, runtime requirements and recording budgets. Keep UI camera/colormap settings in a separate view specification. Current opaque Debug text remains provenance; do not parse it into executable configuration.

**Run instance/attempt:** an immutable resolved specification plus actual runtime/build/device identities, actual accepted time sequence, archive reference, outcome and selected provider reports. Use a stable application-level run identity alongside existing native local IDs; do not reinterpret those IDs as globally unique. Retries are new attempts and cannot overwrite old evidence.

**Run set:** a saved group of variant specifications and/or existing run references, baseline designation and versioned comparison plan. A minimal example is native Jacobi, native SGS and a selected model+fallback, with identical scenario controls and intentionally varied provider bindings. Record those intentional differences explicitly.

Start with a bounded versioned run-set manifest referencing existing per-run SQLite files and immutable configuration/model material. It is an index and analysis recipe, not another numerical archive. Add an application catalog only if browsing scale warrants it; keep numerical BLOB custody in `StepRecorder`.

### Required behavior

- Editing a saved specification/run set creates a new revision. Running captures a resolved immutable revision before work begins.
- Add/reorder/remove set membership without changing the runs themselves. Removing membership does not delete archive files.
- Cold-open saved runs with `RecordingReader` and recorded field descriptors; do not load a provider or regenerate missing fields.
- Missing, corrupt, unsupported, active or incomplete members remain explicit. Never substitute the current simulation, newest run, constructor data or an empty zero field.
- Store archive/run-header and selected publication digests appropriate to their lifecycle. Finalize/close before claiming a whole archive is an immutable artifact; a hash of a live SQLite file is not a stable completed-run identity.
- Persist comparison results as derived metadata referring to exact input publications and analysis version. Recompute only on an explicit analysis request, and retain prior result identity.
- Rerun from a saved specification only when required source, data and runtime are available and compatible. Archive inspection, environment reconstruction, numerical reproduction and integrator restart are separate capabilities.
- Do not offer “resume from selected frame” from current archives. That requires a new complete checkpoint/state capability, including model history and source/emitter/integrator state.

## 9. Quantitative comparison validity

### 9.1 Make alignment a checked part of the comparison

Default to identical physical geometry, association, units, stage and matching actual physical time/interval. A shared ordinal is not sufficient: adaptive dt may produce different accepted times. An interval projection pressure cannot be compared to an endpoint pressure merely because the field names match.

For equal-request stage tests, bind both candidate and reference to the same RHS/operator/context digest. For independent trajectories, each candidate residual is evaluated against its own authoritative request; native and model states can diverge. Label the comparison question clearly.

Initially require exact compatible grid/staggering and matched accepted intervals within an explicitly specified time tolerance. Show unmatched coverage. If later analysis permits temporal/spatial interpolation, record the method, tolerance, source samples, valid overlap and transform. Do not use interpolation across a reset/event discontinuity or silently treat a remapped field's residual as the native discrete residual.

ParaView's multi-input calculator performs elementwise operations and warns that topology/geometry generally need to match; even equal tuple counts do not establish correspondence. Its resampling facilities are useful explicit analysis operations, not automatic proof of equivalence. [ParaView filtering and multi-input analysis](https://docs.paraview.org/en/latest/UsersGuide/filteringData.html).

### 9.2 Metrics to provide first

| Question | First metrics and essential caveat |
|---|---|
| Did the candidate satisfy the pressure equation? | Host-recomputed full true residual L2/L∞, threshold, RHS norm, gauge and predicted divergence; name integrated units/operator/version |
| Did the stored corrected state satisfy the owner? | Actual native staggered stored-velocity divergence, Courant and all current acceptance reasons; not divergence reconstructed from a display grid |
| How different is the field? | Volume-weighted RMSE/L2 and L∞ for compatible pressure/velocity fields, with units, valid-cell count and domain coverage |
| What happened during rollout? | Error over matched physical time, first invalid/rejected step, accepted/attempted horizon, native warmup length and fallback frequency |
| Was transport conserved? | Existing authoritative represented-volume ledger where available, including source/boundary terms; appearance tracer is not a liquid-mass ledger |
| What did execution cost? | Separate inference, IPC/pre/postprocess, qualification, fallback and full accepted-step scopes; startup/export/rendering stated separately |

Define every reduction precisely in the versioned comparison recipe. For weighted RMSE use `sqrt(sum(w * difference²) / sum(w))` on the declared valid support. Relative error must state a denominator and a physically meaningful zero-reference policy; never hide a zero norm with an undocumented numerical epsilon. Keep signed bias if scientifically useful.

Pressure is gauge-dependent. Equal-gauge pressure comparisons can be direct; a gauge-aligned diagnostic must record the removed offset and must not hide a candidate's violation of its required canonical gauge. Cross-physics EOS/absolute and projection pressures are incompatible without a separately justified analysis.

Do not match particles across runs using the same array index or local ID. Current IDs are run-scoped. Use declared aggregate statistics or an explicit spatial comparison/mapping whose error and support are visible. Do not invent a kinetic-energy/conservation metric from cell-averaged preview velocity without specifying the discrete integration and density/association policy.

PETSc distinguishes preconditioned/natural/true residual norms and explicitly notes these are equation residuals, not error norms. Rheon should retain its independently recomputed true full residual, and keep it separate from field error or physical accuracy. [PETSc residual norm semantics](https://petsc.org/release/manualpages/KSP/KSPSetNormType/).

### 9.3 Timing and memory honesty

Use the existing equal-input reference harness patterns: retained warmup, rotated sequential order, repeated samples, exact binary/build/configuration and hardware facts. Keep cold startup and steady inference separate. Report the full distribution/sample count and failures; do not report only successful model-selected steps.

Current CLI stepping timing includes CSV writes whereas GUI stepping excludes preview/rendering. They are not interchangeable columns. Retained array-byte counters are not process RSS or GPU peak allocation. Numerical precision, thresholds, physical horizon, device, CPU threads and concurrent load belong beside any timing comparison. [Existing comparison controls and timing scopes](https://github.com/MrScripty/Rheon/blob/43ed26b645dd42de906d0fac7d1ec7d5ab6e351a/docs/COMPARISON.md).

No speedup, physical-accuracy, real-time or cross-platform benchmark claim follows from this design.

## 10. Compact native UI proposal

Extend the existing desktop rather than introducing a second application:

1. **Simulation:** existing scenario controls plus a named saved specification, revision, qualified stage-plan summary and a pressure-provider selector. A provider detail drawer shows actual named inputs, output meaning, support and runtime status.
2. **Run sets:** a small list of variants with pinned provider/config revisions and explicit differences. “Run set” uses the existing worker; “Add recorded run” uses the existing cold archive reader. Capacity estimate covers all retained tracks, rather than showing only per-track limits.
3. **Compare:** two primary views and an optional difference view. Link camera, physical slice/probe location and time selection; show exact run/stage/time/interval labels. Shared field units and color bounds default to locked; a difference scale is symmetric about zero. Independent color ranges require an obvious label.
4. **Metrics:** a short result summary and time curves for residual, actual divergence, field error and step cost. Click a point to select the exact recorded publications. Show “unaligned,” “missing,” “diagnostic” and “fallback-selected” as meaningful states.
5. **Inspector:** reuse raw numerical field inspection and VTK/CSV export. Quantitative views read native numerical archives; max-pooled/quantized appearance previews remain display-only.

ParaView supplies an established precedent for linked comparative views and varying a parameter/time across views. Borrow the interaction pattern without embedding ParaView or relying on screenshots as evaluation. [ParaView comparative visualization](https://docs.paraview.org/en/latest/ReferenceManual/comparativeViews.html). Its time-series plotting supports the companion metric curves. [ParaView plotting over time](https://docs.paraview.org/en/v6.2.0/Tutorials/ClassroomTutorials/beginningPlotting.html).

Avoid a plugin browser/marketplace, free-form node editor, custom plugin JavaScript panels, distributed run service or complete simulation editor in this first slice. Host-rendered bounded configuration forms are sufficient.

## 11. Alternatives and licensing boundary

| Option | Assessment |
|---|---|
| Continue checkpoint-specific Rust/UI adapters | Lowest immediate work; repeated field/history/runtime edits remain. Useful baseline, not the long-term seam |
| Typed host stage contracts + model adapters | Recommended. Preserves numerical ownership while localizing compatible model changes |
| Arbitrary tensor/PDE graph | Flexible appearance, but type/shape validation cannot verify boundary conditions, splitting stability, conservation or operator consistency. Too broad first |
| Implement all of FMI/BMI | Potential future interoperability adapters; excessive as the core design when Rheon needs a few typed native stages |
| Adopt a multiphysics framework/preCICE | Appropriate only when genuine external solver coupling becomes a requirement; brings new runtime, mapping, iteration and licensing review |
| Turn the model into a learned preconditioner | Promising separate numerical experiment, not a drop-in candidate API. Variable/nonlinear preconditioners can require different Krylov methods |
| Embed ParaView | Large product/runtime change. Existing VTK export plus a compact native comparison view is a smaller first path |

PETSc's flexible GMRES explicitly supports nonlinear/variable preconditioning. That illustrates why a generic “pressure plugin” cannot silently replace the current PCG preconditioner with arbitrary learned behavior; retain a separate typed role and qualification if this is pursued. [PETSc FGMRES](https://petsc.org/release/manualpages/KSP/KSPFGMRES/).

Keep new Rheon contracts and implementation under the project's Apache-2.0 policy, subject to exact dependency review. This report recommends independently written interfaces and reuse of existing approved dependencies. It does not recommend copying GPL solver code or adopting a GPL dependency. OpenFOAM is GPL; preCICE core is LGPLv3, a different license with its own obligations. A subprocess boundary should not be represented as an automatic licensing exemption. [OpenFOAM license](https://openfoam.org/licence/), [preCICE licensing](https://precice.org/fundamentals-license.html).

BMI is MIT; FMI's specification text is CC BY-SA 4.0 while its code is BSD-2-Clause; ONNX code is Apache-2.0. Referencing their concepts is different from copying specification prose or incorporating implementation code. Audit the actual package, transitive dependencies, model weights, adapter source and training-data rights before distribution; do not infer a model's rights from its runtime's license. [BMI license](https://bmi.csdms.io/en/stable/license.html), [FMI licensing](https://fmi-standard.org/docs/3.0.2/), [ONNX license](https://github.com/onnx/onnx/blob/main/LICENSE).

## 12. Phased implementation and acceptance tests

No phase below has been implemented or run in this research.

### Phase 0: reconcile the new integration

- Obtain the inaccessible Rheon target, pin its exact commit, and inspect existing worker/runtime/recording changes before designing replacements.
- Trace every lab channel to its training/export source; record residual sign/units, previous-pressure semantics, coordinate normalization and full-pressure reconstruction.
- Resolve the declared viscous-water versus current main provider-composition gap, and make the adapter's scalar-spacing/no-origin limits explicit. Unsupported spacing/origin must reject before inference unless a new mapping is independently qualified.
- Produce one contract matrix showing what current main, the new branch and the model actually expose. Preserve existing evidence and tests.

Stop condition: a reviewed boundary/gap list and one exact pressure-stage fixture, not a rewrite proposal based on unavailable code.

### Phase 1: typed pressure context and adapter vertical slice

- Add explicit previous accepted pressure and provenance through an additive typed pressure-context version.
- Adapt the current model package to that context and preserve the existing pressure/full-step gates.
- Qualify one native reference and one external CPU model runtime; expose host-rendered configuration and support failures.

Tests: unequal per-axis counts with cubic cells and x/y/z channel-order sentinels; preserve anisotropic-spacing/nonzero-origin geometry in the native context but reject it for the current lab adapter until qualified; reject the unresolved viscosity/plan mismatch; cell/face length mismatch; wrong unit/stage/gauge; RHS-versus-residual distinction; actual shortened dt; missing/stale/other-run history; zero normalization scale; nonfinite/short output; unknown required versions; mutated source/weights; selected-versus-attempted reporting. Preserve manufactured-solution and independently assembled operator oracles; configuration/conformance tests do not replace them.

Stop condition: two independently packaged compatible model/adapter revisions can be selected without editing a numerical owner or UI owner, and both produce the expected physical candidate on an independently checked fixture. A model-only shape match is insufficient.

### Phase 2: accepted recording and saved specifications

- Connect the admitted provider/session producer to the current recorder, with complete provider/context provenance.
- Add stable typed `SimulationSpec` and a run-set manifest referencing current archives.
- Cold-open and inspect accepted native/model+fallback runs without loading model code.

Tests: constructor pressure absent; coherent accepted field/report/time generation; rejection/cancellation produces no accepted record; recording error reports unrecorded tail; uncertain commit requires fresh-reader reconciliation; unavailable plugin still permits generic cold inspection; exact selected ordinal survives another live append; missing/corrupt member never substitutes live data; edits create new revisions; archive frame does not offer resume.

Stop condition: close the producer process, reopen a saved set in a fresh process, verify the committed prefix and reproduce selected field values/provenance from storage alone.

### Phase 3: three-mode comparison

- Implement explicit teacher-stage fixtures and independent diagnostic track ownership, initially stop-on-reject. Qualify a separate Rheon-owned diagnostic advance contract before permitting failed-state continuation.
- Add alignment checks, native numerical metrics, linked views and fallback/warmup annotations.
- Add the required versioned diagnostic record schema/producer within the current recording system; preserve diagnostic evidence without bypassing accepted decoders.

Tests: equal ordinal/different time rejected; interval-versus-endpoint pressure rejected; same dimensions/different origin rejected; gauge-offset diagnostic labeled; zero reference norms; mismatched support/mask; unmatched horizons; fallback frequency includes failed attempts; pressure passes but stored-f32 divergence fails; post-pressure full-step failure preserves state/history; no teacher leakage into independent rollout; invalid diagnostic never appears as accepted production data; cross-run particle IDs are not matched by index.

Stop condition: the UI and exported summary answer three distinct questions correctly: local candidate quality on equal inputs, hybrid accepted behavior, and independent rollout behavior.

### Phase 4: broaden only with evidence

Add viscosity composition, additional native stages, ONNX/GPU runtimes, stateful providers, explicit remapping or coupled solvers one capability at a time. Each new contract gets support admission, cancellation/rollback, memory and numerical tests. Preserve core-only builds and old archive interpretation.

Useful fault tests across phases: hung/crashed worker, malformed protocol, log flood, stale result after cancellation, repeated Start, Close during inference, descendant cleanup, state rollback on final-gate rejection, storage capacity exhaustion, failed terminal-status write, unavailable runtime and deliberately changed dependency versions. Run them on each actually supported OS/runtime; a Linux compile is not macOS/Windows qualification.

## 13. Decisions to settle before implementation

1. Which exact new native integration commit is available for review, and which runtime/recording pieces can be retained?
2. Is the initial external-code trust model explicitly trusted-local, or does the product need malicious-code containment?
3. The reviewed checkpoint uses explicit prior-pressure context without recurrent network memory. Do future providers need hidden evolving state, and, if so, which commit/rollback capability will be required?
4. Which scientific envelope is admitted first: the existing sealed carrier only, or a separately qualified viscous plan as well?
5. Which saved comparisons must support only inspection, rerun, or true restart? These require different material and guarantees.
6. Which metric/alignment policies should be normative for the first pressure model, and what is the held-out validation scope?

The minimal success criterion is change locality with preserved authority: a model's compatible channels, normalization, weights and runtime packaging can evolve in the lab package; Rheon can load, qualify, record and compare it through stable contracts; Tuldok remains generic; and no configuration file can declare unimplemented physics to be supported.