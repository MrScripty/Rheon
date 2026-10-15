# Independent Session constructor configuration review

Reviewed 2026-10-15 by design_review (GPT-6.1 Sol), source-only review of
`/workspace/rheon-accepted-session` over
`c39aa2300b156292bad6d03aaaf8dacc2127064d`.

## Outcome

No material findings in the two-file immutable getter change, the two new public
contract tests, or the added configuration paragraph in the API document.
Source-approved for exact-tree validation and rebinding the existing selected
`src/lib.rs` and `src/session.rs` inventory entries. This is not a test execution
receipt, accepted recording/persistence admission, or complete-plan acceptance.

`SessionConfiguration` is stored from the exact constructor arguments only after
both numerical and boundary-workspace owners construct successfully. The
snapshot returns a shared immutable borrow; its public Copy value can be changed
only as a detached caller copy. Public export is additive. No mutable Session
configuration access, caller Debug parsing or reconstructed future recipe is
introduced. The physics, commit path, workspace matching and array accounting
are unchanged by this getter.

The transport memory limit is the original logical constructor budget, not the
internally adjusted carrier-only limit. `LiquidTransportSimulation::new` partitions
that logical budget among carrier, represented-volume and held-pressure arrays;
the stored raw controls deliberately retain the original total limit. The
separate boundary-workspace limit is likewise a constructor cap rather than
actual allocated bytes, and max_steps is the original quota rather than remaining
quota. Physical allocated-array reporting remains the existing owner API; neither
these logical limits nor allocated-array counts claim whole-process RSS.

Per-call viscosity, provider binding, requested timestep, forces, liquid controls
and future schedules remain separate execution intent. The getter cannot infer
them from a completed step. Applied viscosity and actual shortened interval
belong to the accepted report, while initial field data and geometry remain
separate constructor inputs/views. A recording or frozen-run consumer must retain
those additional required inputs explicitly.

## Test quality

Both public tests cover Jacobi and symmetric Gauss-Seidel implementations. The
first compares every current configuration field (floating controls by exact
bits), deliberately distinguishes numerical and boundary caps, admits distinct
represented density at construction, and mutates every field of a detached
configuration copy before checking the Session and encoded publication unchanged.

The second checks readback after real accepted configured viscous publication,
invalid-viscosity failure, final-checkpoint cancellation with a different attempted
viscosity, and terminal stop. A nonzero initial volume-stamp version distinguishes
original constructor controls from evolved accepted state. It verifies that the
cancelled per-call viscosity does not replace the prior accepted coefficient.
These are meaningful authority/readback regressions; they do not claim numerical
accuracy or a completed persistence codec.

The added API paragraph accurately calls these original logical constructor
controls and separates per-call intent and accepted applied observations.

## Reviewer verification

Reviewer inspected the exact getter/export diff, both new tests, API addition and
existing budget partition in `LiquidTransportSimulation::new`; SHA256 pins and
`git diff --check` passed. Reviewer ran no Cargo command, test, model or numerical
experiment and edited no repository file. Only this external review document was
created. Execution results and fresh qualification remain owned by the lead.

## Exact reviewed SHA256

| File | SHA256 |
| --- | --- |
| `src/session.rs` | `2caa26be33d5497609bfe1b6f16456d217372119d0720a989697fbf5debb0544` |
| `src/lib.rs` | `91f3cdc1a9b0f782e2efcc647eb956230b201e6b0a3429e4b2cd4955f5f2a2eb` |
| `tests/session_configuration_contract.rs` | `4f53c8d35eeb44819791341b8ce1a80421d9562ab8413f528a3234da233632f7` |
| `docs/api/configured-pressure-session.md` (added configuration paragraph) | `d8fd2e4098b3cab3d4adc367e36f95e3237a7103ac71fe1878bd3257f8715af2` |
