# Plan: what Rheon needs before it makes the next models' datasets

**Status:** proposed 2026-11-19, waiting for the owner's approval. Nothing here is built.

**Owner, 2026-11-19 (in short):** new models are trained only on data from the current Rheon, never mixed with the earlier data. The goal is a general fluid solver: fluids defined by their properties, several fluids in one domain (water and air, splashing with a force), many kinds of force (push, vortex, random vibration, waves; several per scene, not only sinusoidal), and walls that are not always hard (open water, inflow and outflow through a tunnel, wrap-around "portal" walls). The dataset must be well defined and not redundant, and it must fit training within 24 GB of VRAM. Two model branches follow: speed (real time, games) and accuracy (engineering). "It may be necessary to do further development on Rheon … or further research."

## What Rheon has today (main, 2026-11-19)

The full inventory is in the pull request that adds this file. In short:

| Goal | Today | Where |
| --- | --- | --- |
| Run from Python, batch | No binding. The `rheon` CLI runs only the smoke carrier. `SimulationSession` exists in Rust | `src/session.rs`, `docs/api/simulation-session.md` |
| Forces | `BodyForce` (constant over a box, any number, changeable per step). `ForceSnapshot` (directional, radial, vortex; sphere or box support; up to 256) only on the closed-box `Simulation`. No time functions, waves, vibration or moving paddles | `src/forces.rs`, `src/force_field.rs` |
| Fluids by property | `FluidMaterial`: density, constant (Newtonian) viscosity, presets. Surface tension only as an approximate cohesion term in the particle preview. Nothing non-Newtonian | `src/material.rs` |
| Water and air | Two-material voxel model: variable-density projection, viscosity by material, gravity. Its limits: only gravity as a force, 4–64 cells per axis, serial CPU f64, first-order interface (it smears), no surface tension | `src/voxel_material/` |
| Boundaries | Sealed box (free-slip). Prescribed inflow/outflow as one uniform speed per whole face. Periodic only in research-scope solvers. Open/absorbing and portal walls: a written plan only (`fluid-portals-open-boundaries`, deferred) | `src/box_flux*.rs`, `docs/plans/fluid-portals-open-boundaries/plan.md` |
| Obstacles | One axis-aligned box in the grid. Collision meshes only for particles | `src/static_obstacle.rs`, `src/collision.rs` |
| Writing data | SQLite recordings (one blob per field per step), VTK (no writer for the voxel model), dense3d JSONL | `src/recording/`, `src/vtk_export.rs` |
| Speed | About 20 ms per step at 16³ and 0.27 s at 32³ (one worker, cold Jacobi PCG). No figures for the voxel model | `docs/plans/accepted-pressure-warm-start/measurement-tables.md` |
| Accuracy | Rheon's own audit finds no validated 3D water solver yet | `docs/audits/general-fluid-20261019/report.md` |

## Work, in the order that unblocks datasets

| # | Work | Size | Kind |
| --- | --- | --- | --- |
| R1 | **A scene-file generator.** `rheon-generate scene.json --out <dir>`: grid, material(s), initial contents, a list of forces with time functions, boundaries, duration and output interval. Writes fields at the output interval in a dataset format the lab reads directly (`.npz` or `.safetensors` per scene, with a small JSON index), plus the scene and the Rheon commit. One process per scene, CPU threads chosen by the caller | M | engineering |
| R2 | **Forces that change in time.** Time functions (constant, sine, pulse, smooth random noise, ramps), base vibration (the whole box shaken), travelling-wave makers at a wall, swirl and jets, several per scene. `ForceSnapshot` accepted on every route, including the session and the voxel model | S–M | engineering |
| R3 | **The voxel water/air model, made usable for data.** Forces in (R2), inflow/outflow faces, the `parallel` path, more than 64 cells per axis, a Courant-limited step, and a dataset writer | M | engineering |
| R4 | **Periodic and portal walls** (plan M1: one periodic axis, conservative momentum transport across it) | M | engineering; the plan exists |
| R5 | **Open water, first approximation.** An absorbing ("sponge") layer next to chosen walls that relaxes velocity and surface toward a far-field state, so waves leave instead of reflecting | M | engineering, standard method |
| R6 | **Sharp, splashing surfaces.** Interface transport that does not smear (VOF with geometric reconstruction, or a level set), robust at a 1000:1 density ratio | L | research first |
| R7 | **Surface tension on the grid** (continuum surface force from the interface curvature) | L | research |
| R8 | **Non-Newtonian fluids** (shear-thinning and yield-stress laws on the variable-viscosity path); viscoelastic ("stringy") later | M; L | light research; research |
| R9 | **Open boundaries done properly** (plan M2: pressure/reservoir faces) | L | research first |
| R10 | **Moving paddles and obstacles** that push fluid | L | research first |
| R11 | **Accuracy evidence** for an engineering tier: refinement studies and comparisons with known solutions for each capability as it lands | L, ongoing | research |

R1–R3 are enough for a first new-Rheon dataset: water and air in a box, many kinds of force, any density. R4 and R5 add the tunnel and open-water scenes. R6–R10 make the dataset richer as they land; each new capability is a new scene family, not a change to the earlier ones.

## The dataset itself (in the lab, alongside R1–R3)

- A **forcing library** in the lab (`surrogate/`): push, swirl/vortex, jet, random vibration, waves, several per scene, each with time functions. Parameters are drawn with a low-discrepancy sequence across force kind and strength, material, grid and density.
- Coverage is checked before generating more, using Tuldok's curation tools (embedding map, near-duplicates, thin regions), so the data stays small and varied rather than large and repetitive.
- Every dataset is labelled `generation: rheon-v2` with its Rheon commit. Tuldok refuses a model that mixes it with the earlier `rheon-reference-v1` data.
- The lab's playground gets each new capability as a backend option, so every scene family can be seen before it is generated.

**Compute:** R1–R5 need only small CPU runs to test. Generating the full dataset and training wait for the owner's word.

## Decisions needed

1. This plan and its order (R1–R3 first).
2. R1's output format: recommended `.safetensors` per scene with a JSON index (read by Tuldok without pickles, memory-mappable for training).
3. Whether to start research on R6 (sharp surfaces) in parallel with R1–R3, since splashing water is a headline goal and it is the longest item.
