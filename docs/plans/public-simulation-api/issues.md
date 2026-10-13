# Issues and dispositions

- API-1 (integration dependency): real material/SPH, quantitative and AI-provider
  contracts are owned by concurrent workers. Parent must relay accepted types/revision
  before M3 integration. No capability is advertised before implementation evidence.
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
