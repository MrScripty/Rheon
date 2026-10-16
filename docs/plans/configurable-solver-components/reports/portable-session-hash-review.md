# Portable Session hash dependency independent review

Read-only review of the sole current `Cargo.toml` delta in `/workspace/rheon-accepted-session`, base `da2e12cf921b2fe5e93337550f792876b32fd9bb`. No Cargo or browser execution performed by this reviewer. The lead owns the active WASM build and later fresh qualification. `git diff --check` passed.

## Assessment and admission pin

No material finding. Approve rebinding the native source-admission inventory's existing `Cargo.toml` entry to SHA-256 **`10a7b438ec5e7bdb2c8eb95137cf5a42d1cd35687ed47bf63b05a3e42dc1674e`**, followed by the required fresh complete source-closure qualification. This approval is for the reviewed manifest bytes; I did not edit the inventory or assert those checks have passed.

The change moves the same unconditional, exact `sha2 = "=0.10.9"` entry from the non-WASM dependency table to common dependencies and adds one explanatory comment. There is no package version, dependency version, SHA feature declaration, Cargo feature/default selection, target list, native capability gate, numerical source or acceptance-bound change.

`src/solver_provider.rs` publicly registers `configured` on portable builds, and `src/solver_provider/configured.rs` unconditionally imports `sha2::{Digest, Sha256}` for exact observed RHS/prior-pressure custody hashes. The browser solver depends on this core crate with default features disabled. Keeping sha2 available only under `not(target_arch = "wasm32")` is inconsistent with that portable module. Moving the existing pure hashing dependency is the direct scope correction; it does not move glTF, filesystem capabilities, SQLite or native UI dependencies into the browser.

## Locks and regression scope

Both checked-in locks already contain the same sha2 dependency and its resolved dependency entries. `git diff --exit-code -- Cargo.lock browser/solver/Cargo.lock` passed; only Cargo.toml is modified in this reviewed checkout.

| Unchanged lock | SHA-256 |
| --- | --- |
| `Cargo.lock` | `3b5836a7be9f1290c698cadaff010e11b93e33b4af839c8549391c23e7d8ec0c` |
| `browser/solver/Cargo.lock` | `d77e825663daceafb6bf841a8ea38e98e1cede8e163a892f466c137afdeb7f90` |

The existing research-book workflow already selects Cargo changes and builds the actual WASM/native browser packet through `docs/education/build_solver.py`, then exercises build/flow/timestep contracts normally and under Python optimization. That builder includes Cargo.toml, both lockfiles, all core Rust source and browser source in its conservative source receipts. A successful native browser reference test alone would not catch the missing WASM dependency; the actual locked WASM build and newly generated complete packet checks are the deciding regression evidence. The lead's requested fresh whole-closure native/rational and browser-packet normal/optimized checks remain appropriate, without altered bounds or stale receipt reuse.

This review establishes the dependency scope and exact admission pin only. It does not transfer earlier native/browser receipts across the manifest change, assert that source hash identity proves authenticity, or claim current hosted Pages/build success.
