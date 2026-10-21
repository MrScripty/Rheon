# Independent source, mathematics and plan review

## Subject and method

A separate read-only reviewer inspected documentation head `4ac466988ce7360b6f766119845e8d355c8af75b`, selected primary-paper passages, current-code statements and the exact source/license inventories. The reviewer independently matched all 37 baseline file hashes and 16 cached external license/source/NOTICE hashes. The review is human-readable mathematical/source assessment, not formal verification, dependency-wide legal clearance, native qualification or browser/PDF inspection.

## Findings and corrections

| ID / severity | Finding at reviewed head | Correction in this documentation revision |
| --- | --- | --- |
| R1 / medium | Chapter40's full-zero weak test trace covered Dirichlet walls but omitted M1's free-slip case. | Section40.4 now separates full Dirichlet and impermeable-slip traces in an explicit H1 space, retains tangential traction work and states pressure/gauge spaces and net-flux compatibility. Energy assumptions include zero normal velocity and zero tangential traction. No existence/inf-sup theorem is asserted. |
| R2 / medium | Section40.7's acoustic linearization did not expose the smooth positive-slope EOS assumption or the current clipped liquid law's kink/flat branch. | States the continuum linearization assumptions, actual clipped EOS and derivative limits; neither characteristic invariants nor the reflection ratio qualifies reference-density liquid-preview behavior. Compressed-branch and isothermal-gas alternatives remain conditional, with no EOS change. |
| R3 / low | An audit's blanket no-scientific-execution wording conflicted with the recorded existing bounded publication prerequisites. | Both audits distinguish source inspection from those documented publication prerequisites; no portal qualification campaign or external implementation ran. |

The reviewer found no substantive sign/algebra defect in the control-volume balances, hydrostatic subtraction, periodic-gravity argument, MAC reservoir coefficient/nullspaces, pressure-work identity including defects, or conditional negative-part backflow calculation. The review accepts neither a full-split discrete energy theorem nor a generic inviscid backflow claim.

The implementation proposal covers a complete periodic MAC milestone, user controls, API ownership, serialization/migration and cold inspection before reservoir and particle stages. Dual-volume boundary completion/CFL, discrete viscous-backflow stability, particle EOS equilibrium and domain-size sensitivity remain future acceptance gates. All A1–A8 remain pending.

The license audit distinguishes citation/independent derivation from implementation copying; PySPH reuse remains conditional and DualSPHysics copying is excluded without excluding scientific citation. No dependency-wide or patent clearance is given.

## Correction re-review and publication limits

The same separate reviewer re-read the three corrected manuscript/audit files against4ac4669 and confirmed R1–R3 resolved, with no new substantive defect and a clean diff-whitespace check. This correction review was read-only and limited to those changes; it did not repeat unchanged hash checks or execute numerical/publication gates. Exact file identities are recoverable from the final committed diff against the reviewed head.

Initial draft PR creation was rejected by automatic approval review, with the denial preserved. After the user’s explicit follow-up authorization for documentation creation/merge/deployment after checks, one retry created draft PR148. Normal CI and exact final-head gates remain required. The sandboxed Chromium launch failed because its SUID helper was not configured; no unsandboxed fallback or security edit occurred. The earlier Library upload remains uncertain, with no verified finalized IDs and no retry/substitute write. These distinct blockers remain open regardless of mathematical review.
