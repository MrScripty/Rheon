# First source revision stopped before any E1 call

Source cd157ed39c9b215e7aeb547c92be0dffb0630e40, tree
cf3ee0742705b9c342defe21a53ab41864fd537c, failed Rust compilation with E0277.
The harness omitted `use candidate::CoupledDiscreteError;`, so scalar.rs's
unchanged `super::CoupledDiscreteError` resolved to the imported production
error rather than the private candidate error. The reviewed scalar source is
unchanged; this was a harness namespace error, not a numerical result.

Formatting passed. The private Clone/test overlays were restored byte-exact
by the build script's finally block. No binary was produced, no native scalar/
baseline preflight completed, no memory qualification passed, and no E1 call
was executed. build.log retains the actual compiler diagnostic; all commands
and source hashes remain frozen. A separate v2 source packet corrects only
the harness import before repeating every required new-binary preflight.
