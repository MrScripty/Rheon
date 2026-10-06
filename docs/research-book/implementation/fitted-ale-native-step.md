# Native nonzero relative transport on the preceding physical flow

The restricted material-frame source `96bb0efb5674d5a9505dcfae1d4ee5d2b73df001` and evidence `d8133c302fa61644062d347c3b387e7f6f5d4e8b` remain frozen. The actual finite-transport research source `b8dc1da4c53b06b4837fce0c241a94a70a7058a2` and evidence `d533f6929ad64d0ae3c39dbf9cb450bc6fd81198` supply the independent physical face-integral reference and preserved negative criteria. This successor implements that connected transport criterion, not a new unrelated flow family or a general pressure-coupled simulator.

## Supported advance and sole accepted authority

The opt-in public `FixedBottomAleFlow` advances the same U=(a,0,w), analytic relative pressure zero physical family with constant positive density, nonnegative constant viscosity and periodic extrusion. Its cap follows the accepted nonnegative constant x velocity. Its bottom mesh coordinates are fixed tangentially; the cap translates and actual interior Powell–Sabin intersections are recomputed. This creates nonzero relative transport and changing nodal liquid masses while total liquid mass is conserved. The cap offset is explicitly limited to one eighth of the period; geometry, thickness, shape, pressure-rank and arithmetic rejections also apply. The mathematical whole-interval certificates and numerical refinement study are specific to the published four-column fixture. The displacement limit alone is not a certificate for arbitrary initial graphs or other topologies.

The new wrapper reuses the preceding accepted-state payload: its full accepted velocity, candidate velocity, clock, stamp and solver vectors. It never exposes that inner material-frame stepping interface. Its public view describes the actual accepted fitted geometry, nodal liquid masses/volumes, all three velocities, common physical clock and analytic pressure. There is one accepted fitted frame and one strictly candidate frame, no separately publishable phase/MAC authority, externally supplied next height or donor velocity. The candidate-only private fitted reassembly method clears/reuses existing capacities and retains constructor coefficients and rejection settings. A failed/cancelled reconstruction can leave candidate scratch incomplete; a subsequent attempt reconstructs it from the unchanged accepted frame.

## Actual integrated transfers and composed equation

For each time interval the candidate workspace reconstructs the declared cap-driven/fixed-bottom trajectory at 8-point and 16-point Gauss nodes. The existing instantaneous assembly differentiates the same periodic Powell–Sabin construction with actual cap velocity and fixed bottom. The third velocity does not affect xy face mass flux because the model is independent of z. Integrate the two nonnegative transfers separately:

    P_ij=integral max(f_ij(t),0) dt,  N_ij=integral max(-f_ij(t),0) dt,
    F_ij=P_ij-N_ij.

The 16-point transfers define the operation; the 8-point values provide a measured quadrature refinement gate. Per-face differences must be within 64 machine epsilons times the incident old-mass sum. No transfer is fitted to endpoint masses and no face circulation is inferred from GCL. This comparison is a numerical gate, not a rigorous quadrature/IEEE error theorem on arbitrary graphs or sign-changing paths. The independent high-precision antiderivatives certify the recorded native fixture's actual transfers; its qualified path has no flux reversal.

Reconstruct the endpoint geometry and verify topology/embedding compatibility. Derive every new mass from its actual triangles. Gate each finite GCL defect d_i=m_new,i-m_old,i+sum_j F_ij at 128 epsilons times m_old,i+m_new,i, and gate total mass independently with the corresponding whole-mass scale. Reject rather than repair these defects. No positive mass, viscosity or density floor is added.

For an oriented i<j face the donor matrix contributions are A_ii+=P_ij, A_ji-=P_ij, A_jj+=N_ij and A_ij-=N_ij, with A initially diag(m_new). The actual existing third trace embedding R is retained. Solve

    R^T(A+dt K_new)R z_new = R^T M_old U_old,z,  U_new,z=R z_new.

The reduced mass is not replaced by its diagonal. Full-space, two-pass modified Gram–Schmidt GMRES with Givens rotations handles the nonsymmetric composed equation; CG remains only in the preceding material-frame mode. Every Krylov/Hessenberg vector is preallocated and charged, with at most the existing third-space dimension of Arnoldi steps and the configured iteration bound. No inverse transfer map, dense new coefficient matrix or hidden restart history is materialized. True composed residual is recomputed directly before acceptance.

Constant xy carrier velocity satisfies the same row balance in exact GCL arithmetic. Its measured restricted x momentum residual from a*d is included in the reported full residual norm and work. The existing x and third embeddings have the same support/weights; their column offsets differ. Y momentum residual is zero in this invariant family. This uses existing velocity owners/basis construction, not a newly asserted pressure degree count. Candidate strong xy divergence is measured from actual element gradients. Pressure remains analytic zero, not a solved pressure field or a proxy for general stress coupling.

## Full work ledger, failure and publication

Let r include the restricted third and constant-carrier momentum residuals. The native acceptance ledger is

    E_new-E_old + increment_loss + donor_loss +dt strain_power
      = full_residual_work - geometric_defect_work,

    increment_loss=1/2 sum_i m_old,i |U_new,i-U_old,i|^2,
    donor_loss=1/2 sum_unordered_ij (P_ij+N_ij)|U_new,i-U_new,j|^2,
    geometric_defect_work=1/2 sum_i d_i |U_new,i|^2.

The strain evaluation uses the preceding constant-preserving element differences and the actual endpoint mesh. Energy, all momentum components, finite mass/GCL, quadrature, true composed residual, actual divergence and this work ledger are measured independently. The work allowance is 128 machine epsilons times the measured energy/loss/residual-work scales, with no constant lower scale floor. Default linear targets and absolute momentum/divergence tolerances retain the preceding settings; no Jacobi fixture or previous numerical tolerance is relaxed.

Nonzero subnormal intermediates, checked-division failure, overflow, unsupported initial carrier/trace, lost time/coordinate resolution, mesh/path/rank failure, iteration or acceptance failure and six cancellation stages reject before publication. The final barrier precedes only infallible frame/velocity swaps and clock/offset/stamp assignments. Public transfer arrays are labelled working scratch and may be incomplete after a rejected attempt; they are not accepted phase state.

The strict path endpoint is not extended by an epsilon tolerance. The initial nominal twenty-step trial exceeded q=1/8 by an ulp and was rejected; its raw failure log is preserved. The example driver requests the remaining physical interval on its last step and records its actual dt. For nominal dt=0.025 its final request is 0.024999999999999856, producing actual time 0.5 and cap offset 0.125. The owner never clamps requested geometry or fabricates a fixed-step time. Independent replay uses each recorded dt and verifies the physical clock and trajectory.

## Bounded payload and actual evidence

On the qualified 64-bit target the payload is

    2688 C^2 +23224 C +448 bytes,

or 136,352 bytes for C=4. The nominal estimate derives from the existing frame/flow plans and actual type sizes. Actual reserved capacities are charged against the shared memory limit. It includes both fitted frames and the preceding velocity/solver arrays, cap/bottom scratch, quadrature carrier/diagnostics/pressure, shared transfer/coarse-transfer rows, GCL scratch and full Arnoldi/Hessenberg/rotation/least-squares buffers. Constructor allocations are bounded; a step allocates no heap. Caller-side example snapshots, JSON output, independent reference dense matrices and rendering remain explicit outside that owner payload.

The packet records six native debug/release runs at nominal dt=0.05,0.025,0.0125; independent normal/optimized replay of every accepted geometry, mass, transfer and composed solve; nine corruptions per validator including a real zero-node-balance face circulation and endpoint-frozen transfer; six cancellation stages after nonzero accepted state in every native run; and seven Rust contracts. A final-source rebuild records complete Rust input hashes and reproduces the independently qualified payloads byte-for-byte. Plots/GIF use only recorded native accepted states.

At time 0.5, native temporal mass-L2 errors versus the independently refined nonautonomous ALE ODE are approximately 0.00326517973851,0.00165313843184,0.00083189297152. The worst recorded cumulative full-momentum drift is below 6.7e-12; the maximum physical face-integral error is below 7e-18. These are actual numerical measurements, not continuum spatial convergence or exact IEEE conservation theorems. The accepted y^2 fixture has both nonzero donor and viscosity dissipation and changing geometry-derived nodal masses.

## Preserved failures and next general coupling gate

The frozen constrained convex-bound failure remains a limitation. An actual native inviscid, nonnegative nodal basis fixture also develops negative third velocity under repeated constrained transport while passing momentum/work gates. Neither the unconstrained donor M-matrix theorem nor its conditional Lean claims is extended to R^T A R. Positive liquid masses come from positive actual geometry, independently of velocity signs. The wrong endpoint transfer's correct mass marginals do not make it physical; the native output validator independently rejects its face provenance and zero-balance circulations.

All frozen evidence, proofs, original Jacobi fixtures, historical accuracy failures and public general-step refusals retain their bytes. No new Lean proof of native execution, uniform inf-sup constant, whole-family pressure rank, pointwise free-surface traction or PDE/continuum spatial accuracy is claimed. Pressure image is actually checked by the existing bounded constructor at each reconstructed geometry; this does not manufacture or prove a general degree count.

The new milestone closes actual nonzero relative transport in the same advancing physical flow. General xy momentum/pressure coupling and its compatible material cap trajectory remain the next missing integrated criterion. Variable density, adhesion, body forces, capillarity, reconstruction/support change, mesh adaptation and a fully validated general liquid simulator remain separate unfinished goals. The general advancing refusal stays in place.
