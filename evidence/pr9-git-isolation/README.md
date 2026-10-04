# Bounded PR9 Git test isolation

This branch descends from PR8 integration head
`1e966357c45a92ee2c318fc47921b3e2f8e539a7`, tree
`4ec1b939de418e210ba78f139d01daf0903cceaf`. Its ordered merge parents
`af15099f920bb046a955d0d93849e3bc1b758f1c` and
`1830b9b0f918f5eb309e6453fd3432afbdb7b408` remain intact.

## Finding and bounded disposition

PR9 review 5408047076, run `225a0b53-15bb-414e-81c8-cdd49a229108`, reviewed
accepted source 1830b9b and reported one
[Minor harness finding](https://github.com/MrScripty/Rheon/pull/9#discussion_r4179210717).
The documented unittest invocation does not clear inherited repository-local Git
variables. This is a valid test-harness isolation issue; no solver or verifier
implementation defect was demonstrated by this finding.

Recorded red probes load the exact original test source from integration commit
1e966357 and were run with the source checkout still at that baseline. With
GIT_DIR/GIT_WORK_TREE/GIT_INDEX_FILE together, seed cloning fails
before any test method executes; the invoking repository remains unchanged in
that case. With GIT_INDEX_FILE alone, the first seed checkout changes the
invoking repository's alternate index, and the following seed checkout fails.
This proves a real index mutation during setup. The probe does not claim that
git add/commit were reached or that the invoking HEAD or tracked files changed.

The functional fix is six lines in test_invariants.py setUpClass:
ask Git for its repository-local environment names, save inherited values,
register their restoration with unittest class cleanup, and remove them before
seed setup. All subsequent temporary-checkout and verifier Git subprocesses use
their intended repository. Cleanup restores inherited values even when setup
raises. No global Git configuration, credentials, permissions, network settings,
solver/verifier, book/proof, fixture or prior evidence bytes are changed.

## Actual validation

probe.py creates a disposable invoking clone, marker file and alternate index.
It executes the real existing 12-method suite in a child Python process with
the chosen inherited environment. Its before/after snapshots compare HEAD,
all refs, tracked file contents, both indexes, marker bytes and working status.
It also checks environment restoration in the same child process after the
complete suite and an injected real setup exception.

```sh
python3 evidence/pr9-git-isolation/probe.py
python3 evidence/pr9-git-isolation/probe.py --variables index
python3 -O evidence/pr9-git-isolation/probe.py
python3 -O evidence/pr9-git-isolation/probe.py --variables index
PYTHONOPTIMIZE=1 python3 evidence/pr9-git-isolation/probe.py
PYTHONOPTIMIZE=1 python3 evidence/pr9-git-isolation/probe.py --variables index
python3 -m unittest discover -s evidence/pr7-verifier-repair -p test_invariants.py -v
```

Each of the six after probes passes all 12 methods, preserves every invoking
snapshot field, and restores the environment after suite completion and injected
setup failure. The direct documented invocation also passes all 12 methods.
Recorded before probes are successful reproductions of failing setup, not
qualifications. The --expect-red path reads original test code from Git, but its
seed clones still start from the invoking source checkout's HEAD. On repaired
head 3cc13d44, the index-only probe aborts its first checkout before changing the
alternate index, so --variables index --expect-red exits 1. Original test code
alone is insufficient to replay the recorded mutation from that repaired head.

For the exact red reproduction, run this recipe from the repaired checkout in a
shell without inherited repository-local Git variables. It makes a separate
baseline checkout and copies only the recorded probe into it; the probe itself
injects the inherited variables. Both commands exit 0 for the expected red
reproduction, with failed child setup and zero test methods. The index-only
report has invoking_repository_preserved=false and only operator_index_sha256
in changed_snapshot_fields.

```sh
rheon_before=$(mktemp -d)
git clone --quiet --shared --no-checkout . "$rheon_before/checkout"
git -C "$rheon_before/checkout" checkout --quiet --detach 1e966357c45a92ee2c318fc47921b3e2f8e539a7
mkdir -p "$rheon_before/checkout/evidence/pr9-git-isolation"
git show 3cc13d44f192a31dc16be17436f9a156b317266f:evidence/pr9-git-isolation/probe.py > "$rheon_before/checkout/evidence/pr9-git-isolation/probe.py"
(
    cd "$rheon_before/checkout"
    python3 evidence/pr9-git-isolation/probe.py --expect-red
    python3 evidence/pr9-git-isolation/probe.py --variables index --expect-red
)
```

The historical receipt and recorded snapshots are retained unchanged and remain
bound to their recorded source commit. This successor corrects reproduction
instructions only; the accepted Git isolation fix and probe are unchanged.

No new Rust, Clippy, Lean, solver benchmark or full historical terminal binary
qualification is claimed. Existing terminal binary limits remain unchanged.
No PR mutation, review request or merge is made. Source-specific receipt and
external final-head handoff record the remaining coordinated review disposition.
