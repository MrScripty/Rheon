# PR143 final post-dependency ordinary qualification

The fresh ordinary qualification passed on clean, unchanged candidate
`159fa74bd271f8fcc07f3837d66e4738925c820c`, full tree
`d66c930d091f8e84cf445f0920922d65afed8f8e`, after the actual ordered
PR142 and PR138 merges. This supporting report is separate from the executed
candidate; its later commit is not an execution or merge-head identity.

## Actual target and complete-source proof

Actual PR142 merge M is `b70dbae515d01ced3762e0fff030df7f8b13363e`,
parents `[15026a966265cf9eaedf464e2b5a066bd0fef3d8,
3cd735fde7655aedffbd0ffb2f041ca6994961a4]`, full tree
`7431605931aa55f6a83135d3476d02da909fc7e2`.
Actual PR138 merge N is `7befee45270280d07693b91f4afa934ec1780b66`,
parents `[M, da1708da79fdf3ede4b4e6ee73906e1b9217a48f]`, full tree
`41192cba5adf6d4bc24d4ce80e3d47190e56b1f4`.

Fresh authenticated GitHub-app metadata and ordinary fetched Git objects agree:
actual main is N; PR143 remains open on base main at exact159. The fresh
GitHub synthetic merge `088a18227d273e3070b9602f92d9e6fbb31f5a00` has
ordered parents `[N,159]` and the entire exact159 tree. Independently repeated
`git merge-tree --write-tree N 159` exits zero and returns that full tree.
The lead's [owned protocol exception](https://github.com/MrScripty/Rheon/pull/143#issuecomment-6329435931)
admits this full-content proof while retaining the tested candidate. It does
not establish that N is a candidate ancestor, or that any builds, binaries,
environments, artifacts or receipts are equivalent.

REST PR143 `base.sha` still reports opening base15026a9. This discrepant field
is retained in both proofs, without substituting it for actual live-main and
synthetic-parent evidence. The same live target/head/synthetic/full-tree guards
passed immediately before and after actual execution. Both phases captured
new real app GET responses after separate recorder nonce requests, through the
reviewed collector. Their raw JSON, unique request hashes, capture times,
authorization identity and exact endpoint queues remain external artifacts.
This is trusted tool-response recording, not cryptographic API attestation.

All 2,052 committed file hashes remained unchanged. Their compact sorted JSON
map SHA256 is `6aebe9dd485c142965bcfda8575f77245e605c4fe768bdbbefa217704efada7c`.
The actual complete selector still admits exactly152 inputs, map SHA256
`9a10497158827e189fe505f65e1aa203cd8a28fe894c469408977071b055375f`.
The v3 inventory and bound review are unchanged, with SHA256
`e3ac26c5c62b909aa1ef1384b8e0baffb5587e14c1794f02f0e68ae655d14195`
and `b4e7fcaf0c33b9e220fc7626d04969a95b31767897e9779eee850a9cfd71d0f9`.
No selector narrowing, admission refresh or parent-inventory union occurred.
The original54 preparation,16 oracle and25 source-review seals remain valid
for their historical source. Current source retains the sole already-admitted
Cargo.toml variance; all16 oracle payloads and the other preparation payloads
remain exact. No selected oracle module was imported by custody checks.
See [independent preflight](source-preflight-review.md).

## Fresh execution and custody

The [preserved prospective protocol](qualification-protocol.md) was reviewed
before execution. Actual task-local driver SHA256 is
`aa78bef6b3fbaf524b54bb751a7e3898ebf370e8a82d850a1f24c380f7f67a0b`;
collector SHA256 is
`1fcc22b4710f21ae6f2fc58b508f708e66a0dcdc7a7fbfc9f832abf9acf20e37`.
Their earlier reviewed, unexecuted versions and original3217 recorder remain
retained. Exactly one final attempt ran; no command failed or was retried.

All11 outer commands passed, including:

- Rust/Cargo1.92.0 identity, unchanged admission, locked metadata and normal
  core dependency graph excluding num-rational, num-bigint and num-integer.
- The unchanged native qualifier's18 commands:35 Rust contracts; format,
  actual example build and strict Clippy; five actual native/rational specimens
  with11 negative controls each; gate5 normal+5 optimized; oracle10+10.
- A real four-file packet bound to the new qualification receipt and actual
  exporter. All26 tests passed normally and all26 under `-O`, with zero skips.
- Strict no-default-feature all-target Clippy.
- The real private adapters: coupled15, variational10 and static13 passed.
  The three CV3 numerical contracts remain explicitly ignored and unexecuted.

The new external output is `/workspace/pr143-final-qualification-1`.
All57 retained artifacts and eight observed executing exporter/test ELF copies
were hash-checked. New journal SHA256 is
`5c981d534e0d95b337e055435fb592407c7ef5d85c48016fc4d0d457298e7c18`;
native receipt SHA256 is
`a7c4c124717d89716fa99f2a6edaa064b030f9a2fe21c05a1c4cfc8a29f5386f`;
packet receipt SHA256 is
`344ea04f492d03276ed7bb247410989cb73cbf0d04caa581a34a3de491707e91`.
The actual retained exporter SHA256 is
`ca6a6e62a6177b73b6445b4665b00727569f695228bd89b2706b61199ece4397`.
[qualification.json](qualification.json) records actual argv, elapsed times,
environment, receipt chain, target proofs, ELF identities and artifact roster.

Python3.12.14, task-owned toolchain/cache, jobs2, incremental0 and dev/test
debug0 were retained; optimization/debug-assertion defaults were unchanged.
Ambient optimization/Rust flags and optional Rheon source/build declarations
were removed, with the latter actually unspecified. Local GitHub SHA/ref/event
variables were independently observed unset. The source was clean local159,
not a GitHub event or execution of N. Warm-cache reuse does not establish a
cold build, new compilation, embedded commit attestation, performance result or
binary/environment reproducibility. The fresh execution receipt has its own
identity even where unchanged source produces the same observed ELF hash.

## Hosted checks and preserved outcomes

The independent [full-CI audit](https://github.com/MrScripty/Rheon/pull/143#issuecomment-6329770051)
and [captured check metadata](hosted-checks.json) bind exact159:20 completed
check runs,19 SUCCESS and the expected PR deployment SKIPPED, zero pending.
Full Rust39695320834 and full book39695320858 passed. Rust's actual checkout
was synthetic `a2e514e2643d023aa557b13ea85c8a8e1de17d2a` with parents
`[15026a9,159]` and full159 tree. The book checked out159 directly; a2e was
its event/artifact-name identity. Native, browser, PDF and reference checks
passed within their recorded scope. Artifact metadata/upload success does not
prove downloaded artifact contents or visual acceptance. The legacy CodeRabbit
success said draft review skipped and was not used as independent review.

Rust top-level suites reported default772 passes/13 ignores, core764/13 and
desktop1156/20, each with three separate doc-test passes. Desktop's four nested
one-test summaries are excluded from the top-level count. The executable job
reported bootstrap7 normal+7 optimized;220 cases with193 passes and27 explicit
conditional skips; real smoke passed. These counts do not erase unmet hardware
or other conditional acceptance boundaries.

Original c7 attempt2 run39377493085 remains CANCELLED at its actual historical
checkout ea5aa6d8. The recorded annotation identifies the30-minute outer job
timeout after successful strict Clippy, during ordinary contracts; no assertion
failure was observed. Attempt1 cancellation and aggregate failures remain.
The stale41c run39692570591 retained its executable failure from the exact
30-versus60 budget-header guard, then its recorded supported supersession
cancellation and aggregate failure. The shared guard repair requires exact60
and rejects30/59/61/missing, retaining all ordinary suites and strict lints.
The superseded f6 Rust39693673824 passed, while its two book runs were
CANCELLED; they are not current159 passes. These outcomes are never relabelled.
The actual31c preliminary qualification and its original receipts remain
separately preserved in the earlier integration report.

A later read-only GitHub CLI status batch returned401 and that credential was
stopped. Independently authorized app identity/repository evidence permitted
the current supported app route; ordinary Git fetch also succeeded. No failed
CLI operation, previously denied artifact download, old PR138 log read or cloud
task read/resume was retried or rerouted. A harmless reviewer local-object
availability check failed before the announced successful Git fetch; it was
not a qualification or numerical failure. No force push or branch deletion
occurred.

## Readiness boundary

Independent actual execution/proof/results review [passed](execution-review.md).
Exact supporting-report review, fresh final head/main/check verification and
the sole lead's normal merge still gate readiness at this report's preparation. The candidate and its source
branches remain159; this evidence branch must not replace that identity.
Actual merge parents/full tree, retained branches and post-merge checks must
then be verified. Raw logs, API responses and binaries remain locally retained
outside Git; the public authored manifest is not a remote raw-evidence archive.

PR143 preserves CV1/CV2 and the nine-case static polygon manufactured-solution
preparation. It neither runs nor accepts that scientific campaign. The packet
performs zero fluid solves and authorizes no stepping; the native qualifier
does not establish Lean checking or a floating-point enclosure. No generic
conditioning/stability/PDE, moving-boundary/GCL, physical/production-water,
native-window, GPU/learned, performance or whole-application production
acceptance follows from this integration.
