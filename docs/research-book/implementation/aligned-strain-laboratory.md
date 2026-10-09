# Aligned-strain laboratory

[Open the live laboratory](../../education/aligned-strain-lab.html) to change
face velocities and inspect the strain, force and dissipation they produce.
The controls evaluate a fixed finite operator. They keep the geometry, active
face space and stationary wall traces fixed.

The specimen is a $3\times3\times3$ grid of one-metre cells, with the central
cell occupied by the box $[1,2]^3$. Density is $1\,\mathrm{kg/m^3}$. Its
source-bound packet contains **48 active velocity faces, 210 retained strain
rows and 6 structurally zero rows**. Independent exact rational assembly gives
$B=17\,\mathrm{m/kg}$. The packet binds the native records, executable and
source digests to a qualification receipt and independent oracle comparison.
It is the same stationary no-slip obstacle and sealed free-slip outer domain
specified in the [reconstruction derivation](reconstructed-aligned-strain.md).

## Follow one row

A row gathers $s_r=\sum_f E_{rf}u_f$. Three normal families measure
$\partial_a u_a$ at fluid cell centres, each with weight $2V$. Three engineering
shear families measure $\gamma_{ab}=\partial_b u_a+\partial_a u_b$ at internal
grid edges. Each actual fluid quadrant contributes its own volume weight
$v_q$. Here the centred unit geometry gives $v_q=1/4\,\mathrm{m^3}$; the general
construction uses represented centre-to-edge lengths and cell widths.

Stationary obstacle and sealed outer normal traces are prescribed zeros and
eliminated from the coefficients. The six empty normal rows remain quadrature
samples, even though their strain is always zero. Outer engineering shear
vanishes analytically under sealed-normal and free-slip conditions before
edge enumeration. Zero rows on included internal samples remain present.

The same coefficients scatter $Ku=E^TWEu$ back to the active faces. With
scalar dynamic viscosity $\mu$, the force and positive loss are

$$
F=-\mu Ku,\qquad D=\mu\sum_r w_rs_r^2,\qquad \sum_f u_fF_f=-D.
$$

Inspect the row’s coefficients, gathered strain, weighted loss and support in
the slice. The total ledger includes all rows, including those outside the
selected slice. The exact
[transpose-work statement](https://github.com/MrScripty/Rheon/blob/f75cd66c6dc4167ad1ec3febb72e6fb2aa6b4d0b/proofs/Rheon/AlignedStrain.lean#L54)
and [nonnegative-loss statement](https://github.com/MrScripty/Rheon/blob/f75cd66c6dc4167ad1ec3febb72e6fb2aa6b4d0b/proofs/Rheon/AlignedStrain.lean#L59)
explain this ledger.

| Quantity | Meaning | SI unit |
|---|---|---|
| $u_f$ | Active face speed | $\mathrm{m/s}$ |
| $s_r$ | Normal strain rate or engineering shear | $\mathrm{s^{-1}}$ |
| $w_r$ | Normal $2V$ or shear-sector volume | $\mathrm{m^3}$ |
| $\mu$ | Dynamic viscosity | $\mathrm{Pa\,s}$ |
| $D$ | Viscous dissipation rate | $\mathrm{W}$ |
| $F_f$ | Viscous force on a face | $\mathrm{N}$ |
| $H=\frac12\sum_f m_fu_f^2$ | Kinetic energy, $m_f=\rho A_fd_f$ | $\mathrm{J}$ |

Lean’s `strainLoss` excludes $\mu$ and has units $\mathrm{m^3/s^2}$; the browser
multiplies it by $\mu$ to display $D$ in watts. The force-work ledger is power,
not a change of energy over a timestep.

## See why a corner couples components

Reflect coordinates and vector components so the solid quadrant is southwest.
For the two surviving samples $U,V$, choose
$u_x=U\max(y,0)/\ell_y$ and $u_y=V\max(x,0)/\ell_x$. These reconstructions reach
zero on both solid half-walls. On the unit grid, $\ell_x=\ell_y=1/2\,\mathrm m$,
so the northeast, northwest and southeast rows are $2U+2V$, $2U$ and $2V$.
Their three quarter-volume contributions give

$$
D_{\mathrm{corner}}/\mu
=\tfrac14(2U+2V)^2+\tfrac14(2U)^2+\tfrac14(2V)^2
=2U^2+2UV+2V^2.
$$

Thus the local block is $\begin{pmatrix}2&1\\1&2\end{pmatrix}$ in these unit,
reflected coordinates. The displayed formula uses numerical SI values; the
block entries have units of metres. The shared quadrant supplies the cross term. The
corner selector retains the actual global component directions: reflections
change the coefficient signs shown by the inspector. This piecewise corner
choice is newly derived; the
[unit-corner identity](https://github.com/MrScripty/Rheon/blob/f75cd66c6dc4167ad1ec3febb72e6fb2aa6b4d0b/proofs/Rheon/AlignedStrain.lean#L256)
checks its quadratic algebra.

## Try the controls

- **Corner samples:** vary the two speeds in $\mathrm{m/s}$ and compare their
  individual sector losses with the shared sector.
- **Normal stretch and cross-component shear:** vary rates in $\mathrm{s^{-1}}$.
  These are affine samples on active faces; prescribed wall traces stay zero.
- **Local rotation:** the selected full-fluid shear rows cancel when the two
  derivatives oppose. Surrounding fixed traces can still produce global loss.
  A rigid rotation across the whole domain generally violates the stationary
  obstacle and sealed outer walls, so it is not a global null-mode claim.
- **Viscosity, slice and row:** viscosity scales force and loss; slice and row
  selection reveal their support. None of these controls advances time or
  performs pressure projection.

## Read the conditional proof correctly

The [coefficient-force bound](https://github.com/MrScripty/Rheon/blob/f75cd66c6dc4167ad1ec3febb72e6fb2aa6b4d0b/proofs/Rheon/AlignedStrain.lean#L135)
derives the finite quadratic bound from fixed nonnegative weights, positive
masses and explicit per-face coefficient bounds. The
[exact Euler theorem](https://github.com/MrScripty/Rheon/blob/f75cd66c6dc4167ad1ec3febb72e6fb2aa6b4d0b/proofs/Rheon/AlignedStrain.lean#L225)
requires exact arithmetic, no forcing, nonnegative time and viscosity, and
$\Delta t\,\mu B\le2$. The
[projection-composition theorem](https://github.com/MrScripty/Rheon/blob/f75cd66c6dc4167ad1ec3febb72e6fb2aa6b4d0b/proofs/Rheon/AlignedStrain.lean#L234)
also requires the matched pressure masses, positive density, areas and
distances, positive time, and an exact full pressure-residual solve.

The browser’s nearest-rounded $B=17$ estimate is **unenclosed and authorizes no
timestep**. A small displayed work-identity residual is a numerical diagnostic.
These exact-real statements do not certify native assembly, JavaScript/IEEE
arithmetic, approximate pressure solves or continuum convergence.

The original variational motivation is
[Batty and Bridson (2008), PDF pp. 5–6, equations (9)–(11) and §5.1](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf#page=5),
which places normal and shear stress at MAC cell centres and edges. The
mass-weighted projection reference is
[Batty, Bertails and Bridson (2007), PDF pp. 3–4, equations (1), (3)–(6)](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf#page=3).
The [reconstruction derivation](reconstructed-aligned-strain.md) separates
those source principles from the new restricted stencil and corner choice.
