# Independent actual post-PR138 PR143 preflight

Read-only source/target preflight PASS by `/root/pr143_review`, 2026-10-22.
This approves the separately reviewed fresh ordinary qualification attempt,
not merge readiness or scientific execution.

## Actual ordered target

Independent authorized GitHub-app reads confirm PR142 and PR138 are closed and
merged on their exact reviewed heads. Local committed objects confirm:

- PR142 merge M `b70dbae515d01ced3762e0fff030df7f8b13363e` has exact parents
  `[15026a966265cf9eaedf464e2b5a066bd0fef3d8,
  3cd735fde7655aedffbd0ffb2f041ca6994961a4]`, full tree
  `7431605931aa55f6a83135d3476d02da909fc7e2`.
- PR138 merge N `7befee45270280d07693b91f4afa934ec1780b66` has exact parents
  `[M, da1708da79fdf3ede4b4e6ee73906e1b9217a48f]`, full tree
  `41192cba5adf6d4bc24d4ce80e3d47190e56b1f4`.
- Fetched `origin/main` equals N and fresh GitHub branch-main API equals N.
- Clean local PR143 head remains
  `159fa74bd271f8fcc07f3837d66e4738925c820c`, tree
  `d66c930d091f8e84cf445f0920922d65afed8f8e`, parents `[f6d44cd, da1708d]`.
- Fresh GitHub PR143 synthetic merge
  `088a18227d273e3070b9602f92d9e6fbb31f5a00` and its fetched local object have
  exact parents `[N,159fa74]` and the entire exact PR143 tree above.
- Independent `git merge-tree --write-tree N 159fa74` exits zero and returns
  exactly `d66c930d091f8e84cf445f0920922d65afed8f8e`. Subsequent app reads retain
  main N, open PR143/base main/head159 and the same synthetic identity.

Content equality does not establish that 159 descends from N or that binaries,
builds, environments or receipts share identity. Stale REST `base.sha` is only
an observation; live main and the synthetic's actual first parent own the target
proof, as separately reviewed. An initial local read before the owner's fetch
returned missing-object exit128 for N; after the announced successful fetch,
the independent actual-object checks above pass. That was a read-only object
availability check, not a qualification attempt or numerical failure.

## Complete source and preserved preparation

I independently reconstruct the actual native selector from complete `src/`,
`.cargo/`, `rust-toolchain*` paths and the literal `QUALIFICATION_INPUTS` tuple.
Its exact 152-path set and every current hash equal the unchanged active v3 map.
Canonical compact sorted map SHA256 is
`9a10497158827e189fe505f65e1aa203cd8a28fe894c469408977071b055375f`;
inventory SHA256 is
`e3ac26c5c62b909aa1ef1384b8e0baffb5587e14c1794f02f0e68ae655d14195`;
bound review SHA256 is
`b4e7fcaf0c33b9e220fc7626d04969a95b31767897e9779eee850a9cfd71d0f9`.
All selected/admission bytes equal actual N and reviewed preliminary PR138.

The complete current 2,052-file tracked source map has canonical compact sorted
SHA256 `6aebe9dd485c142965bcfda8575f77245e605c4fe768bdbbefa217704efada7c`.
There is no production/private example/test/CV3 preparation delta against the
previous actual ordinary execution source31c19585. No production source imports
the CV3 example. The three explicit ignored CV3 numerical contracts and their
assertions, fixed criteria, resources and campaign commands remain unchanged.

All 54 preparation entries, 16 oracle snapshot entries and 25 native/resource
review entries match their original historical PR143 source. Against current
integration, the 54- and 25-entry lists each have only the already-reviewed
Cargo.toml variance (main's force-serde feature); all 16 oracle entries remain
exact. Seals are not refreshed or treated as current binary/execution receipts.
This review imported no selected oracle, ran no native/Rust test or scientific
case/control, and made no change to the frozen candidate.

## Conditional next action

Exact reviewed external recorder SHA256 is
`aa78bef6b3fbaf524b54bb751a7e3898ebf370e8a82d850a1f24c380f7f67a0b`;
app collector SHA256 is
`1fcc22b4710f21ae6f2fc58b508f708e66a0dcdc7a7fbfc9f832abf9acf20e37`;
protocol SHA256 is
`0de6ac12910f0aa97b81dbe5d91fe2dd55efae8759494f9fa4ae5fddffbed65a`.
Their current bytes match the independently approved versions. Binding/output
were absent during this preflight. The owner may now create the actual M/N
binding and run only the reviewed fresh ordinary attempt with fresh nonce-bound
app metadata, outputs and actual receipt/ELF custody. Any target/source mismatch
stops the route. No failed CLI credential, denied log/artifact or cloud route
was accessed by this review.

Full exact159 hosted CI was independently audited in PR143 comment6329770051.
Fresh post-N execution/results/proof review, lead's immediate current-state and
expected-head merge checks, exact actual merge-tree/parents and post-merge checks
remain required. No production, physical, CV3 scientific, hardware/native-window
or performance acceptance follows from this preflight.
