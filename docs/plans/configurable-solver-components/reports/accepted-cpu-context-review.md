# Independent accepted Session CPU contract review

Reviewed 2026-10-15 by design_review (GPT-6.1 Sol), read-only source review of
`/workspace/rheon-accepted-runtime`, four-file uncommitted slice over
`b41e9053d1f0e04fe415bf3903b98e93f160b2b5`.

## Result

No remaining material source or fixture findings in the exact files pinned below.
Source-approved to proceed through exact-tree tests, independent source admission,
fresh qualification and applicable hosted gates. This document is not a test
execution receipt, complete A2/A3/A4/A8 acceptance, measured-model parity, a
production GUI/runtime bridge, or an inference performance/accuracy claim.

The only production change adds synchronous cancellation polling to the existing
ModelProcess write/reply checkpoints. Legacy typed prediction and startup receive
use a false callback. A true callback latches the existing AtomicBool and kills
the same owned ChildSlot; Drop retains its existing child wait and I/O joins.
Dispatch still means acceptance by the writer queue, not child consumption.
Consumed complete invalid replies preserve the existing reason/digest precedence;
runtime EOF without an invalid payload can still classify as cancellation when
the shared flag is set. No general invalid-response-versus-cancellation ordering
claim is made for a queued reply that the consumer has not taken.

The callback is cooperative. Existing receive/write waits are 20 milliseconds,
not 25 milliseconds, and neither number is a latency or hard shutdown guarantee.
A reply can win before another callback poll; accepted Session post-prediction
and final checkpoints remain authoritative. Cancelling a process owner kills
that runtime permanently; retries use a new runtime owner. Descendant processes,
hung callbacks and guaranteed teardown deadlines remain outside the contract.

## Scientific and test scope

The new seven-test child module is registered under the existing CPU contract
module. Its Driver is test-only glue from the public ContextPressureProvider
interface to the existing pinned ModelProcess; it is not a shipped production
adapter or desktop execution path. It checks declared package physical support
before prediction. Dynamic identity pins are fixture declarations, not measured
trained weights. The final runtime description says test-only stdlib CPU
subprocess, avoiding an ambiguous measured-model label.

The independent Python fixture admits exactly two sealed cells, counts [2,1,1],
spacing 1/4, zero origin, density 2, dynamic viscosity 1/128 and timestep 1/16.
A uniform binary X acceleration 1/8 produces a nonzero compatible pressure RHS.
The fixture validates arbitrary named ports against semantic meaning, units,
shape/layout, physical centers, accepted run/generation/time lineage and exact
b-A*p_prior. It solves the authoritative two-cell integrated equation independently:
k=h/rho and p=[0,b1/k]. Full-pressure and correction packages exercise host
reconstruction exactly once, following one native accepted prefix interval.
This is a bounded equation/protocol/composition fixture, not a general flow,
physical-applicability or learned-accuracy benchmark.

The tests compare pressure/fraction/velocity/tracer bits and accepted clocks,
generation and stamp versions with existing viscous native transport. They
require actual configured selection after the seed, same-request support/quality
fallback, native next-prior ownership and exact metadata/array accounting.
Actual child request logs and markers establish consumed requests for the
in-flight cases. Exit/stall-once fixtures permit a fresh owner to retry against
the same accepted request and preserve quota. Cancellation tests include
before-dispatch refusal and a large request to a child that does not read stdin,
exercising blocked-writer cancellation and I/O ownership.

The in-flight test uses a 500ms deadline and observes return within 2 seconds;
these are test windows, not advertised API bounds. Failed intervals retain the
prior dense3D publication and complete prior configured receipt, and retry
observations compare generation/time/RHS/prior/prefix with the failed attempt.
The final accepted retry consumes the remaining quota and the next call refuses.
Linux drop checks use the existing /usr/bin/kill probe and empty ChildSlot; source
ownership still comes from wait/join. The new module does not add a separate
positive PID-probe assertion for every new process, so these checks are not
described as a new independent OS-level liveness proof.

Typed PredictionFailure reason, dispatch and invalid-payload digest remain in
the test Driver.failures vector when mapping to the coarser core provider
rejection. That preservation is test-local and in memory. It establishes no
durable typed failure/report schema or cold consumer. Existing default-call
success/legacy/malformed-reply digest contracts remain in the existing prediction
tests; their production consumed-reply handling is unchanged by this slice.

## Reviewer actions and limits

Reviewer inspected production diff, registration, complete new Rust test module
and Python fixture, and checked hashes and git diff whitespace (passed).
Reviewer ran no Cargo command, test, adapter, model or numerical experiment.
Execution results reported by the lead/fixture owner are not independent
execution receipts here. No repository source was edited; only this external
review document was created. Fresh composed-source validation and inventory
rebinding remain the lead's responsibility.

## Exact reviewed SHA256

| File | SHA256 |
| --- | --- |
| `src/bin/desktop/ai_comparison/model_process.rs` | `77070b2ae78dabf8f2d046a0b83f97053ef6460f8c2f19f47dcc3264e34f9754` |
| `src/bin/desktop/ai_comparison/cpu_contract_tests.rs` | `d7f2be7ea559891450f4dc1baae47183615bab8b999bc9bfe9d0151bb4d27580` |
| `src/bin/desktop/ai_comparison/accepted_provider_contract_tests.rs` | `040ffd5ad5500ffa0275a61f99d0f3105618597175355db526393645c94bbf56` |
| `tests/fixtures/accepted_pressure_adapter.py` | `958ba74ebde596aac8375eb436898a1e2342ba1b7868c08bfee99c09c02dc27c` |
