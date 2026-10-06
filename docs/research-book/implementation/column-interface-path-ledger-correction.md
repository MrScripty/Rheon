# Column repair path-coverage count correction

This narrow successor preserves `bc66591` and the original column/CI repair
receipts unchanged. Their claimed 23 examples and 52 event/path controls were
incorrect. The bound source has 22 tracked examples. The original test walked
those examples plus two verifier paths and one final-PNG evidence path for each
of `pull_request` and `push`: 25 paths per event, 50 controls. The frozen summary
counter overstated coverage by two. Its counter is not qualifying evidence.

The current test now builds an explicit ledger from this checkout's tracked
Git inventory and executes its assertions over those exact rows. An instrumented
runner records that test's actual ledger normally and under optimized Python.
The new receipt and verifier derive all counts from the executed rows and the
frozen source tree; neither accepts a hard-coded summary count. The historical
52 claim is explicitly superseded, not edited into apparent prior success.

The all-example and column verifier/demo path filters, 30-minute budget, every
original Rust mode, smoke/lifecycle assertion and job layout remain unchanged
from `bc66591`. No Rust, Cargo, native evidence, old figure, physical gate or
forcing-feature source is changed. Prior successful artifact-corruption and
local CI runs are preserved; this correction tests the changed path-ledger
method and source/evidence binding. It does not repeat an unchanged hosted job.
A new hosted CI completion remains required. Older viscosity vector-transition
and later wet-pressure qualifications remain separate, unresolved limits.

The packet also binds the exact ordered five-commit parent chain from
`35b00247` through `bc66591`, including source `6e62e2b9`, its parent `300366af`
and evidence `d08ad5cd`. This lane remains separate from uniform forcing.
