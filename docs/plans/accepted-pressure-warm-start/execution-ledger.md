# Execution ledger

- 2026-10-13: drafted proposal while owner contract pending; no production edits.
- Parent relayed PR55 owner agreement. Recorded binding contract; chose coupled
  accepted-pressure reuse as smallest first slice, avoiding standalone seed array.
  Frozen base3525c979 (PR55) fetched; separate local branch moved to that head.
  PR55/57, PR58 and rejected-assessment branches remain unchanged.
- Next: implement explicit seeded numerical solve and owner-local provenance/commit
  behavior within agreed contract, then test before performance admission.

- Implemented borrowed seeded solver, coupled accepted provenance and opt-in Session
  path. Independent read-only review found no blocking defect; unsupported-path
  and preferred-provider tests added in response. Strict work-observation clippy
  passed; numerical/owner/surface/column/provider scoped tests passed (real-model
  checkpoint test remains the existing ignored external-input test).
- Pilot32³/1worker/4steps: steady cold0.9571s/warm0.3763s; localized1.0434/0.9047;
  switch0.7403/0.8107; rest0.1676/0.1492. These single runs are not qualification.
  Switch removal incurred277warm iterations vs34cold; retain this adverse result.
  Saturated fraction1 failed identically on first step for both policies, under
  unchanged fraction-bounds gate. No successful shortened horizon is claimed.
- Full pre-recorded matrix is still required. No performance admission yet.
