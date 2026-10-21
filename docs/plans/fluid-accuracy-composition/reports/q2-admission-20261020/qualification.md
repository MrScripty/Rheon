# Dependent exact-reference build qualification

Fresh qualification passed on clean commit
**`9a9904f3c6f0f96cbe2af9210a0c578f825ccee0`**, tree
`488ab45ed7151d20d17adba519d587550b1e983e`. This separately admits the PR132/134
reference build inputs and verifies aligned native export plus existing focused
contracts. Fixed PR133/8bf542e and its execution at `9c04278` are unchanged. No cut/PDE
method, production solver or physical accuracy is promoted.

## Source and admission

The component checkpoint cd21a995 retains exact PR132
`e6525a116a07af486e4a682cc2b157241d6b9150` and PR134
`f84f65104d6a0f6f8f1a9b903412b1e5155f72a2`, whose tested sources remain `0f4e7e0`
and `06b3190`. The [complete independent admission review](final-admission-review.md)
and [protocol-only follow-up](final-admission-protocol-spacing-followup.json)
approve **135 inputs: 0 added, 2 Cargo changes, 133 unchanged, 0 removed**. All current
Rust/toolchain/checker/qualifier/oracle/test identities are exact reused inputs.
The [source/dependency review](source-review/readiness-review.md) and actual crate
checksums/license hashes establish the three new locked dev dependency identities;
all existing registry identities/checksums remain.

Canonical sorted compact map SHA256:
`c1baa8b487bdacaa746446a0e21540b9605232c39131b32148253909249cb8f1`.
Reviewed inventory SHA256:
`fce71cd78045159972f4bd4500bf4f67305ecc074db9a450ef3f3327d2421a09`.
Append-only bound review SHA256:
`49f7d8e32e29886a22d91b8310e99b961a5e611a0b75f670c1f280e7da29f497`.
Previous review text, v3 schema/base, frozen predecessor bytes, immutable primary
tree and refusal behavior remain exact. Old stale-Cargo admission refusal was
retained; it was a source identity failure before expensive qualification. No
old receipt was transferred, no gate skipped and no allow-dirty override used.

An initial prequalification whitespace check flagged one protocol line after the
metadata commit ba16197. A normal, independently reviewed spacing-only follow-up
commit `9a9904f` corrected it before any qualification. No history was rewritten or
criteria changed. The original eight-path review and separately pinned follow-up
remain retained; source/admission/review bytes did not change in that follow-up.

## Executed checks

| Existing suite | Result on exact clean `9a9904f` |
| --- | --- |
| Native admission preflight | 135 matching inputs; source identity only |
| Unchanged native qualifier | All 18 commands pass; 35 Rust tests, five new native specimens with independent rational/adversarial checks |
| Qualification-gate tests | 5 normal + 5 optimized pass |
| Real-native oracle tests | 10 normal + 10 optimized pass |
| Fresh real packet with RHEON_ALIGNED_PACKET | 26 normal + 26 optimized pass, zero skips |
| Focused no-default workflow/native-family/composition contracts | 23 + 9 + 14 pass |
| Focused no-default original/periodic represented-volume contracts | 18 + 5 pass |
| No-default borrow Rustdoc contracts | 3 pass |
| Private aligned coupled and exact-geometry adapters | 2 + 9 pass |

The focused total is **83 passing, zero failing, zero ignored**. Other filtered
tests are not counted. The exact-geometry suite's known five-artifact regression
completed normally in the prescribed debug configuration; its 186.08s suite time
and over 60s progress notice remain in the raw log. This is an existing fixed
regression, not a rerun of the 24-case scientific field campaign. The native
qualifier includes locked build, formatting and strict Clippy in its unchanged
library/example/aligned-contract scope, not all-target desktop acceptance.

The packet contains exactly native.tsv, data.json, qualification.json and
primary-qualification.json, produced from the new actual executable and new
source receipt. Both 26-test modes exercised real packet and malformed/stale
source/inventory/predecessor/review/binary/receipt/rehashed-payload refusals.
No packet-dependent tests skipped. The [independent execution audit](qualification-9a9904f-review.md)
verified all 1472 committed source hashes, all 135 selected inputs, the actual
built/retained ELF and all 60 archive members/CRCs/hashes without rerunning suites.
No remaining material finding was reported. The original audit
summary binding is preserved: a [metadata-only custody follow-up](summary-custody-followup.json)
adds Library version 0 and the exact file name. Removing only those two fields
reconstructs the audited summary hash exactly; source, results and raw archive
identities remain unchanged.

## Environment and retained evidence

Pinned Rust/Cargo 1.92.0, x86_64-unknown-linux-gnu, actual Python 3.12.14 recorded
before execution. A previously absent external target
`/workspace/rheon-q2-reference-admission-20261020-build` was used with jobs 2,
incremental 0 and dev/test debug information 0; default opt-level 0/debug assertions
remain. Normal Python clears ambient optimization and bytecode writes are off.
Source 1472-file identity, exact commands/exits, outputs and compiler/oracle logs
are bound by the [compact manifest](qualification.json).

Fresh native receipt SHA256:
`b647e5c61f319044067ac10d128bf04e2461677103f1d8f953b7230c02c11739`.
Actual retained ELF SHA256:
`e12fe3ecd5f5f96cd30e30cba9e1f5e25fd9c263033314554cebe44cf97e94d8`.
Raw archive `/workspace/rheon-q2-reference-stage-9a9904f.zip`, 913699 bytes, SHA256
`fa0db9c9e6f98a8c2d98b89967053537c8a2e74237c34276c84614b76bbff16c`,
is saved privately in Library as rheon-q2-reference-stage-9a9904f.zip / version 0;
its exact persistent identity is in the compact manifest. All official crate
archive/license bytes and final admission review are retained in the archive.
No original campaign/failure archive was overwritten.

## Remaining gates and source boundaries

Original PR132 fixed-corpus geometry evidence retains its `0f4e7e0` source, and
PR134's eight-case coupled-algebra fields retain `06b3190`. Root freshly executed
only the declared native/rational/packet and fixed focused contracts. Component
field campaigns and independent scientific reconstruction were not rerun or
relabelled as `9a9904f` evidence. Exact positive/zero geometry is qualified only for
the declared corpus. Separate PR135/4f8bd22 qualifies bounded CV1 cut-cell
operators/physical traces at source `6d99e77`; its square mass condition 6.53e11
leaves coupled conditioning, mixed-pair stability and PDE accuracy open. That
source and its raw campaign are excluded from this admission. Small-cell policy
and MMS/hydrostatic/production-water acceptance remain separately gated.

Hosted Pages/browser/full default/desktop integration is separate. No fresh
release, Lean, model/GPU, native-window/performance or physical acceptance is
claimed. C1-B2/PR136's new 140-input runtime/custody closure is a separate pending
admission, with its assigned legacy G1 cancellation-label repair also separate. The unchanged
legacy path in `9c04278` and `9a9904f` mislabels advection cancellation as Failed
despite no publication; the passing 23-test suite does not resolve it. Full G1-C
outcome-label acceptance remains open for that path.
The future grid→particle/surface→Lagrangian roadmap remains unimplemented beyond
actually delivered bounded slices.

PR121's unchanged nine-green-check proposal remained unmerged after the single
requested retry was again rejected by automatic approval review: the original
trusted no-merge delegation prevailed over tool-relayed confirmation. No other
merge path or further retry was used. Earlier PR120/119 merges remain verified;
main was `718469b` and no source branch was deleted. Public implementation merges
remain paused at that approval blocker and their own required gates.
