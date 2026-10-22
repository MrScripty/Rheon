# PR59 fresh independent raw executable review

2026-10-21. Scope: newly regenerated compiler-output artifacts supplied by the primary worker, assessed afresh. Used read-only GNU Binutils 2.44 `readelf -hW/-lW/-SW/-nW`, `objdump -d`, direct ELF64 parsing and complete byte comparisons. No prior diagnostic-image conclusion was assumed. No objcopy, strip, build, test, timing, binary execution, artifact rewrite or repository mutation was performed. Only this report was written.

## Decision and limits

The supplied raw hashes are verified. **All four load layouts and all 12 program headers match; exactly 21 bytes differ inside the declared file-backed load payloads:** offset 40 in `e_shoff` and the 20-byte GNU build ID at offsets 812–831. Every allocated section has identical metadata/layout. All allocated file-backed payloads except the GNU build-ID note are byte-identical; all zero-fill/TLS extents match. Code, ordinary constants, relocation/dynamic data, unwind tables and initialized state data match. This supports no introduced compiled particle numerical-path or code/data-layout change in these exact remapped raw artifacts, with the observable metadata exceptions explicitly retained.

**Whole-executable identity is not satisfied.** Neither literal complete-file equality nor literal equality of every mapped byte is established. Metadata/debug/self-inspection paths can distinguish the artifacts. No statistical performance acceptance, universal default-consumer equivalence or merge authorization follows. The strict timing stopping rule remains unchanged, and the proposed whole-file-identity clause is both unapproved and unsatisfied. No broader image-based exception is approved by this review.

## Exact artifact binding and source checks

| Raw compiler artifact | Bytes | SHA256 |
|---|---:|---|
| Base: `/workspace/pr59-base-remapped-target/release/examples/cpu_particle_lab` | 957592 | `d1473cb7b75965d489d7cc59c6dec3529890d7978b0584f3fe08f31d2cdda4a4` |
| Candidate: `/workspace/pr59-candidate-remapped-target/release/examples/cpu_particle_lab` | 957576 | `7264603aa88c22073a7c58e7a294a992f0cf4459a750d7b1d968fe575a851bb9` |

The preserved copies `/workspace/pr59-raw-initial-binaries/base-remapped` and `/workspace/pr59-raw-initial-binaries/candidate-remapped` independently hash to those same values. These differ from the earlier objcopy-rewritten diagnostic files; the earlier report and provenance correction remain separate and unmodified.

The raw build receipt identifies initial main `15026a966265cf9eaedf464e2b5a066bd0fef3d8`, candidate source commit `0b3e3fc0e83524fcc9064a2158030f2ee4994a1a`, and `cargo build --release --locked --features parallel --example cpu_particle_lab`, with identical frozen flags:

```text
--remap-path-prefix=/workspace/pr59-base=/rheon --remap-path-prefix=/workspace/Rheon=/rheon
```

Both fresh rebuild logs show successful Cargo release compilation. I verified the baseline worktree HEAD and no tracked baseline Cargo/source/harness differences; all 13 originally reviewed candidate production/test files plus the particle harness match the candidate source commit byte-for-byte. The harness matches in both worktrees, SHA256 `3e6f4eb91403c285805115733fd423e2ed071b6f3024a49daee35db527539e1c`. Thus the inspected artifact hashes and current source checks are independently verified. Exact execution of flags/toolchain and absence of intervening transformations are represented by the primary worker's receipt and custody statement rather than independently reproduced compilation. The worker states these regenerated hashes reproduce original pre-objcopy observations; I verify the fresh artifacts and retained receipt, not a historical untouched build I observed personally.

## Header and every load segment

Both are x86-64 ELF64 little-endian PIE, entry `0x34210`, with 12 program headers at file offset 64 and 35 section headers. The complete raw 672-byte program-header tables are byte-identical, SHA256 `89841a15cbf7f21689ceb55f6a961801735d86fcd95e1d111d352aa7a1fe8023`. Interpreter string is identical: `/lib64/ld-linux-x86-64.so.2`. Dynamic tables and relocation payloads match. Runtime-resolved shared libraries, loader and kernel were not hashed.

The only header-byte difference is zero-based offset 40 in the eight-byte section-header offset field. Base `e_shoff` is **955352 (`0xe93d8`)**, candidate **955336 (`0xe93c8`)**. This is a section-table pointer, not a changed entry point or program-header layout. These values were freshly derived from the raw files and differ from the earlier diagnostic-rewrite report.

The 20 remaining differing load-payload bytes are exactly the build-ID descriptor at offsets 812–831. Base ID `cb1cf9e701a2ef5a0b4da99bf09096dbed6bf76b`; candidate ID `962114c9ad002a85f7e997b047afa4e54e916de0`. The note's header/name match. It is **SHF_ALLOC**, inside the first `PT_LOAD`, and remains an observable loaded-data difference. No exception is silently discarded or normalized.

| Program-header index | File offset | File bytes | Memory bytes | Base payload SHA256 | Candidate payload SHA256 |
|---:|---:|---:|---:|---|---|
| 2 | 0 | 209416 | 209416 | `4f509f926a8513caee60c68f820ec13533dc6162576ff05f92a6c04c90d7e411` | `9e2fb7f6d5bb1e16edf75f5507c2b5ba10c7f98b21fd51e0ab15d1193ef448f8` |
| 3 | 209424 | 463184 | 463184 | `04ec761de6567236b096ca6d5abb68e54154faa558b3b3bea9c0858ee521bf4b` | `04ec761de6567236b096ca6d5abb68e54154faa558b3b3bea9c0858ee521bf4b` |
| 4 | 672608 | 20536 | 23712 | `863ca9eafc605a06c30edbcaadab2ff31cb75cdfdac84ac8f935937299534aa3` | `863ca9eafc605a06c30edbcaadab2ff31cb75cdfdac84ac8f935937299534aa3` |
| 5 | 693144 | 2528 | 2784 | `4fc7620c6a7493887a01415baf366f3323207f1309dc446f37639a92cc32fdcb` | `4fc7620c6a7493887a01415baf366f3323207f1309dc446f37639a92cc32fdcb` |

All file-backed segment bytes were compared, including padding/gaps. Last declared load payload ends at offset **695672** exclusive. First subsequent differing byte is **697432** (one-based cmp byte 697433), outside all declared load payloads. Nonallocated `.symtab` contents differ at equal size (83664 bytes); raw `.strtab` sizes are 175494 versus 175480 bytes, and its contents differ. Section-header offsets/metadata reflect the changed nonallocated string-table length. Raw `.shstrtab` content/layout is identical (339 bytes). Matching NOBITS/program extents prescribe matching zero-filled memory.

Disassembly machine-address/byte fields match across all 123985 parsed lines. Rendered symbol annotations can differ because private LLVM suffixes in static symbol metadata differ; complete code bytes, rather than normalized names, establish code equality.

## Every allocated section

All 30 SHF_ALLOC section raw metadata records match; 27 have file payloads and three are NOBITS. A single hash below means exact payload equality. NOBITS has no file payload and is reported by matching extent instead.

| Section | Size | SHA256 or comparison |
|---|---:|---|
| .interp | 28 | `9097c98ea70f6f3b01d50145eb72bf55ad1f4e9713bccf27bd6405d6254eed00` |
| .note.ABI-tag | 32 | `6e48317ad93a8ac2dc04321465ede06d1274543fc8ab172e68171a9d684bb313` |
| .note.gnu.build-id | 36 | differs: base `af581aa75a3e2cfae3f8864012b6664a0ec0ebad69ead8cf3b02e4ae15ca40bd`; candidate `9642896be6a1cfa3d91c0c85af641f1eadf5363fd5895d26a376c42587cf99ec` |
| .dynsym | 1944 | `6801b53a84566bf34a0ad9ab89d23c807e5f7d32e21ccfe21b8de82d5d361ada` |
| .gnu.version | 162 | `8dfcb0cdcc1d877c80dac710eb80984a093c1df4fdfd703bbb5d0f8a2c30113a` |
| .gnu.version_r | 368 | `e7d52d2ec906dbbe4bb2eef85c73c47338b95c0bae0084f88d39188ecadba2b1` |
| .gnu.hash | 28 | `b97b3a4dc507ae480cac374aa052146b4dd9eba099bda92da2e79b32d39a68b8` |
| .dynstr | 1269 | `ff2b1ec523c700de780b6031e37214eaf5febdec34685c9d311e1e936b78edba` |
| .rela.dyn | 31392 | `d2056298e091d2eb483b09b0652c2c5e38fe51a3b07dd9ba40349bdc6fcd3a8b` |
| .rela.plt | 48 | `1c8e2a88a65d5bcb8817082b3b78d44a57f40dee18e1450d4592aaa4f52524a2` |
| .rodata | 34620 | `fe6eab2dabf211f556f02e53ea841dd249940722923f89729b61f953f8d71e87` |
| .gcc_except_table | 82276 | `6b6281b88610548c9a18da16ccd10ac173bf4b832855c7f18df473ee0e8e08f3` |
| .eh_frame_hdr | 8380 | `38677867c817f0779ccf1f225192d11642fb8e23cd95fbb8fbdf7d68f24bf2bc` |
| .eh_frame | 48072 | `d1377c0bdd33c208985c5de7bccb1410f72ba314f3f2e56e0b9cbcf75de72dcc` |
| .text | 463093 | `38d473ee41faeb6ad2649dbd4f7bcc7c9efb52e40b75f4331dfa9707dd547674` |
| .init | 23 | `268fdd19cb1935140ecf9b7e341b27db4e69e5c7f5031c177d1926625fa78ebc` |
| .fini | 9 | `66e6f54550612182b4ad78f30b140dd08318b968db3878de2db65fef87dc04d7` |
| .plt | 48 | `f27829635fb219cc5df88b1f250a0fc4d2b395de6ad7b479630eff8a387d1c5f` |
| .tdata | 32 | `59e7addb5e068640de3f8fb015017e7aa7495430d2533f87d4ece9f7ee548fa3` |
| .tbss | 128 | NOBITS; equal zero-fill extent 128 |
| .fini_array | 8 | `af5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc` |
| .init_array | 16 | `374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb` |
| .data.rel.ro | 16440 | `f7ad010be434ccbdf356e7ad81cabdae6f45702f459ba803b2161d08ea2f4119` |
| .dynamic | 480 | `04e1829eb373bb28dfc546be849bf080bd33f8ada1eef0392bdcf35fea315f00` |
| .got | 3520 | `36a26ecce01fa5b0397415be067d3d275359a48fce772eb50a916862e0bec1d6` |
| .got.plt | 40 | `0ed6b350b38a865823bcb82675a1e7b5f53a09a0aed73870cb8047af0c271d76` |
| .relro_padding | 3176 | NOBITS; equal zero-fill extent 3176 |
| .tm_clone_table | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| .data | 2528 | `4fc7620c6a7493887a01415baf366f3323207f1309dc446f37639a92cc32fdcb` |
| .bss | 256 | NOBITS; equal zero-fill extent 256 |

## Supporting receipt hashes and remaining gates

- `/tmp/pr59-base-raw-rebuild.log`: `8647ac1806e873fbd60044173e5183fd219b9f85368a786adfec2be157aa32ba`
- `/tmp/pr59-candidate-raw-rebuild.log`: `d5197cafbfff302edeebaf5623f386b175edcc4e9002781708d4a475bf4ec480`
- `/workspace/Rheon/docs/plans/accepted-pressure-warm-start/integration-20261021/artifact-correction-hashes.json`: `90e798b3709a29c56dc6384ee764a36c3ce2d423da806b7a842b95df08c8d93b`
- `/workspace/Rheon/docs/plans/accepted-pressure-warm-start/integration-20261021/initial-raw-remapped-build.json`: `990a21e302a6f217168cf988178eff1d3d41bef20d947279131b2dc194be5791`
- `/workspace/Rheon/docs/plans/accepted-pressure-warm-start/integration-20261021/protocol.md`: `36c1891eb064160dfdded473a164ae9f11ab0687871c43169ddee7bc6ed0c638`

This report freshly establishes the raw-artifact comparison; it does not rehabilitate diagnostic files as untouched outputs or revise earlier records. The harness has no direct executable/ELF/build-ID lookup, but runtime-library, exceptional, backtrace, debug and external observation paths were not exhaustively proven independent of metadata. Runtime scheduling, ASLR, caches, file metadata/path and host conditions remain outside compiled numerical code identity. Equal code does not justify dismissing every possible timing effect or a positive/inconclusive result under the current rule.

No timing was run by this reviewer; the primary worker reports none has run. Preserve all artifacts, correction records, complete commands and eventual trial outcomes. Do not change build-ID/linker/layout flags, strip, patch or normalize artifacts to obtain identity. The final #142 → #138 → #143 composition still needs its own exact source/artifact binding, independent composition assessment, native source admission, required CI, fixed prospective controls under the unchanged rule and lead authorization. Historical 1000-particle **+2.20%** adverse confirmation remains visible and unresolved. No default universal, mixed-particle feature-enabled, device/GPU or new-main speedup claim is admitted.

All requested fresh read-only checks completed. No denied or pending operation arose; final admission is outside this report.
