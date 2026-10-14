# 27 Conservative compressible discretization

The numerical method must carry the state and thermodynamics of Chapter 26 through every face, time stage and boundary. A low-memory implementation can store five conservative scalars per fluid cell and compute one shared face flux at a time. This reduces temporary storage; it does not remove the need for reliable reconstruction, thermodynamic validation and conservative parallel assembly.

## Start from a finite-volume balance

For a stationary cell \(K_i\) with fluid volume \(V_i\), cell average \(\bar U_i\), outward normals \(\mathbf n_{if}\) and open face areas \(A_f\), a forward-Euler inviscid step is

\[
V_i\bar U_i^{n+1}=V_i\bar U_i^n
-\Delta t\sum_{f\subset\partial K_i}A_f\widehat F_{if}.
\]

The same interior flux must enter the neighbour with opposite orientation. It follows by cancellation that the sum of cell contents changes only by boundary fluxes and declared sources. Summing cell averages without volume weights is the wrong conservation audit when cells differ in size. A consistent face solver also satisfies

\[
\widehat F(U,U;\mathbf n)=F_n(U),\qquad
\widehat F(U_R,U_L;-\mathbf n)=-\widehat F(U_L,U_R;\mathbf n).
\]

These are finite algebraic identities. They leave open admissibility, entropy, accuracy, solid geometry and floating-point summation error. Conservation of a computed total cannot establish correct shock speed if the EOS or physical flux is wrong. The positivity analysis of Johnson and Rossmanith starts from this local conservation structure and adds separate conditions on reconstruction and time step. [Johnson and Rossmanith, §§1–2](https://arxiv.org/pdf/1212.4695)

## Local Lax–Friedrichs gives a transparent baseline

For admissible left and right states, the LLF or Rusanov normal flux is

\[
\widehat F_{LLF}=\tfrac12\{F_n(U_L)+F_n(U_R)\}
-\tfrac12\alpha_f(U_R-U_L),
\]

where a customary ideal-gas interface estimate is

\[
\alpha_f=\max(|u_{n,L}|+c_L,\ |u_{n,R}|+c_R).
\]

For a claimed invariant-domain theorem, the speed estimate must satisfy that theorem's required bound on the relevant wave fan, with its specified CFL and admissible states. Endpoint estimates and the label “LLF” alone are not a proof for every EOS or reconstruction. The dissipation term damps all conservative jumps, including contacts and tangential shear. That makes first-order LLF useful for debugging a small teaching fixture, while its contact broadening and acoustic damping are visible limitations. Degond and Tang formulate a first-order LLF-based all-speed method; their semi-implicit treatment is an additional method, not a property of the explicit flux written here. [Degond and Tang](https://arxiv.org/abs/0908.1929)

A dimensional check helps implementation: \(\alpha_f\) has units m/s, so both \(F_n\) and \(\alpha_f U\) have conservative-content flux units. On a uniform one-dimensional grid, each flux difference divides by \(\Delta x\), not \(\Delta x^2\). A three-dimensional fixture must update all three momentum components even when its first verification case has one-dimensional variation.

## HLL separates the outer signals; HLLC restores a contact

Estimate bounding wave speeds \(S_L<S_R\). The HLL flux is

\[
\widehat F_{HLL}=\begin{cases}
F_L,&0\le S_L,\\
\dfrac{S_RF_L-S_LF_R+S_LS_R(U_R-U_L)}{S_R-S_L},
&S_L<0<S_R,\\
F_R,&S_R\le0.
\end{cases}
\]

Here \(F_L=F_n(U_L)\), \(F_R=F_n(U_R)\). The middle formula follows by integrating conservation over a two-wave fan. With \(S_L=-\alpha_f\), \(S_R=\alpha_f\), it reduces to LLF. HLLE denotes HLL with Einfeldt-type robust speed estimates. Its single middle state diffuses the contact and shear fields; suitable wave bounds make it an important admissible fallback for an ideal-gas method under the corresponding time-step assumptions. [HLL/HLLE construction, §4.1](https://arxiv.org/html/1803.04954)

HLLC introduces a contact speed \(S_M\) and two star states. For each side \(k=L,R\), impose the jump relations and common star pressure:

\[
p_* =p_k+\rho_k(S_k-u_{n,k})(S_M-u_{n,k}),\qquad
\rho_{*,k}=\rho_k\frac{S_k-u_{n,k}}{S_k-S_M}.
\]

The star normal velocity is \(S_M\), and tangential velocity is transported consistently. In the left-star branch, for example, use \(F_L+S_L(U_{*,L}-U_L)\). This structure resolves a contact more sharply, but small denominators, a non-admissible star state and multidimensional shock instabilities need explicit treatment. Sangeeth and Mandal analyse shock instability and develop antidiffusion control. Their strong-shock study demonstrates why passing Sod alone cannot qualify a contact-resolving flux for arbitrary multidimensional shocks. This book incorporates none of their source code or figures. [HLLC and shock-instability analysis, §§4–7](https://arxiv.org/html/1803.04954)

Record how speed estimates, degeneracies and fallback choices are handled. A fallback flux must remain shared by both adjacent cells; independently selecting a fallback on each side breaks flux cancellation. Reporting how often a fallback activates helps distinguish a robust calculation from one that spends most of its run at first order.

## Roe's resolution requires additional safeguards

Roe's construction replaces the nonlinear fan with waves from a linearized matrix \(\widetilde A\) satisfying

\[
\widetilde A(U_R-U_L)=F_R-F_L,
\qquad
\widehat F_{Roe}=\tfrac12(F_L+F_R)
-\tfrac12|\widetilde A|(U_R-U_L).
\]

The flux-difference property gives good resolution of an isolated shock. It does not preserve positive density or internal energy for all rarefactions. An unmodified linearized fan can also admit an expansion shock near a transonic rarefaction; an entropy fix changes the offending wave dissipation. A positivity fallback addresses a different failure and does not replace the entropy fix. The authors' Riemann notebook exhibits both failures and contrasts Roe with HLLE; it is a mathematical reference, not an imported dependency. [Roe rarefactions and positivity examples](https://www.clawpack.org/riemann_book/html/Euler_approximate.html)

The design choice should follow the required regime. For the first independently written Euler reference, LLF has the smallest explanatory surface. HLL/HLLE offers a useful next comparison. HLLC and Roe need stronger rarefaction, strong-shock and low-Mach evidence before they become defaults. A solver name is not a complete algorithm specification.

## Reconstruction, limiting and time integration

Piecewise-constant cell states give first-order spatial accuracy. A linear reconstruction can approximate smooth face states more accurately. On a uniform one-dimensional grid a simple limited slope is

\[
\delta W_i=\operatorname{minmod}(W_i-W_{i-1},\ W_{i+1}-W_i),
\quad W_{i\pm1/2}=W_i\pm\tfrac12\delta W_i.
\]

Minmod is zero when the two arguments have opposite signs, and otherwise takes the common sign and smaller magnitude. Specify whether \(W\) contains conservative, primitive or characteristic variables. Reconstructing primitive variables is convenient; the conservative cell average remains the evolved content. Nonlinear conversion means a primitive reconstruction does not automatically define a conservative polynomial with that average. A multidimensional cut-cell method additionally needs a geometry-aware gradient, stencil and limiter.

Limiting suppresses oscillations near discontinuities, usually sacrificing local order near extrema or shocks. A componentwise scalar TVD argument is not a universal Euler-system positivity or entropy proof. WENO and other high-order reconstructions require a separate smoothness selection, quadrature and admissibility analysis; increasing polynomial order does not remove these obligations.

For a conservative reconstruction \(U_h\) whose cell average is \(\bar U\), a positivity limiter may use

\[
U_h^\theta(x)=\bar U+\theta[U_h(x)-\bar U],\qquad0\le\theta\le1.
\]

Choose \(\theta\) to maintain density and internal-energy bounds at the required flux and quadrature points. This scaling preserves the cell average; it cannot repair an already inadmissible average. The flux, quadrature, CFL restriction and every time stage must satisfy the selected positivity argument. Zhang and Shu's original Euler paper is identified in Shu's publication list; Johnson and Rossmanith give an inspected primary derivation of the wider convex-state framework. Their theorem does not automatically cover newly invented cut-cell or moving-wall quadrature. [Original Euler positivity publication](https://www.dam.brown.edu/people/shu/pub.html), [inspected positivity framework](https://arxiv.org/pdf/1212.4695)

Use a time integrator with a declared spatial operator and order. Strong-stability-preserving Runge–Kutta stages can inherit a forward-Euler admissibility property only under their corresponding stage restrictions. Reducing \(\Delta t\) and retrying from the previous coherent state is preferable to accepting an invalid state. Blindly flooring density or pressure changes conservative content and may create a hidden energy source. Publish any deliberate nonconservative repair and its cumulative budgets; do not relabel it as a conservative positivity guarantee.

## Positivity and entropy are different contracts

Positivity keeps the thermodynamic state in its valid domain. Entropy stability controls a specified discrete entropy inequality, including boundary fluxes and source contributions. A positive state can still contain an inadmissible expansion shock. A method with an entropy inequality may still need a positivity limiter to evaluate its logarithms and EOS. Conservation alone proves neither property. Curvilinear entropy-stable methods also need compatible metric identities; Nolasco and colleagues give an original example of that coupling. [Entropy and geometry construction, §§3–4](https://arxiv.org/html/1911.03682)

An entropy diagnostic should state whether it is cellwise or global, semi-discrete or fully discrete, and which entropy flux is used. A global entropy rise in an insulated shock tube is a useful observation, but it can hide local violations and is not a theorem for a reconstructed three-dimensional scheme.

## CFL, diffusion and the low-Mach limit

A useful local advective-acoustic estimate for stationary cells is

\[
\Delta t\le C\min_i\frac{V_i}{\sum_f A_f\alpha_{if}},
\qquad \alpha_{if}\sim|u_{n,if}|+c_{if}.
\]

The allowed \(C\) depends on the actual flux, quadrature, dimension and time integrator. It is an engineering estimate until linked to a specific stability argument. Report the controlling cell and mechanism. Explicit stress and conduction bring further restrictions of order \(h^2/\nu\) and \(h^2/\chi\), where \(\nu=\mu/\rho\) and \(\chi=k/(\rho c_p)\); compressible stress has additional bulk/longitudinal coefficients, so a scalar estimate is not a complete viscous bound. Use the minimum of the selected restrictions and hit the requested final time with the actual accepted final step.

At Mach \(M=U/c\ll1\), acoustic time steps are roughly \(M\) times the flow-advection scale. Sound-speed-scaled LLF diffusion can overwhelm slow vortical motion, while physical pressure variations can be small differences on a large thermodynamic background. High-speed total-energy subtraction presents a different conditioning problem: kinetic energy can dominate internal energy. Dual-energy policies and alternative thermodynamic variables must document which equation is authoritative and how their conservation error is measured.

Low-Mach remedies include preconditioning for steady or pseudo-time iteration, and carefully designed implicit-explicit or asymptotic-preserving time discretization for physical unsteady motion. Simply replacing \(c\) in a physical explicit flux changes acoustic dynamics and its stability argument. Noelle and colleagues demonstrate that a Mach-independent CFL and formal asymptotic consistency still leave an asymptotic stability/accuracy problem in their method. This is a concrete reason to test the Mach limit independently. [Noelle et al., §§1 and 3–6](https://arxiv.org/html/1412.1606)

The initial qualification should retain the physical sound speed, expose its accepted CFL and show its dissipation on an acoustic pulse and a smooth vortex. A broader all-speed claim requires a Mach sweep and a matching error budget. Geometry and boundaries enter the same conservative balance in the next chapter.

The accompanying [Lean source](../../../proofs/Rheon/Compressible.lean) makes three narrow contracts explicit. `shared_face_conservation` proves cancellation for balanced incidence and one shared extensive flux. `admissible_blend` proves convexity of the ideal-gas admissible set over exact reals. `cfl_density_positive` proves strict positivity for its stated donor-density combination with nonnegative coefficients and strict CFL; it is **not** a positivity proof for the Rusanov Euler update. The source assumptions and current proof-qualification records govern these claims. They establish no entropy inequality, arbitrary geometry assembly, floating-point accuracy or production solver.
