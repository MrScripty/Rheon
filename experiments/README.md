# Opt-in aligned Stokes experiment

This directory is deliberately outside Rheon's production library. The
experiment modifies an explicitly supplied binary64 MAC velocity state by a
bounded, unforced viscous update and a pressure proposal. It admits one padded,
exactly grid-aligned internal stationary box, no-slip obstacle walls, sealed
free-slip outer walls and one constant positive density/viscosity.

Read the [derived contract](../docs/research-book/implementation/experimental-aligned-stokes.md)
before invoking the example. A requested step commits only if an outward row
bound, separate actual energy changes, per-face momentum-equation defects and
all wet-cell divergence enclosures pass. Refusals preserve the caller state
and the previous accepted pressure/report/witnesses. Ordinary nearest-rounded
estimates and solver residuals cannot authorize this transaction.

```
cargo run --locked --no-default-features --example aligned_stokes -- unit-curl /tmp/rheon-stokes-new.json
cargo run --locked --no-default-features --example aligned_stokes -- unit-curl 3 /tmp/rheon-stokes-three-new.json
cargo test --locked --no-default-features --test aligned_stokes_contract
python3 tools/qualify_aligned_stokes.py /absolute/external/evidence --lean
```

Each example destination must be a new JSON file outside every Git checkout.
The examples export actual accepted states and certificates; the three-step
example repeatedly updates the same caller field. The qualification destination
must be a new directory outside Git. The default
qualification checks fresh native/rational evidence; `--lean` additionally
kernel-checks the isolated module, its namespace axiom audit and rejection
probes using the repository's pinned Lean/Mathlib environment. A native-only
receipt explicitly records that Lean was not rerun.

The enclosure backend assumes Linux x86_64 and Rust's documented default
scalar binary64 semantics, without altered floating-point controls. Its
observational probes can refuse a detected violation; they do not verify the
compiler, control registers or external code. Nonfinite/subnormal arithmetic,
overflow/underflow and uncertain acceptance inequalities are refusals. Lean
assumes sound enclosures and proves exact-real conditional implications; it is
not an IEEE implementation proof.

The example and integration tests opt in with a path import. Nothing enables
this workspace in `Simulation`, the liquid facade or the default library.
The experiment has no advection, material transport, forces, capillarity,
free surfaces, moving-solid work or independent simulation clock. Workspace
payload caps include the borrowed strain's reported payload and owned vector
capacities, but exclude the geometry owner, caller arrays, stack, allocator
metadata and RSS. The workspace and its audited operator/pressure call paths allocate no heap
memory during a step. Caller cancellation callbacks remain caller code.
