# Repository-lead source inventory through parent

DRAFT PR132 on fix/numerical-evaluator-inputs-20261020 (PR130), frozen numerical
source0f4e7e0944a7e0302690a0f71c08c21fe345cc89. No rebases/merges/cleanup.
New exact write set vs c2fbbea: Cargo.toml/Cargo.lock dev dependencies, private
examples/grid_geometry_reference.rs, its geometry.rs/diagnostics.rs/new exact.rs,
and this new q2-exact owner. Existing test adapter unchanged. No production source,
lib exports, Session, UI, recording, composition, shared canonical lifecycle or
proof changes. Existing Q1 production LiquidVolume changes belong to PR126.

Fresh origin/main now718469beedb9a949f4bb6f3299c7b880b4c47e47 (audit119 andplan120
merged at unchanged heads); this branch remains qualified at its frozen source.
No combined-main or C1/G1 source-admission claim. Lead owns final Cargo/CI/source
inventory reconciliation and exact integration qualification. Preserved PR126,
PR127 andPR130 heads remain dfcc693/7d7306a/c2fbbea. Six earlier numerical/tooling
archives match their recorded hashes. Q1 n64/Full findings remain separate.

Next exclusive proposed owner: examples/grid_coupled_reference.rs and optional
private submodules; tests/grid_coupled_reference_contract.rs; q2-coupled/*. No
Cargo changes or existing production/Session/provider/geometry-owner modifications.
Aligned algebra reference is not an oblique/free-surface extension/DOF admission.
