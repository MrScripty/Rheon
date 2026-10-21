#!/bin/bash
set -euo pipefail
for custody_test in \
 recording::force_fields::tests::binary_rejects_truncation_trailing_unknown_versions_and_invalid_objects \
 recording::force_fields::tests::binary_roundtrip_preserves_all_variants_ids_disabled_and_float_bits \
 recording::force_fields::tests::field_schema_limits_and_accepted_custody_fail_closed \
 recording::force_fields::tests::legacy_run_hash_and_absent_metadata_are_unchanged \
 recording::force_fields::tests::malformed_run_digest_never_panics_and_is_not_rebound \
 recording::force_fields::tests::maximum_valid_scene_fits_bounded_codec \
 recording::force_fields::tests::run_binding_rejects_absent_mismatched_and_unbound_metadata
do
 /workspace/.rheon-tools/target/debug/deps/rheon-84cc08b87dcd124f "$custody_test" --exact --test-threads=1
done
