# PR59 supported capability and pending admission

Current state: preliminary implementation/review/regression complete; **not ready
for merge or whole-application production acceptance**. Final source must follow
verified #142 → #138 → #143 integration. Only the designated PR142 lead merges main.

## Implemented surface

`accepted-pressure-warm-start` is an explicit non-default Cargo feature for sealed
constant-density coupled-liquid CPU consumers. Enabling it exposes borrowed-seed
PressureWorkspace calls, LiquidTransportSimulation/SimulationSession initialization
policy and accepted provenance reports. Callers still opt in per step. Complete-step
acceptance owns provenance; failed/cancelled attempts preserve the publication.
No extra retained pressure array is allocated. Full current RHS compatibility,
residual/gauge, actual divergence, transport and downstream gates remain intact.

Without the Cargo feature, warm solver branches, owner history/identity allocation,
extra report fields and warm entry points are compiled out. Session API version3
reports the actual feature capability. This build boundary is a transparent scope
narrowing, not a universal performance guarantee.

Standalone Simulation remains cold. Provider/fallback, prescribed-boundary,
viscous, surface and column routes remain cold and invalidate native eligibility
only on accepted publication. There is no hidden retry or enlarged iteration
budget. Existing dense3D does not persist warm history; process-local stamps are
not portable replay identities. No GPU/device/native-rendered-performance claim.
Feature-enabled mixed-particle applications have no performance admission here.

## Evidence and limits

Initial-main source0b3e3fc (parents original7f1733f and main15026a9) has independent
exact-source review with no blocking finding. Full release default610pass and
warm626pass;10 existing explicit skips in each. Seven skips are separately bounded
upstream force integrations; three require exported real-model inputs. Warm/core
strict all-target Clippy and formatting pass. Hosted real-AI and executable checks
passed at0b3e3fc. These checks do not qualify later dependency composition.

Original workload-specific gains remain bound to their original PR55 base and
binaries. Original default1000-particle +1.83% screening / +2.20% confirmation,
rejected lost horizons and adverse switch pilot remain unchanged. They cannot be
erased or reinterpreted as universal speedup. No new timing has run. Final fixed
controls and any pre-timing evidence-contract decision are still pending.

Separate source paths confounded initial executable layout. Identical path remaps
were frozen before timing. An objcopy inspection command rewrote diagnostic inputs;
those outcomes are preserved and explicitly corrected. Raw outputs were rebuilt
with unchanged source/flags, original raw hashes reproduced, then freshly reviewed
read-only. Their code/data/load layout matches except explicit section-table
pointer/build-ID metadata; whole-file identity is false. This inspection is neither
statistical acceptance nor authorization to waive a positive/inconclusive control.

Native source admission correctly refuses the unreviewed complete inventory at
0b3e3fc; preliminary book run39687940422 fails there, with no deployment. No inventory
was automatically refreshed and no gate was bypassed. Fresh reviewed source binding
and permitted exact-source qualification remain required after dependency landing.

## Remaining actions

1. Compose actual merged #142/#138/#143 main, retaining every predecessor source
   receipt/history. Independently review exact combined source and source inventory.
2. Add the lead-authorized warm feature entry to ordinary CI while preserving all
   existing commands/aggregate/gates. Complete clean source-bound native admission
   and required exact-head checks; no sealed scientific campaign rerun merely for CI.
3. Execute the prospectively fixed final default controls without concurrent work,
   preserving every outcome. Resolve harmful/ambiguous behavior or obtain an explicit
   owned tradeoff; do not silently change feature scope or acceptance criteria.
4. Record exact ready head/dependencies/review/test results on PR59. Lead performs
   normal serial merge; verify merged SHA/ancestry/checks afterward.

All original branches and unique source remain preserved. Local task worktrees and
raw/diagnostic artifacts are retained for this pending integration, not deleted.
The user-facing production handoff must distinguish merged source from acceptance
of the full Rheon application and must include the other PR owners' remaining limits.

Current-main persistence composition was also inspected: configured_publication
requires the current configured-provider receipt and fails Unsupported when absent.
The initialized Session path clears that receipt only after acceptance, so warm
publications cannot masquerade as the preceding configured-provider record. Existing
transition tests prove receipt clearing/preservation; the recording requirement
provides the refusal by construction. No warm recording/restart codec is introduced.
