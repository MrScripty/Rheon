# Existing endpoints: signed geometry components divided by h

All eight original geometry band failures remain. Four have a dominant-component switch; four do not. None passes the original 1.7–2.3 band when the coarser and finer endpoints are compared using the finer endpoint's same dominant coordinate. Reference-resolution checks already passed and remain unchanged.

Only the existing 46 completed endpoints, two refused prefixes and 16 existing references are read. There are zero new trajectory steps or reference integrations. Missing endpoints at h = 0.00078125 remain missing for initial forward and pressure reversed nonconstant fields; partial accepted prefixes are not common-final-time endpoints.

Coordinates follow the actual frozen `q_to_x` order: left cap x, left cap height, middle cap x, middle cap height. `outcome-v3-normal.json` retains all signed component errors, error/h, exact rational binary-input differences, signs, dominant coordinates, paired-reference differences/h and original ratios for all eight families. The native NumPy-bool serialization failure and additive v2/v3 metadata corrections are preserved; v3 repairs coordinate labels and height-pair metadata without changing numerical errors or ratios.

| Reversed nonconstant family, h pair | Original maximum ratio | Same finer-dominant coordinate ratio | Coarse dominance amplification | Dominant switch |
|---|---:|---:|---:|---|
| Initial, 0.05 → 0.025 | 2.6287525513516203 | 0.4775080733000236 | 5.505147867311441 | middle x → left x |
| Pressure, 0.05 → 0.025 | 2.5210010069037616 | 0.6372543944213753 | 3.956035500065592 | middle x → left x |
| Initial, 0.025 → 0.0125 | 1.4451889226786736 | 1.4451889226786736 | 1.0 | none, left x |
| Pressure, 0.025 → 0.0125 | 1.488677366350941 | 1.488677366350941 | 1.0 | none, left x |

The constant-field versions supply the other four failures and exhibit the same pattern. The exact descriptive factorization is maximum ratio = fixed-coordinate ratio × coarse dominance amplification. Dominance explains part of the first-pair maximum ratio, but does not restore a common-component band. For the second pair the normalized left-x ratios are approximately 0.7226 and 0.7443, with no dominant switch. Signed error/h varies appreciably at these coarser steps. This is descriptive evidence, not an extrapolated convergence proof or a substitute acceptance metric.

`signed-components.svg` displays all four signed components for the existing nonconstant endpoints. Paired-reference differences are empirical observations, not rigorous error enclosures. No geometry remedy is established. The original two E1 refusals and eight geometry band failures remain qualification blockers.

Reader source: `4c747b8002179ba2b39a9c5a978be5fc8fcad947`, tree `088f03fc64acaaad6c74e8243bc19a2347bc6d60`. Original geometry receipt was frozen at `4ee70e4587cb4ce5f5f164bcfea59427905e1565` before the separate native E1 capture and is unchanged. Normal/optimized readers agree. The final joint receipt and read-only verifier live in `../forcing-e1-fixed-candidates-v1/`; they do not run any trajectories, references or native candidate equations.
