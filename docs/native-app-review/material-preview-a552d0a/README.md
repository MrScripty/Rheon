# Native material preview review assets

These three inspected captures are retained in Git at the user's explicit
request for the Rheon app image, book and portfolio review. They are separate
from PR42's native-app screenshots. No renderer or solver was changed to make
the captures.

**Recommended primary:** [material-accepted-orbit.jpg](material-accepted-orbit.jpg).
The [front view](material-accepted.jpg) shows the same held state from a second
camera angle. [Results](material-results.jpg) expands the completed-run summary.

Suggested primary caption: “Rheon's native material preview at held step 1023,
shown from an independently controlled camera. Quantized parcel visualization;
engineering surface and load reconstruction remain future work.”

## Actual source and state

- Clean capture source: `a552d0afda3cf73ff1cc789746898a3ba0f332d3`.
- Source tree: `0b6d6aeef04aede631dc333fc64caa60f53db84d`.
- Native desktop binary SHA256:
  `eef52179384e2df667096658fdb56cc35a0d5a46fcce83e3546e33cd0aa99bd1`.
- Held frame: **1023/1024**, displayed **1.061 s**, **87 particles**,
  **21.202002 kg**, **20.0 °C**. Both angles retain this frame.
- Results disclosure: completed run at **1.0621 s**, accepted maximum speed
  **3.175 m/s**, maximum pressure used for forces **0.0 Pa**. These are the
  final-run diagnostics, not a recomputation for the selected historical frame.

This is the actual native material solver and its quantized voxel preview,
not CAD/free-surface reconstruction or calibrated optics. The record contains
displayed UI readings, not a full-precision particle-state export. Pressure is
evaluated before motion and wall-temperature update; accepted speed describes
a later stage. Calculation timings shown in the UI are not isolated benchmarks.

## Evidence

[Original provenance](provenance.json) retains the actual capture identity,
native binary hash, source hashes, display details and interaction record.
[Handoff](handoff.json) preserves the original visual review and precision
limits. Their wider capture inventories refer to additional internal images
and executor paths; only the three JPEGs linked above are included here.
[Asset hashes](asset-sha256.json) bind these selected files and the unchanged
receipts. [Source comparison](native-source-match-b9d9d87.json) verifies that
all 45 captured native source files match book implementation commit
`b9d9d871bccc1590edb61da84b132f3aef9cd271`. It does not relabel the capture as
having occurred on that later commit.
