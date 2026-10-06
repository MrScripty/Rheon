"""Create an additive diagnosis receipt after the source commit is frozen."""
import json
import sys
import verify_frozen as v


def run(source):
    inherited = json.loads((v.ROOT / v.OLD_ROOT).read_text())['source_sha256']
    sources = dict(inherited)
    added = [str(path.relative_to(v.ROOT)) for path in v.P.glob('*.py')]
    added += ['examples/forcing_temporal_probe.rs', 'docs/research-book/implementation/forcing-temporal-regime-diagnosis.md']
    for path in added:
        sources[path] = v.sha(v.git('show', source + ':' + path))
    # Bind every Rust input plus the complete inherited Python research code.
    # Column workflow/reader edits on normally merged main are separate from
    # this executed numerical source set; the frozen source tree remains bound.
    executed = {path: digest for path, digest in sources.items()
                if path.startswith('src/') or path.endswith('.py') and path.startswith('evidence/')
                or path in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
                            'examples/forced_extruded.rs', 'examples/forcing_temporal_probe.rs',
                            'tests/forced_extruded_contract.rs']}
    return dict(status=v.STATUS, classification=v.CLASSIFICATION,
                original_temporal_gate='FAIL_ORIGINAL_TEMPORAL_BAND', original_geometry_band=[1.7, 2.3],
                qualified_source_commit=source,
                qualified_source_tree=v.git('rev-parse', source + '^{tree}').decode().strip(),
                ordered_source_parents=v.git('show', '-s', '--format=%P', source).decode().split(),
                historical_base=v.BASE, legitimate_changed_baseline_paths=[],
                historical_git_blobs=v.tree(v.BASE), source_sha256=sources,
                executed_source_sha256=executed,
                file_sha256={path: v.sha((v.ROOT / path).read_bytes()) for path in sorted(v.packet_files())},
                prior_frozen_receipt_sha256=v.OLD_ROOT_SHA,
                production_kernel_changed=False, hosted_CI_qualified=False,
                numerical_and_proof_limits='Original five-interval failure and six finer IterationLimit refusals retained. Descriptive regime evidence, no formal asymptotic-order theorem or general-liquid qualification.')


if __name__ == '__main__':
    print(json.dumps(run(sys.argv[1]), indent=2, sort_keys=True))
