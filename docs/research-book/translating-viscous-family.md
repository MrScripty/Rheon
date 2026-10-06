# A bounded advancing translating viscous family

This successor preserves the seam repair source `6fe335cd49488fd88d9281b06ddea843e4d537b4` and evidence `6b9ec40d25e88bb88fd84db2fd69c75ffa2e7ded`. The general liquid advancing refusal remains in place. The new public `TranslatedViscousFlow` advances one invariant physical family with a common accepted geometry, velocity, analytic pressure and clock. It does not qualify the general moving-domain pressure/remap problem.

## Remaining gate and chosen acceptance

General fitted stepping still needs a compatible finite material-cap trajectory, actual integrated shared relative flux, conservative momentum remap and a pressure/strain solve on that same space-time domain. A fixed fitted-frame rank measurement cannot supply uniform pressure stability. The extra trace restrictions and mass lumping do not inherit a published inf-sup theorem automatically. Neither endpoint mass fitting nor separate static pressure and transport demonstrations closes this gate.

The minimal physically closed target here is a genuinely translating varying-height surface with nonconstant, dissipating third velocity. Accept repeated native steps only when independently reconstructed geometry follows the accepted carrier velocity, liquid nodal masses remain bitwise fixed, all three momentum components remain within the stated absolute tolerance, actual divergence and true solver residual pass, and the backward-Euler work identity matches its measured residual work. Cancellation at every checkpoint and arithmetic/iteration failures must preserve the entire accepted state. Measure temporal convergence against the independently exponentiated **same spatial discretization**. An algebraic work identity alone is not an accuracy study.

## Physical family and material frame

Let the domain be periodic in x and z, with fixed extrusion width b and height

\[
H(x,t)=H_0(x-at),\qquad U(x,y,t)=(a,0,w(x-at,y,t)),\qquad \pi=p-p_{atm}=0.
\]

Density rho and dynamic viscosity mu are constant. Gravity, adhesion, variable density and surface tension are absent. The bottom permits tangential slip; its normal velocity remains zero. The entire fitted mesh translates by the accepted offset q, including the bottom's tangential mesh coordinates. The varying cap therefore moves with the actual fluid velocity; on a sloped cap its normal velocity need not vanish even though U_y=0. This is an Eulerian changing cap and nontrivial viscous evolution, not merely an instantaneous reassembly.

The co-moving mesh velocity is (a,0), identical to fluid xy velocity. Every actual fluid-minus-mesh xy face flux is exactly zero, integrated over every finite step. Periodic z flux cancels because all fields are independent of z. The rigid map has Jacobian one: every triangle area, nodal liquid volume, mass, velocity embedding and stiffness remain invariant. This is a legitimate conservative material transport specialization. It does **not** exercise nonzero relative remap flux or establish its positivity/conservation in the general case. The old frame diagnostic's fixed-bottom mesh motion is deliberately not reused as this trajectory's flux.

The xy equations are unforced constant momentum with zero pressure and zero xy strain. The remaining equation is

\[
\rho \partial_t w-\mu\Delta_{xy}w=0
\]

in co-moving coordinates, with periodic x and natural weak Neumann conditions on cap and bottom. The z traction condition is the natural boundary condition of the variational solve, not a pointwise traction measurement. Initial nodal y² data need not satisfy a classical pointwise Neumann condition. The finite-element semidiscrete solution is the comparison target. This milestone does not measure continuum spatial convergence or justify a general free-surface pressure solver.

## One owner and the finite step

The existing corrected `FittedHeightWorkspace` owns the immutable co-moving template, mass, embedding and strain assembly. The new flow owns one accepted full three-component nodal velocity, one bounded candidate velocity, one accepted offset, clock and stamp. The geometry authority is exactly template plus offset; there is no independent phase geometry or legacy MAC state. State views publish nodal liquid masses/volumes, velocity, analytic relative pressure and physical coordinates under that common stamp. Pressure is explicitly analytic zero, with no claimed pressure iterations.

Let R be the existing third-component embedding and M the positive lumped nodal mass. Its reduced mass RᵀMR is generally **not diagonal**. Each step solves

\[
(R^TMR+\Delta t\,R^TKR)z^{n+1}=R^TMU^n_z,
\quad U_z^{n+1}=Rz^{n+1},\quad q^{n+1}=q^n+\Delta t\,U_x^n.
\]

Matrix-free preconditioned CG uses the actual reduced diagonal for preconditioning, not a diagonal approximation to the equation. True residual is recomputed before publication. The native scalar stiffness is applied through local differences d=(w₁−w₀,w₂−w₀) and the symmetric 2×2 element submatrix, with the first nodal force minus the sum of the other two. This is the same exact rational element quadratic form and preserves constants without floating row-sum cancellation. The independent reference uses the full exact-rational assembled matrix, rather than duplicating this native evaluation order.

For full kinetic energy E=½ UᵀMU and measured reduced residual r, acceptance checks

\[
E^{n+1}-E^n+\tfrac12\|U^{n+1}-U^n\|_M^2
+\Delta t\,(U_z^{n+1})^TKU_z^{n+1}
=(z^{n+1})^Tr
\]

up to a documented floating rounding allowance. Momentum and divergence are measured independently. This identity is a work accounting check; temporal accuracy is assessed separately. Default CG relative/absolute residual targets are 1e−12/1e−13; absolute momentum and divergence limits are 1e−11. Reported errors remain measured rather than clipped or repaired by mass/momentum fitting. Nonzero subnormal intermediates, failed checked divisions, overflow, lost clock/coordinate or local physical-edge resolution, unsupported xy/trace fields and iteration/acceptance failures reject before the final infallible swap and scalar publication.

For C columns the additional heap payload is 944C bytes on the qualified 64-bit target: two full nodal velocities, eight reduced scalar vectors, two nodal scalar vectors and the free-node index vector. Together with the existing fitted workspace it is

\[
1056C^2+10520C+208\text{ bytes},
\]

or 59,184 bytes at C=4. Physical x-edge differences after converting template plus offset must match the template within 64 machine epsilons times the local edge 1-norm; this resolution guard has no lower scale floor. Actual reserved capacities are charged. There is no per-step heap allocation inside the owner, dense new solver matrix, trajectory history or hidden accepted snapshot. The existing bounded pressure-rank workspace remains included in this budget even though this family does not solve for pressure. Caller examples/reference matrices/rendering and vector headers/stack/bookkeeping are outside this payload accounting, as in the existing frame contract.

## Evidence and proof limits

The packet `evidence/translated-viscous-flow` records actual debug/release native trajectories at dt=0.05, 0.025 and 0.0125 through time 0.5, cancellation checks, independent dense backward-Euler replays, semidiscrete exponential temporal comparisons, eight corruption controls and plots rendered from the recorded accepted states. The root receipt binds exact source and evidence identities and preserves the complete historical evidence/proofs/book inventory. The failed initial exact-mass test is preserved: the native mass sum is 3.5624999999999996 rather than the exact rational 57/16. Conservation checks use the accepted sum bitwise, while the independent rational comparison measures that representational error separately.

No new Lean theorem claims a native IEEE solve, pressure inf-sup bound, PDE solution or arbitrary mesh trajectory. All prior conditional Lean claims and limitations retain their bytes and scope. This family's rigid-map identities are additionally tested with exact rational finite translated geometry. A transport-only coupled step is still not a fully validated liquid simulator. The general pressure-coupled advancing step, nonzero relative transport/remap, viscosity outside this family, adhesion, variable density and surface reconstruction remain separate unfinished criteria.
