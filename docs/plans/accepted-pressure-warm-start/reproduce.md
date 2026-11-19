# Reproducing the CPU qualification

Run from the final feature checkout with Rust 1.92.0 and Python 3.12. The reference
comparison additionally needs NumPy 2.3.5. Use isolated source and Cargo target
roots. Do not run builds, tests or other workloads concurrently with measurements.
The recorded host has a four-CPU quota; 1, 2 and 4 are worker-pool sizes, not a
claim about physical core isolation. See `environment.json` and raw manifests.

## Frozen build inputs

Baseline: `3525c979e16629b295a6865e3abbb2a36803cc93` (PR 55).
Rejected candidate: `ec535bcfe1ce944c73b61653d83f4bb2e206c7c5`.
Corrected production candidate: `3569d3b08419340c6fbb0762a4a8c660301f3977`.
Later evidence-only changes do not change these measured binaries.

The exact baseline grid harness is `baseline-harness.rs.txt`; its difference from
the candidate example is retained in `baseline-harness.diff`. It uses the legacy
cold call and predates optional reference/dump support. Eight-argument performance
runs use identical normal tolerances and timed loop structure. This does not claim
identical code layout; candidate-cold controls compare the actual frozen binaries.

For example, with a new, empty scratch directory:

```bash
bench_root=$(mktemp -d)
mkdir "$bench_root/base" "$bench_root/candidate"
git archive 3525c979e16629b295a6865e3abbb2a36803cc93 | tar -x -C "$bench_root/base"
git archive 3569d3b08419340c6fbb0762a4a8c660301f3977 | tar -x -C "$bench_root/candidate"
cp docs/plans/accepted-pressure-warm-start/baseline-harness.rs.txt "$bench_root/base/examples/accepted_pressure_lab.rs"
cp examples/cpu_particle_lab.rs "$bench_root/base/examples/cpu_particle_lab.rs"
CARGO_TARGET_DIR="$bench_root/base-target" cargo build --manifest-path "$bench_root/base/Cargo.toml" --locked --release --features work-observation --example accepted_pressure_lab --example cpu_particle_lab
CARGO_TARGET_DIR="$bench_root/candidate-target" cargo build --manifest-path "$bench_root/candidate/Cargo.toml" --locked --release --features work-observation --example accepted_pressure_lab --example cpu_particle_lab
```

Record SHA-256 hashes of sources and executables before running. Build-path and
platform differences can change executable hashes and timings; do not overwrite
or mix the retained qualification cohort with a new run.

## Workloads and analysis

Set `baseline` and `candidate` to the corresponding `release/examples/accepted_pressure_lab`
executables. Run `run_matrix.py` serially for `primary-small`, `primary-64`,
`controls`, then `sgs`, passing the same new output directory. The driver records
all six variant permutations, binary hashes, commands, CSV attempts and child RSS.
Run `analyze_matrix.py OUTPUT` only after all four groups complete. Its default
mode requires the full matrix and exits unsuccessfully on violations. `--progress`
allows partial evidence inspection but cannot qualify a dataset.

`compare_reference.py CANDIDATE NEW_OUTPUT` runs the explicit tighter cold reference
and records terminal field discrepancies and snapshot hashes. Its timings are not
performance evidence, and it is not an independent numerical oracle.
`particle_controls.py BASE_PARTICLE CANDIDATE_PARTICLE NEW_OUTPUT` records the
unchanged SPH control workload, including per-step and phase costs.

The measurement plan records the separate bounded cold confirmations. Their cohorts
must remain separate from screening. `confirm_controls.py BASE CANDIDATE CASES_JSON
NEW_OUTPUT` runs exactly twelve balanced pairs per frozen case, using the original
16-step horizon. Do not extend or pool these samples after seeing their results.

## Retained evidence

`archive_measurements.py unpack ARCHIVE.jsonl NEW_EMPTY_DIRECTORY` restores raw text
files with verified SHA-256 hashes. Packing refuses nested content and destinations
inside the input directory. Derived `summary.json` is intentionally omitted; regenerate
it with the analyzer. Original rejected and interrupted attempts remain distinct
from the corrected cohort. Never interpret a failed or shortened horizon as a gain.

Independent acceptance tests are in `tests/pressure_initial_guess.rs` and
`tests/accepted_pressure_history.rs`, with provider/session integration coverage.
The actual large-grid failure trigger is `check_gauge_regression.py`; the smaller
hand-assembled oracle is not claimed to reproduce the old stagnation.


The final frozen cohort is in `confirmation-plan.json`. Its six cold cases use
`confirmation-cases.json`; run `confirm_rest.py BASE CANDIDATE NEW_OUTPUT` for the
separate rest partition, and `particle_controls.py BASE_PARTICLE CANDIDATE_PARTICLE
NEW_OUTPUT --confirm-single-worker` for the two particle partitions. The exact
original particle screening driver is retained separately. This totals 228 runs;
do not add samples after observing outcomes.
