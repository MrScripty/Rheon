import Mathlib.Data.Real.Basic
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Fin
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring

/-! Exact-real finite coupled backward-Euler work with a momentum defect.
No Rust, geometry, free-surface assembly, convergence, IEEE or experimental
claim. B is integrated outward flux; pressure force is +B^T p. S/W are fixed
in the interval. Homogeneous stationary boundaries only. Forcing and moving
boundaries require explicit extra work. Positive masses are needed for a norm;
the algebraic identity itself holds for arbitrary real masses. -/
namespace Rheon.FollowOn.StokesWork
noncomputable section
open scoped BigOperators

def flux {n c : Nat} (B : Fin c → Fin n → ℝ) (v : Fin n → ℝ) (i : Fin c) : ℝ :=
  ∑ j, B i j * v j

def gradient {n c : Nat} (B : Fin c → Fin n → ℝ) (p : Fin c → ℝ) (j : Fin n) : ℝ :=
  ∑ i, B i j * p i

def strain {n s : Nat} (S : Fin s → Fin n → ℝ) (v : Fin n → ℝ) (a : Fin s) : ℝ :=
  ∑ j, S a j * v j

def viscous {n s : Nat} (S : Fin s → Fin n → ℝ) (w : Fin s → ℝ)
    (v : Fin n → ℝ) (j : Fin n) : ℝ :=
  ∑ a, S a j * w a * strain S v a

def dissipation {n s : Nat} (S : Fin s → Fin n → ℝ) (w : Fin s → ℝ)
    (v : Fin n → ℝ) : ℝ := ∑ a, w a * (strain S v a)^2

def energy {n : Nat} (mass v : Fin n → ℝ) : ℝ := ∑ j, mass j / 2 * (v j)^2

theorem pressure_work {n c : Nat} (B : Fin c → Fin n → ℝ)
    (p : Fin c → ℝ) (v : Fin n → ℝ) :
    (∑ j, gradient B p j * v j) = ∑ i, p i * flux B v i := by
  simp only [gradient, flux, Finset.sum_mul, Finset.mul_sum]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro i _
  apply Finset.sum_congr rfl
  intro j _
  ring

theorem viscous_work {n s : Nat} (S : Fin s → Fin n → ℝ)
    (w : Fin s → ℝ) (v : Fin n → ℝ) :
    (∑ j, viscous S w v j * v j) = dissipation S w v := by
  unfold viscous dissipation
  simp only [Finset.sum_mul]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro a _
  calc
    (∑ j, S a j * w a * strain S v a * v j) =
        (w a * strain S v a) * ∑ j, S a j * v j := by
      rw [Finset.mul_sum]
      apply Finset.sum_congr rfl
      intro j _
      ring
    _ = w a * (strain S v a)^2 := by unfold strain; ring

theorem coupled_work_with_defect {n c s : Nat}
    (B : Fin c → Fin n → ℝ) (S : Fin s → Fin n → ℝ)
    (w : Fin s → ℝ) (mass old v defect : Fin n → ℝ)
    (p : Fin c → ℝ) (dt : ℝ)
    (step : ∀ j, mass j * (v j - old j) + dt * viscous S w v j -
      dt * gradient B p j = defect j) :
    energy mass v - energy mass old + energy mass (fun j => v j - old j) +
      dt * dissipation S w v =
      dt * (∑ i, p i * flux B v i) + ∑ j, defect j * v j := by
  have local_work : ∀ j, mass j / 2 * (v j)^2 - mass j / 2 * (old j)^2 +
      mass j / 2 * (v j - old j)^2 + dt * viscous S w v j * v j =
      dt * gradient B p j * v j + defect j * v j := by
    intro j
    have h := congrArg (fun z : ℝ => z * v j) (step j)
    change (mass j * (v j - old j) + dt * viscous S w v j -
      dt * gradient B p j) * v j = defect j * v j at h
    calc
      _ = (mass j * (v j - old j) + dt * viscous S w v j -
            dt * gradient B p j) * v j + dt * gradient B p j * v j := by ring
      _ = defect j * v j + dt * gradient B p j * v j := by rw [h]
      _ = _ := by ring
  have summed := congrArg (fun f : (Fin n → ℝ) => ∑ j, f j) (funext local_work)
  simp only [Finset.sum_add_distrib, Finset.sum_sub_distrib, mul_assoc] at summed
  unfold energy
  rw [← Finset.mul_sum, viscous_work, ← Finset.mul_sum, pressure_work] at summed
  exact summed

theorem exact_constrained_work {n c s : Nat}
    (B : Fin c → Fin n → ℝ) (S : Fin s → Fin n → ℝ)
    (w : Fin s → ℝ) (mass old v : Fin n → ℝ)
    (p : Fin c → ℝ) (dt : ℝ)
    (step : ∀ j, mass j * (v j - old j) + dt * viscous S w v j -
      dt * gradient B p j = 0)
    (constraint : ∀ i, flux B v i = 0) :
    energy mass v - energy mass old + energy mass (fun j => v j - old j) +
      dt * dissipation S w v = 0 := by
  have h := coupled_work_with_defect B S w mass old v (fun _ => 0) p dt step
  simpa [constraint] using h

theorem exact_constrained_energy_nonincrease {n c s : Nat}
    (B : Fin c → Fin n → ℝ) (S : Fin s → Fin n → ℝ)
    (w : Fin s → ℝ) (mass old v : Fin n → ℝ)
    (p : Fin c → ℝ) (dt : ℝ)
    (hm : ∀ j, 0 ≤ mass j) (hw : ∀ a, 0 ≤ w a) (ht : 0 ≤ dt)
    (step : ∀ j, mass j * (v j - old j) + dt * viscous S w v j -
      dt * gradient B p j = 0)
    (constraint : ∀ i, flux B v i = 0) : energy mass v ≤ energy mass old := by
  have identity := exact_constrained_work B S w mass old v p dt step constraint
  have delta : 0 ≤ energy mass (fun j => v j - old j) := by
    apply Finset.sum_nonneg
    intro j _
    exact mul_nonneg (div_nonneg (hm j) (by norm_num)) (sq_nonneg _)
  have loss : 0 ≤ dissipation S w v := by
    apply Finset.sum_nonneg
    intro a _
    exact mul_nonneg (hw a) (sq_nonneg _)
  nlinarith [mul_nonneg ht loss]

end
end Rheon.FollowOn.StokesWork
