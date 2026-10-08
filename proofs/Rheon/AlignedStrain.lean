import Rheon.BoundedPhysics
import Rheon.ObstacleOperators
import Mathlib.Algebra.Order.BigOperators.Ring.Finset
import Mathlib.Data.Finset.Fold

/-! Exact finite-row contracts for the reconstructed aligned strain operator.
The coefficient bound is derived, including zero rows, from the per-face
absolute-row bound. Assembly, represented geometry, IEEE arithmetic and
approximate projection are external obligations. -/
namespace Rheon.AlignedStrain
noncomputable section
open scoped BigOperators
open Rheon.ObstacleOperators

def gather {n r : ℕ} (E : Fin r → Fin n → ℝ) (u : Fin n → ℝ) (q : Fin r) : ℝ :=
  ∑ f, E q f * u f

def strainOperator {n r : ℕ} (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ)
    (u : Fin n → ℝ) (f : Fin n) : ℝ :=
  ∑ q, E q f * w q * gather E u q

def strainLoss {n r : ℕ} (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ)
    (u : Fin n → ℝ) : ℝ := ∑ q, w q * (gather E u q)^2

def rowAbs {n r : ℕ} (E : Fin r → Fin n → ℝ) (q : Fin r) : ℝ :=
  ∑ f, |E q f|

def faceBound {n r : ℕ} (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ)
    (mass : Fin n → ℝ) (f : Fin n) : ℝ :=
  (∑ q, w q * |E q f| * rowAbs E q) / mass f

/-- Maximum of face bounds, with zero for the empty active space. -/
def rowBound {n r : ℕ} (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ)
    (mass : Fin n → ℝ) : ℝ := Finset.univ.fold max 0 (faceBound E w mass)

theorem row_bound_nonnegative {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (mass : Fin n → ℝ) : 0 ≤ rowBound E w mass := by
  exact Finset.le_fold_max.mpr (Or.inl le_rfl)

theorem face_le_row_bound {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (mass : Fin n → ℝ) (f : Fin n) :
    faceBound E w mass f ≤ rowBound E w mass := by
  exact Finset.le_fold_max.mpr (Or.inr ⟨f, Finset.mem_univ f, le_rfl⟩)

def forceNorm {n r : ℕ} (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ)
    (mass u : Fin n → ℝ) : ℝ :=
  ∑ f, (strainOperator E w u f)^2 / mass f

def euler {n r : ℕ} (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ)
    (mass u : Fin n → ℝ) (dt mu : ℝ) (f : Fin n) : ℝ :=
  u f - dt * mu * strainOperator E w u f / mass f

theorem transpose_work {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (u : Fin n → ℝ) :
    (∑ f, strainOperator E w u f * u f) = strainLoss E w u := by
  exact Rheon.BoundedPhysics.viscous_work E w u

theorem loss_nonnegative {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (u : Fin n → ℝ) (hw : ∀ q, 0 ≤ w q) :
    0 ≤ strainLoss E w u := by
  exact Finset.sum_nonneg (fun q _ => mul_nonneg (hw q) (sq_nonneg _))

theorem transpose_pairing {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (u v : Fin n → ℝ) :
    (∑ f, strainOperator E w u f * v f) =
      ∑ q, w q * gather E u q * gather E v q := by
  unfold strainOperator
  simp only [Finset.sum_mul]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro q _
  calc
    (∑ f, E q f * w q * gather E u q * v f) =
        ∑ f, (w q * gather E u q) * (E q f * v f) := by
      apply Finset.sum_congr rfl
      intro f _
      ring
    _ = w q * gather E u q * gather E v q := by
      rw [← Finset.mul_sum]
      rfl

theorem operator_symmetric {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (u v : Fin n → ℝ) :
    (∑ f, strainOperator E w u f * v f) =
      ∑ f, strainOperator E w v f * u f := by
  rw [transpose_pairing, transpose_pairing]
  apply Finset.sum_congr rfl
  intro q _
  ring

theorem row_abs_nonnegative {n r : ℕ} (E : Fin r → Fin n → ℝ) (q : Fin r) :
    0 ≤ rowAbs E q := Finset.sum_nonneg (fun f _ => abs_nonneg _)

theorem coefficient_le_row_abs {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (q : Fin r) (f : Fin n) : |E q f| ≤ rowAbs E q := by
  exact Finset.single_le_sum (fun g _ => abs_nonneg (E q g)) (Finset.mem_univ f)

theorem zero_row_coefficient {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (q : Fin r) (h : rowAbs E q = 0) (f : Fin n) : E q f = 0 := by
  have hb := coefficient_le_row_abs E q f
  rw [h] at hb
  exact abs_eq_zero.mp (le_antisymm hb (abs_nonneg _))

theorem zero_row_gather {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (u : Fin n → ℝ) (q : Fin r) (h : rowAbs E q = 0) : gather E u q = 0 := by
  unfold gather
  simp_rw [zero_row_coefficient E q h, zero_mul]
  simp

/-- Weighted Cauchy at one face; the zero-row branch needs no division. -/
theorem face_cauchy {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (u : Fin n → ℝ) (hw : ∀ q, 0 ≤ w q) (f : Fin n) :
    (strainOperator E w u f)^2 ≤
      (∑ q, w q * |E q f| * rowAbs E q) *
        ∑ q, w q * |E q f| * (gather E u q)^2 / rowAbs E q := by
  apply Finset.sum_sq_le_sum_mul_sum_of_sq_eq_mul Finset.univ
    (r := fun q => E q f * w q * gather E u q)
    (f := fun q => w q * |E q f| * rowAbs E q)
    (g := fun q => w q * |E q f| * (gather E u q)^2 / rowAbs E q)
  · intro q _
    exact mul_nonneg (mul_nonneg (hw q) (abs_nonneg _)) (row_abs_nonnegative E q)
  · intro q _
    exact div_nonneg (mul_nonneg (mul_nonneg (hw q) (abs_nonneg _)) (sq_nonneg _))
      (row_abs_nonnegative E q)
  · intro q _
    by_cases h : rowAbs E q = 0
    · simp [h, zero_row_coefficient E q h f]
    · field_simp [h]
      ring_nf
      simp only [sq_abs]

/-- The per-face coefficient bound proves K M^-1 K <= B K as quadratic forms. -/
theorem coefficient_force_bound {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (mass u : Fin n → ℝ) (B : ℝ)
    (hw : ∀ q, 0 ≤ w q) (hm : ∀ f, 0 < mass f)
    (hB : 0 ≤ B) (bound : ∀ f, faceBound E w mass f ≤ B) :
    forceNorm E w mass u ≤ B * strainLoss E w u := by
  have hf : ∀ f, (strainOperator E w u f)^2 / mass f ≤
      B * ∑ q, w q * |E q f| * (gather E u q)^2 / rowAbs E q := by
    intro f
    have hn : 0 ≤ ∑ q, w q * |E q f| * (gather E u q)^2 / rowAbs E q := by
      apply Finset.sum_nonneg
      intro q _
      exact div_nonneg (mul_nonneg (mul_nonneg (hw q) (abs_nonneg _)) (sq_nonneg _))
        (row_abs_nonnegative E q)
    have hb : (∑ q, w q * |E q f| * rowAbs E q) ≤ B * mass f :=
      (div_le_iff₀ (hm f)).mp (bound f)
    apply (div_le_iff₀ (hm f)).mpr
    calc
      (strainOperator E w u f)^2 ≤ _ := face_cauchy E w u hw f
      _ ≤ (B * mass f) * _ := mul_le_mul_of_nonneg_right hb hn
      _ = _ := by ring
  have hs : (∑ f, ∑ q, w q * |E q f| * (gather E u q)^2 / rowAbs E q) =
      strainLoss E w u := by
    rw [Finset.sum_comm]
    unfold strainLoss
    apply Finset.sum_congr rfl
    intro q _
    by_cases h : rowAbs E q = 0
    · simp [h, zero_row_gather E u q h]
    · calc
        (∑ f, w q * |E q f| * (gather E u q)^2 / rowAbs E q) =
            (w q * (gather E u q)^2 / rowAbs E q) * ∑ f, |E q f| := by
          rw [Finset.mul_sum]
          apply Finset.sum_congr rfl
          intro f _
          ring
        _ = w q * (gather E u q)^2 := by
          change (w q * (gather E u q)^2 / rowAbs E q) * rowAbs E q = _
          exact div_mul_cancel₀ _ h
  calc
    forceNorm E w mass u ≤ ∑ f, B *
        ∑ q, w q * |E q f| * (gather E u q)^2 / rowAbs E q :=
      Finset.sum_le_sum (fun f _ => hf f)
    _ = B * strainLoss E w u := by rw [← Finset.mul_sum, hs]

/-- Exact coordinate Euler energy expansion, before any step restriction. -/
theorem euler_energy_identity {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (mass u : Fin n → ℝ) (dt mu : ℝ)
    (hm : ∀ f, mass f ≠ 0) :
    kineticEnergy mass (euler E w mass u dt mu) - kineticEnergy mass u =
      -(dt * mu) * strainLoss E w u + (dt * mu)^2 / 2 * forceNorm E w mass u := by
  have local_identity : ∀ f, mass f / 2 * (u f - dt * mu *
      strainOperator E w u f / mass f)^2 - mass f / 2 * (u f)^2 =
      -(dt * mu) * (strainOperator E w u f * u f) +
        (dt * mu)^2 / 2 * ((strainOperator E w u f)^2 / mass f) := by
    intro f
    field_simp [hm f]
    ring
  unfold kineticEnergy euler
  rw [← Finset.sum_sub_distrib]
  calc
    _ = ∑ f, (-(dt * mu) * (strainOperator E w u f * u f) +
        (dt * mu)^2 / 2 * ((strainOperator E w u f)^2 / mass f)) := by
      apply Finset.sum_congr rfl
      intro f _
      exact local_identity f
    _ = -(dt * mu) * strainLoss E w u +
        (dt * mu)^2 / 2 * forceNorm E w mass u := by
      rw [Finset.sum_add_distrib, ← Finset.mul_sum, ← Finset.mul_sum, transpose_work]
      rfl

/-- Unforced exact-real Euler decrease under the derived coefficient restriction. -/
theorem euler_energy_nonincrease {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (mass u : Fin n → ℝ) (dt mu B : ℝ)
    (hw : ∀ q, 0 ≤ w q) (hm : ∀ f, 0 < mass f)
    (ht : 0 ≤ dt) (hmu : 0 ≤ mu) (hB : 0 ≤ B)
    (bound : ∀ f, faceBound E w mass f ≤ B) (step : dt * mu * B ≤ 2) :
    kineticEnergy mass (euler E w mass u dt mu) ≤ kineticEnergy mass u := by
  have hb := coefficient_force_bound E w mass u B hw hm hB bound
  have hd := loss_nonnegative E w u hw
  have hi := euler_energy_identity E w mass u dt mu (fun f => ne_of_gt (hm f))
  have t0 : 0 ≤ dt * mu := mul_nonneg ht hmu
  have hterm : (dt * mu)^2 / 2 * forceNorm E w mass u ≤
      (dt * mu)^2 / 2 * (B * strainLoss E w u) :=
    mul_le_mul_of_nonneg_left hb (div_nonneg (sq_nonneg _) (by norm_num))
  have hc : dt * mu * B * strainLoss E w u ≤ 2 * strainLoss E w u :=
    mul_le_mul_of_nonneg_right step hd
  have hc' := mul_le_mul_of_nonneg_left hc t0
  nlinarith

/-- The computed finite maximum supplies the bound premises automatically. -/
theorem euler_row_bound_nonincrease {n r : ℕ} (E : Fin r → Fin n → ℝ)
    (w : Fin r → ℝ) (mass u : Fin n → ℝ) (dt mu : ℝ)
    (hw : ∀ q, 0 ≤ w q) (hm : ∀ f, 0 < mass f)
    (ht : 0 ≤ dt) (hmu : 0 ≤ mu) (step : dt * mu * rowBound E w mass ≤ 2) :
    kineticEnergy mass (euler E w mass u dt mu) ≤ kineticEnergy mass u := by
  exact euler_energy_nonincrease E w mass u dt mu (rowBound E w mass)
    hw hm ht hmu (row_bound_nonnegative E w mass) (face_le_row_bound E w mass) step

/-- Composition uses exactly the pressure masses and exact full residual. -/
theorem euler_exact_projection_nonincrease {n r c : ℕ}
    (E : Fin r → Fin n → ℝ) (w : Fin r → ℝ)
    (P : Fin c → Fin n → ℝ) (area distance u : Fin n → ℝ) (p : Fin c → ℝ)
    (rho dt mu B : ℝ) (hw : ∀ q, 0 ≤ w q)
    (hr : 0 < rho) (ha : ∀ f, 0 < area f) (hd : ∀ f, 0 < distance f)
    (ht : 0 < dt) (hmu : 0 ≤ mu) (hB : 0 ≤ B)
    (bound : ∀ f, faceBound E w (faceMass rho area distance) f ≤ B)
    (step : dt * mu * B ≤ 2)
    (solved : ∀ i, pressureResidual P rho dt area distance
      (euler E w (faceMass rho area distance) u dt mu) p i = 0) :
    kineticEnergy (faceMass rho area distance)
      (pressureCorrected P rho dt distance
        (euler E w (faceMass rho area distance) u dt mu) p) ≤
      kineticEnergy (faceMass rho area distance) u := by
  have hm : ∀ f, 0 < faceMass rho area distance f :=
    fun f => mul_pos (mul_pos hr (ha f)) (hd f)
  exact le_trans
    (exact_pressure_energy_nonincrease P rho dt area distance _ p hr ha hd
      (ne_of_gt ht) solved)
    (euler_energy_nonincrease E w _ u dt mu B hw hm (le_of_lt ht) hmu hB bound step)

/-- The reflected unit corner has exactly the claimed coupled quadratic block. -/
theorem unit_corner_block (U V : ℝ) :
    (1 / 4 : ℝ) * (2 * U + 2 * V)^2 + (1 / 4 : ℝ) * (2 * U)^2 +
      (1 / 4 : ℝ) * (2 * V)^2 = 2 * U^2 + 2 * U * V + 2 * V^2 := by ring

/-- Geometric area equality is an explicit extra volume hypothesis. -/
theorem flat_conductance (volume area delta : ℝ) (hd : delta ≠ 0)
    (product : volume = area * delta) : volume / delta^2 = area / delta := by
  rw [product]
  field_simp [hd] <;> ring

end
end Rheon.AlignedStrain
