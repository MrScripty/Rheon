# Evidence qualification

This qualifies a bounded negative selection result. No production method,
physical water accuracy, native viscosity, mesh, free-surface trajectory,
acoustic timestep, GPU or desktop claim is made.

Frozen scientific source: e457ac12e874029229c381d30db8481caaee5cf6. Source was
independently reviewed and committed before numerical evaluation. The unchanged
PR121 kernel and EOS are reused by path, without production imports or mutation.
Native release executable SHA256:
6251bff2cfd82a2768ab2cc049cf33e3b6e2cc2d03c7b989aa6a321487a257db.
Linux x86_64, Rust/cargo1.92.0; actual rustc verbose identity retained. SI inputs,
perturbation and mass conventions are frozen in protocol. The closed hydro
probe's 1000Pa top pressure yields about1.06105% exact bottom compression, outside
the production1% envelope at c32. This deliberate derivative probe is not the
plan's original free-surface hydro acceptance fixture. Full committed source
snapshot, binary, raw CSVs, exact evaluator, logs and context are retained.

| Selected claim/check | Outcome | Limit |
|---|---|---|
| A1 equation/adjoint implementation contracts | 6/6 debug and 6/6 release tests pass | Includes unchanged EOS/kernel checks; native f64 only |
| Affected strict Clippy | example and adapter pass, -D warnings | no-default-features CPU targets |
| A2 static causal campaign | exit0; 16 cases, 768 sites, 48 differences | negative selection, no dynamics |
| Directional derivative criteria | all48 pass at frozen scales | finite-difference evidence, not a formal global proof |
| Work/action-reaction criteria | all16 pass | does not establish consistency or physical wall load |
| E0 criterion | gamma0/4, reference4/4 uniform cases | both still unqualified for other distinct failures |
| A3 independent review | pre-freeze and post-run reconstruction passed | read-only separate agent, no physical experiment |
| Scope and hygiene | formatting, diff, frozen original sources preserved | no production/API/dependency/workflow changes |

Commands:

```
cargo test --locked --no-default-features --test particle_adjoint_selection_contract
cargo test --locked --release --no-default-features --test particle_adjoint_selection_contract
cargo clippy --locked --no-default-features --example particle_adjoint_selection -- -D warnings
cargo clippy --locked --no-default-features --test particle_adjoint_selection_contract -- -D warnings
cargo run --locked --release --no-default-features --example particle_adjoint_selection -- FRESH_OUTPUT
python3 docs/plans/fluid-accuracy-composition/p1-adjoint/analyze.py FRESH_OUTPUT
```

The fresh output path must not exist; outputs are explicitly flushed. Assertions
fail closed on unordered/outside sites, nonpositive map numerator/gamma/volume or
a density state crossing the declared wet branch. Later documentation edits do
not change scientific source/protocol/evaluator. No new Lean module or historical
qualification inventory is modified. No existing large CPU regression is rerun:
this slice adds an isolated executable/adapter and has no production write; PR121
retains its complete 582-test release regression. Hosted CI is separate evidence.

Evidence archive `rheon-p1-adjoint-selection-e457ac1.zip` is retained in Library;
[manifest](manifest.json) binds the successful receipt, archive/executable hashes
and file counts. The archive includes full frozen source and the independent
review script and summary. Its external custody receipt is excluded from its own
bytes to avoid a self-hash cycle. Archive CRC/inventory were verified. Actual
shell/process/filesystem access was rechecked after the reported environment
disconnect at00:58 UTC: working, archive unchanged, no campaign process running,
and no duplicate campaign started. Source checkpoint e457ac1 was already pushed.
