# PR59 independent execution-image review

2026-10-21. Read-only analysis of the supplied remapped particle artifacts. Used GNU Binutils 2.44 `readelf -hW/-lW/-SW/-nW`, `objdump -d`, direct ELF64 parsing, hashes and complete byte comparisons. No build, test, timing, binary execution, source edit, binary rewrite, stripping or patching was performed. Only this report was written.

## Finding and scope

**Confirmed, with two precise mapped-metadata exceptions:** all four `PT_LOAD` layouts and all 12 raw program headers match. Across every file-backed `PT_LOAD` byte, exactly 21 bytes differ: one byte in the ELF header's section-table pointer and the 20-byte GNU build-ID payload. All allocated section metadata/layout matches. Every allocated file-backed section except `.note.gnu.build-id` has identical bytes; all NOBITS/TLS/zero-fill extents match. Code, ordinary constants, relocation tables, dynamic linkage metadata, unwind tables and initialized data match exactly.

This supports a narrowly scoped finding: **no compiled particle execution-path or code/data-layout change is present in these two remapped artifacts**. It does not establish whole-file identity, literal equality of every mapped byte, universal default-consumer equivalence or statistical performance non-regression. Build IDs and ELF metadata remain observable, including on diagnostic/self-inspection paths. A software path could inspect them; this review does not claim the files are indistinguishable under all possible executions.

The proposed whole-file-identity clause is **not satisfied**. The existing positive/inconclusive timing stopping rule remains controlling and unamended. No acceptance or merge authorization is supplied. A broader image-based admission criterion would require a concrete separately reviewed amendment and recorded owner/lead agreement before measurements, with these exceptions stated explicitly; it cannot inherit approval from the earlier whole-file proposal.

## Bound artifacts

| Artifact | Bytes | SHA256 |
|---|---:|---|
| Base: `/workspace/pr59-base-remapped-target/release/examples/cpu_particle_lab` | 925944 | `9650df93b8c3b360c22e28b91a69a71b42a8a914b7c1565b5ef47e079f46a0db` |
| Candidate: `/workspace/pr59-candidate-remapped-target/release/examples/cpu_particle_lab` | 925928 | `3a6fcdeb162d5a5e9f2e902851a532d7f0c16d64b2538b3861b0555c43755c36` |

Primary-worker source lineage: initial main `15026a966265cf9eaedf464e2b5a066bd0fef3d8` versus candidate `0b3e3fc0e83524fcc9064a2158030f2ee4994a1a`. This report binds the artifact bytes above; it did not independently reproduce the builds or verify their complete command/flags receipts. Both remapped build logs were read. These are initial-main diagnostics and cannot replace final ordered-composition controls.

## Header and segment assessment

Both are x86-64 ELF64 little-endian PIE executables, entry point `0x34210`, program headers at offset 64, with 12 program headers and 35 section headers. Their complete 672-byte raw program-header tables are equal, SHA256 `89841a15cbf7f21689ceb55f6a961801735d86fcd95e1d111d352aa7a1fe8023`. The interpreter string is the same: `/lib64/ld-linux-x86-64.so.2`. External shared libraries, loader and kernel were not resolved or hashed by this review.

Zero-based differing offsets within declared load payloads:

- Offset **40**, in the eight-byte `e_shoff` field at offsets 40–47. Base section-header offset: 923704 (`0xe1838`); candidate: 923688 (`0xe1828`). All other ELF-header bytes match. Program-header layout and entry point do not change.
- Offsets **812–831** inclusive, the GNU build-ID descriptor. Note header/name are equal. Base ID: `cb1cf9e701a2ef5a0b4da99bf09096dbed6bf76b`; candidate ID: `962114c9ad002a85f7e997b047afa4e54e916de0`. The note is **SHF_ALLOC**, within the first `PT_LOAD`; it is not a nonloaded exception.

| Program-header index | File offset | File bytes | Memory bytes | Base payload SHA256 | Candidate payload SHA256 |
|---:|---:|---:|---:|---|---|
| 2 | 0 | 209416 | 209416 | `6ae3631a71b2ae9b436b9357dbe0de36cb30d67b67c61906543e1b55865590eb` | `94652953fe1b465beb7582aa7653763079a0c1a154b1cf0d85bfa52cde0992e6` |
| 3 | 209424 | 463184 | 463184 | `04ec761de6567236b096ca6d5abb68e54154faa558b3b3bea9c0858ee521bf4b` | `04ec761de6567236b096ca6d5abb68e54154faa558b3b3bea9c0858ee521bf4b` |
| 4 | 672608 | 20536 | 23712 | `863ca9eafc605a06c30edbcaadab2ff31cb75cdfdac84ac8f935937299534aa3` | `863ca9eafc605a06c30edbcaadab2ff31cb75cdfdac84ac8f935937299534aa3` |
| 5 | 693144 | 2528 | 2784 | `4fc7620c6a7493887a01415baf366f3323207f1309dc446f37639a92cc32fdcb` | `4fc7620c6a7493887a01415baf366f3323207f1309dc446f37639a92cc32fdcb` |

This compares complete segment payloads, including padding/gaps, rather than assuming section equality covers all loaded file bytes. Final declared file payload ends at offset **695672** exclusive; first later differing file byte is **696112** (one-based `cmp` byte 696113), outside all declared load payloads. Matching NOBITS/segment extents specify matching zero-filled memory.

The nonallocated `.symtab` contents differ (size remains 83664 bytes), `.strtab` contents and size differ (143856 versus 143842 bytes), and `.shstrtab` shifts while keeping identical contents. Section headers move and record these nonallocated changes. In particular, private LLVM symbol suffixes differ in the static symbol metadata. Therefore rendered `objdump -d` text differs in its symbol annotations even though its instruction addresses and raw machine-byte fields compare exactly. No names were normalized in the binary; full code-segment byte identity independently supplies the stronger code comparison.

## All SHF_ALLOC sections

Every allocated section's raw metadata (including type, flags, file offset, virtual address, size, alignment, links and entry size) matches. One hash means byte equality. NOBITS sections carry no file payload and are reported by matching zero-fill size/layout.

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
| .tbss | 128 | NOBITS zero-fill, 128 bytes; matching metadata |
| .fini_array | 8 | `af5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc` |
| .init_array | 16 | `374708fff7719dd5979ec875d56cd2286f6d3cf7ec317a3b25632aab28ec37bb` |
| .data.rel.ro | 16440 | `f7ad010be434ccbdf356e7ad81cabdae6f45702f459ba803b2161d08ea2f4119` |
| .dynamic | 480 | `04e1829eb373bb28dfc546be849bf080bd33f8ada1eef0392bdcf35fea315f00` |
| .got | 3520 | `36a26ecce01fa5b0397415be067d3d275359a48fce772eb50a916862e0bec1d6` |
| .got.plt | 40 | `0ed6b350b38a865823bcb82675a1e7b5f53a09a0aed73870cb8047af0c271d76` |
| .relro_padding | 3176 | NOBITS zero-fill, 3176 bytes; matching metadata |
| .tm_clone_table | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| .data | 2528 | `4fc7620c6a7493887a01415baf366f3323207f1309dc446f37639a92cc32fdcb` |
| .bss | 256 | NOBITS zero-fill, 256 bytes; matching metadata |

## Interpretation and remaining gates

The inspected `examples/cpu_particle_lab.rs` (SHA256 `3e6f4eb91403c285805115733fd423e2ed071b6f3024a49daee35db527539e1c`) measures in-process particle steps and the capture/hash loop; it has no direct executable, ELF table or build-ID lookup. Equal code, state data and layout support the same numerical workload. This is not an exhaustive proof that all runtime-library, panic/backtrace, debug or external observation paths ignore metadata. Build IDs can guide symbol/debug lookup; section/symbol metadata can be read by self-inspection tools. Physical page placement, ASLR, host scheduling, caches, executable file size/path metadata and process environment remain distinct from compiled algorithm/layout identity. Do not dismiss every possible timing effect solely because the numerical code is equal.

Preserve both raw remapped artifacts and hashes, the earlier unremapped artifacts/diagnosis, complete build receipts and every eventual timing outcome. Do not alter build-ID flags, strip metadata, patch outputs or tune layouts to satisfy an identity criterion after timing. No timing result is accepted, rejected or reinterpreted here. The fixed cohort, strict stopping rule, native source admission, exact-source review, required CI, dependency order and lead authorization still apply to the final composition. The historical 1000-particle **+2.20%** adverse confirmation remains visible and unresolved. No broader default build, feature-enabled mixed-particle, device/GPU or new-main speedup claim follows.

All requested read-only inspections completed. No denied or pending action arose; final source/build provenance and performance admission remain outside this report.
