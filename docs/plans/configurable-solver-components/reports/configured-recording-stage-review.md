# PR103 viscosity-stage and header follow-up independent review

Reviewed exact commit `beb69f65589fbca1005096a6457a540cb33f3d88`, restricted to its three-file delta from previously reviewed `5e6d3c7c83d7c9757c34827e5a37cef8522f0647`, using Git objects in `/workspace/rheon-accepted-session`. The unchanged registration and example retain their prior assessment. No Cargo, test binary, model or numerical execution was performed by this reviewer. `git diff --check 5e6d3c7 beb69f6` passed.

## Assessment

No material finding. This closes the previously assigned missing distinction between disabled viscosity and applied Newtonian viscosity zero in the recorded accepted receipt, and supplies a public header-only constructor decoder for empty/failed-start archives. The source is suitable for scoped admission and fresh composed validation. This does not complete frozen run-set archive binding, production worker/UI integration, numerical comparison integration, trained-model qualification or complete A3/A4 acceptance.

## Final five-path identity

| Path | SHA-256 |
| --- | --- |
| `src/recording/provider.rs` | `91e69991e352013be0ecc48bf7fc0b6c1c75e4a7564d9c7f9ed7d2f312c52ff3` |
| `tests/recording_provider_contract.rs` | `cc1bf048c06054a8be52449383e215d73fb00c430d5d181186a4e8592043e6d2` |
| `docs/api/configured-provider-recording.md` | `bed659ef1e03bd8b5121afd589f9bb4e795bfa4afdf8674ef3e97c5edbb76c60` |
| `src/recording.rs` (unchanged) | `d022f8f46c288538298896ae444c9780265a6bf77fa9a4e1ba4b9732cf026eb2` |
| `examples/recorded_configured_provider.rs` (unchanged) | `d00e9504f395e3e46d016e157eb68bc2b95124a69c1fb8cf88f2b10da58c4ffa` |

## Source conclusions

`configured_publication` obtains actual accepted `LiquidStepReport.viscosity` presence from the held Session snapshot and transfers it into owned `viscosity_applied`. It does not infer stage presence from the coefficient. Constructor rows retain no accepted receipt. Existing generation/time, source binding, interval authority, finite fields, host report and accepted snapshot association remain in force.

The receipt controls are explicitly versioned from `RCP1:` to `RCP2:` and serialize the stage-presence boolean separately from raw f64 coefficient bits. Existing closed boolean decoding applies. Old RCP1 rows are unsupported rather than interpreted with a guessed missing field; native recording profiles and the existing SQLite envelope are unchanged. The constructor configuration codec remains independently RSC1.

Receipt validation rejects nonfinite/negative/nonzero-subnormal coefficients and disabled-stage coefficients other than canonical positive zero. Applied zero remains representable as an explicit stage. When an applied stage is declared, carrier and represented densities must match bit-for-bit, including for applied zero. This mirrors the qualified viscous composition's domain restriction rather than using a positive-coefficient shortcut. These checks are typed receipt coherence checks, not a replay or complete numerical qualification of every diagnostic or physical input.

`configured_configuration(&RecordingRun)` is now public and first validates the generic run and exact configured profile before bounded, closed constructor-codec decoding. It needs neither a publication nor a Session, workspace, provider, model or external runtime. It preserves the original raw logical constructor-budget semantics; it is not a complete executable future recipe or process/RSS budget.

## Regression quality and evidence limits

The new actual-production tests run accepted Session calls for both Disabled and applied Newtonian zero. They verify the real report's stage presence, record constructor plus accepted state, finish and drop the writer/Session, then invoke an exact child test in a separate process. That child constructs only reader/codecs, verifies the full archive, decodes both rows, checks stage presence and exact coefficient bits, and prints a positive marker that the parent requires. Thus the source avoids treating a child process that ran zero matching tests as success. Existing expected snapshot comparison now checks stage presence for every recorded accepted receipt.

The header-only regression uses an actual closed failed-start archive with zero publications and verifies all constructor controls through the existing bitwise comparison, then refuses changed profile, trailing configuration bytes, invalid geometry and empty source identity. Further tests publish generic-valid but typed-invalid archives for old receipt version, nonzero viscosity with stage disabled, unequal density with applied zero, and nonzero subnormal viscosity. Generic verification succeeds before typed decode/full-audit refusal, separating envelope integrity from configured physical-contract coherence.

I inspected but did not execute these Rust tests. The author's and lead's reported test receipts, subprocess observations, fresh source inventory and complete composed qualification remain separate evidence to retain at their exact source commits. The earlier independent recording report remains the basis for unchanged codec/association/clock/history/raw-IEEE behavior; this report supersedes its pending stage/header follow-up limitation only.
