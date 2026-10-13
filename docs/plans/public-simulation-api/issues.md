# Issues and dispositions

- API-1 (resolved design dependency): real material/SPH, quantitative and AI-provider
  contracts are owned by concurrent workers. The API owner now designs the adapters, using published PR #47 and PR #49 directly
  on the stacked feature branch. Capability claims remain limited to their evidence.
- API-2 (deferred): no existing Godot binding/project in Rheon. API owner supplies
  engine-neutral example; a selected binding stack and consumer acceptance are needed
  for actual Godot support. No GPU/desktop/target-device timing claim was tested.
- API-3 (standards evidence): external standards were located after initial source
  work. Policy review is recorded at the exact source pin; no application-purpose
  Standards Engine tool was exposed. Do not describe manual reading as engine approval.
- API-4 (resolved): independent review identified nondeterministic final-callback
  coverage in the real-clock test. Replaced it with a private controllable-clock
  unit test that explicitly observes BeforeCommit, advances to deadline inside
  the callback, verifies exact rollback, and retries. Production uses Instant::now.

- API-5 (boundary, not a hidden fallback): grid provider pressure qualification
  does not satisfy full-step corrected-velocity acceptance. PressureSession exposes
  the qualified stage explicitly. Full trajectory injection needs a physics-owned
  candidate/correction/commit hook and independent divergence/rollback evidence.
- API-6 (retained negative evidence): one authored high-frequency real-model request
  failed qualification and exhausted the 3000-iteration fallback budget. Kept the
  request and original thresholds in the artifact test, asserting typed failure and
  unchanged prior snapshot; do not claim universal fit from the smooth-mode success.
