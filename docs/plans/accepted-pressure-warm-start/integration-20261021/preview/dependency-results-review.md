# PR59 independent preliminary dependency-result review

2026-10-22. Read-only inspection of retained logs, summary and subprocess journal. No test, build, numerical campaign, timing, cloud/GitHub operation, artifact download or binary/source mutation was performed by this reviewer. Only this report was written.

**Conclusion: confirmation ordinary warm-feature release test run is passed as recorded by the journal and matching complete log: 792 passed, 0 failed, 13 ignored. The first outer-tool invocation remains uncertain/unsuccessful at the wrapper level and is not retroactively relabeled passed.** No result-count or source-binding discrepancy found.

## Separate outcomes and exact counts

Command recorded for confirmation: `cargo test --release --locked --features accepted-pressure-warm-start`. No `--ignored` or individual ignored selection occurs in the command. Journal records child return code **0**, start `2026-10-22T00:22:52.742446+00:00`, end `2026-10-22T00:23:51.843621+00:00` (elapsed 59.101175 seconds). Parent reports enclosing tool exit0; this local review verifies the child-return journal, not a separately supplied outer-tool receipt. Both logs are retained independently; no result was deleted or pooled.

| Invocation | Top-level results | Passed | Failed | Ignored | Measured | Filtered | Disposition |
|---|---:|---:|---:|---:|---:|---:|---|
| First | 92 | 792 | 0 | 13 | 0 | 0 | Reported outer exit1; wrapper/child outcome discrepancy remains uncertain |
| Confirmation | 92 | 792 | 0 | 13 | 0 | 0 | Journal child return0 with matching complete log |

Matched each result to one Cargo top-level Running/Doc-tests section: exactly92 sections, exactly one anchored summary each, all status ok. For every section its initial running count equals passed+failed+ignored+measured. There is one additional nested `running 1 test` inside `tests/recording_contract.rs` crash-subprocess output; it has no independent top-level Cargo section or result summary and contributes **zero additional count**. No nested-summary inflation or double-counting occurred. Both logs end with the successful3-test doc-test summary, and no anchored `error:` line is present.

| Retained evidence | SHA256 |
|---|---|
| dependency-warm-first.log | `0f7b58a89a75ecc7c3a4c55ba3439f8b04be5ece4df31c6872f88bf202e68fb0` |
| dependency-warm-confirm.log | `8a8dee9a657de9ac327c85aedf03e385cc06d46197385f1b3b8318975f02bafa` |
| dependency-warm-confirm-journal.json | `966821bbd2372f394aa95458c06c91c3fc5d22467b1a2c21d4ec96b6fb3ef2b8` |
| dependency-warm-summary.json | `6b45363b369871b050e39b7cf85961915c5f1fa100844ab17d24bc5bc3d95869` |

The confirmation hash exactly matches both journal and summary. The first log hash/totals also match summary, but successful printed test summaries do not establish a successful top-level process/tool outcome. Cause of the first outer exit1 is not established here.

## Exact ignored labels

Both logs have the same13 ignored names and reasons, with no ignored case reported executed. Verified each ignore annotation/reason in current source and the inherited PR143 source (the CV3-A integration file includes its example preparation module). This preserves explicit authorization/external-resource boundaries rather than counting skips as passes.

- `tests/force_field_solver_contract.rs::authored_mac_cancellation_retains_accepted_state_and_retry` — bounded solver integration: review and explicit execution authorization required.
- `tests/force_field_solver_contract.rs::authored_mac_matches_legacy_bounded_directional_force` — bounded solver integration: review and explicit execution authorization required.
- `tests/force_field_solver_contract.rs::authored_particle_cancel_retains_state_and_retry` — bounded solver integration: review and explicit execution authorization required.
- `tests/force_field_solver_contract.rs::authored_particle_density_conversion_adds_to_existing_acceleration` — bounded solver integration: review and explicit execution authorization required.
- `tests/force_field_solver_contract.rs::empty_snapshot_preserves_both_legacy_paths` — bounded solver integration: review and explicit execution authorization required.
- `tests/force_recording_integration_contract.rs::carrier_force_archive_round_trip_and_failed_attempt_invalidation` — post-custody numerical/SQLite integration: explicit bounded execution authorization required.
- `tests/force_recording_integration_contract.rs::particle_force_archive_round_trip_and_rejected_control_custody` — post-custody numerical/SQLite integration: explicit bounded execution authorization required.
- `tests/grid_static_mms_reference_contract.rs::preparation::preparation_contracts::continuum_keeps_absolute_pressure_and_outward_traction_sign` — CV3-A preparation: requires the frozen oracle/resource execution admission.
- `tests/grid_static_mms_reference_contract.rs::preparation::preparation_contracts::n16_ceiling_retains_complete_lattice_and_refuses_above_ceiling_before_allocation` — CV3-A preparation: requires the frozen oracle/resource execution admission.
- `tests/grid_static_mms_reference_contract.rs::preparation::preparation_contracts::new_trace_rule_resolves_degree_four_that_old_two_point_rule_misses` — CV3-A preparation: requires the frozen oracle/resource execution admission.
- `tests/pressure_session_contract.rs::frozen_model_selection_and_unsupported_domain_fallback_are_visible` — requires RHEON_AI_EXPORT from the SHA256-verified frozen checkpoint exporter.
- `tests/provider_step_contract.rs::real_model_advances_production_session_for_twenty_four_intervals` — requires RHEON_AI_EXPORT from the SHA256-verified checkpoint exporter.
- `tests/solver_provider_contract.rs::real_checkpoint_support_replay_and_corruption` — requires RHEON_AI_EXPORT from the hash-pinned real lab checkpoint.

Grouping is **7 inherited bounded force/recording integrations + 3 external-model cases + 3 CV3-A execution-admission preparation cases**. None is newly suppressed by this composition. Ordinary tests are separate from sealed scientific/performance campaigns; these log receipts are not those campaigns' execution or admission evidence.

## Exact current-source binding and remaining gates

Verified the full conservative153-path set and every hash against `/tmp/pr59-final-deps-preview-map.json`, current compact map SHA256 `8055a53c319ddec6aade4bed35fdc642c7ab07839e4e0bb2866a00a4a03d3688`. Independently re-enumerated the selector to exclude added/missing paths. All24 earlier non-Cargo review hashes and all recorded unchanged admission/selector/qualifier/oracle/control files remain exact. HEAD is `69754219348d7391c1f5deac685c43b1981382ee` with pending MERGE_HEAD `159fa74bd271f8fcc07f3837d66e4738925c820c`; resolved Cargo is staged. This is still the preliminary dependency composition, not final actual-main qualification. The source bytes reviewed remain unchanged; no fresh source/admission approval is inferred from test success.

This inspection verifies existing receipts and current hashes without independently reproducing runtime results or proving the live process/build provenance beyond retained command/log/journal records. Native inventory remains stale for warm source, final actual ordered #142/#138/#143 qualification is unrun, and explicit source admission, warm CI, exact-head checks, prospective fixed controls and lead authorization remain necessary. Historical **+2.20%** adverse particle evidence and the strict positive/inconclusive stopping rule remain unchanged. No universal performance, scientific or merge admission follows.

No denied action occurred. No expired shell token was retried and no GitHub/cloud/artifact operation was attempted; access/connection status was outside this local result review. The first outcome uncertainty is retained explicitly, while the separate confirmation result has a recorded successful child status and exact log binding.
