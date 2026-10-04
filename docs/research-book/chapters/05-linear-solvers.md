# 5 Solving the pressure system

Pressure often determines both frame cost and volume quality. The solver receives an SPD reduced system or an explicitly handled semidefinite system. It returns pressure, iterations, true residual, termination reason and breakdown diagnostics. A vector alone loses information needed to accept or reject a frame.

## Jacobi as a reference

With diagonal \(D_A\), weighted Jacobi is

\[
p^{k+1}=p^k+\omega D_A^{-1}(b-Ap^k).
\]

Convergence requires spectral radius below one for \(I-\omega D_A^{-1}A\) on the solved subspace. Parallel local access is simple, but smooth error can decay slowly as grids grow. A fixed iteration count is not fixed divergence quality. Isolated zero-diagonal cells must be classified, never divided by zero.

Jacobi is useful as an independent reference and multigrid smoother. Compare time to an accepted residual, including reductions and synchronization, not one iteration against an unrelated solver iteration.

## Conjugate gradients

For an SPD matrix and SPD preconditioner \(M\), initialize \(r_0=b-Ap_0\), \(z_0=M^{-1}r_0\), \(d_0=z_0\). Then

\[
\alpha_k=\frac{r_k^Tz_k}{d_k^TAd_k},\quad
p_{k+1}=p_k+\alpha_kd_k,\quad
r_{k+1}=r_k-\alpha_kAd_k,
\]

\[
z_{k+1}=M^{-1}r_{k+1},\quad
\beta_k=\frac{r_{k+1}^Tz_{k+1}}{r_k^Tz_k},\quad
d_{k+1}=z_{k+1}+\beta_kd_k.
\]

Shewchuk develops Krylov methods and conditioning in detail [S5]. Rheon's operational contract additionally detects nonfinite values, nonpositive curvature, zero denominators and exhausted budgets. Check the initial residual before forming any quotient. Zero right-hand side and zero guess should return immediately.

The exact-arithmetic energy-error bound for SPD systems is

\[
\|e_k\|_A\leq
2\left(\frac{\sqrt\kappa-1}{\sqrt\kappa+1}\right)^k\|e_0\|_A.
\]

It is not a runtime prediction. Spectrum, preconditioner cost, precision and memory traffic matter; a singular system requires a specified subspace. The current Lean package does not prove CG convergence.

## Preconditioner selection

Diagonal scaling is cheap and parallel. Incomplete Cholesky can reduce iterations but adds factorization, ordering dependence and triangular solves. Modified incomplete Cholesky is discussed in Bridson's notes [S4]. Geometric multigrid attacks smooth error, but coarse levels must retain obstacle and free-surface semantics. A coarse operator that reconnects sealed regions changes the nullspace.

A matrix-free seven-point stencil is attractive for a known regular coefficient structure. Reconstruct neighbor contributions from face weights and diagonals. An explicit sparse matrix remains valuable in tests as an inspectable independent oracle. Comparing two wrappers around one kernel cannot catch shared assembly errors.

## Residual and warm starts

Recursive residuals drift from \(b-Ap\) in floating point. Recompute the true residual at termination and periodically where needed. Document direction restarts after replacement. Warm starts help only when operator, pressure gauge and topology remain compatible. New disconnected components or grid rescaling require remapping or reset.

The companion uses an 8-cubed closed grid, removes one row and column, and projects deterministic random velocity. It records CG and weighted-Jacobi histories. This compares mechanisms on that small system, not production frame time. Failure leaves the last accepted state intact or publishes a declared degraded preview according to an explicit lifecycle policy.

## Residual scales and breakdown policy

Let the integrated solve be \(Kp=b\). A relative criterion \(\|r\|_2\leq\tau_{\mathrm{rel}}\|b\|_2\) can be combined with an absolute criterion, but physical acceptance is additionally \(\|\Delta t V^{-1}r\|_\infty\leq\tau_{\mathrm{div}}\). The first measures linear-system progress; the second limits maximum cell divergence. They can disagree in cut cells. Physical acceptance must reconstruct the full ungauged residual on every active cell, including eliminated gauge cells. A reduced-system residual alone can hide accumulated imbalance in the removed row. Independently recompute divergence on every active cell after correction. The companion includes a compatible chain where all retained residual entries pass a threshold but the eliminated row fails it.

For a diagonal preconditioner, require positive diagonal entries for every reduced unknown. If the matrix has a zero row, classify the topology before invoking CG. If \(r^Tz\) or \(d^TKd\) is nonpositive in an expected SPD solve, return a breakdown with the iteration and scalar values. Do not take an absolute value or clamp the denominator; that changes the method and conceals the failed assumption.

The source of a breakdown can be an invalid operator, wrong gauge, loss of precision or an inappropriate preconditioner. Retrying in higher precision is useful only after the operator checks pass. A retry should start from a coherent state, recompute true residual and retain the original failure diagnostic.

## Why multigrid helps

Jacobi damps different error frequencies at different rates. For a one-dimensional uniform Dirichlet Laplacian, the diagonal-scaled Fourier symbol is \(1-\cos\theta\). Weighted Jacobi's error multiplier is \(1-\omega(1-\cos\theta)\). Small \(\theta\) gives a multiplier near one, so smooth error decays slowly. After transferring to a coarser grid, that same physical error becomes less smooth in grid coordinates and easier to correct.

This argument motivates a two-grid method: smooth the current estimate, restrict its residual, solve for a coarse correction, prolong it, and smooth again. Saad provides a primary textbook treatment of multigrid and sparse iterative methods [S22]. The construction below states Rheon's proposed contract rather than asserting an implemented hierarchy.

## Transfer operators

Let P map coarse pressure corrections to fine pressure corrections. For uniform cell-centered grids, construct P by trilinear interpolation at fine cell centers expressed in coarse-cell coordinates. The offset is important: a fine center is not generally a coarse node. Near boundaries, interpolation must respect prescribed pressure and component membership.

Choose restriction R consistently with the inner product. In an integrated symmetric system, \(R=P^T\) yields the Galerkin operator \(K_H=P^TK_hP\). If \(K_h\) is SPD and P has full column rank on the solved subspace, then for nonzero coarse x,

\[
x^TK_Hx=(Px)^TK_h(Px)>0.
\]

Thus coarse SPD follows from construction, with explicit rank and subspace assumptions. For volume-normalized residuals, volume-weighted restriction is more natural; convert consistently instead of mixing formulas.

On a closed graph, preserving constants requires \(P\mathbf1_H=\mathbf1_h\). Coarse connectivity must not join separate fine components. A coarse cell spanning a thin solid barrier needs multiple component degrees of freedom or a hierarchy policy that preserves separation. Otherwise the coarse correction allows nonphysical communication.

## V cycle specification

~~~
v_cycle(level, rhs, estimate):
    if level is coarsest:
        solve the gauge-fixed coarse system accurately
        return estimate
    apply fixed pre-smoothing sweeps
    residual = rhs - K[level](estimate)
    coarse_rhs = transpose(P[level]) * residual
    coarse_error = zero
    coarse_error = v_cycle(level + 1, coarse_rhs, coarse_error)
    estimate += P[level] * coarse_error
    apply compatible post-smoothing sweeps
    return estimate
~~~

As a standalone solver, monitor the finest-grid true residual after cycles. As a CG preconditioner, the V cycle must act as a fixed linear SPD map on the relevant subspace. Matching adjoint pre/post smoothing and an SPD coarse solve are important. Adaptive iteration counts, nonsymmetric Gauss-Seidel order or changing hierarchy can violate the usual PCG assumptions. A flexible Krylov method is a different algorithmic choice, not a label to add after the fact.

The expanded companion constructs a small Dirichlet one-dimensional hierarchy with interpolation P, Galerkin coarse matrix and weighted Jacobi smoothing. It checks symmetry, positive eigenvalues and measured residual reduction. This is an executable mechanism fixture, not a three-dimensional obstacle-aware multigrid implementation.

## Storage and work

A regular three-dimensional geometric hierarchy has cell counts \(N,N/8,N/64,\ldots\); their sum approaches \(8N/7\). This favorable count excludes per-level masks, transfer metadata, coefficients and temporary vectors. An irregular hierarchy can have much more overhead. Count actual simultaneous buffers, especially when used inside PCG.

For low-resolution Rheon scenes, a simpler diagonal-PCG solve may outperform a complex hierarchy once setup and changing geometry are included. Measure complete solve time to a physical tolerance. Multigrid is a promising mechanism, not a required optimization before evidence.
