# CPU scheduling contracts and the Pantograph comparison

Rheon's CPU scheduler changes the decomposition of a numerical kernel. It does
not choose physical timesteps, reorder physics stages, own accepted fields, or
schedule inference models. This note separates the implemented mechanism, its
exact finite contracts, and observations that remain tests rather than proofs.

## Actual ownership and execution order

| Layer | Actual owner and decision | Required boundary |
| --- | --- | --- |
| Accepted physical state | `Simulation` and its step/acceptance path | Scratch work may fail; publication still requires the existing acceptance gates. |
| Linear scratch work | [`parallel.rs`](../../../src/parallel.rs) and Rayon | Each mutable chunk is disjoint; the whole invocation joins before its consumer. |
| Batch feedback | [`scheduling.rs`](../../../src/scheduling.rs) | One run owns per-stage timing metadata; no numerical buffers are stored here. |
| SGS triangular dependencies | [`sgs.rs`](../../../src/sgs.rs) | Each coordinate-sum wave joins before the dependent wave; reverse order for the backward sweep. |
| Scalar numerical reductions | [`pressure.rs`](../../../src/pressure.rs) | Products and compensated accumulation retain their existing serial order. |
| Worker display | [`observation.rs`](../../../src/observation.rs) | Relaxed atomic markers are sampled diagnostics, not utilization estimates or a complete execution trace. |

The scheduler's composition roots bind the run owner around every scoped Rayon
worker. A nested binding restores its predecessor on normal
return and unwinding. `snapshot()` locks each stage independently; it is not an
atomic snapshot of the entire simulation.

For `n` items and `w` workers, the linear policy proposes the caller's preferred
grain capped at `ceil(n/max(w,1))`, then approximately one, two and four batches
per worker. Every proposal is clamped to 64–16,384 items. These are four
*candidate indices*: clamping can produce duplicate batch sizes. A small grid
or one-worker invocation can take the serial path and never enter this policy.
Particle-density and particle-force kernels have a lower parallel threshold
than grid vector kernels.

The feedback clock measures a complete joined kernel invocation, including
scheduling overhead. It is not a sum of worker times. Under the assumptions
below, the first eight successful calls try indices `0,1,2,3,0,1,2,3`; subsequent
calls use the candidate with the smallest recorded trial time. The trial scores
reset at phase 0 of every 128 successful calls. Changing the tracked
`(item_count, worker_count)` shape resets exploration. The displayed smoothed
cost is separate from these minimum trial scores. Neither statistic establishes
an optimum or a repeatable speedup.

SGS calibrates independently: for an eligible multiworker grid it compares
8³ and 4³ tile sweeps in the order `8,4,4,8`, then runs the selected sweep.
These scratch-only passes contribute real work and diagnostics but do not add
physical timesteps or PCG iterations. The workspace retains the selected tile
edge and last calibrated worker count. One-worker or thin-grid fallback returns
before calibration; passing through that fallback does not necessarily erase a
previous multiworker choice.

## Finite contracts and explicit assumptions

[`CpuScheduling.lean`](../../../proofs/Rheon/CpuScheduling.lean) defines natural-number
reference contracts, with seven definitions and fifteen public theorems:

| Checked statement | Concrete interpretation | Boundary not proved |
| --- | --- | --- |
| `batch_size_bounds` | Every clamped grain is positive and at most 16,384. | Rust integer conversion, allocation, performance. |
| `chunk_covers`, `chunk_unique` | For positive grain, each in-bounds index belongs to exactly one clipped half-open chunk. | Refinement of Rayon or Rust borrowing to the finite model. |
| `predecessor_rank`, `same_wave_has_no_dependency` | A read crossing one lower-axis tile boundary has rank exactly one smaller; same-rank tiles cannot have that relation. | Other stencils, diagonal couplings, or arbitrary task graphs. |
| `tile_wave_in_bounds`, `reverse_wave_dependency` | Valid rectangular tiles have bounded rank; descending waves reverse the dependency order. | Machine overflow and memory-model synchronization. |
| `trial_candidate_bounded`, `trial_first_round`, `trial_second_round`, `trial_periodic`, `no_trial_after_exploration` | The exact phase formula selects only indices 0–3, tries each twice in the first eight phases, and repeats after 128. | Four distinct grains or four equally good alternatives. |
| `candidate_revisited_within_period`, `successful_trace_count`, `successful_trace_explores` | An eligible candidate is selected within fewer than 128 further successful-completion positions in the stated trace. | Wall-clock progress, OS/Rayon fairness, or progress when calls keep failing. |

For example, ten indices at grain four occupy `[0,4)`, `[4,8)`, and `[8,10)`.
A 2×2×1 tile grid has waves `{(0,0,0)}`, `{(1,0,0),(0,1,0)}`, and
`{(1,1,0)}`. The middle two tiles have no nearest-axis triangular dependency
on each other. This example is about ready work, not an independence claim for
arbitrary physics operations.

The successful-trace premise has operational content: one outstanding invocation
of the same stage; stable item count, worker count and preferred grain; a
nonempty workload; and exactly one record after each successful joined call.
The natural-number model has no machine-counter overflow. A delay counts
successful completions before selection of the next invocation, never seconds.

Those premises cannot be removed. Two overlapping `BatchRun`s can both observe
phase 0; recording both moves the next call to phase 2. The new adversarial test
executes this counterexample. A dropped or failed pass does not call `complete`,
so repeated failure can retain one phase indefinitely. A whole timestep may
later be rejected even though some earlier kernels completed and supplied valid
kernel timings. Batch exploration is not an accepted-timestep counter.

Other tuning limits follow from the actual metadata key. Preferred grain is not
part of the tracked shape, so a changed preference can reuse an index scored
under an earlier proposal. Overlapping shape changes A→B→A can also make an old
completion look current because there is no plan-generation identity. These are
limits on interpreting the timing policy; the scheduler owns no fields that
could become a second accepted-state authority. This work changes no production
policy and makes no claim that these cases are fair or performance-optimal.

For SGS, preserving the per-cell X/Y/Z summation order is as important as the
topological order. Relaxed atomic bit loads/stores avoid aliased mutable `f64`
storage; they do not perform floating-point accumulation. Visibility relies on
the joined task execution between waves. The Lean rank statements do not prove
the Rust/Rayon memory model. Likewise, swapping the current serial compensated
reductions for unordered parallel reductions would need a new numerical
contract; the scheduling theorems would not justify it.

Rayon documents disjoint mutable chunk iteration and early termination on
fallible parallel work. If several tasks fail, its returned error is not
specified to be the lexicographically earliest one. Rheon's bitwise replay
checks for successful computations do not establish deterministic error choice.
See the pinned [Rayon 1.11 slice API](https://docs.rs/rayon/1.11.0/rayon/slice/trait.ParallelSliceMut.html)
and [fallible iterator contract](https://docs.rs/rayon/1.11.0/rayon/iter/trait.ParallelIterator.html#method.try_for_each).

## Pantograph: inference/workflow scheduling is a different layer

Here Pantograph means the user's
[MrScripty/Pantograph repository](https://github.com/MrScripty/Pantograph/tree/1fba975ab20cb032949aa7221e373f0719e3264b),
reviewed at commit `1fba975ab20cb032949aa7221e373f0719e3264b` on 2026-10-12.
The primary sources are its authored architecture and actual policy code, not a
performance paper. It is unrelated to the similarly named Lean proof-search
interface in arXiv:2410.16429.

Pantograph's scheduler owns workflow-task readiness, resource admission,
compatible batching, runtime/device selection and lifecycle policy. Its
architecture distinguishes these decisions from runtime launch, model storage,
and frontend presentation. The document also labels a transition toward durable
task execution; that target architecture alone is not evidence that every path
is already implemented.
[Primary architecture](https://github.com/MrScripty/Pantograph/blob/1fba975ab20cb032949aa7221e373f0719e3264b/ARCHITECTURE.md).

The reviewed readiness function defers a task when dependency evidence is absent.
Its completion-ranking policy compares bounded eligible offers using supplied
preparation, transfer and execution costs. It distinguishes measured from
synthetic evidence and checks comparison conditions, snapshot association,
sample count and age. Selection remains advisory until ordinary admission and
dispatch validation; the pure ranking function does not acquire a runtime lease.
[Readiness code](https://github.com/MrScripty/Pantograph/blob/1fba975ab20cb032949aa7221e373f0719e3264b/crates/pantograph-scheduler/src/readiness.rs),
[completion-ranking code](https://github.com/MrScripty/Pantograph/blob/1fba975ab20cb032949aa7221e373f0719e3264b/crates/pantograph-scheduler/src/completion_ranking.rs).

The separate frozen-cohort evaluator explores two legal serialized completion
events and then uses a deterministic greedy continuation. It has explicit work
bounds and refuses an incomplete comparison when required evidence is missing,
invalid or over budget. Its source explicitly distinguishes bounded work units
from a wall-time guarantee. This is not an unrestricted global-optimum or
starvation-freedom theorem.
[Bounded cohort code](https://github.com/MrScripty/Pantograph/blob/1fba975ab20cb032949aa7221e373f0719e3264b/crates/pantograph-scheduler/src/cohort_completion.rs).

| Pantograph concern | Rheon mapping | Limit of the analogy |
| --- | --- | --- |
| Evidence before task admission | Complete predecessor waves before consuming their values. | Runtime readiness evidence is not an SGS memory barrier. |
| Comparable completion-cost evidence | Measure the entire joined kernel pass, not worker busyness. | Rheon's four trial minima do not implement Pantograph's resource/residency model. |
| Explicit bounded alternatives and refusal | Four candidate indices, fixed exploration phases; failed passes do not train. | Rheon does not refuse all scheduling when timing is absent; it uses a hardware/workload proposal. |
| State/owner identity across dispatch | Keep numerical state ownership outside timing metadata. | Adding queued workflow tasks or model residency to a fluid step would be new architecture. |

**Engineering inference:** Pantograph's explicit comparison identities are useful
when considering a future Rheon feedback key that includes preferred grain and
a generation for overlapping plans. That is a design suggestion, not an
implemented change or measured improvement. Neither codebase's scheduling
policy proves Rheon's timestep stability, conservation, pressure convergence,
continuum accuracy, or CPU utilization.

## Focused qualification

The new module compiled directly with pinned Lean 4.19.0
(`6caaee842e9495688c1567e78c0e68dbb96942aa`). Printing the axioms of all 22
new declarations reported only the repository's existing allowed axioms:
`propext`, `Classical.choice`, and `Quot.sound`. This standalone check is distinct
from the root import, reviewed inventory and complete project audit, which own
the integrated qualification.

Six new internal Rust tests exercise the private scheduling decisions without
exposing a new public API:

- 256 successful nonoverlapping calls cover two full tuning periods.
- A real failed parallel pass leaves no successful sample; retry visits every
  index once, including the partial final chunk.
- Nested scheduler scopes restore the correct owner after unwinding.
- Two overlapping invocations witness the missing nonoverlap premise.
- All 125 rectangular tile shapes with axes 1–5 satisfy both sweep directions'
  cross-tile dependency checks, with completion marked only after a whole wave.
- Tile edges 1/4/8/16 on one/four threads overwrite NaN-poisoned scratch, produce
  identical output bits, and satisfy an independently reconstructed triangular
  residual within the stated test tolerance on the anisotropic fixture.

The focused command below passed ten selected tests (six new and four existing)
with Rust 1.92.0, no default features, and `parallel`. This is a contract test,
not a timing benchmark. Existing [`parallel_contract.rs`](../../../tests/parallel_contract.rs)
contains broader accepted-state, diagnostic, cancellation and replay checks;
those are not claimed as newly executed by this focused run.

```sh
cargo test --locked --no-default-features --features parallel --lib scheduling -- --test-threads=1
lean proofs/Rheon/CpuScheduling.lean
```

Production scheduler and solver behavior are unchanged. No speedup, CPU fairness,
strict latency bound, race-detector qualification or Rust-to-Lean refinement is
claimed. Historical timing observations remain tied to their original source
and machine in the [scheduling evidence](../../../evidence/scheduling/README.md).
