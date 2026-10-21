# Findings and deferred work

Initial findings below preserve their historical state. Current dispositions are
recorded in the final checkpoint section; resolved integration blockers are not
present requirements for already qualified S1.

| ID / severity | Evidence and relation to objective | Owner / boundary | Disposition and verification / revisit trigger |
| --- | --- | --- | --- |
| CPU-COMP-1 / high | Main has no native Euler thermodynamic owner; research LLF/Lean identities do not provide it | S1 numerical owner | Fix in S1; native C1/C2 evidence. No solver capability claimed by plan |
| CPU-COMP-2 / high | `cfl_density_positive` concerns a donor-density definition, not Rusanov Euler; endpoint speed labels do not establish invariant domain | numerical/proof owners | Runtime admissibility/rollback in S1; defer method-specific positivity theorem to S2; revisit on flux/CFL/reconstruction/domain change |
| CPU-COMP-3 / high | Current collision/reduced obstacle/plate models omit arbitrary swept fluid topology; no validated submarine/aircraft result | geometry/physics owner | Deferred: design vehicle gates; revisit before arbitrary CAD, moving bodies or vehicle advertising |
| CPU-COMP-4 / medium | Open PR69 b1acf2e and PR71 ecfc741 own archive and exact selected inspection; not integrated main | recording integration owner | Defer S4 until owner disposition; re-read current contracts, then real record/cold reload/UI proof |
| CPU-COMP-5 / high | Existing dense3D v2 capture is fixed transport; verified arbitrary compressible Tuldok consumer unavailable in this checkout | producer/Tuldok owners | Defer S5 contract/schema; smallest real native→consumer path before generation; revisit when consumer source and accepted contract identified |
| CPU-COMP-6 / high | Native source admission pins intentionally bind broader crate inputs; implementation changes will invalidate old receipts | book integration owner | Preserve gate; future reviewed inventory + fresh committed-source native/rational/publication evidence; revisit before publishing implementation claims |
| CPU-COMP-7 / medium | No hardware/error budget or physical benchmark fixes vehicle real-time/accuracy target | benchmark/physics owners | No speed/vehicle acceptance claim; freeze workload, data license/regime/uncertainty and independent oracles before promotion |
| CPU-COMP-8 / medium | Pending archive config Debug text is opaque, not a reproducible project codec/checkpoint | producer/API/recording owners | S5 needs complete controls/EOS/boundary/source capture and independently verified consumer; archive remains inspection only |
| CPU-COMP-9 / medium | f64 does not eliminate high-Mach internal-energy cancellation or low-Mach LLF damping | numerical owner | Negative conditioning tests S1; Mach/refinement regimes S2; new dual-energy/all-speed design requires re-plan |
| CPU-COMP-10 / high | Original 8³/12³/20³ conservative-average acoustic study fails strict >0.6 order; native 0.561438404 agrees with independent LLF symbol 0.561438498; [pinned diagnosis](https://github.com/MrScripty/Rheon/blob/b5e868d76aaacf3a538fd1004e0ace7f037edf1b/docs/plans/cpu-cfd-validation/reports/native-s1-diagnosis.md) | S1/validation owners, admission through parent | Re-plan future study only to 16³/24³/32³ with ≥8 cells/shortest coordinate wavelength; retain both failed origin receipts and coarse limitation; all controls/strict threshold fixed. New acceptance blocked until reviewed revision coordinated and fresh native evidence passes |

No systemic defect is repaired by this documentation change. The future shared
geometry/field/control invariant families are explicitly scoped in design; a
discovered repeated authority/consumer defect requires the standards' systemic
finding audit before local patches resume.

## Qualified S1 checkpoint disposition

CPU-COMP-1 is resolved for bounded periodic S1 through C1/C2 at the separately
pinned [release baseline](reports/release-baseline.md). CPU-COMP-6 is resolved for
the reviewed 78-path combined source and full qualified PR76 book/runtime subject;
future source changes still require review and affected fresh qualification.
CPU-COMP-4 recording integration and actual material/carrier software-display
inspection are qualified by PR76, while the S4 compressible adapter remains absent
and Deferred. CPU-COMP-10's original coarse study remains failed; the independently
admitted prospective study passes unchanged gates and does not relabel that failure.
Other physics, producer/consumer and vehicle follow-ons retain their owners/triggers.
No S2 implementation or whole-roadmap acceptance is recorded.
