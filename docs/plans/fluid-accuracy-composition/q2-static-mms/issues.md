# CV3-A preparation issues

- DIM-1: Layout8 is not the only n16 obstruction. Assembly and selected solver
  independently cap velocity rows256; rectangle needs275. Proposed dimension-only
  bounded replan was recorded before edits and subsequently accepted by the
  independent numerical reviewer for preparation. The declared guards are now
  implemented in this child; frozen parent sources remain unchanged.
- UNC-1: Original-coordinate certified numerical bounds must propagate through
  physical norms and refinement differences. CV2 highprecision agreement is
  insufficient. The prospective guaranteed-bound method and complete source are
  independently reviewed and frozen; actual certificates/decisions remain unrun.
- EXEC-1: Current continuation is preparation only. No CV3 selected numerical
  execution, result, PDE/production qualification or merge is admitted here.
- SRC-1: Initial compile-only check exposed a fixture-coupled cfg(test)
  dependency when importing the old CV2 experiment wrapper. The MMS source now
  imports only the actual shared solver and owns its factor CSV exporter.
  Strict compile-only Clippy and formatting pass after iterator lint repairs.
- TEST-1: Three new numerical contracts are explicitly ignored by default
  pending source/oracle/resource execution admission, so publishing preparation
  does not cause ordinary CI to execute them prematurely. They remain unrun;
  an eventual resource-bound command must select them explicitly with --ignored.

- RES-1: Closed for preparation: all-nine source bounds, exact algorithms, fixed
  commands and enforceable resource owners reviewed before numerical output.
  Completion time and actual bit growth remain unmeasured risks under hard stops.
- FREEZE-1: Complete source/command custody is recorded in preparation-freeze.md.
  Lead execution disposition is outside this preparation-only continuation; no
  technical preparation blocker or approval denial remains.
