# 4 Pressure as a constrained correction

Let \(\mathbf u^*\) be velocity after non-pressure updates and \(W\) a diagonal matrix of nonnegative face weights. The graph correction is

\[
\mathbf u^{+}=\mathbf u^*-\Delta t\,WB^Tp,
\qquad A=BWB^T,\qquad b=B\mathbf u^*/\Delta t.
\]

Solving \(Ap=b\) makes \(B\mathbf u^{+}=0\), hence physical divergence \(D=-B\) vanishes. Density and grid-spacing factors can be absorbed into \(W\) and \(B\), but the implementation must state how. A dimensionless graph formula must not be copied into a metre-based solver without scaling.

## The pressure quadratic form

\[
p^TAp=(B^Tp)^TW(B^Tp)
=\sum_e w_e(p_{\mathrm{head}(e)}-p_{\mathrm{tail}(e)})^2.
\]

This is nonnegative when weights are nonnegative; the matrix is symmetric. Theorems pressure_energy, pressure_nonnegative and pressure_symmetric establish these facts. Strictly positive weights make zero energy imply equal pressure across every edge. On a connected closed graph, pressure is constant; on a disconnected graph, each component has its own constant. This connectivity-to-kernel argument is not currently in the Lean inventory.

A closed domain therefore gives a positive semidefinite operator. Pin one pressure per component by eliminating the row and column, or solve in the mean-zero subspace. Replacing only a row with an identity row breaks symmetry unless the column is handled consistently. Prescribed pressure is eliminated and its known contribution moved to the right-hand side.

## Compatibility

Each closed component needs a zero-sum pressure right-hand side. Balanced incidence columns imply \(\mathbf 1^TB=0\). A net inlet into a sealed domain violates compatibility. Subtracting the right-hand-side mean changes the requested problem. It may be an explicitly chosen correction policy but cannot be silent recovery.

The constant_pressure_nullspace theorem proves the constant nullspace under balanced columns. Internal_flux_conservation proves matching cancellation. A nonzero total divergence in a fully closed graph, beyond expected rounding, should trigger assembly, boundary timing or unit checks before increasing solver iterations.

## Residual as a diagnostic

For approximate pressure \(p_k\), let \(r_k=b-Ap_k\). Substitution gives, without a convergence assumption,

\[
B\mathbf u^{+}=\Delta t\,r_k,
\qquad D\mathbf u^{+}=-\Delta t\,r_k.
\]

Projection_residual is the central checked theorem. It links stopping to a velocity constraint. A residual norm means different things if the matrix has been multiplied by \(h^2\), density or time step. Recompute divergence through the production operator and report units. Relative residual alone is insufficient near zero right-hand side; use a documented absolute floor.

## Two cells worked completely

Take time step 1, weight 1 and face velocity 3. Then \(b=(-3,3)\). Fix \(p_0=0\); the reduced equation gives \(p_1=3\). Corrected velocity is \(3-(3-0)=0\). At \(p_1=2.9\), residual is \((-0.1,0.1)\), velocity is 0.1 and physical divergence is \((0.1,-0.1)\). Residual and divergence agree exactly, while acceptable magnitude remains a scene-dependent engineering decision.

For positive weights, minimize velocity change in the \(W^{-1}\) metric subject to incompressibility. A Lagrange-multiplier calculation yields the same normal equations. This explains nonincrease of the corresponding kinetic norm for compatible exact projection without boundary work. The current Lean package proves operator identities, not the complete constrained-minimization theorem or its finite-precision version.

The matrix can be algebraically correct and geometrically wrong. The theorem accepts arbitrary \(B\); it cannot distinguish a neighboring cell from an incorrectly indexed distant one. Independent geometry and stencil tests are essential.

## Assembly by boundary case

Every candidate face should be classified before assembling a pressure row. A fluid-fluid face contributes coefficient w to both diagonals and -w to both off-diagonals. A fluid-solid face with prescribed normal speed supplies known boundary flux; it does not add a fictitious pressure neighbor. A fluid-air face with prescribed pressure contributes w to the fluid diagonal and \(w p_{\mathrm{air}}\) to its right-hand side. An inactive air-air face contributes no liquid pressure equation.

For example, one liquid cell connected to a prescribed zero-pressure surface at half a cell distance has coefficient \(S/(\rho h/2)=2S/(\rho h)\). Treating the surface as a whole-cell distance halves the coefficient and changes the correction. Conversely, treating a solid as zero-pressure air creates a leaking wall. These two common mistakes can both produce plausible-looking motion while violating the intended boundary.

An isolated liquid cell surrounded entirely by fixed walls has no pressure correction degree of freedom through internal faces. If its prescribed net flux is zero, its pressure is undetermined and velocity is already constrained by boundary data. If net flux is nonzero, the problem is incompatible. It is not repaired by adding a tiny diagonal unless the model explicitly permits compressibility or leakage.

## Connected components and gauges

Build a graph using pressure unknowns and strictly positive fluid-fluid transmissibilities. Traverse it with a queue or union-find and record component IDs. A component touching a prescribed-pressure face is anchored by that boundary. A closed component needs a compatibility test and one gauge.

Compute compatibility from integrated flux, using a numerically careful reduction. Its tolerance should account for the magnitudes accumulated and the accepted flux budget. A component with a large inlet and equally large outlet can have cancellation error much larger than an otherwise identical quiescent component. A fixed unscaled epsilon cannot distinguish these cases reliably.

For a closed compatible component, choose a deterministic gauge cell, such as the smallest stable cell index. Remove its row and column; set its known pressure to zero. If using a mean-zero projection instead, apply the projection consistently to residuals, directions and preconditioning. A preconditioner that introduces constant components can corrupt a nominally mean-zero solve.

The topology snapshot should retain the mapping between full cell indices and reduced unknown indices. Do not rebuild it nondeterministically inside every matrix application. When geometry changes, invalidate the operator, component map, preconditioner and any pressure warm start whose gauge mapping is no longer meaningful.

## Acceptance in physical space

After solving, apply the pressure gradient once using the same transmissibilities or metric factors as assembly. Then recompute physical divergence independently from the corrected face velocities and prescribed boundary data. Compare this with the predicted residual-derived divergence.

Their difference has two sources: floating arithmetic and inconsistent implementation. The companion quantifies a small arithmetic discrepancy on its reference graph. A much larger mismatch is evidence of wrong coefficients, stale geometry, inconsistent density or a boundary overwrite after correction. This paired diagnostic localizes failures better than either pressure residual or divergence alone.

The worked matrix appendix supplies exact chain and cycle fixtures. Add production tests for one closed component, two disconnected components, one prescribed-pressure opening, an isolated cell and incompatible moving-wall flux. These cases test the domain logic that the arbitrary-B theorem intentionally leaves open.
