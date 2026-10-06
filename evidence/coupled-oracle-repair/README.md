# Narrow qualification-oracle successor

Frozen research `ef9c3dc3/598e9a06` and native `3f0271d6/a9428641` packets remain
unchanged. Actual stored trajectories and convergence data are corroborated.
`reproduce_old.py` retains actual malformed studies accepted by the frozen
validators; this is an oracle failure, not a false physical trajectory.

`verify_trajectory.py` requires five distinct ordered intervals, complete case
counts, the common exact real endpoint, physical step fields, related regenerated
DAE references, recomputed endpoint/errors/ratios and original convergence bands.
`replay.py` requires every native geometry/mass/topology/pressure field, the actual
state stamp, canonical schedule and strictly validated host references. The
additive Rust example `coupled_discrete_identity` exports ID/version and step
version from `state.stamp`; it uses unchanged simulation code. Focused regressions
reject the previously accepted substitutes in normal and optimized execution.
`qualify.py` captures strict complete replays, current native runs, exact equality
of prior numerical fields, Rust formatting/Clippy and frozen ancestor validation.

This repair does not change governing equations, physical gates, native owner,
rollback or allocation contracts, pressure semantics or the actual convergence
results. No new physics, continuum convergence or Lean theorem is claimed. The
separate governing-equation consistency investigation is not part of this packet.
