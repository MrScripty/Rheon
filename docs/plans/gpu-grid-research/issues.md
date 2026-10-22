# GPU research issues and dispositions

| ID | Finding | Disposition / owner |
| --- | --- | --- |
| GPU-D01 | Parent confirms FEM occupies Chapters 30–36 at `9489aa3` | Resolved: only new GPU files moved to 37–39; parent owns shared integration, existing material untouched |
| GPU-D02 | Full book/Pages build includes scientific/reference/native qualification; Word builder lacks dedicated Markdown-table rendering | No build run under this task. Parent must inspect/authorize full scope and perform output review separately |
| GPU-D03 | No installed device/API/compiler evidence has been collected | Explicitly outside scope; future implementations must query and qualify each backend/device configuration |
| GPU-D04 | General evolving 3D free-surface, arbitrary mesh stress and coupled physics are not supplied by the bounded CPU carrier | Numerical research/qualification precedes port; Chapters 37–39 retain model-family gates |
| GPU-D05 | Moving official documentation is not an immutable SDK/toolchain snapshot | Bibliographies record consulted version/access date; future implementation pins actual environment |
| GPU-D06 | Existing host-slice pressure provider cannot establish whole-step device residency | Separate pressure-only diagnostic prototype from a later resident backend; future implementation owner |

No blocked or denied operation has occurred at this checkpoint. Publication,
hardware and scientific qualification are excluded work, not uncertain outcomes.
