# Findings and dispositions

- Shared files: do not modify particle/boundary, API or book work. Any extension
  requires coordination through parent. CPU pressure files are declared owned.
- CPU environment evidence cannot establish device/GPU performance.
- Warm starts: deferred from workspace-only slice. `solve_velocity` succeeds
  before f32 divergence/Courant/tracer and later cancellation gates. Cache must
  commit at `commit_prepared`/`commit_column_mac`, not solve success; equal-sized
  domains can still change operator/density/surface. Cross-owner work required.
- Particle locality: linked bins are already present. Reordering particles or
  neighbor accumulation changes deterministic sums and particle identity unless
  carefully mapped; particle/emitter files now have active collision/control
  owners. No production edit or particle speedup claim in this slice.
  Dense particle control measurements cover unchanged behavior and resource use.
- Stronger preconditioning: existing SGS is measured alongside Jacobi. No new
  preconditioner admitted without evidence beating its complexity and existing
  whole-frame results. A lower iteration count alone is not a speedup.
- Preview/export: owned-field CPU capture is timed separately; it does not
  represent GPU rendering or PNG export. Neither accuracy nor preview quality
  will be reduced to manufacture a faster solver result.

- Historical rejected-candidate adverse control: 4096 particles / one worker slows in all three
  pairs, median 9.7%, nonoverlapping ranges. Cause is not established. No source
  edits in particle code does not prove timing equivalence. Draft publication
  is permitted, but regression-free acceptance and merge readiness are blocked
  pending attribution/resolution. Preserve raw data; do not dismiss as noise.

- Follow-up disposition: original 4096/1 signal was not stable across 20-pair
  controls. Fresh 4096/4 was adverse in 7/8 pairs. No runtime pressure path exists
  in particle binaries; layouts differ but cause is not proven. Production
  changes reverted. Shipping this candidate is rejected, not declared harmless.
  Detailed report: reports/particle-follow-up.md. Book ownership remains intact.
