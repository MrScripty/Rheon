import Lean

/-! Logical host publication model only, not Rust/IEEE or provider side-effect
rollback. `valid` is the predicate elected by the host, not all of physics.
Immutable candidates exclude external mutable histories. -/
namespace Rheon.FollowOn.Publication

def publish {State : Type} (valid : State → Bool) (old candidate : State) : State :=
  if valid candidate then candidate else old

theorem rejected_preserves_state {State : Type} (valid : State → Bool)
    (old candidate : State) (rejected : valid candidate = false) :
    publish valid old candidate = old := by simp [publish, rejected]

theorem accepted_is_candidate {State : Type} (valid : State → Bool)
    (old candidate : State) (accepted : valid candidate = true) :
    publish valid old candidate = candidate := by simp [publish, accepted]

theorem checked_invariant_preserved {State : Type} (valid : State → Bool)
    (old candidate : State) (initial : valid old = true) :
    valid (publish valid old candidate) = true := by
  cases h : valid candidate <;> simp [publish, h, initial]

/-- Same immutable old state; not a model of provider computation/side effects. -/
def fallback {State : Type} (valid : State → Bool)
    (old primary native : State) : State :=
  if valid primary then primary else publish valid old native

theorem fallback_checked_invariant {State : Type} (valid : State → Bool)
    (old primary native : State) (initial : valid old = true) :
    valid (fallback valid old primary native) = true := by
  cases h : valid primary
  · simpa [fallback, h] using checked_invariant_preserved valid old native initial
  · simp [fallback, h]

theorem both_rejected_preserve_state {State : Type} (valid : State → Bool)
    (old primary native : State) (hp : valid primary = false)
    (hn : valid native = false) : fallback valid old primary native = old := by
  simp [fallback, publish, hp, hn]

/-- Logical composition of checks, not compatibility of physical operators. -/
theorem two_checked_stages {State : Type} (valid : State → Bool)
    (old first second : State) (initial : valid old = true) :
    valid (publish valid (publish valid old first) second) = true :=
  checked_invariant_preserved valid _ second
    (checked_invariant_preserved valid old first initial)

/-- A checked invariant alone cannot imply an unrelated accuracy predicate. -/
theorem checked_does_not_imply_accuracy :
    let checked : Nat → Bool := fun _ => true
    let accurate : Nat → Prop := fun n => n = 0
    checked (publish checked 0 1) = true ∧ ¬ accurate (publish checked 0 1) := by
  decide

end Rheon.FollowOn.Publication
