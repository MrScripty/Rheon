# Issues

- Inspector floating window overlaps CAD controls/footer: in scope, replace with
  scrollable sidebar tab and verify at native minimum size.
- API48 supports transport only: preserve separate particle path using PR47;
  no invented material-session contract or fake particle pressure slices.
- Constructor pressure is allocated zero storage, not a solved pressure field:
  expose availability and stage, never advertise it as a physical measurement.
- Desktop ten-second lifecycle test is load-sensitive: run final native suite
  serially without competing test/build jobs; do not weaken the threshold.

Resolved: floating inspector replaced by dock; screenshots and geometry test show
no overlap at both supported sizes. Constructor pressure remains unavailable and
accepted interval/gauge explicit. New PR53 MaterialSnapshot is supported directly
by VTK; desktop's existing material worker capture is preserved.

Remaining milestones (not implied complete): large-domain performance; background
export without capture stalls; replay/import; GPU/native touchpad qualification;
material session migration coordinated with desktop worker owner; free-surface
and obstacle session integration only once their canonical API contracts exist.
