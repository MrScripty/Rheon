# Separate affine pressure closure research

No pressure implementation is selected here. The viscous wrench accepts no
pressure and does not change the existing stationary pressure owner or metric.
The adjacent-cell unit-cube hydrostatic force error remains a decision gate.

One exact-real alternative avoids needing a second outward normal sample by
using the fluid cell centers on **opposite sides** of the padded aligned box.
For wall positions `a<b` and adjacent outside centers `cl<a<b<ch`, linear
interpolation evaluates both wall traces exactly for affine pressure:

```
H_low  = [(ch-a)/(ch-cl), (a-cl)/(ch-cl)]
H_high = [(ch-b)/(ch-cl), (b-cl)/(ch-cl)].
```

Each trace reproduces constants and the wall position, with positive weights.
In the center-unit cube, the rows are `[3/4,1/4]` and `[1/4,3/4]`.
For `p=b0+k*z`, paired unit-area wall force is `-k*(b-a)`.

The matched transpose is essential: with signed outward-fluid wall map `J`,
the virtual pressure constraint must use `C=H^T J`, and the solid wrench is
`C^T p`. On the unit pair under normal translation `V`, this distributes flux
`[V/2,-V/2]`, whereas physical adjacent-cell wall flux is `[V,-V]`. Thus the
two-center trace is affine-exact but changes the virtual constraint. It cannot
be appended to an unchanged physical wall-flux map or asserted to represent
local swept-volume conservation. It also samples across the solid; non-affine
accuracy, intended support and kinetic metric require an explicit decision.

The exact Fraction script `tools/research_affine_pressure_closure.py` checks
three fixed rational pairs, constant/gauge reproduction, affine force and
matched work. These are research calculations, not native pressure comparisons,
PDE qualification or a new solver. Three-dimensional wall-tile force/moment
quadrature, stored rounding, compatibility, conditioning and eventual moving
boundary equations remain open obligations. This positive-weight alternative
does not resolve them or select the owner’s policy.

[Batty, Bertails and Bridson (2007)](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf),
sections 2–3, motivates matching pressure/body velocity and generalized-force
maps within a consistent kinetic formulation. The paired-wall trace above is
new Rheon research; their complementary-volume map is not imported into the
current reduced active space.
