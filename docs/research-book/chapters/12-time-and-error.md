# 12 Time stepping and error budgets

Physical time and display frames are separate clocks. A frame can contain substeps; a slow frame must not silently enlarge the physical step beyond its accepted regime. Decide whether overload slows simulation, drops presentation frames or changes a declared quality tier. Record the policy in replay metadata.

## Restrictions

Directional Courant numbers are \(c_x=|u|\Delta t/h_x\) and analogues. Multidimensional positivity depends on the update and can involve their sum. Characteristic transport may remain bounded without that restriction, but travel limits still help trajectory accuracy, collision detection and narrow-band validity.

Explicit diffusion depends on \(\nu\Delta t\sum_dh_d^{-2}\). Surface tension introduces a capillary scale proportional to \(\sqrt{\rho h^3/\sigma}\), with method-dependent constant. Acceleration and moving walls add constraints. Return a step and its limiting reason, not an unexplained number. If required substeps exceed a cap, return a typed outcome rather than label a partial interval complete.

## Error accounting

A conceptual budget is

\[
E_{\mathrm{total}}\lesssim E_{\mathrm{model}}+E_{\mathrm{space}}
+E_{\mathrm{time}}+E_{\mathrm{solve}}+E_{\mathrm{round}}
+E_{\mathrm{surface}}+E_{\mathrm{render}}.
\]

This is an accounting guide, not a universal inequality across incompatible norms. Residual controls one constraint error, not advection displacement or mesh extraction. Machine-precision pressure with a coarse staircase obstacle may overspend on a subdominant error.

Refine space at fixed Courant number for smooth transport. Refine time with sufficiently small spatial error for temporal order. Simultaneously changing both mixes effects. For interfaces measure volume, surface distance and silhouette displacement; small average error can hide catastrophic filament loss.

## Manufactured solutions

A periodic stream function \(\psi=\sin x\sin y\) gives velocity \((\partial_y\psi,-\partial_x\psi)\) with zero continuous divergence. Sampling it does not automatically give zero discrete divergence; a discrete curl construction can ensure cancellation. Distinguish those tests.

Choosing discrete pressure \(p\), computing \(b=Ap\) and solving tests the solver for that operator, not the operator's physical accuracy. Add independent analytic and stencil checks. Expected values should not reuse the same indexing routine whose correctness is in question.

## Long horizons and outcomes

Track volume or tracer amount, energy, divergence, surface position and rejection count after sources stop. Include rest, translation, rotation and eventually liquid and obstacle cases. Sensitive flows may not match pointwise indefinitely; use bounded time windows and defined geometric or statistical observables. Exact replay remains a separate implementation property.

Outcomes should distinguish converged, iteration limit, invalid input, nonfinite state, incompatible flux and resource limit. A nonconverged preview may be permitted explicitly but must not masquerade as accepted scientific state. Candidate and accepted buffers need clear ownership so cancellation cannot corrupt future steps.

## One mode, three time updates

A clean test removes spatial error before asking about timestep error. In a
periodic unit-metre cube with density 1 kg/m³, prescribe

\[
\mathbf u(x,y,z,t)=(A(t)\sin(2\pi y),0,0),\qquad
\nu=\frac{1}{4\pi^2}\ \mathrm{m^2/s}.
\]

There is no x dependence, so divergence and self-advection vanish. With no
forcing and constant pressure, viscosity gives \(A'=-\lambda A\), with
\(\lambda=\nu(2\pi)^2=1\ \mathrm{s^{-1}}\). Initially A = 1 m/s.
The exact amplitude is \(e^{-t}\) m/s and the unit-cube kinetic energy is
\(K=A^2/4\) J: the integral of the squared sine over one period is one half.
The cube is periodic; its edges in the picture are not stationary walls.

For \(a=\lambda\Delta t\), explicit Euler multiplies amplitude by \(1-a\),
whereas implicit Euler multiplies it by \(1/(1+a)\). The browser actually repeats
these scalar updates. It samples their resulting three-dimensional velocity
field at fixed locations, without solving a spatial stencil or pressure system.
The field varies only in y; repeated x/z samples show that symmetry. All arrows
share a fixed scale, and dark arrows show exact decay at the same positions.
The energy readout uses the integral above, not the number of visible arrows.

Start with target time **2 s**, **one step**, **Explicit Euler**. Predict the
sign before inspecting the arrows: A becomes −1 m/s while K remains 0.25 J.
The exact flow still points in its original direction, with A ≈ 0.135335 m/s
and K ≈ 0.00457891 J. Energy nonincrease alone did not detect the reversed flow.
The explicit factor is nonnegative for \(0\le a\le1\), reverses sign for
\(a>1\), and has magnitude at most one only for \(0\le a\le2\).
At a = 2 there is no decay at all, despite the energy bound.

## Implicit decay and fixed-time refinement

Now choose **Implicit Euler** without changing time or step count. It yields
A = 1/3 m/s and K = 1/36 J, neither the explicit result nor the exact answer.
Its energy decays for every nonnegative a, but a single large step still has
substantial temporal error. Finally keep target time fixed at 2 s and change
step count to 2, 4, 8 and 16. Compare both amplitude-error columns at the same
physical time. Changing final time while refining would answer a different
question. **Displayed step** inspects intermediate states; it does not animate
or advance a native simulation.

## An energy-growing step

The 3 s, one-step explicit preset gives A = −2 m/s and K = 1 J: viscosity has
numerically increased energy above its initial 0.25 J. That is a rejected method
choice for this mode, not a physical instability of the specified unforced flow.
The lesson's controls bound amplitudes to ±2 m/s, so arrows need no hidden
normalization or clipping. Choosing Exact changes only which field is displayed.

## Research connection and what the test leaves out

[Stam's original *Stable Fluids* paper](https://www.dgp.toronto.edu/people/stam/reality/Research/pdf/ns.pdf)
(1999), section 2.2, replaces an explicit viscous update with an implicit
diffusion solve; section 2.3 explains the simpler Fourier-space form for a
periodic domain. Its introduction also separates graphical utility from
engineering accuracy. Those observations motivate inspecting amplification
rather than treating “stable” as an accuracy certificate. The scalar comparison
here isolates the time update; it does not reproduce the paper's whole split
algorithm or its total dissipation. In particular, this single mode's implicit
amplitude at coarse timesteps exceeds the exact amplitude.

The derivation above assumes constant coefficients, a periodic smooth shear,
no force and no changing geometry. It contains no spatial discretization error;
the display samples are not a resolution control. A general diffusion grid has
many eigenmodes, and its most restrictive mode determines an explicit timestep
bound. Advection, moving boundaries, capillarity, nonlinear materials, approximate
linear solves and finite arithmetic add separate error and acceptance conditions.
Neither this lesson nor a passing amplitude comparison establishes a stable
whole Rheon step, physical calibration, or runtime performance.

## Checked modal amplification proof bridge

[ModalDecay.lean](../../../proofs/Rheon/ModalDecay.lean) proves the exact-real
explicit energy-change identity \(K(A^+)-K(A)=a(a-2)K(A)\) and derives energy
nonincrease for \(0\le a\le2\). Sign preservation has its separate hypothesis
\(a\le1\) with nonnegative incoming amplitude; strictly positive amplitude
reverses for \(a>1\). The a = 2 witness formally preserves energy while reversing
unit amplitude.

For the implicit relation \((1+a)A^+=A\), the file proves
\(K(A^+)-K(A)=-a(2+a)K(A^+)\) and energy nonincrease for nonnegative a.
It also checks that division by \(1+a\) satisfies that update under this
hypothesis. These are scalar amplification theorems. The PDE ansatz, sine
integral, exponential solution, convergence order, JavaScript and Rust execution,
IEEE arithmetic and rendering are outside the Lean statements. The current
kernel status is source-bound in the [coverage ledger](../../education/COVERAGE.md),
separate from numerical oracle and browser evidence.
