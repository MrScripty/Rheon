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

Actual red probes load the exact original test source from integration commit
1e966357. With GIT_DIR/GIT_WORK_TREE/GIT_INDEX_FILE together, seed cloning fails
before any test method executes; the invoking repository remains unchanged in
that case. With GIT_INDEX_FILE alone, the first seed checkout changes the
invoking repository's alternate index, and the following seed checkout fails.
This proves a real index mutation during setup. The probe does not claim that
git add/commit were reached or that the invoking HEAD or tracked files changed.

The only existing-file change is six lines in test_invariants.py setUpClass:
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
python3 evidence/pr9-git-isolation/probe.py --expect-red
python3 evidence/pr9-git-isolation/probe.py --variables index --expect-red
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
Before probes are successful reproductions of failing setup, not qualifications.
The --expect-red path reads original source from Git, so it remains reproducible
on the repaired branch without editing or weakening tests.

No new Rust, Clippy, Lean, solver benchmark or full historical terminal binary
qualification is claimed. Existing terminal binary limits remain unchanged.
No PR mutation, review request or merge is made. Source-specific receipt and
external final-head handoff record the remaining coordinated review disposition.
