"""Bind final diagnostic source, preserving all unrelated frozen history."""
import json
import sys
import verify_frozen as v

source = sys.argv[1]
inherited = json.loads((v.ROOT / 'evidence/forcing-temporal-diagnosis/final-receipt.json').read_text())['source_sha256']
names = set(inherited) | set(v.ALLOWED) | {str(path.relative_to(v.ROOT)) for path in v.P.glob('*.py')}
names.add('docs/research-book/implementation/forcing-finer-native-refusals.md')
receipt = dict(status=v.STATUS, qualified_source_commit=source,
    qualified_source_tree=v.git('rev-parse', source + '^{tree}').decode().strip(),
    ordered_source_parents=v.git('show', '-s', '--format=%P', source).decode().split(),
    historical_base=v.BASE, legitimate_changed_baseline_paths=v.ALLOWED,
    historical_git_blobs={path: blob for path, blob in v.tree(v.BASE).items() if path not in v.ALLOWED},
    source_sha256={path: v.sha(v.git('show', source + ':' + path)) for path in sorted(names)},
    file_sha256={path: v.sha((v.ROOT / path).read_bytes()) for path in sorted(v.inventory())},
    original_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND', original_geometry_band=[1.7, 2.3],
    original_newton_threshold=1e-13, original_iteration_window=7, original_equation_call_budget=200,
    actual_original_refusals=6, actual_preserved_state_retries=6,
    observed_candidates_publish=False, arithmetic_floor_proved=False, general_convergence_rate_proved=False,
    production_repair_implemented=False, hosted_CI_qualified=False,
    execution_facts='Initial nine Rust commands captured the same marked Rust source using explicit known-cfg flags. Four later ordinary commands qualified the marked Cargo cfg-name declaration without diagnostic RUSTFLAGS. Earlier command receipts and outputs are preserved.')
print(json.dumps(receipt, indent=2, sort_keys=True))
