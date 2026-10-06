# Additive clarification of the frozen one-step experiment

The one-step packet at e3b6858d2d29fc8837dbf2acd84db0c01784a352 and every prior
file remain unchanged. Its passing pressure-forward result reports **22 counted
Newton equation evaluations**. The two final acceptance equations (coarse
order 16 and fine order 32) bring the actual completed equation evaluations to
**24**. Four Newton checks contain three corrections, not four corrections.

The controller's Newton target remains **1e-13**. The physical momentum,
constraint and conservation gates retain **LIMIT = 1e-11**. The earlier outcome
reader tested the observed successful norms against 1e-13; that extra check was
an observation of this particular result, not a change to the solver's physical
acceptance criteria. Its wording must not imply that the physical gates use
the Newton target. The ordinary quadrature limit remains 1e-15, and the work
allowances retain their original 128*EPSILON scale.

This clarification changes no result, accepted field, arithmetic, iteration
budget, equation budget, memory certificate or production limit.
