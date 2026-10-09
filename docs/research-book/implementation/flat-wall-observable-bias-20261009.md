# Wall-observable bias and misleading force cancellation

The retained N6/N9/N12 body-force Ritz fields expose a physical observation
problem that small algebraic residuals cannot settle. This note describes
post-processing of those existing fields, not a new solve or convergence study.
The original one-shot campaign remains refused; its failed evidence is preserved
outside Git. The fields remain supplied, immutable and physically unqualified.

## An independent manufactured reference

On the declared lower fluid strip, take
`psi=A B(x-1) B(z-1)(z-1/2) y^4(1-y)^2`, with
`B(s)=s^6(1-s)^6`, `A=12012^2/2`, `u*=(psi_y,-psi_x,0)` and `p*=0`.
Exact polynomial integrals give force `(-1,0,0)` N and torque
`(0,-1/60,-1/2)` Nm about `(1.5,1.5,1.5)`. The solver receives only its declared
body-force polynomial; this reference field is independently constructed for
diagnostics and never enters the provider.

Differences of exact layer integrals of psi divided by exact geometric areas
give continuum face averages. The stored curl map uses rounded inverse areas;
its small representation effect is kept separate. For a linear wall observable
`Lh`, the exact signed diagnostic is

`Lh(u_native)-J* = [Lh(u_native)-Lh(C_stored q_ref)]`
`                 + [Lh(C_stored q_ref)-Lh(u_bar*)]`
`                 + [Lh(u_bar*)-J*]`.

These are, respectively, an aggregate field/model discrepancy, an area/inverse
representation effect, and reference wall-observation/surface-quadrature bias.
This identity measures this known reference; it does not bound errors of an
unknown physical solution or separate source, energy, trial-space and solver
errors inside the first bracket.

At N12 the original P1 x-force is `-0.548770144` N. Its reference observation
bias is `+0.685452408` N: about `+0.679578132` N comes from the wall secant and
`+0.005874276` N from x quadrature. The field/model bracket is `-0.234222551` N.
Thus the wall observable is the dominant measured baseline contribution.
Normal traction also gives spurious reference torque and remains unchanged by
the tangential option below.

## Reconstructing a derivative from averages

Let `s=wall_y-y`, with stationary trace `v(0)=0`. Assume the first two tangential
samples are cell means over `[0,H1]` and `[H1,H2]`, where `0<H1<H2`.
Eliminating the quadratic coefficient of `v(s)=a s+b s^2` gives

`a_hat=w1 v1+w2 v2`,
`w1=2(H1^2+H1 H2+H2^2)/(H1 H2^2)`, `w2=-2H1/H2^2`.

Uniform cells give `(7v1-v2)/(2h)`. These are average-based weights; ordinary
point-center quadratic weights have different moments. The optional
`TwoCellAverageP2` Rust observable uses the actual stored grid planes and outward
coordinate-subtraction intervals. Constant viscosity and a smooth stationary
flat no-slip wall let `mu*a_hat` approximate fluid-on-body tangential traction.
The original P1 API, normal P1/P2 observations, energy, source and solver retain
their existing paths. No pressure or timestep coupling is added.

For a C3 averaged profile with zero wall trace, the derived exact-real remainder
is bounded by `H1 H2/12 * sup|v'''|`. Sample errors additionally contribute
`|w1| epsilon1+|w2| epsilon2`: on uniform cells, equal input bounds become
`4 epsilon/h`, compared with P1's `2 epsilon/h`. Quadratic exactness removes
the quadratic-profile bias. For this manufactured reference the measured
observation bias decreases, while equal uniform-grid sample-error bounds have
twice the P1 amplification.
Arithmetic intervals cover the implemented arithmetic on supplied samples,
not sampling, field, trace or PDE error. No new Lean theorem certifies this
physical interpretation; existing finite-work proofs have their own premises.

## Why a force near -1 N is insufficient

Using the optional tangential reconstruction and unchanged normal P1, actual
Rust retained-data observations give:

| N | Fx (N) | Tz (Nm) |
| --- | ---: | ---: |
| 6 | -0.428113547 | -0.107028387 |
| 9 | -0.598440128 | -0.197347034 |
| 12 | -1.005841242 | -0.365728085 |

At N12, `Fx=-1-0.438878915+0.433037673` N. Two errors of about 0.43 N cancel,
leaving a small net force error. Torque remains visibly different from its
`-0.5` Nm reference: `Tz=-0.5-0.160883820+0.295155735` Nm. These measurements do
not establish accuracy, an application tolerance or convergence. Unknown
source-to-field and wall-trace error propagation still prevents qualification.

The primary research context is [Batty and Bridson (2008), sections 5 and
5.1](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf) for symmetric
stress and MAC fluid-volume quadrature, and [Batty, Bertails and Bridson (2007),
equations (4)-(6)](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf)
for the separate mass-weighted pressure projection. The average-based wall
formula is newly derived here, not a stencil attributed verbatim to either
paper. The [full derivation](../../flat-wall-physical-error-model-20261009.md),
`src/flat_wall_ritz.rs`, `tools/flat_wall_physics_model.py` and their scoped tests
are reproducible source; actual records and review receipts remain external.
