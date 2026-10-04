import Rheon.Physics

/-! Exact finite contracts for a planar collision and an unforced viscous step.
They do not certify mesh queries, stencil assembly, material fits or IEEE code.
The wall normal points into the permitted half-space; it need not be unit length.
The strain matrix and viscosity weights are fixed during the implicit step. -/
namespace Rheon.BoundedPhysics
noncomputable section
open scoped BigOperators

def segment (a b : Fin 3 → ℝ) (t : ℝ) (i : Fin 3) : ℝ :=
  (1 - t) * a i + t * b i

def wallValue (normal : Fin 3 → ℝ) (offset : ℝ) (x : Fin 3 → ℝ) : ℝ :=
  (∑ i, normal i * x i) - offset

def wallHit (normal : Fin 3 → ℝ) (offset : ℝ) (a b : Fin 3 → ℝ) : ℝ :=
  wallValue normal offset a / (wallValue normal offset a - wallValue normal offset b)

theorem wall_segment_affine (normal : Fin 3 → ℝ) (offset : ℝ)
    (a b : Fin 3 → ℝ) (t : ℝ) :
    wallValue normal offset (segment a b t) =
      (1 - t) * wallValue normal offset a + t * wallValue normal offset b := by
  unfold wallValue segment
  simp only [mul_add, Finset.sum_add_distrib]
  have ha : (∑ i, normal i * ((1 - t) * a i)) = (1 - t) * ∑ i, normal i * a i := by
    rw [Finset.mul_sum]
    apply Finset.sum_congr rfl
    intro i _
    ring
  have hb : (∑ i, normal i * (t * b i)) = t * ∑ i, normal i * b i := by
    rw [Finset.mul_sum]
    apply Finset.sum_congr rfl
    intro i _
    ring
  rw [ha, hb]
  ring

theorem wall_hit_range (normal : Fin 3 → ℝ) (offset : ℝ) (a b : Fin 3 → ℝ)
    (ha : 0 < wallValue normal offset a) (hb : wallValue normal offset b < 0) :
    0 < wallHit normal offset a b ∧ wallHit normal offset a b < 1 := by
  have hd : 0 < wallValue normal offset a - wallValue normal offset b := by linarith
  unfold wallHit
  constructor
  · exact div_pos ha hd
  · apply (div_lt_one hd).2
    linarith

theorem wall_hit_on_surface (normal : Fin 3 → ℝ) (offset : ℝ) (a b : Fin 3 → ℝ)
    (ha : 0 < wallValue normal offset a) (hb : wallValue normal offset b < 0) :
    wallValue normal offset (segment a b (wallHit normal offset a b)) = 0 := by
  have hd : wallValue normal offset a - wallValue normal offset b ≠ 0 := by linarith
  rw [wall_segment_affine]
  unfold wallHit
  field_simp [hd]
  ring

theorem wall_first_hit (normal : Fin 3 → ℝ) (offset : ℝ) (a b : Fin 3 → ℝ)
    (ha : 0 < wallValue normal offset a) (hb : wallValue normal offset b < 0)
    (t : ℝ) (_ht0 : 0 ≤ t) (ht : t < wallHit normal offset a b) :
    0 < wallValue normal offset (segment a b t) := by
  have hd : 0 < wallValue normal offset a - wallValue normal offset b := by linarith
  have hmul : t * (wallValue normal offset a - wallValue normal offset b) <
      wallValue normal offset a := (lt_div_iff₀ hd).1 ht
  rw [wall_segment_affine]
  nlinarith

theorem clipped_segment_in_halfspace (normal : Fin 3 → ℝ) (offset : ℝ)
    (a b : Fin 3 → ℝ) (ha : 0 < wallValue normal offset a)
    (hb : wallValue normal offset b < 0) (s : ℝ) (_hs0 : 0 ≤ s) (hs1 : s ≤ 1) :
    0 ≤ wallValue normal offset (segment a (segment a b (wallHit normal offset a b)) s) := by
  rw [wall_segment_affine, wall_hit_on_surface normal offset a b ha hb]
  have h : 0 ≤ (1 - s) * wallValue normal offset a :=
    mul_nonneg (by linarith) (le_of_lt ha)
  simpa using h

def strain {n m : ℕ} (E : Fin m → Fin n → ℝ) (v : Fin n → ℝ) (e : Fin m) : ℝ :=
  ∑ i, E e i * v i

def viscousOperator {n m : ℕ} (E : Fin m → Fin n → ℝ) (mu : Fin m → ℝ)
    (v : Fin n → ℝ) (i : Fin n) : ℝ :=
  ∑ e, E e i * mu e * strain E v e

def dissipation {n m : ℕ} (E : Fin m → Fin n → ℝ) (mu : Fin m → ℝ)
    (v : Fin n → ℝ) : ℝ :=
  ∑ e, mu e * (strain E v e)^2

theorem viscous_work {n m : ℕ} (E : Fin m → Fin n → ℝ)
    (mu : Fin m → ℝ) (v : Fin n → ℝ) :
    (∑ i, viscousOperator E mu v i * v i) = dissipation E mu v := by
  unfold viscousOperator dissipation
  simp only [Finset.sum_mul]
  rw [Finset.sum_comm]
  apply Finset.sum_congr rfl
  intro e _
  calc
    (∑ i, E e i * mu e * strain E v e * v i) =
        ∑ i, (mu e * strain E v e) * (E e i * v i) := by
      apply Finset.sum_congr rfl
      intro i _
      ring
    _ = (mu e * strain E v e) * strain E v e := by
      rw [← Finset.mul_sum]
      rfl
    _ = mu e * (strain E v e)^2 := by ring

theorem dissipation_nonnegative {n m : ℕ} (E : Fin m → Fin n → ℝ)
    (mu : Fin m → ℝ) (v : Fin n → ℝ) (hm : ∀ e, 0 ≤ mu e) :
    0 ≤ dissipation E mu v := by
  exact Finset.sum_nonneg (fun e _ => mul_nonneg (hm e) (sq_nonneg _))

theorem backward_euler_work {n m : ℕ} (E : Fin m → Fin n → ℝ)
    (mu : Fin m → ℝ) (mass u v : Fin n → ℝ) (dt : ℝ)
    (step : ∀ i, mass i * v i + dt * viscousOperator E mu v i = mass i * u i) :
    (∑ i, mass i * (u i - v i) * v i) = dt * dissipation E mu v := by
  calc
    (∑ i, mass i * (u i - v i) * v i) =
        dt * ∑ i, viscousOperator E mu v i * v i := by
      rw [Finset.mul_sum]
      apply Finset.sum_congr rfl
      intro i _
      have hi : mass i * (u i - v i) = dt * viscousOperator E mu v i := by
        nlinarith [step i]
      rw [hi]
      ring
    _ = dt * dissipation E mu v := by rw [viscous_work]

theorem backward_euler_energy_nonincrease {n m : ℕ} (E : Fin m → Fin n → ℝ)
    (mu : Fin m → ℝ) (mass u v : Fin n → ℝ) (dt : ℝ)
    (hm : ∀ i, 0 ≤ mass i) (ht : 0 ≤ dt) (hmu : ∀ e, 0 ≤ mu e)
    (step : ∀ i, mass i * v i + dt * viscousOperator E mu v i = mass i * u i) :
    (∑ i, mass i * v i ^ 2) ≤ ∑ i, mass i * u i ^ 2 := by
  exact Rheon.Physics.implicit_energy_nonincrease mass u v dt (dissipation E mu v)
    hm ht (dissipation_nonnegative E mu v hmu) (backward_euler_work E mu mass u v dt step)

end
end Rheon.BoundedPhysics
