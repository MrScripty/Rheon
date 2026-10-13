# Issues and dispositions

- Prior pressure-pass and initial-residual experiments were rejected. Preserve their
  branches/data; they are not acceptance evidence for warm starts.
- PR55 is stacked. Use its frozen head3525c979 as implementation base, separate
  feature branch; compare against that head, not a misleading main-only diff.
- Owner contract received via parent and recorded in owner-contract.md. No direct
  cross-thread channel exists here; parent remains coordination path for conflicts.
- First implementation uses coupled owner's existing accepted pressure. Standalone
  warm allocation and changing-surface/provider warm starts remain deferred.
- Public provenance must distinguish initialization from provider identity and
  full-step acceptance from PressureSession qualification. Dense3D wire format
  does not carry this new history; no book evidence or wire-version claim changed.

- Saturated fraction1 may reject tiny transport overshoots. Original bounds stay
  strict; tests preserve accepted seed/state on that failure. Benchmark both
  policies' acceptance, do not clamp or exclude failed runs.
- Abrupt force removal can make an accepted previous pressure a poor guess; pilot
  switch was slower. No hidden cold retry or source-based invalidation was added.
  Judge complete recorded horizons and disclose adverse individual frames.
