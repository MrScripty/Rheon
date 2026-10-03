# 11 Particle alternatives and hybrid transfers

Dense grids pay for empty space; particles pay for occupied material and neighborhood organization. Include neighbors, sorting, scratch and surface reconstruction when comparing memory. Particle count alone is not a budget.

## SPH

A smoothing-kernel density estimate is

\[
\rho_i=\sum_jm_jW_h(\mathbf x_i-\mathbf x_j).
\]

Support defines neighborhoods, often found through uniform bins or sorting. Pairwise force formulations can preserve antisymmetry when consistently evaluated, but free-surface neighbor deficiency and boundary quadrature remain. Müller, Charypar and Gross present an interactive SPH method [S13]; its published performance is not inherited by Rheon.

Weakly compressible pressure laws turn density deviation into force. Higher stiffness reduces compression but introduces fast waves and smaller steps. Visual softness may reflect tolerated compression. Measure density and volume errors before claiming incompressibility.

## Position based fluids

Position based fluids iteratively corrects predicted positions using density constraints [S15]. For \(C_i=\rho_i/\rho_0-1\), a local multiplier has the form

\[
\lambda_i=-\frac{C_i}
{\sum_k|\nabla_{\mathbf x_k}C_i|^2+\epsilon}.
\]

Regularization prevents singular corrections but changes behavior. Lists may become stale as positions change; rebuilding trades work for accuracy. Reconstructed velocity includes position corrections. Finite iterations yield approximate density constraints, not exact incompressibility or calibrated viscosity.

Compare complete frame cost at fixed occupied volume, obstacles, density tolerance, surface metric and memory cap. One density iteration and one pressure iteration are incomparable work units.

## PIC and FLIP

PIC transfers particle momentum to a grid, applies grid forces and pressure, then samples velocity back. Repeated averaging dissipates detail. FLIP instead adds grid velocity changes:

\[
\mathbf v_p^{+}=\mathbf v_p+
\sum_iw_{ip}(\mathbf u_i^{+}-\mathbf u_i^{old}).
\]

Blending PIC and FLIP trades damping against retained particle noise. Zero or tiny grid mass needs explicit handling. Parallel deposits need race-free accumulation.

## APIC

APIC adds a local affine matrix \(C_p\) to particle velocity. A transfer includes

\[
m_i\mathbf u_i=\sum_pw_{ip}m_p
\bigl(\mathbf v_p+C_p(\mathbf x_i-\mathbf x_p)\bigr).
\]

Jiang and collaborators derive conservation properties for specified transfer rules [S16, S17]. They depend on weights, moment matrices and updates. Merely adding nine floats does not inherit angular-momentum conservation. Boundary-truncated support also needs analysis.

Position, velocity, affine matrix and one scalar use 16 single-precision values, or 64 bytes before IDs, padding and scratch. Eight particles per cell in a filled 64-cubed region require 128 MiB for those fields alone; at 128 cubed, 1 GiB. These calculated costs matter in a model-memory-constrained host.

Choose dense grids for bounded low-resolution domains, sparse grids when empty space dominates, and particles when material motion justifies their overhead. Compare identical initial geometry, camera and metrics. No candidate wins every scene.

## Complete PIC and FLIP update sequence

At particle positions \(x_p\), evaluate weights \(w_{ip}\) on the selected grid lattice. Require partition of unity and, when claiming linear reproduction, \(\sum_iw_{ip}x_i=x_p\). Deposit mass and momentum:

\[
m_i=\sum_pm_pw_{ip},\qquad
g_i=\sum_pm_pw_{ip}v_p,\qquad
u_i=g_i/m_i\quad(m_i>0).
\]

Store this normalized velocity as the old grid velocity before forces, pressure and collision changes. Apply the grid update to obtain \(\widetilde u_i\). Then

\[
v_p^{\mathrm{PIC}}=\sum_iw_{ip}\widetilde u_i,\qquad
v_p^{\mathrm{FLIP}}=v_p+
\sum_iw_{ip}(\widetilde u_i-u_i).
\]

A blend with parameter \(\beta\in[0,1]\) is \((1-\beta)v_p^{\mathrm{PIC}}+\beta v_p^{\mathrm{FLIP}}\). Advect positions using a declared grid-velocity interpolant and integrator, rather than assuming noisy FLIP particle velocity is always the best trajectory field. Collisions and reseeding follow a defined stage order.

A face-centered grid performs these deposits separately for each component lattice. Face mass is a transfer weight, not automatically the physical control-volume mass used in a pressure derivation. Keep the transfer normalization and fluid density model distinct. Near empty faces, a threshold must distinguish unavailable velocity from a valid zero.

Constant particle velocity is preserved by PIC transfer when all required mass and weight conditions hold. FLIP can preserve particle modes invisible to the grid because only grid changes are added. A pair of opposite particle velocities depositing to the same grid sample gives zero grid velocity; PIC removes that mode while FLIP retains it under a zero grid update. This is a concrete noise/dissipation tradeoff, not simply an implementation defect.

## Affine transfer moments

For frozen particle positions define offsets \(r_{ip}=x_i-x_p\) and moment matrix

\[
D_p=\sum_iw_{ip}r_{ip}r_{ip}^T.
\]

A local affine particle field is \(v_p+C_pr_{ip}\). Deposit

\[
g_i=\sum_pm_pw_{ip}(v_p+C_pr_{ip}).
\]

After grid evolution, a frozen-position least-squares affine reconstruction is

\[
v_p^+=\sum_iw_{ip}\widetilde u_i,\qquad
H_p=\sum_iw_{ip}\widetilde u_i r_{ip}^T,\qquad
C_p^+=H_pD_p^{-1}.
\]

These equations assume partition of unity, zero first moment and invertible D. They describe a transfer subproblem. Full time-integrated APIC variants require matching position and affine updates; conservation results from [S16, S17] cannot be transferred to an arbitrary integrator.

To see affine reproduction, let grid velocity be \(a+Cx_i\). The particle average becomes \(a+Cx_p\) because the weighted offset sum is zero. The cross moment is \(CD_p\), so multiplying by \(D_p^{-1}\) returns C. The companion tests this with an independently constructed affine field. It does not test a full APIC liquid simulation.

## Moment degeneracy and kernels

Linear hat weights in one dimension give \(D=h^2t(1-t)\), which vanishes when a particle lies exactly on a grid point. An inverse then becomes undefined. Ignoring the case can generate huge affine coefficients near nodes. A pseudo-inverse, a limiting formula or a higher-order kernel are different choices that require their own consistency tests.

A tensor-product quadratic B-spline uses one-dimensional kernel

\[
N(r)=
\begin{cases}
3/4-r^2,& |r|<1/2,\\
\tfrac12(3/2-|r|)^2,&1/2\leq|r|<3/2,\\
0,&\text{otherwise}.
\end{cases}
\]

With complete regular-grid support, its moment is \(D=(h^2/4)I\), giving a simple inverse and 27 collocated nodes per particle in three dimensions. Staggered component lattices require their own shifted supports; three components can mean 81 scalar deposits. The cost is not equivalent to eight-corner trilinear sampling.

Truncating support at a wall breaks moment identities. Renormalizing surviving weights restores their sum but generally not the zero first moment. Recentring changes the effective sample location. Ghost support, constrained reconstruction or boundary-specific transfer can address this, but each must be specified. The companion deliberately removes one support node and measures the resulting moment defect instead of silently claiming affine preservation.

## Momentum bookkeeping

Partition of unity gives total grid mass equal to total particle mass. Zero first moment removes the affine contribution to total linear momentum. Angular momentum has an internal affine contribution:

\[
L_p=m_p x_p\times v_p+
m_p\sum_iw_{ip}r_{ip}\times(C_pr_{ip}).
\]

Comparing only \(m_px_p\times v_p\) would omit represented subparticle rotation. A transfer test should compare the full particle representation with grid angular momentum using the same support. Boundaries can exchange momentum; those impulses must be included rather than blamed on transfer error.

## SPH kernels and gradients

For an explicit three-dimensional density fixture, use the compact poly6 kernel

\[
W(r)=\frac{315}{64\pi h^9}(h^2-|r|^2)^3
\quad\text{for }|r|<h,
\]

and zero outside. Its gradient is

\[
\nabla W(r)=
-\frac{945}{32\pi h^9}(h^2-|r|^2)^2r.
\]

The gradient is zero at coincident positions and at the support boundary under this formula. Other kernels may improve pressure behavior, but replacing the gradient with that of a different kernel changes the constraint derivative. The companion finite-differences the actual density constraint to test this exact gradient relationship.

For equal particle mass m and \(C_i=\sum_jmW(x_i-x_j)/\rho_0-1\),

\[
\nabla_{x_i}C_i=\frac m{\rho_0}\sum_{j\ne i}\nabla W(x_i-x_j),
\qquad
\nabla_{x_j}C_i=-\frac m{\rho_0}\nabla W(x_i-x_j).
\]

These derivatives sum to zero, expressing invariance under uniform translation. Include self density \(mW(0)\) while excluding a spurious self-force. A finite-difference test that accidentally omits self density can still have the right gradient but the wrong density target.

## PBF correction and neighbor validity

With equal solver weights, set

\[
\lambda_i=-\frac{C_i}
{\sum_k|\nabla_{x_k}C_i|^2+\epsilon}.
\]

Accumulating all constraint contributions gives

\[
\Delta x_i=\frac m{\rho_0}
\sum_{j\ne i}(\lambda_i+\lambda_j)\nabla W(x_i-x_j).
\]

Use Jacobi-style separate correction buffers if all multipliers were computed from the same predicted positions. In-place updates change the iteration into an order-dependent method. Unequal masses require the corresponding inverse-mass-weighted constraint formulation; do not reuse the equal-weight equation unchanged.

A full step predicts positions from forces, builds neighborhoods, iterates density/collision corrections, reconstructs velocity from corrected displacement, then applies any declared viscosity or vorticity postprocess. Recompute density error after the final correction. A fixed iteration count defines a quality budget, not an incompressibility guarantee.

Uniform bins of width h need the 27 neighboring bins in three dimensions for support radius h. Hash collisions must be resolved by cell keys, and every candidate still needs an exact distance test. A cached neighbor list needs a skin radius and a displacement bound before reuse; otherwise particles entering support are missed. Sorting, prefix sums, bin ranges and particle permutations must preserve particle IDs and all state fields consistently.

## Complete particle memory accounting

For P particles and K stored directed neighbor entries, a possible PBF layout uses old and predicted positions, velocity and correction vectors: 12 floats, or 48P bytes. Density and multiplier add 8P bytes. Two 32-bit sort/index arrays add 8P bytes. CSR neighbor offsets add approximately 4(P+1) bytes and neighbor IDs add 4K bytes. This totals roughly \(68P+4K\) bytes before boundary data, capacity padding, bin tables, renderer fields and alternative sort scratch.

At P=100,000 and mean directed neighbor count 60, these named arrays occupy about 30.8 MB in decimal units. A dense cluster can increase K sharply unless a cap changes the method; dropping neighbors silently changes density and forces. A checked capacity failure is preferable to an unreported truncated neighborhood.

For APIC, the earlier 64P-byte state estimate excludes old/new grid velocity, grid mass, sorting, particle output buffers and interface reconstruction. Count peak concurrent state for the actual update schedule. Particle methods save empty space only when these complete costs beat the selected grid alternative for the target scene.
