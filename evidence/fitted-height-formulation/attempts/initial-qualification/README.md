Initial local qualification failures, retained without replacing their logs.
Lean rejected compact absolute-value syntax and the reserved variable `by`;
its two generic algebra proofs also needed explicit substitution/factoring.
The first corruption fixture modified a node where the affine third component
was zero, so the rotation check detected it first. The corrected fixture corrupts
a nonzero affine cap node and detects the intended traction mismatch. These were
research harness repairs, with unchanged physical tolerances and production.
