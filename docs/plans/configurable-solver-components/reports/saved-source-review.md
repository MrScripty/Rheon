# Independent saved-source adapter review

Read-only source review of `/workspace/rheon-saved-source`, base HEAD `be0ed8efd74160d3b42b96b948d342288aa44b49` plus exactly the three working paths below. No repository mutation, Cargo, model, numerical qualification or test execution performed. Read all new source/test code and the existing native/configured revision open/validate/hash/member owners used by the adapter.

**No material source finding. Approve bounded source admission at these exact bytes, pending the author's final commit binding and root's fresh composed checks.** This adapter alone does not implement configured quantitative page/artifact handling, remedy separately reviewed cold-runtime chronology gaps or establish a complete workflow acceptance claim.

## Source findings

SavedRunSet serializes only the native/configured inner revision; it introduces no native JSON tag, schema rewrite or new digest framing. Native serialization and hashes delegate to the exact native owner. The custom untagged deserializer validates the selected revision after typed structural decoding. Both underlying revisions deny unknown fields; configured additionally requires its distinct kind and closed configured schema. Configured payloads cannot silently pass as native because of the configured kind/shape, and future native/configured revisions fail owner validation. A public enum can still be constructed or mutated directly by app code: callers must use validate/open or a validating envelope boundary before treating metadata as admitted.

File open tries the configured owner only for the exact Unsupported("manifest version") result from native open. It preserves native codec, integrity, byte-limit, regular-file and I/O refusals rather than treating arbitrary errors as a configured fallback. The configured owner then independently checks its expected envelope version, strict payload, revision validation and digest. Unsupported cross-version envelopes remain unsupported; malformed schema-one configured contents do not acquire a fallback path. Existing manifest byte/path policies remain owned and unchanged. Reads are bounded by the existing manifest limits; a concurrent path replacement is not made safe or authenticated by this adapter.

Member and attempt views borrow the actual revision data, including exact expected run/source, archive relative path and finalized pin. Limits are copied from the actual member's declared recording controls; labels/differences are derived from closed typed variants. No alternate baseline/difference inference, fabricated native spec or accepted promotion occurs. A present pin is explicitly declarational until SavedMember::open delegates to the actual native/configured archive owner. Ready, Incomplete, Unpinned, Missing, Unsupported and Invalid semantics are not reimplemented here. No Session, Simulation, deployment/runtime, numerical loop, reducer, cache or store is created by source selection/open. Underlying SQLite open retains its existing hot-journal recovery contract; this is not a guarantee of zero filesystem writes under recovery.

The consumer of this untagged inner payload must still bind its own envelope version to the decoded source variant (schema-one native versus explicit schema-two configured), and retain the native page/recipe/result bytes and digests. This adapter does not enforce a version belonging to a different enclosing artifact. Actual page invalidation, exact member readiness/pin checks and quantitative field loading remain the existing application owners' integration work.

## Regression quality and scope

Six source-reviewed contracts are meaningful: exact native serialized key set/bytes/digest and roundtrip; configured strict roundtrip plus future versions/kind/unknown fields/spec hash/stale pin changes; owner manifest opening with actual save_new envelopes and digest corruption/version/byte-limit/I/O refusals; borrowed member access, real limits/attempts and typed differences; actual constructor SQLite archives progressing Incomplete to Unpinned to pinned Failed Ready then changed-file refusal; and a real producer OS process exiting before a fresh cold reader opens both manifests/archives without resolving the nonexistent configured deployment. The exact-test child output is required to report one actual passing test; the ordinary parent harness counts as one case, not a separate independent simulation per phase.

Configured fixtures explicitly stop at constructor failure before any interval/runtime, and the cold phase asserts it cannot enter fixture numerical-owner construction. Therefore these cases demonstrate the adapter's cold owner delegation and declarations without a deployment, not accepted configured runtime evidence, RCP3/RCT1 terminal validation, field comparison, native GUI behavior or model execution. The child does not clear PATH; no PATH-empty capability claim is admitted. The test-only path-import dead_code allowance covers public owner APIs exercised by other suites, not production warning suppression. Parent registration is exactly the new path/module declaration.

The author subsequently reports 40 passing entries (new six/native seventeen/intent nine/cold eight), strict four-target Clippy and fmt, with unchanged exact source pins. These are attributed reports, not this reviewer's execution receipts; later commit/qualification must bind the actual final files. In particular this base's configured archive owner predates the independently allocated runtime-custody extension. Root must compose the final reviewed cold owner and its clock fix rather than infer stronger Ready guarantees from this adapter's delegation. No blocker exists in these three additive paths themselves. `git diff --check` passed during this read-only review.

## Exact working-source pins

| Path | SHA-256 |
| --- | --- |
| `src/bin/desktop/run_sets/source.rs` | `c948c24fbe3d0bbb4d281c427244f7908a16506236c0f24b194490c64033cd11` |
| `src/bin/desktop/run_sets.rs` | `e194a48350954edf13cdcd01371d320cc210069b4256ba955f2ddd1132301cc1` |
| `tests/saved_source_contract.rs` | `39cb5152583c1662f8dfb71217409025084c4e05be1a18e1b7b3912fa4d21ca5` |
