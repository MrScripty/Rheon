# 37 GPU algorithms for a compatible grid fluid

*Research chapter for the Puma-credited Rheon book. Reviewed against the CPU source at `fd7c0f0` and primary literature on 21 October 2026. The designs below specify future work; no GPU solver, hardware result or general three-dimensional coupled liquid implementation is claimed.*

A GPU helps a grid fluid when enough independent samples can use the same arithmetic on resident data. It cannot remove the elliptic pressure constraint, fix a mismatched boundary operator, or turn a fixed iteration count into a physical accuracy guarantee. The useful design question is therefore: which dependencies belong to the mathematics, which come from a particular numerical method, and which were introduced by the implementation?

Chapters [3](03-staggered-grids.md)–[10](10-forces-and-viscosity.md) establish the discretization; [13](13-memory-and-parallelism.md) and [15](15-floating-point.md) establish storage and arithmetic cautions. This chapter develops their consequences for GPU algorithms without choosing CUDA or Vulkan. [Chapter 38](38-independent-cuda-and-vulkan.md) concerns backend mechanics; [Chapter 39](39-gpu-validation-and-roadmap.md) and the [implementation research plan](../implementation/gpu-research-plan.md) govern qualification. Source identifiers G1–G9 resolve to the [verified GPU bibliography](../sources/gpu-algorithm-bibliography.md).

## 37.1 The reference problem and the transfer boundary

Consider incompressible Newtonian fluid with positive density, a Cartesian marker-and-cell (MAC) grid, pressure at cell centers, and each velocity component on faces normal to its direction. A split step transports velocity, applies body forces and possibly viscosity, solves pressure, corrects velocity, and transports the selected scalar or interface representation. This order is a numerical choice: moving operations across one another changes splitting error.

The bounded native reference is more specific. [`PressureWorkspace`](../../../src/pressure.rs) owns six cell-length `f64` arrays: pressure, right-hand side, residual, search direction, matrix product and preconditioned residual. It offers diagonal-Jacobi PCG and symmetric-Gauss–Seidel PCG. The ordinary [`Simulation`](../../../src/simulation.rs) stores face velocity and tracer as `f32`, prepares a candidate, checks pressure and independently reconstructed stored-velocity divergence, and commits an accepted generation. [`PressureOperator`](../../../src/operator.rs) owns the integrated SI coefficient convention. Its baseline is a connected sealed rectangular domain; additional slab, column and fixed flat-column modes have explicit restricted geometry contracts. The material particle owner and its voxelized appearance preview are separate from this MAC pressure state.

The [CPU oracle inventory](../implementation/gpu-cpu-oracle-inventory.md) records the exact stage and ownership boundaries. A first GPU parity target should name that bounded problem, including boundary faces, zero pressure initialization, tolerance conjunction, cancellation and failed-step behavior. Merely producing similar smoke does not establish parity. A later obstacle-aware or free-surface solver is a new numerical qualification even when it reuses a pressure kernel. There is no inference from a height-column capability to arbitrary overhangs, droplets or moving cut-cell topology.

Historical sources require the same care. Harris's 2004 chapter treats a two-dimensional homogeneous rectangular fluid, while Crane, Llamas and Tariq's 2007 chapter uses collocated textures for exposition and explicitly notes advantages of a staggered MAC grid. They demonstrate how grid passes map to GPUs, rather than supplying Rheon's pressure discretization [G1, G2]. McAdams, Sifakis and Teran develop an important parallel multigrid pressure method, but their reported implementation targets shared-memory CPU platforms [G4]. Their algorithmic ideas can motivate GPU research without becoming GPU performance evidence.

## 37.2 Work, span, bandwidth and residence

Let a pass have total arithmetic work W, a longest dependent chain of length S, and P effective concurrent arithmetic lanes. Even an ideal scheduler cannot complete it in less than a constant times `max(W/P, S)`. A cellwise source addition has large W and a short chain. A lexicographic triangular solve has large W but a long chain. Describing both as one loop over cells hides the difference.

For a sequence of GPU passes, a useful accounting model is

\[
T_{\rm step}\approx T_{\rm upload}+T_{\rm download}
+\sum_{k=1}^{m}\left(L_k+
\max\{B_k/\mathcal B_k,\;F_k/\mathcal P_k\}\right)
+T_{\rm dependencies}+T_{\rm host}.
\]

Here `L` is submission/dispatch cost, B is transferred device-memory bytes, F is floating arithmetic, and the calligraphic quantities are achieved bandwidth and arithmetic rates for that workload. The maximum assumes ideal overlap within a pass and is an optimistic model, not a measured identity. Cache reuse, occupancy, addressing, branch behavior and contention affect the achieved quantities. Keeping the fluid resident removes repeated full-field transfer terms; it does not remove reduction dependencies or the host application's own memory needs.

A seven-point constant-coefficient stencil performs only a modest amount of arithmetic per output. Its obvious uncached implementation reads seven scalars and writes one. At scalar width s that is `8sN` requested bytes for N cells. Neighboring rows reuse values, so actual external-memory traffic can be lower; coefficient loads, masks and misses can make it higher than a simple idealized model. A GPU with a large nominal arithmetic rate may spend this pass waiting on data. Conversely, adding expensive geometry or constitutive evaluation can move the bottleneck to arithmetic or control flow.

The parallel break-even point depends on this whole expression. Small grids, small liquid islands, many tiny coarse levels, frequent host waits, short runs and resource contention can erase the benefit of high fine-grid throughput. Pressure changes that picture further: total cost is setup plus iteration count times iteration cost, measured at an accepted physical residual. A cheaper iteration that needs many more iterations may lose. Repeatedly reducing a grid diagnostic on the GPU and downloading one scalar is different from downloading all fields; the remaining scalar wait may still serialize the next decision.

An Amdahl-style check prevents overclaiming. If fraction f of a measured end-to-end workload is accelerated by factor a while other work is unchanged, the ideal overall factor is `1 / ((1-f)+f/a)`, before transfer and integration overhead. The formula supplies a limit under stated assumptions, never a forecast for Rheon. A busy GPU, a fast pressure kernel and a faster accepted frame are three different observations.

## 37.3 Preserve the physical operator before optimizing it

Use the incidence convention from Chapter 3: an oriented internal face column of B has −1 at its tail and +1 at its head. Let S contain open areas, L contain normal pressure-sample distances, R contain face density, V contain liquid cell volumes, and Q contain known outward prescribed-boundary volume flux. With normal face speed u,

\[
D(u)=V^{-1}(Q-BSu),\qquad
G=L^{-1}B^T,\qquad
u^+=u^*-\Delta t R^{-1}Gp.
\]

The symmetric integrated pressure system is

\[
Kp=b,\qquad K=BSR^{-1}L^{-1}B^T,
\qquad b=(BSu^*-Q)/\Delta t.
\]

This assumes a two-point normal gradient on the stated orthogonal pressure-sample geometry. It does not justify arbitrary skew cut-cell centroid connections. A face transmissibility `t_e=S_e/(rho_e ell_e)` is nonnegative, shared by its two neighboring rows, and has units m⁴/kg. Cellwise gathering evaluates

\[
(Kp)_i=\sum_{j\leftrightarrow i}t_{ij}(p_i-p_j)
+\sum_{e\in\Gamma_D(i)}t_e p_i.
\]

Known Dirichlet pressure contributes `t_e p_D` to b; prescribed solid speed contributes known flux. Classifying a solid face as zero-pressure air creates a different, leaking problem.

One GPU invocation per pressure cell can gather neighboring values and write exactly one output. One invocation per correctable face can gather its two pressures and write exactly one velocity. These ownership rules avoid atomic accumulation. Face scattering can instead preserve paired contributions explicitly, but then two faces may write one cell and require a declared accumulation method. A matrix-free gather still needs one authoritative face coefficient; deriving separate copies with different clipping or geometry versions can break symmetry.

For a closed component, constants lie in the kernel. Gauge elimination must remove a row and column consistently, or a specified mean-zero method must maintain the subspace through every operation. A pressure opening anchors its component. Closed components require compatible integrated flux before solving. A parallel connected-component procedure may converge over several passes; an unfinished label propagation is not a valid component map. Deterministic representative selection helps replay, but the correctness requirement is complete membership and correct anchoring.

Pressure stopping must retain the physical relation

\[
r=b-Kp,\qquad D(u^+)=-\Delta t V^{-1}r.
\]

If the solve uses eliminated gauges, reconstruct r on all active cells, including the removed ones. The prediction is exact for the specified algebra before rounding. It does not include the final `f32` velocity storage error. Recompute divergence from the actual stored corrected faces and the same boundary snapshot. The tolerances

\[
\|r\|_2\le\max(\tau_{\rm abs},\tau_{\rm rel}\|b\|_2),
\quad \|\Delta t V^{-1}r\|_\infty\le\tau_{\rm div},
\quad \|D(u_{\rm stored}^+)\|_\infty\le\tau_{\rm stored}
\]

are separate requirements with different units and purposes. Tiny cells amplify integrated residual; a small global norm can hide one bad local cell.

## 37.4 A dependency graph for one candidate step

The following graph names dependencies, not commands in a particular API. An arrow means producer completion and visibility are required before the consumer reads that version.

~~~mermaid
flowchart TD
    A[Accepted fields and geometry snapshot] --> B[Time-step diagnostics]
    B --> C[Velocity backtrace and gather]
    C --> D[Forces and optional viscous solve]
    D --> E[Boundary flux and pressure RHS]
    E --> F[Pressure iterations and reductions]
    F --> G[Face pressure correction]
    G --> H[Stored-velocity divergence and Courant checks]
    H --> I[Tracer or interface transport]
    I --> J[Finite, bound, resource and cancellation gates]
    J --> K[Commit accepted generation]
~~~

Branches can overlap only if they read immutable versions and write disjoint resources. For example, a scalar source independent of pressure may be prepared separately, but transport driven by the projected velocity waits for that velocity. An interface update that changes pressure coefficients cannot be moved before the current pressure solve without defining which interface geometry belongs to the interval.

Within a transport pass, output samples are independent if all sample the same immutable old velocity and source field. Updating x velocity in place and then tracing y velocity through that changed x component violates this condition. Within pressure, `Kd` is independent across rows, but the scalar `d^T Kd` must complete before alpha, and alpha must be visible before updating p and r. No API can eliminate that mathematical dependency while retaining the same recurrence.

A workgroup-local barrier only orders participating local invocations. It does not make arbitrary other groups complete. A multi-pass phase boundary, with the required backend memory visibility, is the conservative algorithmic primitive. Spinning until another unscheduled group produces a value can deadlock. More advanced cooperative or persistent scheduling requires its own supported execution assumptions; it is not a portable baseline. Harris's reduction tutorial provides the historical tree-reduction construction and kernel decomposition motivation [G5]. Its old hardware-specific synchronization statements are not a modern API specification.

The accepted state should remain immutable until all gates succeed. Failed pressure, invalid geometry, cancellation or insufficient sparse capacity discards the candidate. Device completion and numerical acceptance are distinct: completion makes a result available; the acceptance logic decides whether it becomes simulation state. Chapter 39 defines the evidence needed to test that lifecycle.

## 37.5 Stencils, tiling and reductions

### Matrix-free pressure application

A minimal stencil contract is:

~~~text
apply_K(snapshot, x, y):
    require x and y do not alias
    parallel for each active pressure cell i:
        total = 0
        for each face in fixed axis/side order:
            read its one authoritative coefficient and boundary class
            fluid neighbor: total += t * (x[i] - x[neighbor])
            pressure boundary: total += t * x[i]
            prescribed-flux wall: add no pressure coupling
        y[i] = total
    publish all y writes before a consumer reads them
~~~

The real implementation must additionally apply its reduced-system map and gauge convention. Inactive cells have declared outputs, rather than uninitialized values that enter a later reduction. A separate regular-interior kernel can avoid repeated boundary branches; a boundary kernel owns the exceptional rows. Their output regions must be disjoint and together cover the intended unknowns.

Use x-contiguous invocations with an x-fastest layout as a candidate for neighboring memory accesses. A tile can load a shared pressure patch plus a one-cell halo and reuse it for several outputs. For a `b_x × b_y × b_z` tile, a full bounding-box halo contains `(b_x+2)(b_y+2)(b_z+2)` samples, though a seven-point stencil does not need every corner. Reuse competes with shared storage, registers, occupancy, redundant halos and synchronization. Direct cached global loads may win for some sizes. A tile shape is a tunable parameter backed by measurements, not a theorem about optimal layout.

Temporal blocking of multiple Jacobi sweeps has a larger dependency cone. Performing k exact local sweeps needs a halo reaching k cells in the appropriate graph distance, or shrinking valid output regions. Reusing a one-cell halo for many sweeps freezes neighboring blocks and changes the global iteration to a block method. That can be useful if specified, but its residual history cannot be reported as ordinary global Jacobi.

### Dot products, maxima and invalid flags

Compute partial sums per workgroup, then reduce those partials through fixed additional stages. For dot products, the product precision and accumulation precision are separate choices. For a maximum diagnostic, reduce the maximum absolute finite value and separately OR an invalid-value flag; NaN behavior of a plain maximum must not decide validity accidentally. Compatibility sums also need the magnitude sum used to interpret cancellation.

A fixed reduction tree can provide repeatability for pinned executable, precision, group size and device assumptions. It does not reproduce the CPU's ordered compensated sum. Atomic floating additions typically admit different orders; using them for a pressure norm changes numerical replay behavior even when every update is race-free. Stochastic ordering error is especially troublesome when a stopping decision changes iteration count.

Fuse compatible vector updates and local reduction contributions to avoid rereading a vector. For example, updating r and accumulating local `r*r` contributions uses values already in registers. The global result still needs a reduction phase before a dependent decision. Fusing `Kd` with contributions to `d^T Kd` is possible when every row has the necessary d values. Fusing the scalar-dependent update into that same arbitrary multi-group pass is not justified without an additional global completion mechanism.

## 37.6 Pressure methods on a GPU

### Weighted Jacobi: a baseline with a spectral limitation

For diagonal D of K,

\[
p^{k+1}=p^k+\omega D^{-1}(b-Kp^k).
\]

Use distinct input and output pressure arrays per sweep. On the gauge-fixed subspace, convergence requires the eigenvalues of the error propagator `I-omega D^-1 K` to lie inside the unit disk. With K SPD and D positive, a sufficient and necessary interval for positive omega is `0<omega<2/lambda_max(D^-1/2 K D^-1/2)`. Choosing omega from a familiar uniform Laplacian without considering changed coefficients is not a qualification for irregular geometry.

For a one-dimensional uniform interior, the normalized Laplacian symbol is `1-cos(theta)`. Small theta leaves a Jacobi error multiplier near one. Increasing resolution introduces slowly varying errors that many local sweeps remove poorly. This explains why Jacobi is useful for a transparent baseline and a smoother, yet a fixed small number of sweeps is an unreliable pressure policy. GPU suitability of one sweep does not establish suitability of the full solve.

Diagonal preconditioning in PCG is different from repeatedly solving pressure by Jacobi. It computes `z_i=r_i/D_i` once per preconditioner application. The reference name `JacobiPcgV1` denotes that diagonal PCG preconditioner, not a standalone fixed-sweep Jacobi pressure solver.

### PCG: parallel vectors around ordered scalars

On a fixed SPD reduced K and fixed linear SPD preconditioner M, PCG has the following dependency structure. Scalar operations below occur only after their reductions complete; all breakdown checks precede division.

~~~text
p = specified initial guess                         # CPU parity target: zero
r = b - K*p                                         # true residual
if full residual and predicted-divergence gates pass: return pressure success
z = inverse_M(r); gamma = reduce_dot(r, z)
d = z
for iteration in finite budget:
    restart_direction = false
    q = K*d
    delta = reduce_dot(d, q)
    require finite positive gamma and delta
    alpha = gamma / delta
    parallel: p += alpha*d; r -= alpha*q
    recursive_norm = reduce_l2(r)
    candidate = recursive_norm <= max(tau_abs, tau_rel*norm_b)
    if candidate or scheduled_true_check or final_budget_iteration:
        rt = b - K*p                                # dedicated scratch/lifetime
        if full ungauged residual and predicted-divergence gates pass:
            return pressure success
        if declared residual-replacement rule triggers:
            r = rt; restart_direction = true
    if final_budget_iteration: return IterationLimit without commit
    z = inverse_M(r)
    gamma_next = reduce_dot(r, z)
    require finite positive gamma_next
    if restart_direction: d = z
    else: parallel d = z + (gamma_next/gamma)*d
    gamma = gamma_next
return IterationLimit without commit
~~~

This is an algorithm sketch, not a drop-in replacement for `solve_loaded`. Pressure success returns pressure and its report; the separate step then corrects velocity, stores it as `f32`, and checks actual stored-face divergence before a possible commit. The CPU does not perform a stored-velocity correction at each PCG candidate. Acceptance details and scratch reuse must match the chosen implementation contract. In particular, a true-residual computation must not overwrite q while a later recurrence still needs q, or silently replace r without updating the direction policy. The native implementation already distinguishes a true linear-residual replacement from continuing because the physical divergence gate needs more work.

Ordinary PCG needs at least two ordered inner-product reductions per nonterminal iteration: curvature and the new preconditioned residual product. Residual norms and predicted-divergence candidates add work; compatible sums may share reduction stages. A diagonal preconditioner makes more fusion possible than a multilevel solve. The critical path contains global scalar dependencies even when every vector operation is highly parallel. Bolz and colleagues establish an early GPU formulation of sparse CG and regular-grid multigrid [G3]; modern source in AmgX exposes the same broad apply/dot/update/precondition structure [G8]. Neither source qualifies Rheon's termination policy.

Pipelined and communication-avoiding Krylov variants alter recurrences and rounding behavior to reduce synchronization. They should follow a working ordinary PCG baseline with true-residual checks. Larger numerical drift, extra vectors and more complex restart logic can outweigh saved waits. Selecting such a method means specifying and testing that variant, rather than silently reordering standard CG equations.

### Which preconditioner exposes enough concurrency?

| Candidate | Parallel work and synchronization | Main mathematical condition | Limitation to investigate |
|---|---|---|---|
| Positive diagonal | Independent cells; reduction of `r*z` | Positive D on reduced unknowns | Weak control of smooth error and large coefficient contrasts |
| Disjoint block Jacobi | Independent blocks, small local solves | Each block solve fixed and SPD | Ignores interblock low-frequency coupling; block metadata and factors |
| Symmetric colored GS | Independent cells within each valid color; ordered color sweeps | Paired forward/reverse construction must define SPD M | More barriers; coloring depends on actual operator graph |
| Lexicographic SGS wavefront | Ready anti-diagonals/tiles execute together | Respect every predecessor and reverse dependency | Narrow wavefronts and long span; direct CPU layout may expose little GPU work |
| Polynomial approximation | Stencil/vector stages without triangular solves | Fixed polynomial positive on the relevant spectrum for SPD preconditioning | Reliable spectral bounds; degree versus reduction and storage cost |
| Geometric multigrid V cycle | Fine/coarse stencils, transfers, small coarse solve | Fixed linear symmetric positive action for ordinary PCG | Boundary/topology preservation; underfilled coarse levels |
| Algebraic multigrid | Sparse matrix and hierarchy operations | Configuration must meet outer-method assumptions | Setup, graph/index traffic, hierarchy memory and rebuild costs |

For a nearest-neighbor seven-point Cartesian graph, red/black parity separates neighboring cells. Wider stencils, merged cells, skew corrections and coupled viscosity can invalidate that two-color independence. A red sweep followed by a black sweep alone is generally a nonsymmetric action; an appropriate symmetric sequence or another derivation is required for ordinary PCG.

MIC(0) and incomplete Cholesky are important CPU references, described in the Bridson/Müller-Fischer notes [G6]. Their triangular dependencies do not disappear when factored entries are copied to GPU memory. Level scheduling, reordering or approximate triangular solves expose different amounts of parallel work and may change the preconditioner. A lower iteration count on the CPU is insufficient reason to select a GPU factorization path.

For block Jacobi, a clean baseline partitions reduced unknowns into disjoint sets and applies fixed exact SPD block inverses. Overlapping additive methods require explicit restriction, extension and weighting; simply allowing neighboring blocks to race on an output vector is neither a defined preconditioner nor a conservation method. Polynomial and multilevel candidates likewise need mathematical configuration checks before timing.

## 37.7 Multigrid with geometry and GPU resource limits

Multigrid converts smooth fine-grid error into more oscillatory coarse-grid error. For prolongation P and integrated symmetric K, the Galerkin coarse operator `K_H=P^T K_h P` preserves SPD when P has full column rank on the solved subspace. Closed-domain constants require `P 1_H=1_h` together with consistent gauges or nullspace projection. These algebraic conditions should be checked independently of the GPU implementation.

A V cycle computes pre-smoothing, fine residual, restriction, coarse correction from a zero correction guess, prolongation and post-smoothing. Transfers are stencil-like gathers when the chosen interpolation support permits them; transpose restriction may instead require scatter or a separately derived equivalent gather. Use the actual transpose, including boundary modifications, when the construction requires it. A convenient averaging stencil is not automatically `P^T` for a truncated interface neighborhood.

As a PCG preconditioner, the cycle should be a fixed linear SPD map: fixed hierarchy, fixed smoothing counts and weights, matched adjoint pre/post smoothing, and a suitable SPD coarse solve. A coarse solve stopped by a data-dependent tolerance can change that map. Adapting the hierarchy during the outer recurrence requires restarting or selecting an outer method with an appropriate variable-preconditioner contract. These are algorithm changes, not backend optimization switches.

A regular three-dimensional hierarchy contains about `N(1+1/8+1/64+...)=8N/7` cells. That ratio counts cells only. Coarse masks, graph mappings, coefficients, vectors, transfer operators and an irregular Galerkin stencil can dominate the extra bytes. A Galerkin product need not retain the original seven-point sparsity. Re-discretizing a seven-point coarse operator can reduce storage but demands an independent consistency and convergence argument.

Two fine liquid components separated by a sub-coarse-cell solid barrier must not become one coarse component accidentally. Coarse cells may need multiple degrees of freedom or another component-aware hierarchy policy. A thin channel disappearing at coarse resolution is the dual failure: coarse correction can lose the connection needed to represent its smooth pressure mode. Cut-cell multigrid research addresses geometry-specific hierarchy construction [G7]; its published claims must not be generalized to every embedded boundary or imported as current Rheon capability.

Fine levels expose many independent cells. Coarse levels eventually expose too few to use a large GPU well, and each additional pass has an overhead. Candidate responses include reducing the coarsest resolution, grouping coarse work into fewer dispatches, using a fixed small device solve, or transferring a very small coarse system to the CPU. The last choice introduces a round trip inside every cycle unless carefully amortized. Choose using complete solve cost and accepted residual, not the finest stencil bandwidth alone. McAdams et al. motivate a lightweight matrix-free geometric cycle used inside CG on irregular voxelized domains [G4]; their CPU timings are not a GPU break-even estimate.

## 37.8 Transport, forces and viscosity as GPU work

### Backtrace and gather

For frozen velocity, midpoint tracing uses `k1=u(x)`, `k2=u(x-dt*k1/2)` and `x_d=x-dt*k2`. Each destination sample independently gathers its old scalar or component at `x_d`. It also gathers all three velocity components at their respective MAC offsets to construct the tracing velocity. Hardware interpolation is an implementation option; it must be shown to match coordinate normalization, center offsets, boundary extension and precision rules. A cell-centered `float4` texture from a visual-effects example is not a storage substitute for three staggered component shapes [G2].

A first-order transport pass has a relatively simple dependency graph and irregular reads around departure points. Long traces reduce spatial locality. They also make thin-wall intersection and boundary policy more important. Semi-Lagrangian stability does not guarantee trajectory accuracy, conserve liquid volume, or allow unchecked time steps.

Corrected transport needs multiple versions. For the Chapter 7 formula, compute `q_f=S_dt(q)`, then `q_b=S_-dt(q_f)`, then `q_c=q_f+(q-q_b)/2`, followed by the declared limiter/fallback. The old q must survive until correction; `q_f` survives the reverse pass. A reverse sample of `q_f` cannot be computed from only the current destination's forward value because it gathers neighboring forward values. That requires a phase boundary. Limiter bounds and validity masks can be prepared with the forward pass if their complete lifetimes are retained. Count fallback samples and pre-limiter overshoots in diagnostics.

Conservative scalar transport should compute one face flux and then gather its signed contribution into adjacent cells. This yields shared-flux cancellation without atomic cell updates. Computing nominally identical face fluxes twice in neighboring cell invocations risks different limiter or boundary decisions. Volume-of-fluid transport additionally needs geometric interface reconstruction and bounded multidimensional outflow accounting; a fast trilinear texture advection pass does not supply those properties.

### Forces and local derivatives

Gravity and prescribed accelerations are independent face updates once their spatial sampling and units are defined. Buoyancy gathers cell quantities to the appropriate face and may require a source snapshot. Vorticity confinement typically computes curl, then a gradient of curl magnitude, then the force; those are dependent stencil passes unless a correct expanded dependency cone is used. Guard zero normalization and report injected work. Fusion should preserve the same spatial stencil and stage ordering.

Maximal speed or acceleration for a time-step policy is a reduction problem. A stale diagnostic can admit an interval outside the selected Courant or force limit. Reduction of finite maxima and an invalid flag must complete before that diagnostic controls the step. A device-resident time-step decision can reduce host waits, but accepted interval metadata must record the actual chosen value.

### Viscosity is sometimes a coupled vector solve

For constant coefficient scalar diffusion, explicit rectangular-grid stability requires

\[
\nu\Delta t(h_x^{-2}+h_y^{-2}+h_z^{-2})\le\tfrac12.
\]

This follows by requiring the center weight of the seven-point update to remain nonnegative. Each sample is parallel, but finer spacing can increase the number of required substeps severely. Backward Euler moves that restriction into a solve; it does not make large-step decay accurate.

For variable viscosity and free-surface stress, use the strain formulation of Chapter 10:

\[
(M_u+\Delta t E^T M_\mu E)u^*
=M_u u^n+\Delta t f.
\]

E maps staggered velocities to strain samples; `M_mu` contains positive viscosity/quadrature weights with consistent tensor factors. The action can be computed as strain gather followed by a compatible transpose action, or by a derived coupled gather stencil. Separate x/y/z diffusion solves generally omit coupling. A preconditioner and smoother suitable for scalar pressure may fail on this vector operator. Aanjaneya and colleagues specifically investigate box smoothers for the coupled viscous problem [G9]. The primary author abstract was verified; its reported performance is not evidence for Rheon or for an arbitrary GPU port.

Nonlinear viscosity adds an outer constitutive iteration: compute strain, evaluate coefficients, solve the frozen system, then check the original nonlinear residual. Updating coefficients independently inside a Krylov iteration changes its operator. Dynamic two-way rigid coupling, capillary forces and changing fluid volume add further physical dependencies; parallel execution should follow their derived model rather than using a pressure-only pass as a general substitute.

## 37.9 Dense storage, sparse tiles and geometry changes

A dense array offers simple addressing, regular traversal and explicit capacity. It pays for every represented cell. A sparse tile map pays for occupied tiles, including partly empty tile interiors, halos and metadata. Let tiles have b³ interior cells and one full-box halo layer. The sample multiplier is `((b+2)/b)^3`: about 1.95 for b=8 and 1.42 for b=16, before any metadata. These are geometric storage ratios, not measured memory or speed. A seven-point optimized halo can store less, while multistep transport can require more.

An advection departure point needs all interpolation corners, not just the destination tile. A pressure stencil needs adjacent active/air/solid classification. Interface normals and curvature expand the neighborhood again. Sparse activation therefore depends on the union of these supports and predicted motion, rather than the set of currently nonzero appearance samples. Absence from a sparse table must resolve to a declared physical category or an invalid request; returning zero by convenience can create artificial pressure openings.

One face shared by two tiles still has one physical value. Duplicated halo copies need one source owner and a completed refresh before use. Shared coefficients and component IDs require the same coherence. Deterministic tile ordering can help reproducibility and stable reduction trees, at a sorting/compaction cost that belongs in performance and memory accounting.

Before a candidate geometry change becomes visible, construct or reserve the next active set, coefficient snapshot, component map and solver workspace. If capacity is insufficient, reject coherently. Reusing old pressure after topology changes needs remapping compatible with the new gauges; the bounded CPU parity target's zero guess avoids that issue initially. A sparse hierarchy may need rebuilding even when only a few fine tiles change. Amortizing setup is legitimate only over intervals whose hierarchy remains valid.

The pinned modern NVIDIA Flow source demonstrates sparse-table addressing, pressure textures and separate multilevel pressure passes [G8b]. Its inspected divergence shader gathers collocated velocity components with centered differences. This is useful implementation evidence for resource organization, but it does not implement the MAC incidence/gradient pair specified here. Importing its pressure expression alone would mix discretizations.

## 37.10 A resource ledger before a kernel plan

Let `N=n_x n_y n_z` and

\[
F=(n_x+1)n_yn_z+n_x(n_y+1)n_z+n_xn_y(n_z+1).
\]

Use `s_u`, `s_p` and `s_q` bytes for velocity, pressure-work and scalar samples. The following ledger describes candidate allocations, with no padding, allocator, descriptor, driver or rendering overhead. Rows marked optional must be attached to an actual selected method. Scratch sharing requires disjoint lifetimes and the necessary completion dependency.

| Resource | Payload | Required lifetime or reason |
|---|---:|---|
| Accepted and candidate MAC velocity | `2 s_u F` | Accepted state survives all candidate failures |
| Accepted and candidate scalar | `2 s_q N` | Candidate transport and accepted rollback |
| Six pressure arrays, matching CPU ownership | `6 s_p N` | p, b, r, d, Kd, z; true-check reuse must be proven safe |
| Diagonal, if explicitly cached | `s_p N` | Optional; constant box diagonals may be generated |
| Three-direction face transmissibilities | `s_p F` | Optional variable-geometry representation; unused boundary slots still count |
| Cell volumes | `s_p N` | Optional nonuniform liquid volumes |
| Cell class/component/reduced map | Actual integer widths and counts | Geometry/gauge representation; not implied by six pressure vectors |
| Reduction scratch | `s_a sum_{l=1}^L ceil(N/g^l)`, `L=ceil(log_g N)`, `g>1` | Approximate partial counts; include every simultaneous diagnostic and scalar status |
| Corrected-advection intermediate | At least one extra transported field | Forward field survives reverse gather; reverse/correction output may share only with justified lifetimes |
| Multigrid levels | Sum of actual per-level resources | `8N/7` is only a regular cell-count estimate |
| Sparse table, tile storage and halos | Actual capacities, not active payload only | Include reserved growth and candidate topology |
| Staging/readback/history/rendering | Application-specific | Coexist with simulation and other host GPU workloads |

For one cubic field layout matching the first three rows, with `s_u=s_q=4` and `s_p=8`, the payload is `8F+56N` bytes. Calculated examples are:

| Cubic cells | N | F | Base payload | With one stored `f64` coefficient per face and `f64` volume per cell |
|---|---:|---:|---:|---:|
| 64³ | 262,144 | 798,720 | 20.094 MiB | 28.188 MiB |
| 128³ | 2,097,152 | 6,340,608 | 160.375 MiB | 224.750 MiB |
| 256³ | 16,777,216 | 50,528,256 | 1,281.500 MiB | 1,795.000 MiB |

These totals deliberately differ from Chapter 13's eight-single-precision-cell-array illustration: this ledger retains the native six-double-precision pressure arrays in addition to two single-precision scalar versions. They are alternative declared layouts, not competing process-memory measurements. Reducing all pressure-work arrays to `f32` saves `24N` bytes but changes the arithmetic experiment. It cannot be called equivalent until pressure and stored-divergence gates pass on the qualified workload range.

Resource admission must use checked products, capacity and peak overlap. A buffer fitting its individual limit does not show the full candidate step fits available device memory. A CPU fallback also needs its memory and transfer staging reserved; a fallback that tries to allocate after device failure may fail a second time. Scientific evidence and preview history have different lifetimes and should be budgeted separately.

## 37.11 Precision, reproducibility and diagnostics

Mixed precision is a family of designs. Separate field storage, coefficient evaluation, stencil arithmetic, local sum, global sum and scalar recurrence precision. Promoting a reduction to `f64` does not recover digits already lost in `f32` products or `Kd`. Conversely, pressure in `f64` cannot avoid rounding when a small pressure correction is finally added to a large `f32` stored velocity.

Nondimensionalization or equation scaling can improve numerical range, provided residual-to-physical-divergence conversion retains its exact scale. Row scaling by varying cell volume usually destroys Euclidean symmetry; ordinary CG needs the integrated symmetric operator or an explicitly justified transformed inner product. A uniform scalar rescaling changes norm thresholds even when it leaves the mathematical solution unchanged.

A first arithmetic experiment should retain the CPU's `f64` pressure work where the device supports the required operations, then isolate changes one at a time. That is a comparison design, not a requirement that every consumer device deliver useful double-precision throughput. Other candidate policies include `f32` vectors with wider reductions and true residual evaluation, or an outer higher-accuracy correction method. Each needs its own breakdown and convergence evidence. Half-width pressure storage is especially risky for high coefficient contrast and near-rest cancellation; bandwidth savings are not accuracy evidence.

Report the gap between recursive residual and recomputed `b-Kp`. A pressure norm plateau may come from precision, poor conditioning, invalid coefficients, wrong topology, a nonsymmetric preconditioner or a stale resource. Increasing the iteration budget is informative only after those causes are separated. Curvature `d^T Kd<=0` or preconditioned product `r^Tz<=0` in an expected SPD solve is a breakdown; taking an absolute value hides a failed assumption.

A repeatability statement should name its scope: same pinned executable/device configuration, tolerance-based cross-backend agreement, or a stronger bitwise contract actually tested. Group size, reduction tree, fused multiply-add policy, compiler transformations, subnormal handling and coefficient generation can affect results. Race freedom and determinism are separate properties; physical correctness is separate again. Bitwise repetition of a wrong boundary operator remains wrong.

The minimal step diagnostic record should retain geometry/operator version, chosen time interval, precision policy, solver/preconditioner identity, iteration count, true full residual norms, predicted physical divergence, actual stored-face divergence, Courant value, invalid flags, termination reason and accepted generation. For sparse or multigrid runs, add active/capacity counts, hierarchy identity and setup/rebuild costs. Timings should separate pressure setup, pressure solve, transport, reductions, transfers and full accepted step.

## 37.12 Failure mechanisms and discriminating evidence

| Symptom | Mechanism to distinguish | Evidence that separates it |
|---|---|---|
| Pressure norm small, one cell leaks | Tiny volume or omitted gauge-cell residual | Full volume-scaled residual and stored divergence per active cell |
| GPU pressure differs but both velocities agree | Different compatible gauge | Compare pressure differences/gradients after gauge mapping |
| Smooth pressure error remains after many cheap sweeps | Weak low-frequency damping | Manufactured smooth mode and residual history versus refinement |
| Good single-pass timing, slow frame | Reductions, setup, transfer, other application work | Complete accepted-step timeline and end-to-end distributions |
| Results change with group size | Reduction order or latent race | Fixed-tree replay plus independent ownership and bounds checks |
| Grid boundary smoke smears or leaks | Sampler extension/offset/path collision | Affine offset fixture, thin-wall trace and boundary flux ledger |
| Sparse result changes with activation radius | Missing dependency support | Dense/sparse comparison with tile-edge and departing-trace fixtures |
| Multigrid fails on separated liquid pockets | Coarse topology joins components | Per-level component/nullspace maps and sealed-barrier fixture |
| SGS becomes slow despite fewer iterations | Long wavefront span or excessive joins | Preconditioner complete cost and ready-wave widths |
| Rest tank develops motion | Unbalanced force/gradient or stored precision floor | Hydrostatic cancellation, force work, stored correction and surface drift |
| Viscous material looks wrong with low divergence | Wrong stress model or inadequate temporal accuracy | Shear/channel decay and traction tests in addition to continuity |
| Cancellation corrupts next step | Candidate wrote accepted resources | Generation checks and injected failure at each phase boundary |

Each row distinguishes a hypothesis from a diagnosis. Chapter 39 specifies experiments and evidence retention; no result in this chapter is an executed GPU fixture. Documentation/source inspection, the resource arithmetic and the stated algebraic derivations do not establish wall-clock speed, cross-device replay, a Rust-to-shader refinement or continuum validation.

The practical sequence is to preserve the fixed-box MAC operator and acceptance lifecycle, establish independent stencil/reduction evidence, then compare pressure preconditioners at equal accepted quality. Geometry, sparse activation, constitutive complexity and coupled motion each expand the problem and should enter with their own contracts. CUDA and Vulkan can then implement a specified numerical experiment whose successes and limits are interpretable.
