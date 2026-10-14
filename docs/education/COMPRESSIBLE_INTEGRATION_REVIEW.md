# Combined book and compressible research source review

This continuation merges frozen PR67 `24b38606e8c8354e56a2f83f1d0c44d6cd7b932e`
and PR68 `5bd5079b42c26bc021df7551d8ae49ce7a63dda9` on a separately named
integration branch. PR67 remains unchanged. Main and Pages remain held for
actual image review. Production compressible implementation is not started.

The eleven conflicts are resolved by retaining the existing reader/workspaces,
WASM packet, native qualification and source-admission paths, then adding the
compressible packet/qualification alongside them. The builder accepts both
packets, keeps the text/split/example modes and all prior laboratory links,
and adds the three chapters and separate computed Euler field laboratory.
PDF and Markdown source bundles include both continuations.

Every existing proof-module digest is preserved; only the reviewed root import
and audit files change. The new Compressible module is byte-identical to
PR68's reviewed digest. Every shared theorem module is unchanged. The root
imports all modules; the expected audit list unions the existing 275 names and
twenty compressible names. Counts are 224 public statements and 295 explicit
expected declarations. The fresh combined pinned kernel run
[38493519897](https://github.com/MrScripty/Rheon/actions/runs/38493519897)
passed 355 audited declarations and both negative-probe runs. Its checked merge
`d427059b16848adbf7e00f30e0e76ac24c021e50` has the identical tree as
source `1be45f33d52c5e354fccb4080413c1d57f8502be`. Both historical
qualification records retain their original inventories. Current metadata
records this new exact-proof-byte receipt; final whole-source qualification
remains separate.

Both native admission inventories remain byte-identical to PR67, including
the separately reviewed 73-path runtime epoch. No Rust, Cargo, operator, oracle
or native qualifier source changes. No hash refresh or qualification fallback
is introduced. Fresh clean-source native qualification is still required.

Browser qualification retains `chromium_sandbox=True`, including the source-only
preview. The explicit Chrome path uses the same already-supported hosted
renderer as PR67. The preview's time budget accommodates the combined reader
checks. Enabling the back/forward cache allows the new cached-viewer restoration
check; it does not remove a control, field comparison or admission gate.

On 2026-10-14, a fresh local sandboxed launch at exact PR67 head/tree failed
before any page opened: Chromium requires the SUID helper to be root-owned,
but `/usr/lib/chromium/chrome-sandbox` is owned by nobody. No sandbox flags,
ownership or security settings were changed. The launch record is external
`pr67-sandboxed-renderer-check.json`. Supported artifact download also returned
HTTP403. Actual desktop/mobile image inspection therefore remains blocked
until an authorized renderer or readable images/receipt are supplied. Hosted
geometry/raster/control tests are distinct from that image inspection.

Combined native, Lean, full PDF/link/browser and runtime checks must run on the
new committed source. A source-only preview does not qualify native runtime
composition, and no earlier whole-tree receipt is relabelled as current.
