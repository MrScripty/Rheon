import Rheon.Transport

/-! Conditional finite masked-donor contracts. The visibility bits and original
nonnegative weights are inputs, not certified geometric predicates. A positive
retained total is essential; IEEE transport and mass conservation are excluded. -/
namespace Rheon.TracerBarrier
noncomputable section
open scoped BigOperators

def masked {n : ℕ} (w : Fin n → ℝ) (keep : Fin n → Bool) (i : Fin n) : ℝ :=
  if keep i then w i else 0

def total {n : ℕ} (w : Fin n → ℝ) (keep : Fin n → Bool) : ℝ := ∑ i, masked w keep i

def normalized {n : ℕ} (w : Fin n → ℝ) (keep : Fin n → Bool) (i : Fin n) : ℝ :=
  masked w keep i / total w keep

def acceptedValue (reverted : Bool) (arrival sample : ℝ) : ℝ :=
  if reverted then arrival else sample

theorem masked_nonnegative {n : ℕ} (w : Fin n → ℝ) (keep : Fin n → Bool)
    (hw : ∀ i, 0 ≤ w i) (i : Fin n) : 0 ≤ masked w keep i := by
  unfold masked
  split
  · exact hw i
  · exact le_refl 0

theorem masked_blocked {n : ℕ} (w : Fin n → ℝ) (keep : Fin n → Bool)
    (i : Fin n) (hk : keep i = false) : masked w keep i = 0 := by
  simp [masked,hk]

theorem normalized_nonnegative {n : ℕ} (w : Fin n → ℝ) (keep : Fin n → Bool)
    (hw : ∀ i, 0 ≤ w i) (ht : 0 < total w keep) (i : Fin n) :
    0 ≤ normalized w keep i := by
  exact div_nonneg (masked_nonnegative w keep hw i) (le_of_lt ht)

theorem normalized_partition {n : ℕ} (w : Fin n → ℝ) (keep : Fin n → Bool)
    (ht : 0 < total w keep) : (∑ i, normalized w keep i) = 1 := by
  unfold normalized
  simp only [div_eq_mul_inv]
  rw [← Finset.sum_mul]
  exact mul_inv_cancel₀ (ne_of_gt ht)

theorem normalized_blocked {n : ℕ} (w : Fin n → ℝ) (keep : Fin n → Bool)
    (i : Fin n) (hk : keep i = false) : normalized w keep i = 0 := by
  simp [normalized,masked,hk]

theorem visible_bounds {n : ℕ} (w q : Fin n → ℝ) (keep : Fin n → Bool) (lo hi : ℝ)
    (hw : ∀ i, 0 ≤ w i) (ht : 0 < total w keep)
    (hlo : ∀ i, keep i = true → lo ≤ q i) (hhi : ∀ i, keep i = true → q i ≤ hi) :
    lo ≤ ∑ i, normalized w keep i * q i ∧
      (∑ i, normalized w keep i * q i) ≤ hi := by
  have low (i : Fin n) : normalized w keep i * lo ≤ normalized w keep i * q i := by
    by_cases hk : keep i = true
    · exact mul_le_mul_of_nonneg_left (hlo i hk) (normalized_nonnegative w keep hw ht i)
    · simp [normalized,masked,hk]
  have high (i : Fin n) : normalized w keep i * q i ≤ normalized w keep i * hi := by
    by_cases hk : keep i = true
    · exact mul_le_mul_of_nonneg_left (hhi i hk) (normalized_nonnegative w keep hw ht i)
    · simp [normalized,masked,hk]
  constructor
  · calc
      lo = ∑ i, normalized w keep i * lo := by
        rw [← Finset.sum_mul,normalized_partition w keep ht]; ring
      _ ≤ ∑ i, normalized w keep i * q i := Finset.sum_le_sum (fun i _ => low i)
  · calc
      (∑ i, normalized w keep i * q i) ≤ ∑ i, normalized w keep i * hi :=
        Finset.sum_le_sum (fun i _ => high i)
      _ = hi := by rw [← Finset.sum_mul,normalized_partition w keep ht]; ring

theorem normalized_constant {n : ℕ} (w : Fin n → ℝ) (keep : Fin n → Bool)
    (ht : 0 < total w keep) (c : ℝ) : (∑ i, normalized w keep i * c) = c := by
  rw [← Finset.sum_mul,normalized_partition w keep ht]
  ring

theorem accepted_bounds (reverted : Bool) (arrival sample lo hi : ℝ)
    (ha : lo ≤ arrival ∧ arrival ≤ hi) (hs : lo ≤ sample ∧ sample ≤ hi) :
    lo ≤ acceptedValue reverted arrival sample ∧ acceptedValue reverted arrival sample ≤ hi := by
  unfold acceptedValue
  split
  · exact ha
  · exact hs

end
end Rheon.TracerBarrier
