# First trajectory harness source stopped before execution

Source 160e9e3bb99c1e42e7fed9419c15b5deaa045878 (tree
b312d9e572b87d85f1e1b11e4cfd53de63b5d730) failed Rust compilation with E0382.
The refusal diagnostic moved the non-Copy error into a comparison Result and
then tried to format it again. No binary, scalar/baseline/memory qualification
or E1 trajectory was produced. The test overlays were restored byte-exact.

All original source, protocol and actual failed build logs are retained here.
The separate v2 source borrows that error in the diagnostic comparison. It
changes no constructor, arithmetic, equation, step size, load, threshold,
budget, interruption schedule or stopping rule, and repeats the full preflight.
