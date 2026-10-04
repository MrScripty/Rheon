import Mathlib.Data.Real.Basic
import Mathlib.Algebra.BigOperators.Ring.Finset
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Fin
import Mathlib.Tactic.Ring

/-! Exact finite-dimensional contracts. B is a cells-by-faces incidence matrix
with +1 at a face head and -1 at its tail. Physical divergence is -B u.
No theorem in this module is a floating-point or continuum-flow theorem. -/
namespace Rheon.Discrete
noncomputable section

def gradient {n m : ℕ} (B : Fin n → Fin m → ℝ) (p : Fin n → ℝ) (e : Fin m) : ℝ :=
  ∑ i, B i e * p i

def incidence {n m : ℕ} (B : Fin n → Fin m → ℝ) (u : Fin m → ℝ) (i : Fin n) : ℝ :=
  ∑ e, B i e * u e

def laplace {n m : ℕ} (B : Fin n → Fin m → ℝ) (w : Fin m → ℝ)
    (p : Fin n → ℝ) (i : Fin n) : ℝ :=
  incidence B (fun e => w e * gradient B p e) i

def corrected {n m : ℕ} (B : Fin n → Fin m → ℝ) (w u : Fin m → ℝ)
    (p : Fin n → ℝ) (dt : ℝ) (e : Fin m) : ℝ :=
  u e - dt * w e * gradient B p e

theorem adjoint_identity {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (u : Fin m → ℝ) (p : Fin n → ℝ) :
    (∑ i, p i * incidence B u i) = ∑ e, gradient B p e * u e := by
  simp only [incidence, gradient, Finset.mul_sum, Finset.sum_mul]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro e he
  apply Finset.sum_congr rfl
  intro i hi
  ring

theorem pressure_energy {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (w : Fin m → ℝ) (p : Fin n → ℝ) :
    (∑ i, p i * laplace B w p i) = ∑ e, w e * (gradient B p e)^2 := by
  unfold laplace
  rw [adjoint_identity]
  apply Finset.sum_congr rfl
  intro e he
  ring

theorem pressure_nonnegative {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (w : Fin m → ℝ) (p : Fin n → ℝ) (hw : ∀ e, 0 ≤ w e) :
    0 ≤ ∑ i, p i * laplace B w p i := by
  rw [pressure_energy]
  exact Finset.sum_nonneg (fun e _ => mul_nonneg (hw e) (sq_nonneg _))

theorem pressure_symmetric {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (w : Fin m → ℝ) (p q : Fin n → ℝ) :
    (∑ i, p i * laplace B w q i) = ∑ i, q i * laplace B w p i := by
  unfold laplace
  rw [adjoint_identity, adjoint_identity]
  apply Finset.sum_congr rfl
  intro e he
  ring

theorem constant_gradient_zero {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0) (c : ℝ) :
    gradient B (fun _ => c) = 0 := by
  funext e
  simp [gradient, ← Finset.sum_mul, balanced]

theorem constant_pressure_nullspace {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (w : Fin m → ℝ) (balanced : ∀ e, (∑ i, B i e) = 0) (c : ℝ) :
    laplace B w (fun _ => c) = 0 := by
  have h := constant_gradient_zero B balanced c
  funext i
  simp [laplace, incidence, h]

theorem internal_flux_conservation {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0) (u : Fin m → ℝ) :
    (∑ i, incidence B u i) = 0 := by
  unfold incidence
  rw [Finset.sum_comm]
  simp_rw [← Finset.sum_mul, balanced]
  simp

theorem conservative_update {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (balanced : ∀ e, (∑ i, B i e) = 0) (q : Fin n → ℝ)
    (flux : Fin m → ℝ) (dt : ℝ) :
    (∑ i, (q i + dt * incidence B flux i)) = ∑ i, q i := by
  rw [Finset.sum_add_distrib, ← Finset.mul_sum, internal_flux_conservation B balanced]
  ring

theorem projection_residual {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (w u : Fin m → ℝ) (p b : Fin n → ℝ) (dt : ℝ)
    (rhs : ∀ i, incidence B u i = dt * b i) (i : Fin n) :
    incidence B (corrected B w u p dt) i = dt * (b i - laplace B w p i) := by
  unfold incidence corrected laplace
  simp only [mul_sub, Finset.sum_sub_distrib]
  have h : (∑ e, B i e * (dt * w e * gradient B p e)) =
      dt * ∑ e, B i e * (w e * gradient B p e) := by
    rw [Finset.mul_sum]
    apply Finset.sum_congr rfl
    intro e he
    ring
  rw [h]
  have hr := rhs i
  unfold incidence at hr
  rw [hr]
  unfold incidence
  ring

theorem exact_projection {n m : ℕ} (B : Fin n → Fin m → ℝ)
    (w u : Fin m → ℝ) (p b : Fin n → ℝ) (dt : ℝ)
    (rhs : ∀ i, incidence B u i = dt * b i)
    (solved : ∀ i, laplace B w p i = b i) :
    incidence B (corrected B w u p dt) = 0 := by
  funext i
  rw [projection_residual B w u p b dt rhs i, solved]
  simp

end
end Rheon.Discrete
