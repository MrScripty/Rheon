# 36 Two complete minimal FEM algorithms and their verification boundary

This chapter closes the numerical and dataflow choices for **one hyperelastic solid path and one compressible material-mesh fluid path**. Both are finite-dimensional algorithms in original pseudocode, not executable Rheon implementations. Equations, analytic checks and reasoning review are distinct from numerical validation and machine proofs. The earlier mixed incompressible P2/P1 equations remain an advanced candidate in [Chapter34](34-incompressibility-and-fluids.md); they are not silently substituted for this selected fluid.

## Fixed model, units and run contract

Use scales \(L_*=1\) m, \(\rho_*=1\) kg/m\(^3\), \(V_*=1\) m/s. Consequently \(t_*=L_*/V_*=1\) s, \(M_*=\rho_*L_*^3=1\) kg, \(p_*=\rho_*V_*^2=1\) Pa, \(F_*=p_*L_*^2=1\) N, and \(E_*=p_*L_*^3=1\) J. Divide coordinates/displacements by L*, velocities by V*, time by t*, mass by M*, force residuals by F*, and energies by E*. All algorithm arrays below are **dimensionless**. Their numerical values happen to equal the stated SI values, but the normalization makes merit gradients and line-search steps dimensionally valid. A different scale choice changes the nondimensional inputs coherently, not the equations.

Both bodies start as \(\Omega_0=[0,1]^3\), material density \(\rho_0=1\), \(x_0=X\), no body acceleration or applied dead traction, and no contact, heat, viscosity, gravity or remeshing. Material choice is immutable for a run:

- **Solid S:** \(\mu=\lambda=1\), with exactly the Chapter31 energy \(W_s=(F:F-3)/2-\ln J+(\ln J)^2/2\). All three displacement components are fixed to zero on the reference x=0 face; all other faces have zero natural traction. The clamp supplies reactions. There is no pressure DOF.
- **Fluid G:** inviscid isentropic gas with \(\gamma=5/3\), \(p_0=\rho_0=1\), \(W_g=3J^{-2/3}/2\), \(p=J^{-5/3}\), \(\rho=1/J\), \(e=3J^{-2/3}/2\). Each coordinate face fixes only the displacement component normal to that plane. For example x=0 and x=1 fix d_x=0; tangential components are free except where another face imposes its normal condition. Tangential natural traction is zero. Positive absolute pressure creates normal wall reactions; no outside pressure offset, pressure pin or incompressibility constraint is introduced.

Initialize nodal velocity by

\[
 v_{a,1}=64U X_{a,1}(1-X_{a,1})X_{a,2}(1-X_{a,2})X_{a,3}(1-X_{a,3}),
 \qquad v_{a,2}=v_{a,3}=0,
\]

with input \(|U|\le1/16\), default \(U=1/16\). It vanishes at all boundary nodes and satisfies either selected essential condition. It is interpolated by the P1 basis; no claim that this coarse piecewise-linear field is globally smooth is made. U=0 supplies the exact rest checks below. Use fixed \(h=1/128\), eight steps to \(T=1/16\); no step doubling, rejection retry or adaptive timestep is hidden in this minimal path. A failed attempted step terminates the run with the previous accepted state intact.

These are declared teaching cases and algorithms, not calibrated materials, a general input-file format, a guarantee that every requested step converges, or production admission thresholds. Fluid G approximates a smooth isentropic/barotropic model; it supplies no shock-admissibility or shock-detection guarantee. Forcing, arbitrary geometry, thermal entropy evolution and the mixed incompressible model are explicit extensions rather than unspecified switches.

## Exact mesh and element choices

Input mesh resolution is integer \(n\in\{2,4\}\), default2. Nodes have coordinates \((i/n,j/n,k/n)\), \(0\le i,j,k\le n\), and global ID \(i+(n+1)[j+(n+1)k]\). Assign cube ID i+n*(j+n*k), then element ID6*cube_id+permutation_rank, with ranks0–5 for the lexicographic permutations below. For every small cube with lower index \(c=(i,j,k)\), enumerate the six permutations \(\pi\) of axes in lexicographic order. Form a tetrahedron with lattice vertices

\[
 c,\quad c+e_{\pi_1},\quad c+e_{\pi_1}+e_{\pi_2},\quad c+(1,1,1).
\]

If this construction's reference determinant is negative, swap its second and third local vertices once to select positive **reference** orientation. This is a mesh-construction rule, never a repair of an inverted deformed element. Perform the reference determinant/orientation tests on integer lattice edge matrices B, whose determinant is plus or minus one. After the swap require det(B)=1. The selected n values have exactly represented binary64 coordinates and physical edge matrices B/n. Compute each physical reference determinant and inverse from these edge matrices, cache V0=det(B/n)/6 and gradients (B/n)^(-T) grad_xi N, and require the determinant to equal the exactly represented dyadic 1/n^3. The exact-real volume is 1/(6n^3); division by6 is rounded once in the stored V0, so the check on V0 compares with that same declared binary64 division. Do not replace a measure computed from actual reference geometry with a nominal measure to hide a mismatch. Neighboring cubes use the same axis convention, producing matching face triangulations. Boundary faces are those incident to one tetrahedron; orient their normals outward by testing against the opposite element vertex. Assert the expected node/tetrahedron counts \((n+1)^3\) and \(6n^3\), two incident elements on every other face, no duplicate tetrahedra, and reference volume \(1/(6n^3)\) for every element. All boundary triangles lie on one of the six labelled planes.

Use the P1 barycentric shapes of Chapter32 for **all three motion and velocity components**. Reference gradients \(G_{e,a}=\nabla_XN_a\) are constant. There are no edge nodes or pressure/energy basis DOFs. For each element store four node IDs, \(V_{0,e}\), gradients, and the material identity. With default n=2 there are27 nodes and48 tetrahedra. There are54 free scalar unknowns for S and27 for G. The largest permitted mesh has125 nodes/384 tetrahedra and at most300 free unknowns (S); do not infer a larger mesh from a successful allocation.

The integration choices are exact for this selected discrete field:

- **Mass:** use the analytic consistent scalar P1 element matrix \(M^s_{e,ab}=\rho_0V_{0,e}(1+\delta_{ab})/20\), copied independently to the three vector component blocks. Do not use one-point mass or lumping in these implicit paths.
- **Internal energy, force and tangent:** use one barycentric point \((1/4,1/4,1/4,1/4)\) with physical reference weight \(V_{0,e}\). F and homogeneous material coefficients are constant on an affine P1 tetrahedron, so this integrates W, P times a basis gradient and the tangent contraction exactly for the finite-element field, despite W being nonlinear in F.
- **External loads:** zero in S/G. No boundary traction quadrature is called. Normal/clamp reactions are recovered from the full residual, including consistent inertia.

Assembly is serial in increasing element ID, then local node/component order; DOFs use node ID then x,y,z. Floating-point summation order is thereby specified at the pseudocode level, without claiming bitwise portability of logarithm/power libraries. All accepted fields are computed from the same nodal motion. The fluid has one geometric density/pressure authority: \(J_e\) determines \(\rho_e,p_e,e_e\); none is independently advected or repaired.

## State, constraints and constitutive evaluation

Build a list of fixed scalar DOFs from the face conditions, sorted by global index; all prescribed displacement/velocity values are zero. R injects free coordinates into the full vector and fills fixed entries with zero. Shared edge/corner constraints are unioned, not counted twice for the same component. Cache full consistent M and reduced \(M_f=R^TMR\). Because fixed values are zero, this reduction introduces no missing prescribed acceleration/load term. Moving supports are outside the chosen path.

One accepted state owns \((d_n,v_n,t_n)\) and derived diagnostics. One candidate workspace owns trial d, velocity, geometry, W/P/tangent evaluation, residual, dense Jacobian and LU scratch. No history is committed during iterations. Both selected materials depend only on current F and immutable reference coefficients. Internal energies for G are derived from J, not hidden thermal state.

At each element,

\[
 F_e=I+\sum_ad_a\otimes G_{e,a},\quad J_e=\det F_e,
 \qquad f^{\mathrm{int}}_{e,a}=V_{0,e}P_eG_{e,a},
\]

\[
 K_{e,ai,bj}=V_{0,e}(G_{e,a})_A\mathcal A_{iAjB}(F_e)(G_{e,b})_B.
\]

Use exactly \(P_s,\mathcal A_s\) from Chapter31 for S and \(P_g,\mathcal A_g\) from Chapter34 for G. Each A is the derivative of its P, not an extra geometric stiffness added to it. Scatter-add all element contributions; never overwrite a shared-node contribution. Sum \(\Psi=\sum_eV_{0,e}W_e\). For G report total mass \(\sum_e\rho_eJ_eV_{0,e}=1\), local density/pressure extrema and positive specific internal energy. For S report W and elastic stress without interpreting solid density as an independent evolved variable.

## Admissibility and arithmetic policy

The **mathematical sufficient geometry condition** uses the following convex-body argument, specialized to P1: require \(\|\nabla_Xd\|_F\le\theta=1/4\) on every tetrahedron. Each gradient is constant, so there is no unsampled element interior. It implies local nonsingularity and global injectivity on the convex reference cube in exact real arithmetic, and holds along the straight segment between two admissible displacement fields. For G it also keeps the boundary on the same six planes and the material domain the same box, allowing local compression/expansion but no mass flux through a wall.

**Conditional proof.** Continuity and the piecewise gradient bound on a convex body give \(|d(X)-d(Y)|\le\theta|X-Y|\) by integrating along the straight segment. Consequently \(|\chi(X)-\chi(Y)|\ge(1-\theta)|X-Y|\), proving injectivity. Singular values of \(I+\nabla d\) are at least \(1-\theta>0\); the path \(I+s\nabla d\), s from0 to1, remains nonsingular and starts at identity, so its determinant remains positive. The same norm bound holds along convex interpolation between two admissible displacement fields. It covers the chosen Newton segments and straight accepted mesh path in exact real arithmetic. It rejects many valid large rotations/shears and is not extended to nonconvex bodies by assertion.

For G's same-box claim, write C for the closed cube and \(\Pi_C\) for componentwise projection onto C. Given interior y, the map \(X\mapsto\Pi_C(y-d(X))\) is a contraction with constant at most theta, so it has a unique fixed point in C. That point cannot lie on a coordinate face: there the corresponding d component is zero, while y's corresponding coordinate is strictly between0 and1. Thus projection is inactive and \(X+d(X)=y\). Boundary target points follow by continuity/compactness. Conversely the zero normal-face components and gradient bound keep each coordinate of chi inside C. Together with injectivity this proves onto the same cube under these exact-real assumptions; no computed collision certificate is inferred.

In a future binary64 implementation this is a **computed acceptance diagnostic**, not a rigorous outward-rounded geometry certificate. Evaluate all nine gradient entries, sum squares with finite checks, and require the computed sum \(\le1/16\). Independently require finite \(J>0\), finite inverse F, positive G density/pressure/energy and finite W/P/A. A finite geometry-domain violation (gradient norm above the bound or J<=0) marks a line-search trial INFEASIBLE and permits the next alpha. Any nonfinite arithmetic, division by zero, invalid EOS result or undefined log/power is terminal INVALID_ARITHMETIC for the attempted step; it is never hidden by further backtracking. An inadmissible initial/current accepted-iterate geometry terminates as INVALID_STATE. Do not take an absolute determinant, clamp a density or floor an internal energy. The exact-real injectivity proof must not be relabelled as verification of these rounded operations.

Specify binary64 round-to-nearest arithmetic for a prospective implementation, ordinary real log and positive-base power interfaces with failure checks, and no fast-math assumptions. A model implementation still needs environment/toolchain identification before execution evidence or bitwise-reproducibility claims. Check integer products before allocating. Reject construction beyond the declared n/count limits or if the requested raw numeric buffers exceed16 MiB; allocator/runtime overhead is separate and not measured here. Allocate dense M, reduced Jacobian, LU copy, element state and vectors once, and reuse them. No model/runtime download is part of construction.

## Backward Euler and the actual nonlinear equation

Eliminate endpoint velocity using \(d=d_n+hv\). With \(d_{\mathrm{pred}}=d_n+hv_n\), the full displacement residual and tangent are

\[
 r_{\mathrm{full}}(d)=M(d-d_{\mathrm{pred}})/h^2+f^{\mathrm{int}}(d),
 \quad K_{\mathrm{full}}=M/h^2+K_{\mathrm{int}}(d),
 \quad r=R^Tr_{\mathrm{full}},\quad K=R^TK_{\mathrm{full}}R.
\]

These formulas apply separately to S or G with its selected energy. They do not assert a universal energy inequality for either nonlinear W. At acceptance set \(v_{n+1}=(d-d_n)/h\) and \(a_{n+1}=(v_{n+1}-v_n)/h\), and recover the full support reaction as \(r_{\mathrm{full}}\) on fixed DOFs. A positive internal residual is on the left of momentum balance; the required applied support force equals this constrained residual, with opposite force on the support.

## Fully selected algebra and globalization

Use a **dense direct LU solver with partial pivoting**, not an unspecified Krylov method. Its left diagonal equilibration is \(D_{ii}=1/\max_j|K_{ij}|\). Form \(A=DK\), \(b=-Dr\), factor A using Gaussian elimination with largest-absolute remaining column pivot and smallest-row-index tie-break; apply the same row swaps to b, then forward/back substitution. An empty row, nonfinite arithmetic or pivot \(\le2^{-44}\max(1,\|A\|_\infty)\) marks the direct attempt unusable. This is a numerical pivot policy, not a proven condition estimate. There is no additional preconditioner, iterative refinement or Krylov iteration. Recompute the unscaled full free linear defect \(Ks+r\) and require

\[
 \|Ks+r\|_\infty\le2^{-36}(1+\|r\|_\infty).
\]

Do not call that recomputed defect an exact floating-point residual. It checks the represented operator through a new evaluation, subject to rounding.

Use merit \(m=\tfrac12r^Tr\), with r already in force-normalized units and free displacement coordinates normalized by L*. For a usable direct direction s, compute \(q=r^TKs\) and require finite q<0. If factorization/linear-defect/descent fails, use the explicitly selected fallback \(g=K^Tr\), \(s=-g/\max(1,\|g\|_\infty)\); if g is zero/nonfinite or q is not negative, reject the step as stagnated or invalid. This fallback is a descent direction for the same residual merit; it changes no constitutive equation. Without normalization, a raw K^T r would have the wrong units to be a displacement direction.

Scale the chosen direction by a positive factor so \(\|s\|_\infty\le1/16\), and recompute q after scaling. Try exactly the ordered candidates \(\alpha=2^{-j}\), \(j=0,\ldots,19\). For each, form \(d_{\mathrm{trial}}=R(z+\alpha s)\), check geometry/constitutive admissibility, evaluate its complete residual and merit, and accept the first satisfying

\[
 m(z+\alpha s)\le m(z)+2^{-10}\alpha q.
\]

Finite domain-infeasible trials do not evaluate log/power or merit after the failed geometry gate. Nonfinite geometry, constitutive, residual or merit arithmetic instead terminates the attempted step immediately. If none passes, reject; no twentieth halving outside this roster is added. At most32 nonlinear updates are allowed per step. Test convergence at the initial residual and after every accepted update, including after update32:

\[
 \|r\|_\infty\le2^{-32}
 \left(1+\|R^TM(d-d_{\mathrm{pred}})/h^2\|_\infty+
             \|R^Tf^{\mathrm{int}}\|_\infty\right).
\]

The constant1 is the declared force scale, not a dimensionless replacement for an unknown physical load. If residual convergence fails and an accepted update has norm \(\le2^{-40}(1+\|z\|_\infty)\), reject as stagnation. Small displacement alone is never convergence. These fixed values make the reference algorithm concrete; they are not experimentally tuned or certified tolerances and do not alter Rheon thresholds.

## Complete construction, solve and publication pseudocode

~~~text
construct(kind in {S,G}, n in {2,4}, |U|<=1/16):
    create the specified structured cube nodes and six-permutation tetrahedra
    validate oriented reference geometry, counts, incidence and boundary planes
    build P1 gradients/volumes; assemble analytic consistent full M
    build exact fixed/free scalar maps from S clamp or G normal-only walls
    cache M_free; allocate all bounded numeric scratch before stepping
    accepted = {d=0, v=nodal_bubble(U), t=0}
    verify fixed velocity components zero and initial geometry/model finite
    return immutable reference/material/constraints + accepted + scratch

assemble(d, need_tangent):
    require fixed d components zero
    fint=0; Kint=0 if requested; Psi=0
    for e in increasing element ID:
        grad_d=sum_a outer(d[node[a]], G[e,a]); F=I+grad_d
        require finite grad_d and finite squared_Frobenius(grad_d), else INVALID_ARITHMETIC
        if squared_Frobenius(grad_d)>1/16: return INFEASIBLE
        J=det(F); require finite J, else INVALID_ARITHMETIC
        if J<=0: return INFEASIBLE
        require finite inverse(F), else INVALID_ARITHMETIC
        (W,P,A)=selected_S_or_G_constitutive(F)  # formulas in ch31/ch34
        require finite W,P,A and finite positive G rho,p,e, else INVALID_ARITHMETIC
        Psi += V0[e]*W
        for local a,i: fint[node[a],i] += V0[e]*(P*G[e,a])[i]
        if need_tangent:
            for local a,i,b,j:
                Kint[ai,bj] += V0[e]*G[a]_A*A[i,A,j,B]*G[b]_B
    return fint, Kint_if_requested, Psi, element_diagnostics

attempt_step(accepted):
    h=1/128; require finite advancing t_new=t_n+h
    d_pred=d_n+h*v_n; z=free_coordinates(d_n)  # feasible initial guess
    converged=false
    for updates in 0..32 inclusive:
        d=inject_free(z); (fint,Kint,Psi,diag)=assemble(d,true)
        if INFEASIBLE: reject INVALID_STATE; any arithmetic error: reject INVALID_ARITHMETIC
        inertia=M*(d-d_pred)/h^2; r_full=inertia+fint
        r=restrict_free(r_full); K=restrict_free(M/h^2+Kint)
        require every evaluated scalar finite; honor cancellation
        if the specified nonlinear residual gate passes:
            converged=true; break
        if updates==32: reject NONLINEAR_LIMIT
        s=equilibrated_partial_pivot_LU(K,-r) or unusable
        q=r^T*K*s if s usable, otherwise unusable
        if s unusable or recomputed_linear_gate_nonfinite_or_fails or q_nonfinite_or_nonnegative:
            g=transpose(K)*r
            require finite nonzero g; s=-g/max(1,norm_inf(g))
        s *= min(1,(1/16)/norm_inf(s)); q=r^T*K*s
        require finite q<0
        accepted_trial=false
        for j in 0..19:
            alpha=2^(-j); trial=inject_free(z+alpha*s)
            evaluate trial geometry; if finite domain INFEASIBLE: continue to next alpha
            evaluate selected constitutive model and whole residual from accepted d_n,v_n
            any nonfinite/model/residual/merit arithmetic: reject INVALID_ARITHMETIC
            if merit obeys Armijo inequality:
                delta=alpha*s; z+=delta; accepted_trial=true; break
        if no accepted_trial: reject LINE_SEARCH_LIMIT
        if norm_inf(delta)<=2^-40*(1+norm_inf(z)):
            recompute trial residual gate; if it fails: reject STAGNATION
    require converged
    recompute final full residual/model diagnostics independently of cached flags
    require the same convergence/geometry/finite gates still pass
    v_new=(d-d_n)/h; a_new=(v_new-v_n)/h
    compute energy, mass/volume, wall/clamp reactions and balance ledger below
    require finite derived outputs and exact fixed components in represented state
    final cancellation barrier
    publish one state {d,v_new,t_new,diagnostics}; never publish candidate scratch

run:
    construct once; for ordinal in 1..8: attempt_step or stop with typed failure
    return exactly the accepted prefix and the terminal outcome
~~~

Each residual call inside line search uses the **same accepted** d_n,v_n and h, not the previous rejected trial. An unusable LU attempt or nonfinite direct-direction/linear-defect/descent computation may trigger the specified gradient fallback only when r,K remain finite and valid. This expressly limited algebra-solver recovery differs from invalid geometry/model/residual/merit arithmetic, which terminates the attempt. Nonfinite fallback or scaled-direction arithmetic also terminates; finite domain violations alone skip a line-search candidate. Cancellation at construction, assembly, linear solve, line search and final barrier returns a distinct cancelled outcome with no new accepted clock/state. There is no checkpoint/restart, recorder, GUI or hardware implementation claim.

Report nonnegative geometry metrics, mass, kinetic/internal energies and the measured discrete work defect, but do not require a fabricated energy theorem. Let \(r_f\) be the final recomputed full residual and \(\Delta d=d_{n+1}-d_n\). Define \(T=\tfrac12v^TMv\) and \(E=T+\Psi\). Because fixed displacements do no work,

\[
 E_{n+1}-E_n+\tfrac12\|v_{n+1}-v_n\|_M^2
 +\left[f^{\mathrm{int}}(d_{n+1})^T\Delta d-(\Psi_{n+1}-\Psi_n)\right]
 =r_f^T\Delta d.
\]

This exact-arithmetic **identity** follows by dotting backward-Euler momentum with Δd. The bracket has no universally claimed sign. A future floating-point run reports each term and its discrepancy; this exposition supplies no measured work closure or acceptance certificate. Similarly report \(M(v_{n+1}-v_n)/h+f^{\mathrm{int}}\) and support reactions rather than claiming whole-body momentum conservation with fixed walls.

## Worked analytic checks for the selected paths

**Reference mass and initial kinetic energy.** At default n=2 each tetrahedron has volume1/48 and mass1/48 kg. Its scalar consistent mass diagonal is1/480 kg, off-diagonal1/960 kg. The centre node belongs to24 tetrahedra: six in each of the two cells where it is the long-diagonal endpoint, and two in each of the other six incident cells. Thus its assembled scalar diagonal is1/20 kg. It is the only node with nonzero bubble velocity, U=1/16 m/s, so initial kinetic energy is \(\tfrac12(1/20)(1/16)^2=1/10240\) J. This is an exact combinatorial/element integral calculation, not a mesh execution. Whole physical mass is1 kg; component blocks are repeated mass metrics, not three kilograms.

**Exact rest.** With U=0, S has F=I, W_s=P_s=0 and zero residual. G has F=I, rho=p=1 and e=W_g=3/2, so unit-box internal energy is3/2 J. Its full pressure residual is nonzero on constrained normal DOFs but zero on free DOFs; the continuum integrated wall reaction on fluid is −p n per square metre, one newton inward on each face. Opposite fluid load on the wall is outward. Fixed wall motion does zero work. These states satisfy every backward-Euler step algebraically; no numerical run is claimed.

**Solid element shear.** With \(F=I+k e_1\otimes e_2\), \(k=1/8\), J=1, the selected S law gives \(W_s=k^2/2=1/128\) Pa and \(P_{12}=P_{21}=k\), other components zero. Its gradient norm1/8 is below theta. On an element, substitute those entries and the stored constant gradients into \(V_0PG_a\). This is a constitutive/assembly check, not a whole-box field satisfying the x=0 clamp, and must not be labelled an accepted global shear trajectory.

**Fluid compression and linear response.** The exact isentropic compression values and breathing tangent in Chapter34 check pressure sign, J powers and A_g. At initial rest the first Newton matrix is \(M_f/h^2+K_{g,0}\), with \(K_{g,0}[w,z]=\int(5/3)(\nabla\cdot w)(\nabla\cdot z)\,dV_0\) after assembled fixed-wall geometric cancellation. This is symmetric positive definite because M_f is positive definite, even though the gas has no shear stiffness. It does not prove later nonlinear matrices are positive definite; LU/fallback/failure rules remain necessary.

For a continuum **linearized** acoustic check in the same impermeable box, take \(k=\pi\), \(c_0=\sqrt{5/3}\), \(\omega=c_0k\), and small amplitude A:

\[
 v_1=A\sin(kX_1)\cos(\omega t),\quad v_2=v_3=0,
 \qquad \rho'=-\frac{\rho_0Ak}{\omega}\cos(kX_1)\sin(\omega t),
 \quad p'=c_0^2\rho'.
\]

Direct differentiation verifies \(\rho'_t+\rho_0\partial_1v_1=0\) and \(\rho_0\partial_tv_1+\partial_1p'=0\). It satisfies wall normal velocity conditions. This is a small-amplitude continuum target, not an exact finite-amplitude solution, not the default bubble initial data, and not an executed spatial/time convergence study. It supports a future separately admitted refinement check without conflating analytic and numerical evidence.


## Open-source architectures worth studying

The [source map](../lagrangian/source-map.md) pins versions, source paths and license evidence. The comparisons below are source-guided architectural observations, not comparative benchmarks or adoption decisions.

| Project and inspected version | Common architecture and lesson | Tradeoff/domain boundary |
| --- | --- | --- |
| [MFEM v4.7 ex10](https://github.com/mfem/mfem/blob/v4.7/examples/ex10.cpp) | FE forms, mass and hyperelastic operators feed a separate ODE solver; reduced implicit nonlinear operator | Compact mechanics example; its reference diffusion damping is not a full moving-fluid viscous stress |
| [DOLFINx v0.9.0 hyperelasticity](https://github.com/FEniCS/dolfinx/blob/v0.9.0/cpp/demo/hyperelasticity/hyperelasticity.py) | Symbolic energy/form differentiation generates residual and Jacobian; assembly, boundary constraints and PETSc own numerical execution | Reduces hand-derivative maintenance; generated derivatives still depend on correct form, domains and quadrature |
| [deal.II v9.6.2 step44](https://github.com/dealii/dealii/blob/v9.6.2/examples/step-44/step-44.cc) | Explicit finite-strain material/quad-point state, mixed blocks, optional static condensation | Makes material and algebra inspectable; three-field conventions/spaces must remain consistent |
| [Laghos v3.1](https://github.com/CEED/Laghos/blob/v3.1/laghos_solver.hpp) | Continuous kinematic and discontinuous thermodynamic spaces, shared force operator, full/partial assembly | Compressible Euler miniapp; high-order tensor-product action can reduce global matrix traffic, with different preconditioning/work costs |
| [Kratos PFEM pinned 9452ba2](https://github.com/KratosMultiphysics/Kratos/blob/9452ba205627aa4a182368b2fbcbc41bb68b62ab/applications/PfemFluidDynamicsApplication/python_scripts/pfem_fluid_solver.py) | Python orchestration, ModelParts and separate pressure/velocity strategies with moving-mesh settings | Production application infrastructure is larger than an element loop; remeshing/strategy/stabilization details are formulation-specific |
| [FEBio v4.7 solid domain](https://github.com/febiosoftware/FEBio/blob/v4.7/FEBioMech/FEElasticSolidDomain.cpp) | Elements/material points and domain-level internal-force/stiffness assembly separated from analysis/solver state | Constitutive history and mechanics domain are first-class; broader mechanics coverage is not proof of this reference fluid branch |

Full assembly stores global sparse operators and enables established factorizations/AMG. Partial assembly stores quadrature/operator data and applies local basis maps; high-order tensor products can reduce work/storage. Neither label alone determines performance. MPI ghost accumulation must gather trial values before element evaluation and sum shared residual contributions afterwards; matrix-free and assembled paths must implement the same constrained action. Quadrature histories need accepted/candidate semantics even when a framework offers convenient mutable material objects.

A future Rheon implementation could adopt an existing library or implement a narrow operator set. That decision needs dependency/license/build/resource review and actual equation fit. The book imports no third-party code and does not prescribe a Rust wrapper or framework registry.

## Verification ladder and meaningful failures

A complete minimal algorithm is not completed scientific qualification. The selected S/G paths need their own future checks; Stokes, mixed-pressure, viscous, capillary, contact and FSI items below belong to the explicitly separate advanced extensions. Establish evidence in increasing dependency order, with independent oracles and explicit bounds:

1. Check basis partition/linear reproduction, affine maps, mass entries, traction signs and objective rigid motions against Chapter32's exact hand calculations. Test deliberately reversed orientation and zero-volume input.
2. Check material stress/energy/tangent directional consistency and assembled residual derivative using independent symbolic or finite-difference references over admitted positive-J states. A finite-difference comparison needs a scale sweep to expose truncation/cancellation; it is numerical evidence, not proof.
3. Check constrained free/support force balance, internal resultant/torque and fixed-topology mass. Reject perturbed load area, omitted tangent block, pressure nullspace and swapped coordinate gradients at the intended boundary.
4. Check linear axial/elastic waves and manufactured smooth solutions with independently derived source/boundary data. Separate spatial refinement, timestep refinement and quadrature refinement so one error does not mask another. Report L2/H1 displacement or velocity, pressure modulo only a legitimate gauge, stress/traction, volume and work errors.
5. For fluid, include fixed-domain Stokes, rigid translation/rotation, hydrostatics under correctly signed boundary traction, viscous decay, free-surface motion and the endpoint-divergence counterexample. Measure total and local J/density errors, actual mixed conditioning, pressure/traction convergence and artificial work.
6. Qualify contact, remap and FSI only after their own equations are implemented: gap/multiplier complementarity, frictional loss, continuous path feasibility, overlap balances, rigid-motion transfer, interface work, added-mass sensitivity and rollback. Breakup/merging need new domain-volume/transfer evidence.
7. Test rejection/cancellation at nonzero accepted state, iteration/memory limits, restart identity if supported and stale-output protection. Native/cloud/GPU timings or accuracy require runs in each claimed environment with exact source/input and resource bounds.

For a smooth stable conforming elliptic problem with coercivity constant \(\alpha\), continuity constant \(C\), exact integration and Galerkin orthogonality, Céa's argument yields \(\|u-u_h\|_V\le(C/\alpha)\inf_{w_h}\|u-w_h\|_V\). Proof: test the error relation against \(u_h-w_h\), use coercivity and continuity, then divide by the error norm (zero error is immediate). Interpolation estimates on shape-regular meshes can give order-k energy errors and, with extra regularity/duality assumptions, higher L2 orders. This conditional **linear** result is not a convergence proof for finite-strain contact, moving mixed free surfaces, mesh degeneration or the proposed nonlinear timestep.

A future bounded qualification should start with one owned small mesh, one exact case, explicit maximum DOFs/steps/iterations, wall/CPU/memory/output limits and an independently reviewed oracle. Broader parameter sweeps, adversarial corrupted outputs and native performance follow only through a separate admission. The [review and qualification guide](../lagrangian/review-guide.md) names those pending needs. No existing numerical threshold or proof receipt is broadened by these chapters.
