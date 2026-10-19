# P0 qualification and retained evidence

Scope: this PR supplies a plan, research, sample interactive visual and isolated proof proposals. It changes no production solver/UI/recording source. The frozen audit's 582 CPU tests and 98 numerical checks are historical evidence, **not rerun results for these changes**. None of G1/C1/P/Q/V1 production acceptance is asserted.

## Source and environment

- Base remote main: bc2f472787060d9a69cb619ed2c2fea10fd754b6, includes PR115. Separate branch `plan/fluid-accuracy-composition-20261019`; no merge or history rewrite. Coding Standards at 0fc92a24279f3a4574c80af0a2f31ea347a21597. Original audit and PR59 protected.
- Linux x86_64 CPU. Official Lean 4.19.0 release reports compiler commit `6caaee842e94`. Download URL `https://github.com/leanprover/lean4/releases/download/v4.19.0/lean-4.19.0-linux.tar.zst`; actual SHA256 `6fe3ce97a58f44e2b3567d455b994eacec5bfe9ae7774f2a573444480ba813fe`. Root `proofs/lean-toolchain` and `lake-manifest.json` unchanged; mathlib c44e0c8ee63ca166450922a373c7409c5d26b00b plus its locked dependencies.
- Elan toolchain retrieval failed with proxy CONNECT HTTP403. Direct official binary download succeeded. Initial cache attempt hit read-only `/home/agent/.cache/mathlib`; retry used task-owned XDG_CACHE_HOME, then cache downloads returned HTTP403. A normal pinned source build was used instead. Those failures did not justify changing toolchain/dependency versions or axioms.

## Exact bounded proof target

[Publication.lean](../proofs/Publication.lean) models immutable tentative candidates and elected boolean validation. Seven theorem statements cover rejection retaining old state, acceptance selecting candidate, common checked-predicate preservation, same-old fallback, two checked publications, and an explicit counterexample to inferring unrelated accuracy. It does not prove Rust rollback, external history/RNG/network rollback, process safety or physical validity.

[StokesWork.lean](../proofs/StokesWork.lean) contains five theorem statements: pressure adjoint work, assembled strain dissipation, the coupled backward-Euler identity with momentum defect and pressure/continuity work, exact constrained work, and energy nonincrease under nonnegative mass/weights/dt. It defines finite real B/S/M/W reference operators, not Rheon's assembled cut geometry. Exact coordinate step and divergence constraint are hypotheses; no existence, uniqueness, solver convergence, IEEE refinement, mesh validity, general wall/free-surface assembly or scientific validation theorem is claimed.

The new modules live beside this plan, outside the root `proofs/` source tree, and are deliberately **opt-in**, outside historical `Rheon.lean`, `AxiomAudit.lean` and `source-inventory.json`. Those old bytes and receipts are unchanged; no full-root requalification is claimed. Before production integration, selected statements need explicit root import/audit/inventory review. This prevents the old qualification from appearing to cover new source automatically.

Reproduce from repository root using the pinned compiler on PATH:

```sh
cd proofs
XDG_CACHE_HOME=/workspace/rheon-plan-proof-tools/cache lake build Mathlib.Data.Real.Basic Mathlib.Algebra.BigOperators.Ring.Finset Mathlib.Algebra.Order.BigOperators.Group.Finset Mathlib.Data.Fintype.Fin Mathlib.Tactic.Linarith Mathlib.Tactic.Ring
python3 ../docs/plans/fluid-accuracy-composition/proofs/check.py /workspace/rheon-fluid-plan-evidence/kernel
```

`check.py` compiles both modules, runs the expected-declaration axiom audit with only `propext`, `Classical.choice`, `Quot.sound` allowed, and runs three actual refusal probes for a custom axiom, admitted proof and skipped expected declaration. These negative generated inputs are not accepted proof sources. Final outcome: **PASS**, outer command exit 0. Both modules compiled; 12 theorem statements and 8 definitions were checked through a 25-declaration axiom audit including generated declarations. All three negative refusal probes passed in the final plan-local location; the unchanged root source-inventory gate also passed. No accepted `sorry`, custom axiom or native-evaluation axiom is used. Three normal pinned source-build commands completed successfully. Earlier development attempts failed on missing imports and a tactic proof; their logs are retained, and only the final source-bound successful checks qualify these modules.

## Authored prototype checks

Exact command:

```sh
python /workspace/rheon-fluid-plan-evidence/workflow-preview-check.py > /workspace/rheon-fluid-plan-evidence/workflow-preview-check.txt 2>&1
```

Observed: 20/20 node-detail selections, both authored modes, Arrow/Home/End selection, reset/focus, unavailable metric display and RSC1/RCP2/RCP3/RCT1 labels passed; 390px viewport had no document overflow; zero JavaScript page errors. Headless Chromium 151.0.7922.173 loaded local bytes with `page.set_content`, using the environment's `--no-sandbox` browser test mode. This verifies a source-controlled static design aid with simulated data, **not** native egui, actual provider execution or recorded workflow custody. A `file://` navigation attempt was denied by browser administrator policy (`ERR_BLOCKED_BY_ADMINISTRATOR`); it was not retried or presented as successful hosting. No server/deployment was created. Screenshot retained only locally, excluded from Library deliverables.

Prototype source SHA256: `b729ea0da75cbde9a322a0760fb962347b561d0cd6dced4d279ce3a13818ebfe`. Source and script checks must bind the same bytes; a later edit invalidates the prior source identity. The generated test script/log are retained with evidence, not committed as simulation observations.

## Other checks and review

Primary-reference correlation/linear-EOS calculations independently evaluated with Python standard library: nu=1.0038068523342016e-6 m²/s, IAPWS mu=.0010015672646030015 Pa s, sigma=.07273614042160757 N/m, capillary length=.002725875433712895 m, c_min=31.393633007279462 m/s for 1m/1%, and c=20 compression=2.4819628591899487%. These are formula evaluations, not native fluid tests.

Three disjoint independent reviewers examined mathematics, architecture and standards after the research authors finished. All initial findings were recorded before material revision. [Review dispositions](reviews.md) documents the three mathematical, three architecture and three standards findings and rechecks. No reviewer independently reran Lean. Product acceptance and method-selection gates remain open.

Final supporting checks: 41 local Markdown/HTML links resolved; Markdown table-column consistency and Python syntax passed; `git diff --check` passed; protected root proof inputs exactly matched main and `src` had no changes. Native fluid/GUI tests were not selected for this source-only plan/proof change. Independent reviews and corrected source are retained; source commit/PR and durable handoff identities accompany the package. Final qualification uses the plan-local proof location and the unchanged root source gate. [Draft PR120](https://github.com/MrScripty/Rheon/pull/120) is open and unmerged. `manifest.json` binds source/evidence bytes without being a numerical pass oracle. Generated caches, fields, logs, screenshots and archives stay outside Git under the repository output policy.

## Actual final command outcome

`PATH=/workspace/rheon-plan-proof-tools/lean-4.19.0-linux/bin:$PATH python3 docs/plans/fluid-accuracy-composition/proofs/check.py /workspace/rheon-fluid-plan-evidence/kernel-final` from the repository root returned exit 0:

```text
PASS Publication
PASS StokesWork
PASS axiom-audit
PASS custom-axiom
PASS admitted-proof
PASS skipped-expected
PASS isolated follow-on kernel/axiom qualification (not Rust/physics validation)
```

The audit's final line is `Follow-on axiom audit passed for 25 declarations`. Raw per-module, positive-audit and negative-probe logs, source-build logs, formula evaluations and browser prototype script/output are retained in `/workspace/rheon-fluid-plan-evidence`. The compact manifest binds their bytes. Screenshots are local inspection evidence only and excluded from Library retention. P0-L is satisfied within this exact stated scope; no production acceptance changes.

### Root inventory placement correction

Final repository-gate inspection established that the old source checker inventories every Lean file under root `proofs/`, including subdirectories. The initial proposal location would therefore fail that gate even with old inventory bytes unchanged. The proposals were moved beside the plan, with the checker explicitly using root Lake dependencies and plan-local source paths. The root gate was neither weakened nor regenerated. Final kernel/axiom and root-source checks were rerun at this location; first-location logs remain historical development evidence only.
