# Shared browser view used by the research book

`view.js`, `shell.css` and `package.json` are copied byte-for-byte from Rheon's
browser GUI at `bdd64de8dbc9d81e3bfeeeb5eb990ff877a25415`
(`development/focused-gui-current-main-20261009`). This is a narrow source reuse,
not a merge of that development branch or its solver/posing work.

The book uses `RecordedView`'s renderer, camera, orbit controls, lifecycle and
scene group. Its six reference geometries retain fixed physical display scales;
they do not use `show(profile)`, which normalizes each profile independently.
The shared shell stylesheet is scoped under `.example-workspace` during build.
The builder copies the existing locked Three.js/OrbitControls locally.

No native pressure API, simulation, training or Kenoma component runs here.
