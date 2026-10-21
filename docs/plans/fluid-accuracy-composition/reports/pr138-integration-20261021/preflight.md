# Retained PR138 preparation results

Preflight examined retained PR138 source
`a6bfcc94481b1bafbe61be804a1abc3426927bbd`, before PR142 integration. It does not
qualify the future combined source. The new protocol was authored during this
preflight; implementation, Cargo and test bytes remained unchanged. No clean
whole-tree qualification receipt is claimed for these preliminary checks.

Independent read-only review by delegated reviewer `/root/review_pr138` compared
that exact head with `8bf542eddd8198008b6d77d52e009e9384998f18` and found no material
defect. It verified PR132/134 ancestry, all 28 recorded component pins, exact
tested implementation/test bytes, complete membership and hashes of the 135-input
selector, the historical review's exact prefix and frozen v1/v2 inventories.
Only the two Cargo files change in the selected source closure. No production
`src`, CI, qualifier or oracle changes belong to this reference delta.

The reviewer identified the deciding stale-composition limit: retained PR142
`4f2d55323825c8907cf9209914f29130ec7de511` selects 140 inputs, with five additional
paths and 13 differing shared hashes versus PR138. The 135-input map cannot
qualify combined source. A read-only Git merge-tree comparison predicts seven
shared-document conflicts, including both native-admission records and canonical
composition status; no reference or production source conflict occurs between
those two retained heads. This is preparation, not conflict resolution against
the lead's eventual repaired source.

Fresh local checks with Rust/Cargo 1.92.0 and Python 3.12.14:

- `cargo test --locked --no-default-features --test grid_geometry_reference_contract
  --test grid_coupled_reference_contract`: 9 geometry and 2 coupled tests pass,
  zero failed or ignored. The existing five-artifact geometry regression completed;
  neither sealed scientific field campaign was rerun.
- Strict Clippy on both private examples and both contract adapters with
  `--locked --no-default-features` and `-D warnings`: passes.
- `cargo tree --locked --no-default-features --edges normal`: all three newly
  added exact-arithmetic crates are absent from the normal production graph.
- Downloaded Cargo registry archives for num-bigint 0.4.8, num-integer 0.1.47 and
  num-rational 0.4.2 match the lock checksums. Their actual MIT/Apache license bytes
  match every retained dependency-review hash. Existing package identities remain
  unchanged; no version broadening occurred.
- Both component pin manifests independently rechecked: 16 PR132 and 12 PR134
  identities match and exact owner heads remain ancestors.
- `git diff --check`: passes.

Raw logs and dependency graph remain outside Git in
`/workspace/pr138-evidence/preflight-a6bfcc9/`; the build target is
`/workspace/pr138-build`. Jobs 2, incremental 0 and dev/test debug information 0
were used. These outputs remain preliminary and are not borrowed for final
combined-source native/packet acceptance.

GitHub read-only metadata confirms workflow run
[39367910080](https://github.com/MrScripty/Rheon/actions/runs/39367910080), attempt 2,
completed successfully on exact retained `a6bfcc9`. Attempt 1 cancellation and
aggregate failure remain historical. No rerun was requested here.

Final dependency remains pending: PR142's landed head, lead agreement for shared
admission reconciliation, fresh complete-source review/qualification and required
hosted checks, then the lead's exact-head merge authorization. No main merge,
cloud-task access, historical artifact download, branch deletion or force push was
performed. No additional scientific or whole-application production claim follows.
