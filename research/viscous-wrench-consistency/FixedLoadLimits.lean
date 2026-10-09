import Mathlib.Data.Real.Basic
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Data.Fintype.Fin
import Mathlib.Tactic.Linarith

/- Research-only limits for correcting a boundary load while freezing the
   existing finite action. These are exact-real statements, not IEEE proofs. -/
namespace Rheon.WrenchConsistencyResearch
open scoped BigOperators

/-- With fluid and outer resultants fixed, exact balance fixes the solid load. -/
theorem fixed_resultants_unique {n : ℕ} (fluid outer solid candidate : Fin n → ℝ)
    (old_balance : ∀ k, fluid k + solid k + outer k = 0)
    (new_balance : ∀ k, fluid k + candidate k + outer k = 0) :
    candidate = solid := by
  funext k
  have h := old_balance k
  have h' := new_balance k
  linarith

/-- Equality of every virtual-work pairing fixes every component of a wrench. -/
theorem all_pairings_unique {n : ℕ} (solid candidate : Fin n → ℝ)
    (same_work : ∀ test : Fin n → ℝ,
      (∑ k, test k * candidate k) = ∑ k, test k * solid k) :
    candidate = solid := by
  funext k
  have h := same_work (fun j => if j = k then 1 else 0)
  simpa using h

/-- A load cannot be recovered even by a nonlinear function of samples when
    two admitted fields share all samples but have different physical loads. -/
theorem identical_samples_block_exact_load {Field Samples : Type}
    (sample : Field → Samples) (load : Field → ℝ) (a b : Field)
    (same_samples : sample a = sample b) (different_loads : load a ≠ load b) :
    ¬ ∃ recover : Samples → ℝ, ∀ field, recover (sample field) = load field := by
  rintro ⟨recover, h⟩
  apply different_loads
  calc
    load a = recover (sample a) := (h a).symm
    _ = recover (sample b) := congrArg recover same_samples
    _ = load b := h b

end Rheon.WrenchConsistencyResearch

#print axioms Rheon.WrenchConsistencyResearch.fixed_resultants_unique
#print axioms Rheon.WrenchConsistencyResearch.all_pairings_unique
#print axioms Rheon.WrenchConsistencyResearch.identical_samples_block_exact_load
