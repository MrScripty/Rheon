# Native water training preview

`rheon-training-viewer` inspects a pinned recorded water simulation. It uses the
same voxel geometry, palette, camera and mesh path as the regular Rheon desktop.
Water velocity, passive tracer and pressure magnitude have separate views, with
translucent voxel opacity and a fixed scale over the whole loaded timeline.

```bash
cargo run --offline --locked --features desktop --bin rheon-training-viewer -- \
  --session /path/to/preview-cube-long/session.json \
  --session /path/to/preview-rect-long/session.json
```

Choose a simulation in the top selector. Playback stays in that scene until you
select another one. Pause and scrub recorded states to compare the marker over
time. Drag to orbit, Shift-drag to pan, and scroll to zoom. A path can also be
loaded manually. Loading an invalid file clears the preceding scene.

The water fills the sealed domain; the tracer is only a passive marker. Cyan
arrows show the actual cell-centered corrected velocity. The orange widget shows
the prescribed force's Gaussian core and base direction. Its inspector reports
the moving center, core/support radii, acceleration, pulse, swirl, fluctuations,
frequency and spatial modes. The support is clipped by the domain. Prescribed
fluctuations do not establish that turbulence is resolved at this grid size.

The producer is surrogate lab's versioned `water-preview-v1` probe. It replays a
reserved training scene through the native solver without changing its physical
inputs. The producer checks all originally retained fields byte for byte at
native indices 15 and 63 before publishing. A longer preview continues the same
scene after step 64 for observation; these later steps are not silently added to
training. The viewer displays recorded provenance, not an independent physics
qualification. It accepts a bounded, finite, increasing timeline and uses actual
timestamps when samples are decimated. Every recorded cell remains cubic.

The original one-second prefix can show less than one voxel of tracer movement.
A longer continuation reveals more transport. Rheon's semi-Lagrangian tracer is
diffusive and not mass-conservative; a widening marker alone is not evidence of
physical stretching. This preview does not run the AI solver or claim model
accuracy. Use `rheon-ai-compare` for independent native/AI simulations and signed
pressure differences.

Verification covers known one-cell tracer translation in the native producer,
exact original replay fields, physical-time playback, frame resets and missing
references, invalid reloads, cubic translucent geometry, and moving tracer
geometry through the actual egui frame-selection path. The ordinary desktop
regressions also run because its voxel rendering is shared.

The local branch `feat/live-training-force-preview` is a private proposal based
on `66e5038`, owned by the surrogate-lab task for Jeremy's review. It targets the
existing live-comparison work before integration to Rheon main. No push or merge
is authorized. Its worktree is retained while the local viewer and Tuldok project
reference this source; after accepted integration and migration of those
references, remove the clean task worktree with its commits protected by the
accepted branch. Existing dirty Rheon worktrees remain outside this change.
