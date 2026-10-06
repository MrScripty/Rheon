# Deferred affine rounding with reconciled ephemeral increments

This is a minimal experimental specification, not an implementation or execution authorization. It appends to the [stored-endpoint contract](forcing-increment-state-contract.md) and [E1 supplement](forcing-increment-state-contract-supplement.md). Base: `b46d26c8fe06f9c331c5cee4e06af6e34fa9190c`, tree `96045fb873ad9f1e00134a46e6e3d6efcda69567`. The [fixed-candidate diagnosis](../../../evidence/forcing-e1-fixed-candidates-v1/RESULTS.md) is still under independent review. Its disposition is unavailable here. Neither a new candidate nor a trajectory may execute until the coordinator records that disposition and resolves any resulting contract changes.

## Three meanings of “same candidate”

Let B(x) mean an existing binary64 input interpreted exactly as a real number, H mean the existing checked 106-bit scalar operation graph, and RN64 mean its checked nearest-even binary64 conversion. H operations round individually; H is not exact rational arithmetic. Accepted q0, eta0, z0, U0, masses, pressure, clock and stamp remain those borrowed from the one existing owner.

1. **Identical stored fields:** the same q/z/U/m/R/D/pressure/forces/transfers and path samples. Exact sums/products or an increment re-expression on these binary inputs describe the same mathematical residual. Finite operation order can change the measured residual, so even this is not a promise of bit-identical controller behavior. The baseline native norm and an exact rational squared norm must be reported separately.
2. **Identical candidate parameters:** the same accepted inputs and binary acceleration/pressure unknown vector, but a different path/rounding graph. Deferring eta0+s*alpha rounding can change known coefficients, dependent chart coefficients, embedded velocity, forces, mesh velocity, sign partitions and transfers. These are changed stored candidate fields. They must be fully rebuilt and reported as such; “same unknowns” is not “same fields.” At a fixed s the unchanged native q graph gives the same geometry, but new partitions can visit different s values.
3. **Unrounded latent endpoint:** qualifying a chart increment before final endpoint rounding defines a different residual from the authoritative stored endpoint residual, unless the discarded part is explicitly reconciled. An ephemeral lifetime does not make this difference an exact re-expression. Persistent low state, initial projection, a latent geometry path or a new rounding impulse would require an owner-level semantic decision; none is selected here.

The proposed first prototype E2 selects (2) for construction and reconciles to stored fields before qualification. Its qualified increment remains the existing native stored-field graph in (1). It does not select (3). The diagnostic's below-target continuous-known variants are counterfactuals and do not predict that E2 will pass after reconciliation.

## One fixed construction and arithmetic graph

Use the existing 106-bit significand, checked signed exponent, nearest-even operations, 15-column factorization/back-substitution, selected rows and last-row pivot tie rule, with zero iterative refinement. Preserve every checked division and nonzero-subnormal/range failure. No clipping, discarded nonzero underflow, fallback precision or alternate norm is permitted.

At every existing point time s, retain the current native quadratic q graph, cap, material and geometry assembly. Compute all six affine known values only in the existing scalar scratch:

    etaH_i = H_add(H_from(eta0_i), H_mul(H_from(s), H_from(alpha_i))).
    zH_K[k] = sigma_k etaH_COORD[k].
    deltaH_K = H_sub(zH_K, H_from(z0_K)).

The seventh known value has the existing negative sign. Use the same binary geometry D as input to the existing increment chart solve, including the entire accepted-state constraint defect:

    D_rows,U delta_U = -D_rows z0 - D_rows,K delta_K.

Over exact real arithmetic this equals the absolute solve for the same D and known targets. It does not equal E1's absolute solve when E1 rounded those known targets earlier. Do not drop D0*z0 or impose delta(0)=0 on an off-chart accepted state. H elimination is approximate and all 24 measured constraint rows remain required.

Produce one stored zB by RN64 of every known and dependent endpoint coefficient. Make stored etaB agree with its six known coordinates; do not leave a stale native eta paired with new zB. Embed stored UB with the unchanged native embedding graph. Recompute every dependent term from this single stored point: full constraints, differentiated constraints, mesh motion, Ddot, pressure image/adjoint, symmetric strain, masses, positive/negative relative fluxes and their inspections. New sign partitions and quadrature points must evaluate this graph afresh. Captured fixed-operator forces or fluxes from the diagnosis are not an E2 equation. A derivative of the exact chart is not a theorem about a rounded program's derivative.

The accepted start overrides reconstructed start z/U/m exactly as in E1. After quadrature, use the existing native checked subtraction and matvec on the already stored end zB and borrowed accepted z0:

    deltaStoreB_j = native_add(zB_j, -z0_j).
    incrementB_i,d = native_dot_in_existing_j_order(R_i,d, deltaStoreB).

The ephemeral H increment is a chart solve variable only. It never enters qualified inertia. Reconciliation is performed by deriving inertia from the stored fields rather than relying on a low-part correction to a latent residual. Direct inertia, body load, endpoint donor transport, force assembly, projected accumulation/division and sequential native norm retain their E1 operation order. The only proposed arithmetic change is deferred working affine-known rounding within the existing chart solve. Every existing finite-difference Newton column must rebuild the complete E2 graph. A norm computed by an offline exact oracle never replaces the native Newton norm.

For exact-input comparison, define hat_z as the captured H endpoint before RN64, deltaLatent=hat_z-B(z0), deltaStore=B(zB)-B(z0), and rho=deltaStore-deltaLatent. Then

    deltaStore = deltaLatent + rho,
    I_stored - I_latent = diag(m1) R rho,
    rate_stored - rate_latent = R^T diag(m1) R rho / h

when all other inputs are held fixed. The public evaluator loads stored differences directly, so no low component of hat_z is needed to qualify the endpoint. Capturing latent parts for this comparison is observational only. The native stored subtraction and dot retain their rounding errors, which are measured separately; they are not claimed exact.

Retain the prior exact stored embedding identities e_n=R z_n-U_n and

    I_stable-I_direct = diag(m1)(e1-e0).
    U1 dot rN-W_R = -e1 dot rN-(U1+e1) dot diag(m1)(e1-e0).

Rounding discrepancies are not physical viscosity, extra work allowances or corrections added to manufacture acceptance. Stored three-component energy, BE/mixing/strain/pressure/GCL/body work and both residual gates remain authoritative. The third solver and its two shears remain unchanged and use the same stored endpoint geometry and transfers. A planar pass never supplies an unexecuted third solve or full step.

## Ownership, live storage and failure

Reuse the existing owner, its candidate/geometry workspace and one existing ChartWorkspace loan. Accepted input is borrowed; the existing extracted 22-coefficient accepted_z is not a second authority. No new owner, accepted-state snapshot, clone of Point, separate clock or restart tail is permitted. Fully initialize every scratch value read. Dirty/new/cancelled workspace must give the same next result from identical accepted inputs. Cancellation, checked failure and refusal preserve every accepted velocity/pressure/geometry/mass/clock/stamp bit through the existing final publication barrier.

The proposed scalar layout reuses the existing 1,934 slots of 32 bytes, 61,888 bytes of scalar storage, within the existing 62,096-byte ChartWorkspace plus its 8-byte descriptor. Six affine lanes may occupy existing coordinate slots 1917–1922; the existing DELTA lanes hold working chart increments only. The native stored-difference vector is the existing equation scratch, not a new high-precision buffer. These are proposed nonoverlapping lifetimes, not compiled offsets or a proven new memory bound. No additional scalar array is allocated. Existing layout, alignment and all simultaneously live workspace, stack, kernel frames, callbacks and constructor/public-call paths must be freshly audited on the actual new ELF before execution. The unchanged additional allowance is 66 KiB = 67,584 bytes. The old diagnosis's 65,616-byte bound and 1,968-byte margin apply to that old capture graph, not E2.

A single point's latent lanes are overwritten by the next quadrature point. Stream observational parts at their evaluation time; never retain an unbounded journal in native memory. Derive endpoint stored increments after quadrature from the existing Equation endpoint using the unchanged native equation code. Do not retain a high-precision endpoint copy throughout quadrature, nor silently exclude it from live memory. External evidence storage/offline rational work is separate from the native numerical allowance and cannot act as an accepted state.

## Frozen comparison roster and measurements

The machine-readable [policy](../../../evidence/forcing-affine-rounding-contract-v1/policy.json) fixes cases 41 and 47, their complete original failed prefixes, final seventh-correction unknowns, h=0.00078125 and orders 16/32. The existing inputs and journals are pinned by hash. The first future comparison evaluates four fixed equations, with zero accepted owners constructed/advanced, zero new Newton corrections, no third solve and no publication. It is blocked now. A baseline using original E1 must reproduce both order-16 failed norms bit for bit. Failure to reproduce is a provenance failure, not permission to select another state.

Keep three distinct comparisons, without search:

| Comparison | Inputs/meaning | Required independent observation |
|---|---|---|
| C0 frozen E1 replay | Existing stored candidate and full captured equation | Native components/norm, forces and transfers reproduce; original refusals retained. |
| C1 same stored fields | Exact binary-input products/sums and stored increment re-expression | All 22 terms, exact squared norm, embedding and rounding defects; no candidate change. |
| C2 proposed E2 | Same parameters, deferred working known/chart graph, unchanged native stored-difference inertia | All changed stored bits and downstream terms, original native norms and gate margins; no latent increment substitution. |

C2 must round/reconcile the actual candidate and recompute every downstream term and original gate from it. Stream point observations while reusing one workspace. Compare both the frozen native graph and an independently derived exact-binary-input equation on C2's changed stored fields. The fixed comparison constructs zero accepted owners. An independently written scalar conformance oracle must check actual H operation parts against exact rational rounding, including ties, negative signs, overflow, nonzero subnormals and division failures. A rational 15x15 reference must distinguish exact continuous affine knowns, the actual captured H knowns, and rounded stored knowns, reporting all three and all 24 constraint rows. This separates working-affine error, factorization error and endpoint rounding; none is native acceptance.

For every complete equation, preserve all 22 stable/direct rate components, actual sequential norms, exact squared norms, original-target signed margins, start/end stored z/U/m, pre-store H parts, known-rounding gaps, rho and projected rho/h, embedding defects, force/body/transport/inertia term groups, full constraints and refinement differences. Keep complete native geometry/force/sample provenance, positive and negative transfers, physical-path and endpoint convection, partitions, sample counts and all rejection reasons. Norm diagnostics at 80 and 120 decimal digits and normal/optimized readers must agree; classifications use exact squared norms when labeled exact. Every value must identify its units and whether it is stored, latent, finite-H, exact binary-input or native rounded. Counterfactual norms never supply acceptance.

A future public-call comparison is a separate stage requiring its own frozen source and execution receipt. Start with the original constructor and exact accepted prefix. If E2 changes an earlier accepted step, report a different trajectory instead of substituting it for the fixed-prefix comparison. Bind all actual physical/refinement/third/work/GCL/range/publication gates. Dirty/new workspace, scheduled cancellation, failure/retry, same-owner repeated refusal and all accepted bits require independent replay. No checkpoint API is invented.

Any later trajectory qualification retains the original 48-case/8-lifecycle roster, five coarse intervals, two finer intervals, original time, loads/initial conditions, reference settings and original geometry maximum metric. Seven corrections, 200 counted equations, the final authorized-correction validation and the two original uncounted final acceptance equations keep their source semantics. Newton 1e-13, physical momentum/full constraints 1e-11, quadrature 1e-15 and work/GCL 128*eps remain unchanged. There is no extra iteration or parameter search. Both original refusals and all eight original geometry band failures remain in comparison reports. Signed components/error over h supplement the original 1.7–2.3 geometry band; they do not replace it. No tighter references, new endpoints, thresholds or runs are authorized by this contract.

## Review gate and owner decisions

Implementation and execution are separate milestones. Before any new candidate evaluation: record the independent diagnostic review disposition and its exact reviewed source/evidence; resolve objections; freeze matching E2 implementation/source/policy; pass scalar conformance, formatting, compilation, Clippy and actual simultaneous-memory qualification; freeze the specific execution authorization. This packet provides no native runner and marks disposition unavailable.

No owner semantic decision is needed to choose an ordinary stored-endpoint experimental arithmetic graph within these limits. A real owner decision is required if the desired improvement relies on qualifying deltaLatent instead of deltaStore, treating RN64 as display-only, retaining a tail, projecting initial state, changing q geometry arithmetic/physics, hiding a metric or enlarging the allowance. Those choices conflict with the inherited stored-state contract and are excluded. Report the conflict rather than silently selecting a latent-state law. If E2 remains refused, publish that result without adding corrections or claiming a floor.

The small analytical checks attached to this contract exercise reconciliation and changed intermediate rounding only. They perform no Rheon point, chart candidate, native evaluation, integration or trajectory. Existing Jacobi fixtures, research chapters, citations and Lean real-algebra claims are untouched. No theorem about IEEE execution, arbitrary fine-step convergence, a numerical floor, general three-dimensional liquids, new free-surface traction, material physics or surface reconstruction is added.
