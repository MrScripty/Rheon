# Combined PR3 qualification

The integration preserves both reviewed histories: qualification commit
`7e1a76dd487549a496e04b9305fa531fcc7ab2f9` and harness repair head
`464eab005a8d363bd3e052370af86150ebba689c`. Merge source commit
`305acadf1e21a49961262531d621a3900c2672f6` combines them without conflicts.
The enclosing evidence commit changes only this evidence directory.

On that combined source, local Rust 1.92.0 checks pass:

- `cargo test --locked`: 49 tests.
- `cargo test --locked --no-default-features`: 44 tests.
- `cargo test --locked --features desktop`: 55 tests.
- Strict `cargo clippy --locked --all-targets -- -D warnings` in all three
  corresponding feature configurations; `cargo fmt --all --check`.
- `python3 -m unittest discover -s tools -p 'test_*.py' -v`: eight tests,
  covering real CLI success/timeout, explicit dt, wrong/missing dt metadata,
  invalid deadlines, TERM/KILL cleanup/reaping, cancellation, output ownership,
  selection and no subsequent job after timeout.

The release solver was reused, not rebuilt. Its SHA256 is recorded in
`receipt.json`. Rust source/tests exactly match the qualified commit; harness
source and documentation exactly match the reviewed repair. Both commits are
ancestors. Original numerical fixtures, research book and proof paths remain
unchanged from `0f2db35009624467a1f338700d8967a350443c93`.

The benchmark deadline is **optional and disabled by default**. Supplying
`--run-timeout SECONDS` enables a positive finite per-process wall-clock limit,
including warmups. Default execution is not bounded by this safeguard. It does
not impose a production solver or native desktop timeout. Timeout enforcement
depends on polling and OS termination; see `docs/COMPARISON.md` for lifecycle
and API details.

Archived benchmarks remain evidence of their original measured source and
binary. Their manifests are not rewritten to attribute measurements to this
integration. The archived `evidence/cloud-qualification/verify.py` was rerun in
the unchanged `7e1a76dd` checkout, because its source hashes deliberately bind
that historical snapshot. `historical-evidence.log` records successful checks
of all six matched comparisons and the original byte-identical demo fixtures.
Running that historical source-hash check against a newer harness should fail;
use the recorded snapshot to reproduce that check.

Fresh combined tests do not requalify native window interaction, shared-host
performance, other operating systems, accessibility, or full physical accuracy.
Hosted exact-head CI is recorded separately in the PR description/publication
receipt after pushing; these local logs do not claim its result. Trailing log
whitespace is normalized for Git checks. No PR merge or external review request
is part of this integration.
