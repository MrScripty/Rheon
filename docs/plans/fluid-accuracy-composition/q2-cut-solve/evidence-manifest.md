# CV2 sealed evidence identity

Draft PR: <https://github.com/MrScripty/Rheon/pull/139>.
Branch: `implementation/q2-cut-coupled-solve-20261020`.
Dependency/base: CV1 PR135, `4f8bd22669efed62e84c0062ee1baec27393e830`.
Numerical source freeze: `b0a9d52cc266cb651c5e67c07ec5f4f82419db55`.
Qualification/report snapshot: `d8575238464a5b3f2ea0465bf6da2f71d621413d`.
This later documentation-only identity closure is outside the sealed archive;
it changes no numerical source, input, protocol, oracle or campaign result.

## Archive and retained Library identity

- Local archive: `/workspace/rheon-q2-cut-solve-b0a9d52.zip`.
- Size: 10,060,236 bytes.
- Archive SHA256: `4c40b60923bac393498388f7702fde0a182a38c09d415d6c41d7d43f43560616`.
- Payload `MANIFEST.sha256` SHA256: `49afc06c449ff85dee378ea378d85262487be662dfe2524258c4110667fd6c50`.
- Library file: `libfile_a0f447ac54d881919df6a29415eeddd2`.
- Retained file: `file_0000000050208230a415327db9f8b5f9`, version 0.
- Library path: `/rheon-q2-cut-solve-b0a9d52.zip`.

The archive contains 346 regular payloads and records 2,478 derived fixture
symlinks in `symlink-manifest.json`, without duplicating their immutable targets.
It retains all 178 native CSV files, including the 65 byte-identical CV1 operator
files; frozen source, protocol, commands, executable and toolchain receipts;
independent numerical references and all 45 evaluator checks; the unscaled
comparator's six square failures; and final qualification/review snapshots.
Archive CRC and every payload manifest entry were checked before retention.

## Source and independent review bindings

- Release executable SHA256: `c1f115f54e2867a2429b6d04305240ab242a1745b4c249e1e18c5512d3babfde`.
- Independent oracle freeze SHA256: `5382622591188efb6d9148077f8940d4269ac7717424631423b18169b47e0705`.
- Independent final review SHA256: `34a437a5018efe908cf49e3fdfe829fb118e290049b13bd9aaf81625d8b6c477`.
- Independent final manifest SHA256: `612781a988a84fe927e0b0dc2b99d3fa3aae2d747071c8cbb3a74a6077b6e4ec`.

The sole native campaign began at `2026-10-20T04:23:38.329611+00:00` after source
and oracle freezes. All 24 cases passed the fixed `1e-10` gates. The largest
original-coordinate velocity error against independently integrated operators
was `1.861216501617988e-12`; maximum normalized pressure error was
`3.354808230044388e-13`. The original square mass conditioning remains about
`6.53e11`; scaling changes solve coordinates, without changing that mass.

## Ownership and qualification limits

The exact new write set is `examples/grid_cut_coupled_reference.rs`,
`examples/grid_cut_coupled_reference/{solver,experiment}.rs`,
`tests/grid_cut_coupled_reference_contract.rs`, and this documentation directory.
CV1, exact geometry and aligned reference dependencies are read-only and retain
their frozen hashes. Production, Session/recording, desktop composition, provider
and source-admission code are unchanged. No merge or branch cleanup is performed.

This qualifies only the fixed static planar dense numerical reference. Generic
cut stability, PDE convergence, mesh refinement, moving geometry, transport/GCL,
solid Dirichlet lifting and production water remain unqualified. No GPU, native
UI, screenshot or physical-impact verification is claimed. Repository-lead
integration and merge coordination remain separate.
