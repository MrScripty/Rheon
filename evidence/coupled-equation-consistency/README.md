# Governing-equation and pressure-meaning audit

The research/native checkpoints remain frozen; simulation Rust, acceptance gates,
old packets and existing claims are unchanged. `taylor.py` independently rebuilds
PS geometry/operators and derives exact rational limiting/first-defect coefficients,
checking the full tangent and its derivative against separate full systems.
`study.py` measures actual saved cell defects against those coefficients and
independent regenerated DAE pressure/impulse references, retaining old false gates.
It compares repeated physical pressure values at common time by an explicit ALE
pullback. `finer.py` retains finer actual attempts, including the unchanged-gate
initial `.00078125` Newton refusal. `test_reference.py` retains wrong mass/constraint
variants and the invalid pressure-as-evolved-state derivative comparison.

The method approaches the declared semidiscrete donor ALE tangent system. Its
pressure is a newly solved algebraic multiplier. The published endpoint pressure
value is first-order; the force term is an endpoint-weighted interval impulse.
It is neither a literal average nor an evolved pressure state. Nonzero scaled
truncation coefficients and the failing first pressure difference quotient must
not be called nonvanishing governing-equation defects. Continuum spatial/traction
accuracy, IEEE/mesh-family/sign theorems and material/contact properties remain
unqualified. The book proposes nonzero third velocity on the existing constant
material extrusion as the next bounded physics slice; this packet implements none.

The independent qualification-oracle repair lives on a separate branch/prefix.
This audit does not claim to repair the frozen reader gaps or falsify the actual
stored trajectories. New equation comparisons regenerate references and exact
coefficients rather than using a self-convergence ratio as model validation.
